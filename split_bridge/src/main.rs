use std::env;
use std::process::{self, Command};
use std::path::Path;
use std::fs;
use std::io::Read;
use std::os::unix::fs::symlink;

fn main() {
    let args: Vec<String> = env::args().collect();
    let home = env::var("HOME").expect("HOME not found");
    let user = env::var("USER").expect("USER not found");
    let split_root = format!("{}/split_bridge", home);

    // ==========================================
    // 1. نظام التحديث الموحد (The Core Rebuild)
    // ==========================================
    if args.contains(&"update".to_string()) {
        println!("🚀 Split Engine Master vMAX: Rebuilding Hybrid Core...");
        let build_rust = format!(
            "cd {} && cargo build --release && sudo install -m 755 target/release/split_bridge /usr/local/bin/split",
            split_root
        );
        let _ = Command::new("fish").arg("-c").arg(build_rust).status();
        println!("✅ VMAX System Synchronized.");
        return;
    }

    // ==========================================
    // 2. تحليل المسار ومعمارية الملف (Binary Analysis)
    // ==========================================
    let mut exe_path_raw = String::new();
    for arg in args.iter().skip(1) {
        if arg != "gpu" && arg != "update" { exe_path_raw = arg.clone(); break; }
    }
    if exe_path_raw.is_empty() {
        println!("❌ Error: No executable specified.");
        process::exit(1);
    }

    let exe_path = Path::new(&exe_path_raw);
    let exe_name = exe_path.file_name().unwrap().to_str().unwrap();
    let game_dir = exe_path.parent().unwrap_or(Path::new("."));
    let game_name = exe_name.replace(".exe", "");

    // فحص إذا كانت اللعبة ثقيلة برمجياً (DirectX 11/12)
    let is_heavy = is_heavy_game(&exe_path_raw);

    // ==========================================
    // 3. تطهير الكاش وإدارة السيف المركزي (Save Arsenal)
    // ==========================================
    let _ = Command::new("rm").arg("-rf").arg(format!("{}/.cache/mesa_shader_cache", home)).status();
    establish_central_save(&game_name, &home, &user);

    // ==========================================
    // 4. واجهة التحكم الاحترافية (VMAX Control Center)
    // ==========================================
    let gui = Command::new("zenity")
    .args(&["--list", "--checklist", "--title=🛡️ Split Control Center vMAX",
          "--column=Enable", "--column=Feature", "--width=450", "--height=350",
          "TRUE", "Performance Overlay (MangoHud)",
          "TRUE", "Zenith Performance (Gamemode)",
          "TRUE", "Activate External Rubber Pipe (ZRAM)",
          "TRUE", "Thermal Guard (Temp Monitor)",
          "FALSE", "Manual FPS Control (Limiter)",
          "FALSE", "Force Vulkan Strict Mode (Intel)",
          "FALSE", "Aggressive RAM Flush (Drop Caches)"])
    .output().expect("Zenity failed");

    let selection = String::from_utf8_lossy(&gui.stdout);
    if selection.trim().is_empty() { process::exit(0); }

    // نظام تحديد الفريمات (FPS Target)
    let mut fps_limit = "0".to_string();
    if selection.contains("Manual FPS Control") {
        let fps_gui = Command::new("zenity")
        .args(&["--list", "--radiolist", "--title=🎯 FPS Limiter", "--column=X", "--column=Limit",
              "FALSE", "30", "TRUE", "60", "FALSE", "90", "FALSE", "120", "FALSE", "OFF"])
        .output().expect("FPS failed");
        let fps_sel = String::from_utf8_lossy(&fps_gui.stdout).trim().to_string();
        if fps_sel != "OFF" && !fps_sel.is_empty() { fps_limit = fps_sel; }
    }

    // ==========================================
    // 5. هندسة البيئة (Intel HD 520 Pipe Tuning)
    // ==========================================
    env::set_current_dir(game_dir).unwrap_or(());
    let mut env_vars: Vec<(&str, String)> = vec![
        ("WINEFSYNC", "1".to_string()),
        ("DXVK_ASYNC", "1".to_string()),
        ("DRI_PRIME", "0".to_string()), // حظر AMD تماماً
        ("MESA_LOADER_DRIVER_OVERRIDE", "iris".to_string()), // إجبار تعريف Intel الحديث
        ("vblank_mode", "0".to_string()), // كسر قيد VSync
        ("DXVK_FRAME_RATE", fps_limit.clone()),
    ];

    // تهيئة MangoHud (البيانات الحقيقية فقط)
    if selection.contains("Performance Overlay") {
        env_vars.push(("MANGOHUD", "1".to_string()));
        let hud_cfg = format!(
            "fps_limit={},cpu_temp,gpu_temp,ram,fps,frame_timing,1percent_low,vsync=0,gl_vsync=0",
            fps_limit
        );
        env_vars.push(("MANGOHUD_CONFIG", hud_cfg));
    }

    if selection.contains("Force Vulkan Strict Mode") || is_heavy {
        env_vars.push(("MESA_VK_DEVICE_SELECT", "8086:1916".to_string()));
        env_vars.push(("VK_ICD_FILENAMES", "/usr/share/vulkan/icd.d/intel_icd.x86_64.json".to_string()));
        env_vars.push(("WINEDLLOVERRIDES", "d3d11,dxgi,d3d9=n,b".to_string()));
    }

    // ==========================================
    // 6. إدارة العتاد والذاكرة (Hardware Level)
    // ==========================================
    if selection.contains("Thermal Guard") { check_thermal_safety(); }
    if selection.contains("Activate External Rubber Pipe") {
        let _ = Command::new("sudo").arg("-n").args(&["zramctl", "--find", "--size", "4G"]).status();
    }
    if selection.contains("Aggressive RAM Flush") {
        let _ = Command::new("sudo").arg("-n").arg("sh").arg("-c").arg("echo 3 > /proc/sys/vm/drop_caches").status();
    }

    // ==========================================
    // 7. إطلاق المحرك (The Command Chain)
    // ==========================================
    let mut cmd_chain = Vec::new();
    if selection.contains("Zenith Performance") { cmd_chain.push("gamemoderun"); }
    if selection.contains("Performance Overlay") { cmd_chain.push("mangohud"); cmd_chain.push("--dlsym"); }
    cmd_chain.push("wine");
    cmd_chain.push(exe_name);

    let mut final_cmd = Command::new(cmd_chain[0]);
    if cmd_chain.len() > 1 { final_cmd.args(&cmd_chain[1..]); }

    println!("🔥 Split VMAX: Launching {} | FPS: {} | Rubber Pipe: ACTIVE", game_name, if fps_limit == "0" { "UNLOCKED" } else { &fps_limit });
    let _ = final_cmd.envs(env_vars).status();
}

// --- وظائف الأنظمة الفرعية (Sub-Systems) ---

fn is_heavy_game(path: &str) -> bool {
    if let Ok(mut file) = fs::File::open(path) {
        let mut buffer = vec![0; 1024 * 1024]; // فحص أول 1MB
        if let Ok(n) = file.read(&mut buffer) {
            let content = String::from_utf8_lossy(&buffer[..n]);
            return content.contains("d3d11") || content.contains("d3d12");
        }
    }
    false
}

fn establish_central_save(game_name: &str, home: &str, user: &str) {
    let wine_prefix = env::var("WINEPREFIX").unwrap_or_else(|_| format!("{}/.wine", home));
    let central_base = format!("{}/Documents/Split_Saves/{}", home, game_name);

    // ربط المجلدات الحيوية للسيف (Documents + AppData)
    let paths = vec![
        (format!("{}/Documents", central_base), format!("{}/drive_c/users/{}/Documents", wine_prefix, user)),
        (format!("{}/AppData", central_base), format!("{}/drive_c/users/{}/AppData/Roaming", wine_prefix, user)),
    ];

    for (central, wine_path) in paths {
        let _ = fs::create_dir_all(&central);
        let wp = Path::new(&wine_path);
        if wp.exists() && !fs::symlink_metadata(wp).map(|m| m.file_type().is_symlink()).unwrap_or(false) {
            let _ = fs::remove_dir_all(wp);
        }
        if !wp.exists() { let _ = symlink(&central, wp); }
    }
}

fn check_thermal_safety() {
    if let Ok(out) = Command::new("sensors").output() {
        let data = String::from_utf8_lossy(&out.stdout);
        if data.contains("+85.0°C") || data.contains("+90.0°C") {
            let _ = Command::new("zenity").args(&["--warning", "--text=🚨 VMAX ALERT: Critical Thermal Levels Detected! Scaling might occur."]).status();
        }
    }
}
