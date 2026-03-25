# 🛡️ Split Bridge VMAX (The Zenith Engine)

**Split Bridge VMAX** هو محرك إدارة أداء هجين (Hybrid Performance Engine) مكتوب بلغة **Rust**، مصمم خصيصاً لتحسين تجربة الألعاب على أنظمة Linux (تحديداً CachyOS/Arch) لأجهزة اللاب توب ذات الموارد المتوسطة مثل **Intel HD 520**.

## 🚀 الفلسفة: نظام "المواسير المطاطية" (Rubber Pipes)
يعتمد النظام على مفهوم **Rubber Pipes**؛ حيث لا يتم فرض قيود ثابتة على العتاد، بل تتوسع وتتقلص قنوات نقل البيانات (Memory & GPU Pipes) ديناميكياً لتناسب حجم اللعبة، مما يمنع فقدان الأداء (Performance Loss) في المهام الصغيرة ويوفر القوة الكاملة في الألعاب الثقيلة.

## ✨ المميزات الرئيسية (Core Features)

* **Smart Game Detection (`is_heavy`):** تحليل تلقائي لملفات الـ EXE للكشف عن تقنيات DirectX 11/12 وتفعيل جسر **Vulkan** الصارم لتحقيق أقصى سلاسة.
* **Silent Thermal Guard:** مراقب حرارة حي يعمل في خلفية النظام، يطبق الـ **Undervolt** بصمت تام فور وصول الحرارة لـ 85°C دون الخروج من اللعبة.
* **Dynamic ZRAM:** تخصيص ذكي للذاكرة يمثل 25% من إجمالي الرام المتاح لحظياً (Rubber Pipe Scaling).
* **Central Save Arsenal:** نظام إدارة سيف (Saves) مركزي يحمي ملفاتك عبر روابط رمزية (Symlinks) مع نسخ احتياطي آلي.
* **VMAX Control Center:** واجهة رسومية (Zenity) تمنحك التحكم في (MangoHud, Gamemode, FPS Limiter) أو فتح الأداء بالكامل (Unlocked Mode) لتجارب الـ Benchmarks.

## 🛠️ المتطلبات (Requirements)
* **OS:** CachyOS / Arch Linux.
* **Language:** Rust (Cargo).
* **Dependencies:** `zenity`, `mangohud`, `gamemode`, `intel-undervolt`, `wine`.

## 📦 التثبيت والبناء (Installation & Build)

1. قم بجلب المستودع:
   ```bash
   git clone [https://github.com/your-username/split-bridge-vmax.git](https://github.com/your-username/split-bridge-vmax.git)
   cd split-bridge-vmax
