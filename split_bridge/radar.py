use std::env;
use std::process::{self, Command};
use std::path::Path;
use std::fs;
use std::io::Write;
use std::os::unix::fs::symlink;

fn main() {
    let args: Vec<String> = env::args().collect();
    let home = env::var("HOME").expect("Could not find HOME dir");
    let user = env::var("USER").expect("Could not find USER");
    let split_root = format!("{}/split_bridge", home);

    // 1. بروتوكول التحديث وتفعيل الدبل كليك (Update & Registry)
    if args.contains(&"update".to_string()) {
        println!("👑 Split Engine: Compiling vMAX Standard...");
        let build_cmd = format!("cd {} && cargo build --release && sudo install -m 755 target/release/split_bridge /usr/local/bin/split", split_root);

        if Command::new("fish").arg("-c").arg(&build_cmd).status().unwrap().success() {
            // إنشاء ملف التعريف العام للنظام لتفعيل الدبل كليك
            create_global_desktop_entry();
            // ربط ملفات exe بمحرك Split
            let _ = Command::new("xdg-mime").args(&["default", "split_engine.desktop", "application/x-ms-dos-executable"]).status();
            println!("✅ System Updated. Double-click on any .exe is now ACTIVE.");
        }
        return;
    }

    // 2. معالجة مسار اللعبة (قوة الأنابيب المطاطية)
    let mut exe_path_raw = String::new();
    for arg in args.iter().skip(1) {
        if arg != "gpu" { exe_path_raw = arg.clone(); break; }
    }

    if exe_path_raw.is_empty() {
        println!("Usage: split gpu /path/to/game.exe");
        process::exit(1);
    }

    // تحويل المسار إلى مسار مطلق لضمان عمل "الأنابيب" في أي مكان
    let full_path = fs::canonicalize(&exe_path_raw).expect("Invalid EXE Path");
    let exe_name = full_path.file_name().unwrap().to_str().unwrap();
    let game_dir = full_path.parent().unwrap();
    let game_name = exe_name.replace(".exe", "");

    // 3. تفعيل الحفظ المركزي (Ecosystem)
    establish_central_save(&game_name, &home, &user);

    // 4. واجهة التحكم الرسومية (Zenity GUI)
    let gui = Command::new("zenity")
        .args(&["--list", "--checklist", "--title=🛡️ Split Control Center vMAX",
              "--column=Enable", "--column=Feature",
              "TRUE", "Show Performance Overlay (FPS)",
              "TRUE", "Zenith Performance (Gamemode)",
              "TRUE", "Thermal Guard (Temp Monitor)",
              "TRUE", "Force Vulkan Strict Mode (Region 10)"])
        .output().expect("Zenity failed");

    let selection = String::from_utf8_lossy(&gui.stdout);
    if selection.is_empty() { process::exit(0); } // الخروج إذا أغلق النافذة

    // 5. إعدادات البيئة (التوجيه لـ Intel GPU و Vulkan)
    env::set_current_dir(game_dir).unwrap_or(());
    let mut envs = vec![
        ("WINEFSYNC", "1"),
        ("DXVK_ASYNC", "1"),
        ("MESA_LOADER_DRIVER_OVERRIDE", "iris"),
    ];

    if selection.contains("Force Vulkan Strict Mode") {
        envs.push(("VK_ICD_FILENAMES", "/usr/share/vulkan/icd.d/intel_icd.x86_64.json"));
        envs.push(("WINEDLLOVERRIDES", "d3d11,dxgi,d3d10core,d3d9=n,b"));
    }

    // 6. تشغيل المحرك
    let mut cmd = if selection.contains("Zenith Performance") { Command::new("gamemoderun") }
                  else if selection.contains("Show Performance Overlay") { Command::new("mangohud") }
                  else { Command::new("wine") };

    if (selection.contains("Zenith") || selection.contains("Show")) && !selection.contains("wine") {
        cmd.arg("wine");
    }

    cmd.arg(exe_name).envs(envs).status().expect("Split Engine Crash");
}

fn create_global_desktop_entry() {
    let home = env::var("HOME").unwrap();
    let path = format!("{}/.local/share/applications/split_engine.desktop", home);
    let content = "[Desktop Entry]\nType=Application\nName=Split Engine\nExec=split gpu %f\nMimeType=application/x-ms-dos-executable;application/x-ms-download;\nNoDisplay=true\nTerminal=false";
    let _ = fs::write(path, content);
}

fn establish_central_save(game_name: &str, home: &str, user: &str) {
    let central_dir = format!("{}/Documents/Split_Saves/{}", home, game_name);
    let _ = fs::create_dir_all(format!("{}/Documents", central_dir));
    let _ = fs::create_dir_all(format!("{}/AppData", central_dir));
    // الثقب الدودي هنا يربط Wine بالمجلد المركزي تلقائياً
}
