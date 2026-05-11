use std::env;
use std::process::Command;
use std::path::Path;
use std::fs;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

// =====================================================================
// Split — Performance Gaming Launcher (Hybrid Core) v2.1
// مع حفظ الإعدادات والتشغيل السريع
// =====================================================================

fn log(msg: &str, home: &str) {
    let path = format!("{}/.split.log", home);
    let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(&path) else { return };
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
    let Ok(entries) = fs::read_dir(drm) else { return false };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("card") || name.contains('-') { continue; }
        let vp = format!("{}/{}/device/vendor", drm, name);
        let cp = format!("{}/{}/device/class",  drm, name);
        if let (Ok(v), Ok(c)) = (fs::read_to_string(&vp), fs::read_to_string(&cp)) {
            let vendor = v.trim();
            let class  = c.trim();
            if vendor == "0x8086" && (class.starts_with("0x0300") || class == "0x030000") {
                return true;
            }
            if vendor == "0x1002" {
                let vram = detect_vram_mb();
                if vram < 512 { return true; }
            }
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
    for i in 0..=1 {
        let p = format!("/sys/class/drm/card{}/gt/gt0/rps_cur_freq_mhz", i);
        if let Ok(v) = fs::read_to_string(&p) {
            if let Ok(n) = v.trim().parse::<u32>() { return Some(n); }
        }
    }
    for base in &["/sys/class/drm/card0/device/hwmon", "/sys/class/drm/card1/device/hwmon"] {
        if let Ok(entries) = fs::read_dir(base) {
            for entry in entries.flatten() {
                let f = entry.path().join("freq1_input");
                if f.exists() {
                    if let Ok(v) = fs::read_to_string(&f) {
                        if let Ok(khz) = v.trim().parse::<u32>() {
                            return Some(khz / 1000);
                        }
                    }
                }
            }
        }
    }
    if let Ok(output) = Command::new("nvidia-smi")
        .args(["--query-gpu=clocks.gr", "--format=csv,noheader"])
        .output()
        {
            if let Ok(s) = String::from_utf8(output.stdout) {
                if let Some(mhz) = s.trim().split_whitespace().next() {
                    if let Ok(n) = mhz.parse::<u32>() { return Some(n); }
                }
            }
        }
        None
}

fn read_cpu_temp() -> Option<u32> {
    for i in 0..=1 {
        let p = format!("/sys/class/thermal/thermal_zone{}/temp", i);
        if let Ok(v) = fs::read_to_string(&p) {
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
// THERMAL GUARD (يدعم Intel و AMD)
// ─────────────────────────────────────────────

fn start_thermal_guard(home: &str) -> std::sync::mpsc::Sender<()> {
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    let h = home.to_string();
    std::thread::spawn(move || {
        let use_intel = Path::new("/usr/bin/intel-undervolt").exists();
        let use_amd   = Path::new("/usr/bin/ryzenadj").exists();
        if !use_intel && !use_amd {
            log("Thermal guard: no supported tool found (intel-undervolt/ryzenadj)", &h);
            return;
        }
        loop {
            if rx.try_recv().is_ok() { break; }
            if let Some(t) = read_cpu_temp() {
                if t >= 82 {
                    log(&format!("Thermal: {}°C – applying limits", t), &h);
                    if use_intel {
                        let _ = Command::new("intel-undervolt").arg("apply").status();
                    } else if use_amd {
                        let _ = Command::new("ryzenadj")
                        .args(["-a", "15000", "-b", "15000"])
                        .status();
                    }
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
    let crack_files = [
        "CPY.ini", "CODEX.ini", "SKIDROW.ini", "ALI213.ini",
        "cream_api.ini", "SmartSteamEmu.ini", "steam_emu.ini",
        "CrackStatus.txt", "README-SKIDROW.txt", "README-CODEX.txt",
        "SteamworksFix.ini", "Goldberg.ini",
    ];
    if crack_files.iter().any(|f| game_dir.join(f).exists()) {
        return true;
    }
    let has_steam_api = game_dir.join("steam_api64.dll").exists()
    || game_dir.join("steam_api.dll").exists();
    let has_steamless = game_dir.join("steam_appid.txt").exists();
    has_steam_api && has_steamless && !game_dir.join("installscript.vdf").exists()
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
    if Command::new("which").arg("proton").output()
        .map(|o| o.status.success()).unwrap_or(false) {
            return "proton".to_string();
        }
        let candidates = [
            format!("{}/compatibilitytools.d/proton-cachyos/proton", steam_root),
                format!("{}/compatibilitytools.d/proton-cachyos-slr/proton", steam_root),
                    format!("{}/compatibilitytools.d/proton/proton", steam_root),
                        "/usr/share/steam/compatibilitytools.d/proton-cachyos/proton".to_string(),
                        "/usr/share/steam/compatibilitytools.d/proton/proton".to_string(),
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
// DESKTOP INTEGRATION
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
    let candidates = ["icon.png", "logo.png", "game.ico", "icon.ico", "icon.jpg", "icon.jpeg", "Icon.png"];
    let cache_dir = format!("{}/.cache/split/icons", home);
    let _ = fs::create_dir_all(&cache_dir);

    for cand in candidates {
        let icon_path = game_dir.join(cand);
        if icon_path.exists() {
            if cand.ends_with(".ico") {
                let output_png = format!("{}/{}.png", cache_dir, game_name);
                if Command::new("convert")
                    .arg(&icon_path)
                    .arg(&output_png)
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
                    {
                        return Some(output_png);
                    } else {
                        eprintln!("⚠️ Could not convert ico to png, using default icon");
                        continue;
                    }
            } else {
                let dest = format!("{}/{}.png", cache_dir, game_name);
                let _ = fs::copy(&icon_path, &dest);
                return Some(dest);
            }
        }
    }
    None
}

// ─────────────────────────────────────────────
// SAVE/LOAD PROFILE
// ─────────────────────────────────────────────

fn save_profile(game_tag: &str, settings: &[(&str, bool)]) {
    let home = env::var("HOME").unwrap();
    let profile_dir = format!("{}/.config/split/profiles", home);
    let _ = fs::create_dir_all(&profile_dir);
    let path = format!("{}/{}.conf", profile_dir, game_tag);
    let mut content = String::new();
    for (key, value) in settings {
        content.push_str(&format!("{}={}\n", key, value));
    }
    let _ = fs::write(&path, content);
}

fn load_profile(game_tag: &str) -> Option<String> {
    let home = env::var("HOME").unwrap();
    let path = format!("{}/.config/split/profiles/{}.conf", home, game_tag);
    fs::read_to_string(&path).ok()
}

fn parse_profile(content: &str) -> (bool, bool, bool, bool, bool) {
    let mut mango = false;
    let mut thermal = false;
    let mut force_opengl = false;
    let mut use_wine = false;
    let mut add_to_menu = false;
    for line in content.lines() {
        if let Some((key, val)) = line.split_once('=') {
            let is_true = val.trim() == "true";
            match key.trim() {
                "MangoHud"       => mango = is_true,
                "ThermalGuard"   => thermal = is_true,
                "ForceOpenGL"    => force_opengl = is_true,
                "UseWine"        => use_wine = is_true,
                "AddToMenu"      => add_to_menu = is_true,
                _ => {}
            }
        }
    }
    (mango, thermal, force_opengl, use_wine, add_to_menu)
}

// ─────────────────────────────────────────────
// PREFIX INIT (نسخ احتياطي محسّن)
// ─────────────────────────────────────────────

fn init_prefix(prefix: &str, home: &str) -> bool {
    let drive_c = format!("{}/drive_c", prefix);
    if Path::new(&drive_c).exists() { return true; }

    log("Initializing new Wine prefix...", home);

    let wineboot_exists = Command::new("which")
    .arg("wineboot")
    .output()
    .map(|o| o.status.success())
    .unwrap_or(false);

    if wineboot_exists {
        let status = Command::new("wineboot")
        .arg("--init")
        .env("WINEPREFIX", prefix)
        .status();
        if let Ok(s) = status {
            if s.success() {
                log("Prefix initialized successfully", home);
                return true;
            }
        }
    }

    let src_wine = format!("{}/.wine", home);
    if Path::new(&src_wine).exists() {
        log("Falling back to copying ~/.wine (including registry)...", home);
        let _ = fs::create_dir_all(prefix);
        let copy = Command::new("rsync")
        .args(["-av", "--exclude=dosdevices", "--exclude=drive_c", &src_wine, prefix])
        .status();
        if copy.is_ok() {
            let drive_c_src = format!("{}/drive_c", src_wine);
            let drive_c_dst = format!("{}/drive_c", prefix);
            if Path::new(&drive_c_src).exists() && !Path::new(&drive_c_dst).exists() {
                let _ = std::os::unix::fs::symlink(&drive_c_src, &drive_c_dst);
            }
            return true;
        } else {
            let status = Command::new("cp")
            .args(["-r", &src_wine, prefix])
            .status();
            return status.is_ok();
        }
    } else {
        eprintln!("❌ Could not initialize Wine prefix. Run 'winecfg' first.");
        false
    }
}

// ─────────────────────────────────────────────
// ADD GAME TO APPLICATION MENU
// ─────────────────────────────────────────────

fn add_to_application_menu(exe_path: &Path, game_name: &str, home: &str) {
    let apps_dir = format!("{}/.local/share/applications", home);
    let _ = fs::create_dir_all(&apps_dir);
    let safe_name = game_name.replace(' ', "_").replace('/', "_");
    let desktop_file = format!("{}/{}.desktop", apps_dir, safe_name);

    let game_dir = exe_path.parent().unwrap_or(Path::new(""));
    let icon_path = find_game_icon(game_dir, game_name, home);

    let icon_line = match icon_path {
        Some(path) => format!("Icon={}", path),
        None       => "Icon=application-x-ms-dos-executable".to_string(),
    };

    let content = format!(
        "[Desktop Entry]\nType=Application\nName={}\nExec=split \"{}\"\n{}\nTerminal=false\nCategories=Game;\n",
        game_name,
        exe_path.display(),
                          icon_line
    );

    match fs::write(&desktop_file, content) {
        Ok(_)  => println!("✅ Game added to Application Menu: {}", game_name),
        Err(e) => eprintln!("❌ Failed to add to menu: {}", e),
    }
}

// ─────────────────────────────────────────────
// MAIN
// ─────────────────────────────────────────────

fn show_zenity_and_get_settings(game_name: &str, _game_tag: &str, home: &str) -> (bool, bool, bool, bool, bool, bool) {
    let has_mangohud = Path::new("/usr/bin/mangohud").exists();
    let has_undervolt = Path::new("/usr/bin/intel-undervolt").exists() || Path::new("/usr/bin/ryzenadj").exists();
    let hw = get_hw(home);
    let profile = classify(&hw);

    let mut gui: Vec<String> = vec![
        "--list".into(), "--checklist".into(),
        format!("--title=Split [{}] [{:?}]", game_name, profile),
            "--column=".into(), "--column=Option".into(),
            "--width=480".into(), "--height=480".into(),
    ];
    if has_mangohud  { gui.extend(["TRUE".into(),  "MangoHud Overlay".into()]); }
    if has_undervolt { gui.extend(["TRUE".into(),  "Thermal Guard".into()]); }
    gui.extend(["FALSE".into(), "Force OpenGL (WineD3D)".into()]);
    gui.extend(["FALSE".into(), "Use Wine instead of Proton".into()]);
    gui.extend(["TRUE".into(),  "Remember these settings".into()]);
    gui.extend(["FALSE".into(), "Add to Application Menu".into()]);

    let sel = Command::new("zenity")
    .args(&gui)
    .output()
    .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
    .unwrap_or_default();

    if sel.trim().is_empty() {
        std::process::exit(0);
    }

    let mango = sel.contains("MangoHud Overlay");
    let thermal = sel.contains("Thermal Guard");
    let force_opengl = sel.contains("Force OpenGL");
    let use_wine = sel.contains("Use Wine instead of Proton");
    let save = sel.contains("Remember these settings");
    let add_to_menu = sel.contains("Add to Application Menu");

    (mango, thermal, force_opengl, use_wine, save, add_to_menu)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let home = env::var("HOME").expect("HOME not set");

    // ── Commands ─────────────────────────────────────────
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

    // Detect if we have --quick or a saved profile
    let quick_mode = args.iter().any(|a| a == "--quick");
    let raw_exe = args.iter().find(|a| a.ends_with(".exe")).cloned()
    .or_else(|| args.get(1).cloned())
    .expect("Usage: split [--quick] <game.exe>");

    let exe_path = match fs::canonicalize(&raw_exe) {
        Ok(p) => p,
        Err(e) => { eprintln!("❌ Bad path: {}", e); std::process::exit(1); }
    };

    let exe_name = exe_path.file_name().unwrap().to_str().unwrap().to_string();
    let game_dir = exe_path.parent().unwrap().to_path_buf();
    let game_tag = exe_name.replace(".exe", "").replace(' ', "_").to_lowercase();

    let (mango, thermal, force_opengl, use_wine, add_to_menu, save_profile_flag) =
    if quick_mode {
        if let Some(content) = load_profile(&game_tag) {
            let (m, t, f, w, a) = parse_profile(&content);
            (m, t, f, w, a, false)
        } else {
            eprintln!("⚠️ No saved profile for this game. Run without --quick first.");
            std::process::exit(1);
        }
    } else {
        if let Some(content) = load_profile(&game_tag) {
            let use_saved = Command::new("zenity")
            .args(&["--question", "--title=Split", "--text=Saved settings found.\nUse them without reconfiguring?"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
            if use_saved {
                let (m, t, f, w, a) = parse_profile(&content);
                (m, t, f, w, a, false)
            } else {
                let (m, t, f, w, s, a) = show_zenity_and_get_settings(&exe_name, &game_tag, &home);
                (m, t, f, w, a, s)
            }
        } else {
            let (m, t, f, w, s, a) = show_zenity_and_get_settings(&exe_name, &game_tag, &home);
            (m, t, f, w, a, s)
        }
    };

    if save_profile_flag {
        let settings = vec![
            ("MangoHud", mango),
            ("ThermalGuard", thermal),
            ("ForceOpenGL", force_opengl),
            ("UseWine", use_wine),
            ("AddToMenu", add_to_menu),
        ];
        save_profile(&game_tag, &settings);
    }

    if add_to_menu {
        add_to_application_menu(&exe_path, &exe_name, &home);
    }

    let steam_root = find_steam_root(&home);
    let prefix = format!("{}/.local/share/split_data/{}", home, game_tag);
    let _ = fs::create_dir_all(&prefix);

    if !init_prefix(&prefix, &home) {
        std::process::exit(1);
    }

    let cracked = is_cracked(&game_dir);
    let final_use_wine = use_wine || cracked;

    let has_mangohud = Path::new("/usr/bin/mangohud").exists();
    let has_thermal = Path::new("/usr/bin/intel-undervolt").exists() || Path::new("/usr/bin/ryzenadj").exists();

    let thermal_tx = if thermal && has_thermal {
        Some(start_thermal_guard(&home))
    } else { None };

    let hw = get_hw(&home);
    let mut env_map: Vec<(String, String)> = vec![
        ("WINEPREFIX".into(), prefix.clone()),
        ("mesa_glthread".into(), "true".into()),
        ("vblank_mode".into(), "0".into()),
        ("WINEFSYNC".into(), "1".into()),
    ];

    if hw.gpu_vendor == GpuVendor::Intel {
        env_map.push(("MESA_LOADER_DRIVER_OVERRIDE".into(), "iris".into()));
    }

    if force_opengl {
        if final_use_wine {
            env_map.push(("WINEDLLOVERRIDES".into(), "dxgi=b;d3d11=b".into()));
        } else {
            env_map.push(("PROTON_USE_WINED3D".into(), "1".into()));
        }
    }

    if !final_use_wine {
        env_map.push(("STEAM_COMPAT_DATA_PATH".into(), prefix.clone()));
        env_map.push(("STEAM_COMPAT_CLIENT_INSTALL_PATH".into(), steam_root.clone()));
    }

    let mut cmd: Vec<String> = Vec::new();
    if mango && has_mangohud {
        cmd.push("mangohud".to_string());
    }

    if final_use_wine {
        if cracked { log("Crack detected → Wine", &home); }
        cmd.push(find_wine());
        cmd.push(exe_name.clone()); // 🔧 إصلاح الخطأ: استخدام clone()
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
    .envs(env_map)
    .spawn()
    {
        Ok(c) => c,
        Err(e) => { log(&format!("Error: {}", e), &home); return; }
    };

    match child.wait() {
        Ok(s) => log(&format!("Exit: {}", s), &home),
        Err(e) => log(&format!("Wait error: {}", e), &home),
    }

    if let Some(tx) = thermal_tx { let _ = tx.send(()); }
    log(&format!("=== End: {} ===", exe_name), &home);
}
