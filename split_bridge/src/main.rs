use std::env;
use std::process::{self, Command};
use std::path::Path;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::symlink;
use std::thread;
use std::time::Duration;

// ==========================================
// 0. نظام التسجيل (The Black Box Logger)
// ==========================================
fn vmax_log(msg: &str, home: &str) {
    let log_path = format!("{}/.split_vmax.log", home);
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
        let timestamp = Command::new("date").arg("+%Y-%m-%d %H:%M:%S").output().ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
        let _ = writeln!(file, "[{}] {}", timestamp, msg);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let home = env::var("HOME").expect("HOME not found");
    let user = env::var("USER").expect("USER not found");
    let split_root = format!("{}/split_bridge", home);

    vmax_log("--- VMAX Engine Started ---", &home);

    // ==========================================
    // 1. نظام التحديث الموحد
    // ==========================================
    if args.contains(&"update".to_string()) {
        println!("🚀 Split Engine Master vMAX: Rebuilding Hybrid Core...");
        let build_rust = format!(
            "cd {} && cargo build --release && pkexec install -m 755 target/release/split_bridge /usr/local/bin/split",
            split_root
        );
        let _ = Command::new("fish").arg("-c").arg(build_rust).status();
        vmax_log("System Updated Successfully.", &home);
        return;
    }

    // ==========================================
    // 2. تحليل المسار وتحديد نوع اللعبة (is_heavy)
    // ==========================================
    let mut exe_path_raw = String::new();
    for arg in args.iter().skip(1) {
        if arg != "gpu" && arg != "update" { exe_path_raw = arg.clone(); break; }
    }

    if exe_path_raw.is_empty() {
        vmax_log("Error: Launch failed, no path provided.", &home);
        process::exit(1);
    }

    let exe_path = Path::new(&exe_path_raw);
    let exe_name = match exe_path.file_name().and_then(|n| n.to_str()) {
        Some(name) => name,
        None => process::exit(1),
    };

    let game_dir = exe_path.parent().unwrap_or(Path::new("."));
    let game_name = exe_name.replace(".exe", "");

    // فحص المحرك: هل اللعبة تتطلب DirectX حديث؟
    let is_heavy = is_heavy_game(&exe_path_raw);

    // ==========================================
    // 3. تطهير الكاش وإدارة السيف المركزي
    // ==========================================
    let _ = Command::new("rm").arg("-rf").arg(format!("{}/.cache/mesa_shader_cache", home)).status();
    establish_central_save(&game_name, &home, &user);

    // ==========================================
    // 4. واجهة التحكم (VMAX Control Center)
    // ==========================================
    let gui = Command::new("zenity")
    .args(&["--list", "--checklist", "--title=🛡️ Split Control Center vMAX",
          "--column=Enable", "--column=Feature", "--width=450", "--height=350",
          "TRUE", "Performance Overlay (MangoHud)",
          "TRUE", "Zenith Performance (Gamemode)",
          "TRUE", "Activate Dynamic ZRAM (Rubber Pipe)",
          "TRUE", "Thermal Guard (Auto Undervolt)",
          "FALSE", "Manual FPS Control (Limiter)", // معطل افتراضياً لتجارب الأداء الأقصى
          "FALSE", "Force Vulkan Strict Mode (Intel)",
          "FALSE", "Aggressive RAM Flush (Drop Caches)"])
    .output().expect("Zenity failed");

    let selection = String::from_utf8_lossy(&gui.stdout);
    if selection.trim().is_empty() { process::exit(0); }

    // نظام تحديد الفريمات الاختياري
    let mut fps_limit = "0".to_string(); // 0 تعني غير محدود (Unlocked)
    if selection.contains("Manual FPS Control") {
        let fps_gui = Command::new("zenity")
        .args(&["--list", "--radiolist", "--title=🎯 FPS Limiter", "--column=X", "--column=Limit",
              "FALSE", "30", "TRUE", "60", "FALSE", "90", "FALSE", "120", "FALSE", "OFF"])
        .output().expect("FPS failed");
        let fps_sel = String::from_utf8_lossy(&fps_gui.stdout).trim().to_string();
        if fps_sel != "OFF" && !fps_sel.is_empty() {
            fps_limit = fps_sel;
            vmax_log(&format!("FPS Limit set to: {}", fps_limit), &home);
        }
    } else {
        vmax_log("FPS Limit: UNLOCKED for maximum performance.", &home);
    }

    // ==========================================
    // 5. هندسة البيئة (Environment Logic)
    // ==========================================
    env::set_current_dir(game_dir).unwrap_or(());
    let mut env_vars: Vec<(&str, String)> = vec![
        ("WINEFSYNC", "1".to_string()),
        ("DXVK_ASYNC", "1".to_string()),
        ("DRI_PRIME", "0".to_string()),
        ("MESA_LOADER_DRIVER_OVERRIDE", "iris".to_string()),
        ("vblank_mode", "0".to_string()), // كسر قيد VSync للوصول لأعلى فريمات
        ("DXVK_FRAME_RATE", fps_limit.clone()),
    ];

    if selection.contains("Performance Overlay") {
        env_vars.push(("MANGOHUD", "1".to_string()));
        // في المانجو هود، نضع الليمت فقط إذا لم يكن 0
        let hud_cfg = if fps_limit != "0" {
            format!("fps_limit={},cpu_temp,gpu_temp,ram,fps,vsync=0", fps_limit)
        } else {
            "cpu_temp,gpu_temp,ram,fps,vsync=0".to_string()
        };
        env_vars.push(("MANGOHUD_CONFIG", hud_cfg));
    }

    // استخدام is_heavy لتفعيل Vulkan الصارم للألعاب المتطلبة
    if selection.contains("Force Vulkan Strict Mode") || is_heavy {
        vmax_log("Smart Detection: Activating High-Performance Vulkan Bridge.", &home);
        env_vars.push(("MESA_VK_DEVICE_SELECT", "8086:1916".to_string()));
        env_vars.push(("VK_ICD_FILENAMES", "/usr/share/vulkan/icd.d/intel_icd.x86_64.json".to_string()));
        env_vars.push(("WINEDLLOVERRIDES", "d3d11,dxgi,d3d9=n,b".to_string()));
    }

    // ==========================================
    // 6. إدارة العتاد (Hardware Level)
    // ==========================================

    if selection.contains("Thermal Guard") {
        let home_clone = home.clone();
        thread::spawn(move || {
            active_thermal_guard(&home_clone);
        });
    }

    if selection.contains("Activate Dynamic ZRAM") {
        if let Ok(mem_info) = fs::read_to_string("/proc/meminfo") {
            if let Some(total_kb) = mem_info.lines().find(|l| l.contains("MemTotal")).and_then(|l| l.split_whitespace().nth(1)) {
                if let Ok(kb) = total_kb.parse::<u64>() {
                    let zram_size_mb = (kb / 1024) / 4;
                    let _ = Command::new("pkexec").args(&["zramctl", "--find", "--size", &format!("{}M", zram_size_mb)]).status();
                }
            }
        }
    }

    if selection.contains("Aggressive RAM Flush") {
        let _ = Command::new("pkexec").arg("sh").arg("-c").arg("echo 3 > /proc/sys/vm/drop_caches").status();
    }

    // ==========================================
    // 7. إطلاق المحرك
    // ==========================================
    let mut cmd_chain = Vec::new();
    if selection.contains("Zenith Performance") { cmd_chain.push("gamemoderun"); }
    if selection.contains("Performance Overlay") { cmd_chain.push("mangohud"); cmd_chain.push("--dlsym"); }
    cmd_chain.push("wine");
    cmd_chain.push(exe_name);

    let mut final_cmd = Command::new(cmd_chain[0]);
    if cmd_chain.len() > 1 { final_cmd.args(&cmd_chain[1..]); }

    println!("🔥 Split VMAX: Launching {} | Target FPS: {}", game_name, if fps_limit == "0" { "MAX/UNLOCKED" } else { &fps_limit });
    let _ = final_cmd.envs(env_vars).status();
}

// --- وظائف الأنظمة الفرعية ---

fn is_heavy_game(path: &str) -> bool {
    if let Ok(mut file) = fs::File::open(path) {
        let mut buffer = vec![0; 1024 * 1024];
        if let Ok(n) = file.read(&mut buffer) {
            let content = String::from_utf8_lossy(&buffer[..n]);
            // فحص وجود مكتبات DirectX 11/12
            return content.contains("d3d11") || content.contains("d3d12");
        }
    }
    false
}

fn establish_central_save(game_name: &str, home: &str, user: &str) {
    let wine_prefix = env::var("WINEPREFIX").unwrap_or_else(|_| format!("{}/.wine", home));
    let central_base = format!("{}/Documents/Split_Saves/{}", home, game_name);
    let paths = vec![
        (format!("{}/Documents", central_base), format!("{}/drive_c/users/{}/Documents", wine_prefix, user)),
        (format!("{}/AppData", central_base), format!("{}/drive_c/users/{}/AppData/Roaming", wine_prefix, user)),
    ];
    for (central, wine_path) in paths {
        let _ = fs::create_dir_all(&central);
        let wp = Path::new(&wine_path);
        if wp.exists() && !fs::symlink_metadata(wp).map(|m| m.file_type().is_symlink()).unwrap_or(false) {
            let _ = Command::new("cp").arg("-r").arg(format!("{}/.", wine_path)).arg(&central).status();
            let _ = fs::remove_dir_all(wp);
        }
        if !wp.exists() { let _ = symlink(&central, wp); }
    }
}

fn active_thermal_guard(home: &str) {
    let mut is_undervolted = false;
    loop {
        if let Ok(out) = Command::new("sensors").output() {
            let data = String::from_utf8_lossy(&out.stdout);
            if data.contains("+85.0°C") || data.contains("+90.0°C") {
                if !is_undervolted {
                    vmax_log("🔥 Thermal Spike! Applying Silent Undervolt.", home);
                    let _ = Command::new("pkexec").arg("intel-undervolt").arg("apply").status();
                    is_undervolted = true;
                }
            } else if data.contains("+70.0°C") {
                is_undervolted = false;
            }
        }
        thread::sleep(Duration::from_secs(30));
    }
}
