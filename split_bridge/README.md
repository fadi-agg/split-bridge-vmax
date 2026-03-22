# 🛡️ Split Engine: Zenith Edition v1.0
**The Ultimate Performance Bridge for CachyOS & Low-End Hardware**

[cite_start]Split Engine is a high-performance gaming bridge built with **Rust** and **C** integration[cite: 1, 2, 4]. [cite_start]It is designed to bypass the overhead of standard operating systems by implementing a "Rubber Pipe" data transfer philosophy, specifically optimized for hardware like the **Intel HD 520** and **i5-6200U**[cite: 3, 4].

---

## 🚀 Key Features (vMAX Core)

* [cite_start]**Dynamic Rubber Pipe**: Automatically expands memory allocation by 10% for large data requests (>16MB) to reduce page faults and micro-stutters[cite: 4].
* [cite_start]**Integrated Hybrid Core**: Native C-level memory management (`malloc` override) integrated directly into the Rust execution chain for zero-latency processing[cite: 3, 4].
* [cite_start]**Intel GPU Force-Tuning**: Automatically forces the `iris` driver and strict Vulkan modes to maximize FPS on integrated graphics[cite: 4].
* [cite_start]**Centralized Save Arsenal**: A "wormhole" system that redirects game saves to a central, safe directory, preventing data loss across different Wine prefixes[cite: 4].

## ⚙️ Customization (For Other Devices)

[cite_start]The engine is currently tuned for specific hardware[cite: 4]. [cite_start]To adapt it, modify these lines in `main.rs`[cite: 4]:
1.  [cite_start]**GPU Driver**: Change `MESA_LOADER_DRIVER_OVERRIDE` (Line 95)[cite: 4].
2.  [cite_start]**Vulkan Device ID**: Replace `8086:1916` with your GPU ID (Line 112)[cite: 4].
3.  [cite_start]**ZRAM/Pipe Size**: Adjust the external rubber pipe size (Default: `4G`)[cite: 4].

## 🛠️ Installation & Usage

1.  **Build the Engine**:
    ```bash
    split update
    ```
2.  **Launch a Game**:
    ```bash
    split gpu /path/to/game.exe
    ```

---

# 🛡️ محرك سبلت: إصدار زينيث (v1.0)
**جسر الأداء الأقصى لنظام CachyOS والأجهزة الضعيفة**

[cite_start]محرك "سبلت" هو جسر أداء متكامل تم تطويره باستخدام لغتي **Rust** و **C**[cite: 1, 2, 4]. [cite_start]صُمم خصيصاً لكسر قيود الأنظمة التقليدية عبر فلسفة "الأنابيب المطاطية" (Rubber Pipe)، وهو محسن بشكل أساسي لعتاد **Intel HD 520** ومعالجات الجيل السادس[cite: 3, 4].

---

## 🚀 المميزات الرئيسية (VMAX Core)

* [cite_start]**الأنبوب المطاطي الديناميكي**: يقوم بتوسيع تخصيص الذاكرة بنسبة 10% تلقائياً عند معالجة الملفات الكبيرة (>16 ميجا) لتقليل التقطيع (Stutters)[cite: 4].
* [cite_start]**النواة الهجينة المدمجة**: تحكم مباشر في الذاكرة على مستوى لغة C مدمج داخل محرك الرست لضمان سرعة معالجة فائقة[cite: 3, 4].
* [cite_start]**تحسين كروت إنتل**: إجبار النظام على استخدام تعريفات `iris` الحديثة وتفعيل أوضاع Vulkan الصارمة لرفع الفريمات[cite: 4].
* [cite_start]**ترسانة الحفظ المركزي**: نظام "ثقب دودي" يقوم بتوجيه ملفات حفظ الألعاب إلى مجلد مركزي واحد تلقائياً[cite: 4].

## ⚙️ التخصيص (للأجهزة الأخرى)

[cite_start]المحرك مبرمج حالياً لعتاد محدد[cite: 4]. [cite_start]لتعديله، قم بتغيير الأسطر التالية في ملف `main.rs`[cite: 4]:
1.  **تعريف الكارت**: عدل `MESA_LOADER_DRIVER_OVERRIDE`.
2.  **معرف Vulkan**: استبدل الرقم `8086:1916` بمعرف الكارت الخاص بك.
3.  **حجم الأنبوب المطاطي**: يمكنك تعديل حجم الـ ZRAM (الافتراضي: `4G`).

## 🛠️ طريقة التثبيت والاستخدام

1.  **بناء المحرك**:
    ```bash
    split update
    ```
2.  **تشغيل الألعاب**:
    ```bash
    split gpu /path/to/game.exe
    ```

---
[cite_start]**Developed with Precision using Rust & C.** [cite: 3, 4]
