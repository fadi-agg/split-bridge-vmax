use std::env;
use std::process::Command;
use std::path::Path;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

// =====================================================================
// Split — Performance Gaming Launcher (Hybrid Core)
// =====================================================================

fn log(msg: &str, home: &str) {
    let path = format!("{}/.split.log", home);
    let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) else { return };
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let _ = writeln!(f, "[{}] {}", ts, msg);
}

// ─────────────────────────────────────────────
// HARDWARE DETECTION
// ─────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum GpuVendor { Intel, Amd, Nvidia, Generic }

#[derive(Debug, Clone, PartialEq)]
enum Profile { Potato, Mid, High }

#[derive(Debug, Clone)]
struct HwInfo {
    gpu_vendor:  GpuVendor,
    vram_mb:     u64,
    is_igpu:     bool,
    cpu_threads: u32,
    cpu_mhz:     u32,
    ram_mb:      u64,
}

fn detect_hw() -> HwInfo {
    HwInfo {
        gpu_vendor:  detect_gpu_vendor(),
        vram_mb:     detect_vram_mb(),
        is_igpu:     detect_is_igpu(),
        cpu_threads: detect_cpu_threads(),
        cpu_mhz:     detect_cpu_mhz(),
        ram_mb:      detect_ram_mb(),
    }
}

fn detect_gpu_vendor() -> GpuVendor {
    let drm = "/sys/class/drm";
    let Ok(entries) = fs::read_dir(drm) else { return GpuVendor::Generic };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if !name.starts_with("card") || name.contains('-') { continue; }
        let p = format!("{}/{}/device/vendor", drm, name);
        if let Ok(v) = fs::read_to_string(&p) {
            return match v.trim() {
                "0x8086" => GpuVendor::Intel,
                "0x1002" => GpuVendor::Amd,
                "0x10de" => GpuVendor::Nvidia,
                _        => GpuVendor::Generic,
            };
        }
    }
    GpuVendor::Generic
}

fn detect_vram_mb() -> u64 {
    let drm = "/sys/class/drm";
    if let Ok(entries) = fs::read_dir(drm) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with("card") || name.contains('-') { continue; }
            let p = format!("{}/{}/device/mem_info_vram_total", drm, name);
            if let Ok(v) = fs::read_to_string(&p) {
                if let Ok(b) = v.trim().parse::<u64>() {
                    if b > 0 { return b / (1024 * 1024); }
                }
            }
        }
    }
    let ram = detect_ram_mb();
    if ram >= 16384 { 1024 } else if ram >= 8192 { 512 } else { 256 }
}

fn detect_is_igpu() -> bool {
    let drm = "/sys/class/drm";
    let Ok(entries) = fs::read_dir(drm) else { return true };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("card") || name.contains('-') { continue; }
        let vp = format!("{}/{}/device/vendor", drm, name);
        let cp = format!("{}/{}/device/class",  drm, name);
        if let (Ok(v), Ok(c)) = (fs::read_to_string(&vp), fs::read_to_string(&cp)) {
            if v.trim() == "0x8086" && c.trim().starts_with("0x0300") { return true; }
            if c.trim() == "0x030000" { return true; }
        }
    }
    false
}

fn detect_cpu_threads() -> u32 {
    fs::read_to_string("/proc/cpuinfo")
    .map(|c| c.lines().filter(|l| l.starts_with("processor")).count() as u32)
    .unwrap_or(2)
}

fn detect_cpu_mhz() -> u32 {
    fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq")
    .ok().and_then(|s| s.trim().parse::<u32>().ok())
    .map(|k| k / 1000).unwrap_or(2000)
}

fn detect_ram_mb() -> u64 {
    fs::read_to_string("/proc/meminfo").ok()
    .and_then(|c| c.lines().find(|l| l.starts_with("MemTotal:"))
    .and_then(|l| l.split_whitespace().nth(1))
    .and_then(|v| v.parse::<u64>().ok())
    .map(|k| k / 1024))
    .unwrap_or(4096)
}

fn detect_ram_free_mb() -> u64 {
    fs::read_to_string("/proc/meminfo").ok()
    .and_then(|c| c.lines().find(|l| l.starts_with("MemAvailable:"))
    .and_then(|l| l.split_whitespace().nth(1))
    .and_then(|v| v.parse::<u64>().ok())
    .map(|k| k / 1024))
    .unwrap_or(2048)
}

fn read_gpu_freq_mhz() -> Option<u32> {
    for p in &[
        "/sys/class/drm/card0/gt/gt0/rps_cur_freq_mhz",
        "/sys/class/drm/card1/gt/gt0/rps_cur_freq_mhz",
    ] {
        if let Ok(v) = fs::read_to_string(p) {
            if let Ok(n) = v.trim().parse::<u32>() { return Some(n); }
        }
    }
    None
}

fn read_cpu_temp() -> Option<u32> {
    for p in &[
        "/sys/class/thermal/thermal_zone0/temp",
        "/sys/class/thermal/thermal_zone1/temp",
    ] {
        if let Ok(v) = fs::read_to_string(p) {
            if let Ok(t) = v.trim().parse::<u32>() { return Some(t / 1000); }
        }
    }
    None
}

fn get_hw(home: &str) -> HwInfo {
    let path = format!("{}/.config/split/hw.cache", home);
    if let Ok(meta) = fs::metadata(&path) {
        if let Ok(modified) = meta.modified() {
            let age = SystemTime::now().duration_since(modified)
            .unwrap_or_default().as_secs();
            if age < 7 * 24 * 3600 {
                if let Some(hw) = load_hw_cache(&path) { return hw; }
            }
        }
    }
    let hw = detect_hw();
    save_hw_cache(&hw, &path);
    hw
}

fn load_hw_cache(path: &str) -> Option<HwInfo> {
    let content = fs::read_to_string(path).ok()?;
    let get = |key: &str| -> String {
        content.lines().find(|l| l.starts_with(key))
        .and_then(|l| l.split('=').nth(1))
        .map(|v| v.trim().trim_matches('"').to_string())
        .unwrap_or_default()
    };
    Some(HwInfo {
        gpu_vendor: match get("gpu_vendor").as_str() {
            "Intel"  => GpuVendor::Intel,
            "Amd"    => GpuVendor::Amd,
            "Nvidia" => GpuVendor::Nvidia,
            _        => GpuVendor::Generic,
        },
         vram_mb:     get("vram_mb").parse().ok()?,
         is_igpu:     get("is_igpu") == "true",
         cpu_threads: get("cpu_threads").parse().ok()?,
         cpu_mhz:     get("cpu_mhz").parse().ok()?,
         ram_mb:      get("ram_mb").parse().ok()?,
    })
}

fn save_hw_cache(hw: &HwInfo, path: &str) {
    if let Some(p) = Path::new(path).parent() { let _ = fs::create_dir_all(p); }
    let _ = fs::write(path, format!(
        "gpu_vendor  = \"{:?}\"\nvram_mb     = {}\nis_igpu     = {}\n\
cpu_threads = {}\ncpu_mhz     = {}\nram_mb      = {}\n",
hw.gpu_vendor, hw.vram_mb, hw.is_igpu,
hw.cpu_threads, hw.cpu_mhz, hw.ram_mb,
    ));
}

fn classify(hw: &HwInfo) -> Profile {
    if hw.is_igpu || hw.vram_mb < 2048 || hw.ram_mb < 6144 { return Profile::Potato; }
    if hw.vram_mb >= 8192 && hw.ram_mb >= 16384 { return Profile::High; }
    Profile::Mid
}

// ─────────────────────────────────────────────
// THERMAL GUARD
// ─────────────────────────────────────────────

fn start_thermal_guard(home: &str) -> std::sync::mpsc::Sender<()> {
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    let h = home.to_string();
    std::thread::spawn(move || {
        loop {
            if rx.try_recv().is_ok() { break; }
            let temp = fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")
            .ok().and_then(|s| s.trim().parse::<u32>().ok()).map(|t| t / 1000);
            if let Some(t) = temp {
                if t >= 82 {
                    log(&format!("Thermal: {}°C", t), &h);
                    let _ = Command::new("intel-undervolt").arg("apply").status();
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(15));
        }
    });
    tx
}

// ─────────────────────────────────────────────
// CRACK DETECTION
// ─────────────────────────────────────────────

fn is_cracked(game_dir: &Path) -> bool {
    ["CPY.ini", "CODEX.ini", "SKIDROW.ini", "ALI213.ini", "cream_api.ini"]
    .iter().any(|f| game_dir.join(f).exists())
    || game_dir.join("steam_api64.dll").exists()
    || game_dir.join("steam_api.dll").exists()
}

// ─────────────────────────────────────────────
// PATH HELPERS
// ─────────────────────────────────────────────

fn find_steam_root(home: &str) -> String {
    let candidates = [
        format!("{}/.local/share/Steam", home),
            format!("{}/.steam/steam", home),
                "/usr/share/steam".to_string(),
                format!("{}/.var/app/com.valvesoftware.Steam/data/Steam", home),
    ];
    for p in &candidates {
        if Path::new(&format!("{}/steamapps", p)).exists()
            || Path::new(&format!("{}/compatibilitytools.d", p)).exists() {
                return p.clone();
            }
    }
    candidates[0].clone()
}

fn find_proton(steam_root: &str) -> String {
    let candidates = [
        "/usr/share/steam/compatibilitytools.d/proton-cachyos/proton".to_string(),
        "/usr/share/steam/compatibilitytools.d/proton-cachyos-slr/proton".to_string(),
        format!("{}/compatibilitytools.d/proton-cachyos/proton", steam_root),
    ];
    for p in &candidates {
        if Path::new(p).exists() { return p.clone(); }
    }
    if let Ok(o) = Command::new("find")
        .args(["/usr/share/steam", "-name", "proton", "-type", "f", "-maxdepth", "5"])
        .output()
        {
            if let Some(line) = String::from_utf8_lossy(&o.stdout).lines().next() {
                return line.trim().to_string();
            }
        }
        "proton".to_string()
}

fn find_wine() -> String {
    for bin in &["wine", "wine-staging", "wine-stable"] {
        if Command::new("which").arg(bin).output()
            .map(|o| o.status.success()).unwrap_or(false) {
                return bin.to_string();
            }
    }
    "wine".to_string()
}

// ─────────────────────────────────────────────
// DESKTOP INTEGRATION (لجعل Split المشغل الافتراضي)
// ─────────────────────────────────────────────

fn integrate(home: &str) {
    let dir = format!("{}/.local/share/applications", home);
    let _ = fs::create_dir_all(&dir);
    let _ = fs::write(format!("{}/split.desktop", dir),
                      "[Desktop Entry]\nType=Application\nName=Split\n\
Exec=split \"%f\"\nMimeType=application/x-ms-dos-executable;\n\
NoDisplay=true\nTerminal=false\n");
    let _ = Command::new("xdg-mime")
    .args(["default", "split.desktop", "application/x-ms-dos-executable"])
    .status();
    println!("✅ Split set as default .exe handler");
}

// ─────────────────────────────────────────────
// EXTRACT ICON FROM GAME FOLDER
// ─────────────────────────────────────────────

fn find_game_icon(game_dir: &Path, game_name: &str, home: &str) -> Option<String> {
    // أسماء الملفات المحتملة للأيقونة
    let candidates = ["icon.png", "logo.png", "game.ico", "icon.jpg", "icon.jpeg", "Icon.png"];
    let cache_dir = format!("{}/.cache/split/icons", home);
    let _ = fs::create_dir_all(&cache_dir);

    for cand in candidates {
        let icon_path = game_dir.join(cand);
        if icon_path.exists() {
            // إذا كان الملف .ico، نحاول تحويله إلى .png
            if cand.ends_with(".ico") {
                let output_png = format!("{}/{}.png", cache_dir, game_name);
                if Command::new("convert")
                    .arg(&icon_path)
                    .arg(&output_png)
                    .status()
                    .is_ok()
                    {
                        return Some(output_png);
                    } else {
                        eprintln!("⚠️ Could not convert ico to png, using default icon");
                        continue;
                    }
            } else {
                // نسخ الملف إلى ذاكرة التخزين المؤقت
                let dest = format!("{}/{}.png", cache_dir, game_name);
                let _ = fs::copy(&icon_path, &dest);
                return Some(dest);
            }
        }
    }
    None // لا توجد أيقونة
}

// ─────────────────────────────────────────────
// ADD GAME TO APPLICATION MENU (START MENU)
// ─────────────────────────────────────────────

fn add_to_application_menu(exe_path: &Path, game_name: &str, home: &str) {
    let apps_dir = format!("{}/.local/share/applications", home);
    let _ = fs::create_dir_all(&apps_dir);
    let desktop_file = format!("{}/{}.desktop", apps_dir, game_name.replace(' ', "_").replace('/', "_"));

    // البحث عن أيقونة
    let game_dir = exe_path.parent().unwrap_or(Path::new(""));
    let icon_path = find_game_icon(game_dir, game_name, home);

    let icon_line = match icon_path {
        Some(path) => format!("Icon={}", path),
        None => "Icon=application-x-ms-dos-executable".to_string(),
    };

    let exec_line = format!("Exec=split \"{}\"", exe_path.display());
    let content = format!(
        "[Desktop Entry]\n\
Type=Application\n\
Name={}\n\
{}\n\
{}\n\
Terminal=false\n\
Categories=Game;\n",
game_name, exec_line, icon_line
    );

    match fs::write(&desktop_file, content) {
        Ok(_) => println!("✅ Game added to Application Menu: {}", game_name),
        Err(e) => eprintln!("❌ Failed to add to menu: {}", e),
    }
}

// ─────────────────────────────────────────────
// MAIN
// ─────────────────────────────────────────────

fn main() {
    let args: Vec<String> = env::args().collect();
    let home = env::var("HOME").expect("HOME not set");
    let user = env::var("USER").expect("USER not set");

    match args.get(1).map(|s| s.as_str()) {
        Some("integrate") => { integrate(&home); return; }
        Some("update") => {
            let _ = Command::new("sh").arg("-c")
            .arg("cargo build --release && \
BIN=$(find target/release -maxdepth 1 -type f -executable ! -name '*.so' | head -1) && \
pkexec install -m755 \"$BIN\" /usr/local/bin/split && echo '✅ Updated'")
            .status();
            return;
        }
        Some("info") => {
            let hw      = get_hw(&home);
            let profile = classify(&hw);
            let free    = detect_ram_free_mb();
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("  Split — System Info");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("  GPU:     {:?}{}", hw.gpu_vendor,
                     if hw.is_igpu { " (integrated)" } else { "" });
            println!("  VRAM:    {} MB", hw.vram_mb);
            if let Some(f) = read_gpu_freq_mhz() { println!("  GPU MHz: {} (live)", f); }
            println!("  CPU:     {} threads @ {} MHz", hw.cpu_threads, hw.cpu_mhz);
            println!("  RAM:     {} MB / {} MB free", hw.ram_mb, free);
            if let Some(t) = read_cpu_temp() { println!("  Temp:    {}°C (live)", t); }
            println!("  Profile: {:?}", profile);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            return;
        }
        _ => {}
    }

    let raw = args.iter().skip(1).find(|a| a.ends_with(".exe")).cloned()
    .unwrap_or_else(|| {
        eprintln!("Usage: split <game.exe> | split info | split integrate | split update");
        std::process::exit(1);
    });

    let exe_path = match fs::canonicalize(&raw) {
        Ok(p)  => p,
        Err(e) => { eprintln!("❌ Bad path: {}", e); std::process::exit(1); }
    };

    let exe_name = exe_path.file_name().unwrap().to_str().unwrap().to_string();
    let game_dir = exe_path.parent().unwrap().to_path_buf();
    let game_tag = exe_name.replace(".exe", "").replace(' ', "_").to_lowercase();

    let hw      = get_hw(&home);
    let profile = classify(&hw);
    log(&format!("=== {} | {:?} | {:?} ===", exe_name, profile, hw.gpu_vendor), &home);

    let has_mangohud  = Path::new("/usr/bin/mangohud").exists();
    let has_undervolt = Path::new("/usr/bin/intel-undervolt").exists();

    let mut gui: Vec<String> = vec![
        "--list".into(), "--checklist".into(),
        format!("--title=Split [{:?}]", profile),
            "--column=".into(), "--column=Option".into(),
            "--width=480".into(), "--height=380".into(),
    ];
    if has_mangohud  { gui.extend(["TRUE".into(),  "MangoHud Overlay".into()]); }
    if has_undervolt { gui.extend(["TRUE".into(),  "Thermal Guard".into()]); }
    gui.extend(["FALSE".into(), "Force OpenGL (WineD3D)".into()]);
    gui.extend(["FALSE".into(), "Use Wine instead of Proton".into()]);
    gui.extend(["FALSE".into(), "Add to Application Menu (Start Menu)".into()]); // الخيار الجديد

    let sel = Command::new("zenity").args(&gui).output()
    .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
    .unwrap_or_default();

    if sel.trim().is_empty() { log("Cancelled", &home); return; }

    // إضافة اللعبة إلى قائمة ابدأ إذا طلب المستخدم
    if sel.contains("Add to Application Menu") {
        let game_display_name = exe_name.replace(".exe", "");
        add_to_application_menu(&exe_path, &game_display_name, &home);
    }

    let steam_root = find_steam_root(&home);
    let prefix     = format!("{}/.local/share/split_data/{}", home, game_tag);
    let _ = fs::create_dir_all(&prefix);

    let cracked  = is_cracked(&game_dir);
    let use_wine = sel.contains("Use Wine instead of Proton") || cracked;

    let dst_drive = format!("{}/drive_c", prefix);
    if !Path::new(&dst_drive).exists() {
        let src_drive = format!("{}/.wine/drive_c", home);
        if Path::new(&src_drive).exists() {
            log("Copying ~/.wine/drive_c to new prefix (first run)...", &home);
            let _ = Command::new("cp").args(["-r", &src_drive, &prefix]).status();
            log("Prefix ready", &home);
        } else {
            eprintln!("❌ ~/.wine not found. Please run 'winecfg' first.");
            std::process::exit(1);
        }
    }

    let thermal_tx = if sel.contains("Thermal Guard") && has_undervolt {
        Some(start_thermal_guard(&home))
    } else { None };

    let mut env_vars = vec![
        ("WINEPREFIX", prefix.as_str()),
        ("MESA_LOADER_DRIVER_OVERRIDE", "iris"),
        ("mesa_glthread", "true"),
        ("vblank_mode", "0"),
        ("WINEFSYNC", "1"),
        ("WINEDLLOVERRIDES", "dxgi=n,b;d3d11=n,b"),
    ];

    if sel.contains("Force OpenGL") {
        env_vars.retain(|(k, _)| *k != "WINEDLLOVERRIDES");
        env_vars.push(("PROTON_USE_WINED3D", "1"));
    }

    if !use_wine {
        env_vars.push(("STEAM_COMPAT_DATA_PATH", &prefix));
        env_vars.push(("STEAM_COMPAT_CLIENT_INSTALL_PATH", &steam_root));
    }

    let mut cmd: Vec<String> = Vec::new();
    if sel.contains("MangoHud") && has_mangohud {
        cmd.push("mangohud".to_string());
    }

    if use_wine {
        if cracked { log("Crack detected → Wine", &home); }
        cmd.push(find_wine());
        cmd.push(exe_name.clone());
    } else {
        let proton = find_proton(&steam_root);
        log(&format!("Proton: {}", proton), &home);
        cmd.push(proton);
        cmd.push("run".to_string());
        cmd.push(exe_path.to_string_lossy().to_string());
    }

    log(&format!("Launch: {}", cmd.join(" ")), &home);

    let mut child = match Command::new(&cmd[0])
    .args(&cmd[1..])
    .current_dir(&game_dir)
    .envs(env_vars)
    .spawn()
    {
        Ok(c)  => c,
        Err(e) => { log(&format!("Error: {}", e), &home); return; }
    };

    match child.wait() {
        Ok(s)  => log(&format!("Exit: {}", s), &home),
        Err(e) => log(&format!("Wait error: {}", e), &home),
    }

    if let Some(tx) = thermal_tx { let _ = tx.send(()); }
    log(&format!("=== End: {} ===", exe_name), &home);
}
