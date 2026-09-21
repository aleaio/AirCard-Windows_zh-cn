//! 多语言国际化支持模块 (i18n)
//! 提供简体中文 (ZhCn) 与英语 (EnUs) 的完整双语支持。

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    ZhCn,
    EnUs,
}

impl Language {
    pub fn toggle(&mut self) {
        *self = match self {
            Language::ZhCn => Language::EnUs,
            Language::EnUs => Language::ZhCn,
        };
    }

    pub fn button_label(&self) -> &'static str {
        match self {
            Language::ZhCn => "🌐 简体中文",
            Language::EnUs => "🌐 English",
        }
    }

    // --- 顶部导航 ---
    pub fn tab_wallet(&self) -> &'static str {
        match self {
            Language::ZhCn => "钱包卡面",
            Language::EnUs => "Wallet",
        }
    }

    pub fn tab_passcode(&self) -> &'static str {
        match self {
            Language::ZhCn => "密码按键",
            Language::EnUs => "Passcode",
        }
    }

    pub fn tab_help(&self) -> &'static str {
        match self {
            Language::ZhCn => "使用说明",
            Language::EnUs => "Help",
        }
    }

    pub fn refresh(&self) -> &'static str {
        match self {
            Language::ZhCn => "刷新设备",
            Language::EnUs => "Refresh",
        }
    }

    pub fn no_device(&self) -> &'static str {
        match self {
            Language::ZhCn => "未连接设备",
            Language::EnUs => "No device",
        }
    }

    // --- 底部状态与日志 ---
    pub fn status_ready(&self) -> &'static str {
        match self {
            Language::ZhCn => "就绪。请通过 USB 连接 iPhone 并解锁屏幕。",
            Language::EnUs => "Ready. Connect iPhone via USB and unlock it.",
        }
    }

    pub fn btn_logs(&self, open: bool) -> &'static str {
        match (self, open) {
            (Language::ZhCn, true) => "日志 [开]",
            (Language::ZhCn, false) => "日志",
            (Language::EnUs, true) => "Logs [x]",
            (Language::EnUs, false) => "Logs",
        }
    }

    pub fn window_logs_title(&self) -> &'static str {
        match self {
            Language::ZhCn => "运行日志",
            Language::EnUs => "Logs",
        }
    }

    pub fn btn_copy_logs(&self) -> &'static str {
        match self {
            Language::ZhCn => "复制日志",
            Language::EnUs => "Copy Logs",
        }
    }

    pub fn btn_save_logs(&self) -> &'static str {
        match self {
            Language::ZhCn => "保存到文件...",
            Language::EnUs => "Save to File...",
        }
    }

    pub fn btn_clear(&self) -> &'static str {
        match self {
            Language::ZhCn => "清空",
            Language::EnUs => "Clear",
        }
    }

    pub fn logs_entries_count(&self, count: usize) -> String {
        match self {
            Language::ZhCn => format!("{} 条记录", count),
            Language::EnUs => format!("{} entries", count),
        }
    }

    pub fn logs_empty(&self) -> &'static str {
        match self {
            Language::ZhCn => "暂无日志事件。",
            Language::EnUs => "No events logged yet.",
        }
    }

    // --- Wallet 标签页 ---
    pub fn syslog_scanning_title(&self) -> &'static str {
        match self {
            Language::ZhCn => "正在监听系统日志...",
            Language::EnUs => "Scanning syslog...",
        }
    }

    pub fn syslog_scanning_hint(&self) -> &'static str {
        match self {
            Language::ZhCn => "请在 iPhone 上打开“钱包”App 并轻点对应卡片",
            Language::EnUs => "Open Wallet on iPhone and tap your card",
        }
    }

    pub fn btn_stop(&self) -> &'static str {
        match self {
            Language::ZhCn => "停止",
            Language::EnUs => "Stop",
        }
    }

    pub fn btn_scan(&self) -> &'static str {
        match self {
            Language::ZhCn => "扫描卡片",
            Language::EnUs => "Scan",
        }
    }

    pub fn wallet_config_title(&self) -> &'static str {
        match self {
            Language::ZhCn => "卡片配置",
            Language::EnUs => "Card Configuration",
        }
    }

    pub fn wallet_config_subtitle(&self) -> &'static str {
        match self {
            Language::ZhCn => "定位目标卡片并选择替换卡面素材",
            Language::EnUs => "Target your card and choose replacement artwork",
        }
    }

    pub fn label_target_card_hash(&self) -> &'static str {
        match self {
            Language::ZhCn => "目标卡片凭据哈希 (Card Hash)",
            Language::EnUs => "Target Card Hash",
        }
    }

    pub fn hint_card_hash(&self) -> &'static str {
        match self {
            Language::ZhCn => "Base64 格式的卡片 Hash...",
            Language::EnUs => "Base64 pass hash...",
        }
    }

    pub fn label_saved_cards(&self) -> &'static str {
        match self {
            Language::ZhCn => "已保存卡片历史",
            Language::EnUs => "Saved cards",
        }
    }

    pub fn label_select_card(&self) -> &'static str {
        match self {
            Language::ZhCn => "选择卡片...",
            Language::EnUs => "Select...",
        }
    }

    pub fn label_card_skin_artwork(&self) -> &'static str {
        match self {
            Language::ZhCn => "卡面皮肤图片",
            Language::EnUs => "Card Skin Artwork",
        }
    }

    pub fn label_skin_artwork_desc(&self) -> &'static str {
        match self {
            Language::ZhCn => "PNG, JPG, WebP - 自动缩放至 1536x969",
            Language::EnUs => "PNG, JPG, WebP - auto-scaled to 1536x969",
        }
    }

    pub fn btn_choose_image(&self) -> &'static str {
        match self {
            Language::ZhCn => "选择图片...",
            Language::EnUs => "Choose Image...",
        }
    }

    pub fn btn_export_png(&self) -> &'static str {
        match self {
            Language::ZhCn => "导出处理后 PNG",
            Language::EnUs => "Export PNG",
        }
    }

    pub fn label_write_to_iphone(&self) -> &'static str {
        match self {
            Language::ZhCn => "写入到 iPhone",
            Language::EnUs => "Write to iPhone",
        }
    }

    pub fn btn_apply_card_skin(&self) -> &'static str {
        match self {
            Language::ZhCn => "写入卡面皮肤",
            Language::EnUs => "Apply Card Skin",
        }
    }

    pub fn need_prefix(&self) -> &'static str {
        match self {
            Language::ZhCn => "需要先满足: ",
            Language::EnUs => "Need: ",
        }
    }

    pub fn need_connect_iphone(&self) -> &'static str {
        match self {
            Language::ZhCn => "连接 iPhone",
            Language::EnUs => "connect iPhone",
        }
    }

    pub fn need_enter_hash(&self) -> &'static str {
        match self {
            Language::ZhCn => "输入卡片哈希",
            Language::EnUs => "enter card hash",
        }
    }

    pub fn need_choose_image(&self) -> &'static str {
        match self {
            Language::ZhCn => "选择卡面图片",
            Language::EnUs => "choose image",
        }
    }

    pub fn preview_wallet_title(&self) -> &'static str {
        match self {
            Language::ZhCn => "卡面预览",
            Language::EnUs => "Wallet Preview",
        }
    }

    pub fn preview_wallet_subtitle(&self) -> &'static str {
        match self {
            Language::ZhCn => "1536 x 969 像素卡面画布",
            Language::EnUs => "1536 x 969 px pass canvas",
        }
    }

    pub fn preview_no_artwork(&self) -> &'static str {
        match self {
            Language::ZhCn => "未加载卡面素材",
            Language::EnUs => "No artwork loaded",
        }
    }

    pub fn preview_ready(&self) -> &'static str {
        match self {
            Language::ZhCn => "就绪",
            Language::EnUs => "Ready",
        }
    }

    pub fn preview_no_image(&self) -> &'static str {
        match self {
            Language::ZhCn => "未选图片",
            Language::EnUs => "No image",
        }
    }

    pub fn preview_post_apply_wallet_hint(&self) -> &'static str {
        match self {
            Language::ZhCn => "写入完成后，请在 iPhone 上彻底上滑关闭“钱包”App 并重新打开。",
            Language::EnUs => "After applying, force close Apple Wallet and reopen it.",
        }
    }

    // --- Passcode 标签页 ---
    pub fn passcode_theme_title(&self) -> &'static str {
        match self {
            Language::ZhCn => "密码主题配置",
            Language::EnUs => "Passcode Theme",
        }
    }

    pub fn passcode_theme_subtitle(&self) -> &'static str {
        match self {
            Language::ZhCn => "支持来自 Cowabunga 或 Nugget 的锁屏密码键盘主题",
            Language::EnUs => "Custom lockscreen keypad from Cowabunga or Nugget",
        }
    }

    pub fn label_theme_package(&self) -> &'static str {
        match self {
            Language::ZhCn => "主题安装包",
            Language::EnUs => "Theme Package",
        }
    }

    pub fn label_theme_package_desc(&self) -> &'static str {
        match self {
            Language::ZhCn => "选择包含锁屏键盘素材的 .passthm 归档文件",
            Language::EnUs => "Choose a .passthm archive containing dialer artwork",
        }
    }

    pub fn btn_choose_passthm(&self) -> &'static str {
        match self {
            Language::ZhCn => "选择 .passthm 文件...",
            Language::EnUs => "Choose .passthm...",
        }
    }

    pub fn theme_assets_summary(&self, fname: &str, count: usize) -> String {
        match self {
            Language::ZhCn => format!("{} - 包含 {} 个按键素材", fname, count),
            Language::EnUs => format!("{} - {} assets", fname, count),
        }
    }

    pub fn label_target_ios_cache(&self) -> &'static str {
        match self {
            Language::ZhCn => "目标 iOS 缓存版本",
            Language::EnUs => "Target iOS Cache",
        }
    }

    pub fn label_target_ios_cache_desc(&self) -> &'static str {
        match self {
            Language::ZhCn => "根据已连接设备的 iOS 版本选择键盘缓存格式",
            Language::EnUs => "Select cache format based on connected iOS version",
        }
    }

    pub fn telephony_auto(&self) -> &'static str {
        match self {
            Language::ZhCn => "自动检测 (TelephonyUI-10)",
            Language::EnUs => "Auto (TelephonyUI-10)",
        }
    }

    pub fn telephony_10(&self) -> &'static str {
        match self {
            Language::ZhCn => "TelephonyUI-10 (iOS 18+)",
            Language::EnUs => "TelephonyUI-10 (iOS 18+)",
        }
    }

    pub fn telephony_9(&self) -> &'static str {
        match self {
            Language::ZhCn => "TelephonyUI-9 (iOS 16-17)",
            Language::EnUs => "TelephonyUI-9 (iOS 16-17)",
        }
    }

    pub fn telephony_8(&self) -> &'static str {
        match self {
            Language::ZhCn => "TelephonyUI-8 (旧版本)",
            Language::EnUs => "TelephonyUI-8 (Legacy)",
        }
    }

    pub fn label_keypad_language(&self) -> &'static str {
        match self {
            Language::ZhCn => "键盘字母语言",
            Language::EnUs => "Keypad Language",
        }
    }

    pub fn label_keypad_language_desc(&self) -> &'static str {
        match self {
            Language::ZhCn => "键盘数字下方的字母排版（英文、俄文、乌克兰文、日文或通用）",
            Language::EnUs => "Subtext alphabet layout (English, Russian, Ukrainian, Japanese, or Universal)",
        }
    }

    pub fn keypad_lang_english(&self) -> &'static str {
        match self {
            Language::ZhCn => "英语 (English)",
            Language::EnUs => "English",
        }
    }

    pub fn keypad_lang_russian(&self) -> &'static str {
        match self {
            Language::ZhCn => "俄语 (Russian)",
            Language::EnUs => "Russian",
        }
    }

    pub fn keypad_lang_ukrainian(&self) -> &'static str {
        match self {
            Language::ZhCn => "乌克兰语 (Ukrainian)",
            Language::EnUs => "Ukrainian",
        }
    }

    pub fn keypad_lang_japanese(&self) -> &'static str {
        match self {
            Language::ZhCn => "日语 (Japanese)",
            Language::EnUs => "Japanese",
        }
    }

    pub fn keypad_lang_universal(&self) -> &'static str {
        match self {
            Language::ZhCn => "全语言通用 (Universal)",
            Language::EnUs => "All Languages (Universal)",
        }
    }

    pub fn checkbox_bold_text(&self) -> &'static str {
        match self {
            Language::ZhCn => "粗体文本 (iOS 辅助功能)",
            Language::EnUs => "Bold Text (iOS Accessibility)",
        }
    }

    pub fn bold_text_desc(&self) -> &'static str {
        match self {
            Language::ZhCn => "为在 iPhone“设置 -> 辅助功能 / 显示与亮度”中开启了“粗体文本”的设备生成 *-bold.png",
            Language::EnUs => "Generates *-bold.png for devices with Bold Text turned ON in iPhone Settings -> Display",
        }
    }

    pub fn btn_apply_passcode_theme(&self) -> &'static str {
        match self {
            Language::ZhCn => "写入密码主题",
            Language::EnUs => "Apply Passcode Theme",
        }
    }

    pub fn need_select_theme(&self) -> &'static str {
        match self {
            Language::ZhCn => "选择主题文件",
            Language::EnUs => "select theme",
        }
    }

    pub fn preview_keypad_title(&self) -> &'static str {
        match self {
            Language::ZhCn => "键盘预览",
            Language::EnUs => "Keypad Preview",
        }
    }

    pub fn preview_keypad_subtitle(&self) -> &'static str {
        match self {
            Language::ZhCn => "锁屏密码按键素材",
            Language::EnUs => "Dialer button artwork",
        }
    }

    pub fn preview_no_theme(&self) -> &'static str {
        match self {
            Language::ZhCn => "未加载主题",
            Language::EnUs => "No theme loaded",
        }
    }

    pub fn preview_keypad_layout_label(&self) -> &'static str {
        match self {
            Language::ZhCn => "3x4 键盘布局",
            Language::EnUs => "3x4 Keypad",
        }
    }

    pub fn preview_post_apply_keypad_hint(&self) -> &'static str {
        match self {
            Language::ZhCn => "写入完成后，锁定 iPhone 屏幕即可看到全新按键风格。",
            Language::EnUs => "After applying, lock your iPhone to see the new keypad.",
        }
    }

    // --- Help 标签页 ---
    pub fn help_setup_title(&self) -> &'static str {
        match self {
            Language::ZhCn => "准备工作与卡片哈希获取指南",
            Language::EnUs => "Setup & Card Hash Guide",
        }
    }

    pub fn help_setup_subtitle(&self) -> &'static str {
        match self {
            Language::ZhCn => "连接设备与捕获卡片哈希所需的一切指引",
            Language::EnUs => "Everything you need to connect and capture your card",
        }
    }

    pub fn help_prerequisites_title(&self) -> &'static str {
        match self {
            Language::ZhCn => "必要前置条件",
            Language::EnUs => "Prerequisites",
        }
    }

    pub fn help_prereq_1(&self) -> &'static str {
        match self {
            Language::ZhCn => "- 已安装 64 位 iTunes 或 Apple Mobile Device Support（苹果移动设备支持服务）",
            Language::EnUs => "- 64-bit iTunes or Apple Mobile Device Support installed",
        }
    }

    pub fn help_prereq_2(&self) -> &'static str {
        match self {
            Language::ZhCn => "- 使用 USB 数据线（USB-C 或 Lightning）将 iPhone 连接到电脑",
            Language::EnUs => "- Connect iPhone via USB-C or Lightning cable",
        }
    }

    pub fn help_prereq_3(&self) -> &'static str {
        match self {
            Language::ZhCn => "- 解锁 iPhone 屏幕并在弹出提示时轻点“信任此电脑”",
            Language::EnUs => "- Unlock iPhone and tap \"Trust this Computer\"",
        }
    }

    pub fn help_find_hash_title(&self) -> &'static str {
        match self {
            Language::ZhCn => "如何获取卡片哈希 (Card Hash)",
            Language::EnUs => "Finding Your Card Hash",
        }
    }

    pub fn help_hash_step_1(&self) -> &'static str {
        match self {
            Language::ZhCn => "1. 在“钱包卡面”标签页点击“扫描卡片”按钮",
            Language::EnUs => "1. Click \"Scan\" in the Wallet tab",
        }
    }

    pub fn help_hash_step_2(&self) -> &'static str {
        match self {
            Language::ZhCn => "2. 在 iPhone 上打开官方“钱包 (Wallet)”App",
            Language::EnUs => "2. Open Apple Wallet on your iPhone",
        }
    }

    pub fn help_hash_step_3(&self) -> &'static str {
        match self {
            Language::ZhCn => "3. 点击进入你想要自定义卡面的卡片",
            Language::EnUs => "3. Tap the card you want to customize",
        }
    }

    pub fn help_hash_step_4(&self) -> &'static str {
        match self {
            Language::ZhCn => "4. AirCard 会从系统日志中自动捕获对应的卡片凭据哈希",
            Language::EnUs => "4. AirCard captures the pass hash automatically",
        }
    }

    pub fn help_hash_step_5(&self) -> &'static str {
        match self {
            Language::ZhCn => "5. 捕获成功后点击“停止”，哈希值会自动保存并填入",
            Language::EnUs => "5. Click \"Stop\" once detected",
        }
    }

    pub fn help_theme_title(&self) -> &'static str {
        match self {
            Language::ZhCn => "激活生效与主题说明",
            Language::EnUs => "Activation & Theme Guide",
        }
    }

    pub fn help_theme_subtitle(&self) -> &'static str {
        match self {
            Language::ZhCn => "卡面皮肤与锁屏键盘素材包的生效步骤",
            Language::EnUs => "Applying skins and dialer keypad packages",
        }
    }

    pub fn help_activate_wallet_title(&self) -> &'static str {
        match self {
            Language::ZhCn => "让钱包卡面生效的步骤",
            Language::EnUs => "Activating Apple Wallet Skin",
        }
    }

    pub fn help_wallet_step_1(&self) -> &'static str {
        match self {
            Language::ZhCn => "1. 点击“写入卡面皮肤”，等待操作完成",
            Language::EnUs => "1. Click \"Apply Card Skin\" and wait for completion",
        }
    }

    pub fn help_wallet_step_2(&self) -> &'static str {
        match self {
            Language::ZhCn => "2. 在 iPhone 底部向上轻扫进入后台多任务界面",
            Language::EnUs => "2. Open App Switcher on iPhone (swipe up from bottom)",
        }
    }

    pub fn help_wallet_step_3(&self) -> &'static str {
        match self {
            Language::ZhCn => "3. 向上滑动卡片彻底关闭“钱包”App",
            Language::EnUs => "3. Force close Apple Wallet by swiping up on it",
        }
    }

    pub fn help_wallet_step_4(&self) -> &'static str {
        match self {
            Language::ZhCn => "4. 重新打开钱包 App —— 全新卡面即刻显示！",
            Language::EnUs => "4. Reopen Wallet - your new skin appears!",
        }
    }

    pub fn help_passthm_title(&self) -> &'static str {
        match self {
            Language::ZhCn => "锁屏密码主题 (.passthm) 说明",
            Language::EnUs => "Passcode Themes (.passthm)",
        }
    }

    pub fn help_passthm_desc_1(&self) -> &'static str {
        match self {
            Language::ZhCn => "- 完全兼容 Cowabunga 与 Nugget 制作的主题安装包",
            Language::EnUs => "- Compatible with Cowabunga & Nugget theme packages",
        }
    }

    pub fn help_passthm_desc_2(&self) -> &'static str {
        match self {
            Language::ZhCn => "- iOS 18+ 系统：请选择“自动检测 (TelephonyUI-10)”",
            Language::EnUs => "- iOS 18+: Select \"Auto (TelephonyUI-10)\"",
        }
    }

    pub fn help_passthm_desc_3(&self) -> &'static str {
        match self {
            Language::ZhCn => "- iOS 16-17 系统：请选择“TelephonyUI-9”",
            Language::EnUs => "- iOS 16-17: Select \"TelephonyUI-9\"",
        }
    }

    pub fn help_passthm_desc_4(&self) -> &'static str {
        match self {
            Language::ZhCn => "- 写入完成后锁定屏幕并点亮，即可查看全新按键效果",
            Language::EnUs => "- Lock screen to verify your updated keypad artwork",
        }
    }

    // --- 驱动支持 ---
    pub fn apple_support_ready(&self) -> &'static str {
        match self {
            Language::ZhCn => "已加载并正常工作",
            Language::EnUs => "Loaded and operational",
        }
    }

    pub fn apple_support_missing(&self) -> &'static str {
        match self {
            Language::ZhCn => "未检测到组件（需要安装 64 位 iTunes）",
            Language::EnUs => "Not found (iTunes required)",
        }
    }

    pub fn status_no_devices(&self) -> &'static str {
        match self {
            Language::ZhCn => "未通过 USB 连接任何设备。",
            Language::EnUs => "No devices connected via USB.",
        }
    }

    pub fn status_found_devices(&self, count: usize) -> String {
        match self {
            Language::ZhCn => format!("已检测到 {} 台连接的设备", count),
            Language::EnUs => format!("Found {} connected device(s)", count),
        }
    }

    pub fn status_device_scan_error(&self, err: &str) -> String {
        match self {
            Language::ZhCn => format!("无法获取设备列表: {}", err),
            Language::EnUs => format!("Could not enumerate devices: {}", err),
        }
    }

    pub fn status_prepared_skin(&self, name: &str, w: u32, h: u32, kb: f32) -> String {
        match self {
            Language::ZhCn => format!("已准备素材 {} ({}x{} -> 1536x969 PNG, {:.1} KB)", name, w, h, kb),
            Language::EnUs => format!("Prepared {} ({}x{} -> 1536x969 PNG, {:.1} KB)", name, w, h, kb),
        }
    }

    pub fn status_prepare_image_failed(&self, err: &str) -> String {
        match self {
            Language::ZhCn => format!("无法处理图片: {}", err),
            Language::EnUs => format!("Could not prepare image: {}", err),
        }
    }

    pub fn status_saved_png(&self, path: &str) -> String {
        match self {
            Language::ZhCn => format!("已保存处理后的 PNG: {}", path),
            Language::EnUs => format!("Saved prepared PNG: {}", path),
        }
    }

    pub fn status_save_png_failed(&self, err: &str) -> String {
        match self {
            Language::ZhCn => format!("无法保存 PNG: {}", err),
            Language::EnUs => format!("Could not save PNG: {}", err),
        }
    }

    pub fn status_syslog_stopped(&self) -> &'static str {
        match self {
            Language::ZhCn => "系统日志监听已停止。",
            Language::EnUs => "Syslog scanning stopped.",
        }
    }

    pub fn status_syslog_scanning(&self) -> &'static str {
        match self {
            Language::ZhCn => "正在监听日志... 请在 iPhone 打开“钱包”或轻触卡片。",
            Language::EnUs => "Scanning syslog... Open Wallet or tap your card on iPhone.",
        }
    }

    pub fn status_need_device(&self) -> &'static str {
        match self {
            Language::ZhCn => "请选择已连接的 iPhone。",
            Language::EnUs => "Please select a connected iPhone.",
        }
    }

    pub fn status_need_hash(&self) -> &'static str {
        match self {
            Language::ZhCn => "请输入或扫描目标卡片的凭据哈希。",
            Language::EnUs => "Please enter or scan a target card hash.",
        }
    }

    pub fn status_need_image(&self) -> &'static str {
        match self {
            Language::ZhCn => "请先选择一张卡面图片。",
            Language::EnUs => "Please choose a card skin image first.",
        }
    }

    pub fn status_writing_card(&self) -> &'static str {
        match self {
            Language::ZhCn => "正在将卡面皮肤写入 iPhone...",
            Language::EnUs => "Writing card skin to iPhone...",
        }
    }

    pub fn progress_init_flash(&self) -> &'static str {
        match self {
            Language::ZhCn => "正在初始化写入...",
            Language::EnUs => "Initiating card flash...",
        }
    }

    pub fn flash_card_success(&self) -> &'static str {
        match self {
            Language::ZhCn => "卡面皮肤写入成功！请在 iPhone 后台彻底关闭“钱包”App 并重新打开。",
            Language::EnUs => "Card skin successfully flashed! Force quit Wallet on iPhone and reopen it.",
        }
    }

    pub fn flash_theme_success(&self) -> &'static str {
        match self {
            Language::ZhCn => "密码主题写入成功！请锁定 iPhone 屏幕以查看新按键风格。",
            Language::EnUs => "Passcode theme applied successfully! Lock your iPhone to see changes.",
        }
    }

    pub fn status_need_theme(&self) -> &'static str {
        match self {
            Language::ZhCn => "请先选择 .passthm 主题文件。",
            Language::EnUs => "Please select a .passthm theme file first.",
        }
    }

    pub fn status_writing_theme(&self) -> &'static str {
        match self {
            Language::ZhCn => "正在将密码主题写入 iPhone...",
            Language::EnUs => "Writing passcode theme to iPhone...",
        }
    }
}
