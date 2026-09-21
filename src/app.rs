use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, channel};
use std::thread;

use eframe::egui;

use crate::apple;
use crate::device::{DeviceInfo, list_connected_devices};
use crate::flasher::{flash_passcode_theme, flash_wallet_skin};
use crate::i18n::Language;
use crate::image_skin::PreparedSkin;
use crate::passthm::{PasscodeTheme, parse_passthm_file};
use crate::scanner::{SavedCard, load_saved_cards, scan_syslog_for_cards};

#[derive(PartialEq, Eq)]
enum AppTab {
    Wallet,
    Passcode,
    Help,
}

enum BackgroundTaskMessage {
    Progress { step: usize, total: usize, message: String },
    Log(String),
    CardFound { hash: String, name: String },
    Done(Result<String, String>),
}

#[cfg(windows)]
fn current_timestamp() -> String {
    #[repr(C)]
    struct SystemTime {
        w_year: u16,
        w_month: u16,
        w_day_of_week: u16,
        w_day: u16,
        w_hour: u16,
        w_minute: u16,
        w_second: u16,
        w_milliseconds: u16,
    }
    unsafe extern "system" {
        fn GetLocalTime(lpSystemTime: *mut SystemTime);
    }
    let mut st = std::mem::MaybeUninit::<SystemTime>::uninit();
    unsafe {
        GetLocalTime(st.as_mut_ptr());
        let st = st.assume_init();
        format!(
            "{:02}:{:02}:{:02}.{:03}",
            st.w_hour, st.w_minute, st.w_second, st.w_milliseconds
        )
    }
}

#[cfg(not(windows))]
fn current_timestamp() -> String {
    "00:00:00.000".to_string()
}

pub struct AirCardApp {
    pub lang: Language,
    current_tab: AppTab,
    apple_status: String,
    apple_ready: bool,

    // Device management
    devices: Vec<DeviceInfo>,
    selected_udid: Option<String>,

    // Wallet tab
    card_hash: String,
    saved_cards: Vec<SavedCard>,
    source_path: Option<PathBuf>,
    skin: Option<PreparedSkin>,
    skin_texture: Option<egui::TextureHandle>,
    scanning_syslog: bool,
    scan_stop_flag: Option<Arc<AtomicBool>>,

    // Passcode tab
    theme_path: Option<PathBuf>,
    loaded_theme: Option<PasscodeTheme>,
    forced_telephony_ver: String,
    keypad_language: String,
    passcode_bold: bool,
    keypad_textures: Vec<(String, egui::TextureHandle)>,

    // Worker thread & progress
    is_busy: bool,
    progress_step: usize,
    progress_total: usize,
    progress_msg: String,
    status_msg: String,
    task_rx: Option<Receiver<BackgroundTaskMessage>>,
    logs: Vec<String>,
    show_logs_window: bool,
}

impl AirCardApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        setup_custom_theme(&cc.egui_ctx);

        let (apple_ready, apple_status) = match apple::verify_support() {
            Ok(msg) => (true, msg),
            Err(err) => (false, err.to_string()),
        };

        let lang = Language::ZhCn;
        let mut app = Self {
            lang,
            current_tab: AppTab::Wallet,
            apple_status,
            apple_ready,

            devices: Vec::new(),
            selected_udid: None,

            card_hash: String::new(),
            saved_cards: load_saved_cards(),
            source_path: None,
            skin: None,
            skin_texture: None,
            scanning_syslog: false,
            scan_stop_flag: None,

            theme_path: None,
            loaded_theme: None,
            forced_telephony_ver: "Auto (TelephonyUI-10)".to_string(),
            keypad_language: "English".to_string(),
            passcode_bold: false,
            keypad_textures: Vec::new(),

            is_busy: false,
            progress_step: 0,
            progress_total: 0,
            progress_msg: String::new(),
            status_msg: lang.status_ready().to_string(),
            task_rx: None,
            logs: Vec::new(),
            show_logs_window: false,
        };

        app.add_log("AirCard Windows v1.2.1 initialized");
        app.add_log(format!("Apple Support Runtime: {}", if app.apple_ready { "Loaded and operational" } else { "Not found (iTunes required)" }));
        app.add_log(format!("Loaded {} saved card(s) from database", app.saved_cards.len()));

        if app.apple_ready {
            app.refresh_devices();
        }

        app
    }

    fn add_log(&mut self, text: impl AsRef<str>) {
        let ts = current_timestamp();
        self.logs.push(format!("[{}] {}", ts, text.as_ref()));
        if self.logs.len() > 1000 {
            self.logs.remove(0);
        }
    }

    fn refresh_devices(&mut self) {
        self.add_log("Scanning for connected iOS devices via usbmuxd...");
        match list_connected_devices() {
            Ok(devs) => {
                self.devices = devs;
                if self.selected_udid.is_none() && !self.devices.is_empty() {
                    self.selected_udid = Some(self.devices[0].udid.clone());
                }
                if self.devices.is_empty() {
                    self.add_log("No devices detected. Please plug in your iPhone and tap 'Trust this Computer'.");
                    self.status_msg = self.lang.status_no_devices().to_string();
                } else {
                    let dev_logs: Vec<String> = self.devices.iter().enumerate().map(|(i, d)| {
                        format!("Device #{}: {} - UDID: {}", i + 1, d, d.udid)
                    }).collect();
                    for line in dev_logs {
                        self.add_log(line);
                    }
                    self.status_msg = self.lang.status_found_devices(self.devices.len());
                }
            }
            Err(err) => {
                self.add_log(format!("Device scan error: {}", err));
                self.status_msg = self.lang.status_device_scan_error(&err.to_string());
            }
        }
    }

    fn select_skin(&mut self, ctx: &egui::Context) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
            .pick_file()
        else {
            return;
        };

        self.add_log(format!("Opening skin image: {}", path.display()));
        match PreparedSkin::from_path(&path) {
            Ok(skin) => {
                self.add_log(format!(
                    "Skin processed: source {}x{} resampled to 1536x969 PNG ({:.1} KB)",
                    skin.source_width,
                    skin.source_height,
                    skin.png.len() as f32 / 1024.0,
                ));
                self.skin_texture = Some(ctx.load_texture(
                    "card-skin-preview",
                    skin.preview.clone(),
                    egui::TextureOptions::LINEAR,
                ));
                let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("image");
                self.status_msg = self.lang.status_prepared_skin(
                    fname,
                    skin.source_width,
                    skin.source_height,
                    skin.png.len() as f32 / 1024.0,
                );
                self.source_path = Some(path);
                self.skin = Some(skin);
            }
            Err(error) => {
                self.add_log(format!("Image preparation failed: {error:#}"));
                self.status_msg = self.lang.status_prepare_image_failed(&format!("{error:#}"));
            }
        }
    }

    fn save_prepared_png(&mut self) {
        let Some(skin) = &self.skin else {
            return;
        };
        let Some(path) = rfd::FileDialog::new()
            .set_file_name("aircard-skin.png")
            .save_file()
        else {
            return;
        };
        match std::fs::write(&path, &skin.png) {
            Ok(()) => {
                self.add_log(format!("Exported prepared card skin PNG: {}", path.display()));
                self.status_msg = self.lang.status_saved_png(&path.display().to_string());
            }
            Err(err) => {
                self.add_log(format!("Failed to save PNG: {err}"));
                self.status_msg = self.lang.status_save_png_failed(&err.to_string());
            }
        }
    }

    fn toggle_syslog_scan(&mut self) {
        if self.scanning_syslog {
            if let Some(flag) = self.scan_stop_flag.take() {
                flag.store(true, Ordering::Relaxed);
            }
            self.scanning_syslog = false;
            self.add_log("Syslog scanning stopped by user.");
            self.status_msg = self.lang.status_syslog_stopped().to_string();
            return;
        }

        let stop_flag = Arc::new(AtomicBool::new(false));
        self.scan_stop_flag = Some(Arc::clone(&stop_flag));
        self.scanning_syslog = true;
        self.add_log("Initiating syslog monitor session...");
        self.status_msg = self.lang.status_syslog_scanning().to_string();

        let (tx, rx) = channel();
        self.task_rx = Some(rx);
        let udid = self.selected_udid.clone();

        thread::spawn(move || {
            let tx_card = tx.clone();
            let tx_log = tx.clone();
            let res = scan_syslog_for_cards(
                udid.as_deref(),
                stop_flag,
                move |hash, name| {
                    let _ = tx_card.send(BackgroundTaskMessage::CardFound { hash, name });
                },
                move |msg| {
                    let _ = tx_log.send(BackgroundTaskMessage::Log(msg));
                },
            );
            match res {
                Ok(()) => {
                    let _ = tx.send(BackgroundTaskMessage::Done(Ok("Syslog scan finished".into())));
                }
                Err(e) => {
                    let _ = tx.send(BackgroundTaskMessage::Done(Err(e.to_string())));
                }
            }
        });
    }

    fn flash_card(&mut self) {
        let Some(udid) = self.selected_udid.clone() else {
            self.add_log("Flash failed: No connected iPhone selected.");
            self.status_msg = self.lang.status_need_device().to_string();
            return;
        };
        let hash = self.card_hash.trim().to_string();
        if hash.is_empty() {
            self.add_log("Flash failed: Target card hash is empty.");
            self.status_msg = self.lang.status_need_hash().to_string();
            return;
        }
        let Some(skin) = self.skin.as_ref() else {
            self.add_log("Flash failed: No skin image prepared.");
            self.status_msg = self.lang.status_need_image().to_string();
            return;
        };

        let png_bytes = skin.png.clone();
        let pdf_bytes = skin.pdf.clone();
        if let Some(ref flag) = self.scan_stop_flag {
            flag.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        self.scanning_syslog = false;
        self.is_busy = true;
        self.progress_step = 0;
        self.progress_total = 3;
        self.progress_msg = self.lang.progress_init_flash().to_string();
        self.status_msg = self.lang.status_writing_card().to_string();
        self.add_log(format!("Starting card skin flash for hash: {} (UDID: {})", hash, udid));

        let (tx, rx) = channel();
        self.task_rx = Some(rx);

        let success_msg = self.lang.flash_card_success().to_string();
        thread::spawn(move || {
            let tx_progress = tx.clone();
            let tx_log = tx.clone();
            let res = flash_wallet_skin(
                &udid,
                &hash,
                &png_bytes,
                &pdf_bytes,
                move |step, total, msg| {
                    let _ = tx_progress.send(BackgroundTaskMessage::Progress {
                        step,
                        total,
                        message: msg.to_string(),
                    });
                },
                move |msg| {
                    let _ = tx_log.send(BackgroundTaskMessage::Log(msg.to_string()));
                },
            );

            match res {
                Ok(()) => {
                    let _ = tx.send(BackgroundTaskMessage::Done(Ok(success_msg)));
                }
                Err(e) => {
                    let _ = tx.send(BackgroundTaskMessage::Done(Err(format!("{:#}", e))));
                }
            }
        });
    }

    fn select_theme_file(&mut self, ctx: &egui::Context) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Passcode Theme", &["passthm", "passtheme", "zip"])
            .pick_file()
        else {
            return;
        };

        self.load_theme_from_path(ctx, &path);
    }

    fn load_theme_from_path(&mut self, ctx: &egui::Context, path: &Path) {
        self.add_log(format!("Opening passcode theme package: {}", path.display()));
        let target_ver = match self.forced_telephony_ver.as_str() {
            "TelephonyUI-10" => Some("TelephonyUI-10"),
            "TelephonyUI-9" => Some("TelephonyUI-9"),
            "TelephonyUI-8" => Some("TelephonyUI-8"),
            _ => Some("TelephonyUI-10"),
        };

        match parse_passthm_file(path, target_ver, &self.keypad_language, self.passcode_bold) {
            Ok(theme) => {
                self.keypad_textures.clear();
                for (digit, bytes) in &theme.key_previews {
                    if let Ok(img) = image::load_from_memory(bytes) {
                        let rgba = img.to_rgba8();
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(
                            [rgba.width() as usize, rgba.height() as usize],
                            &rgba,
                        );
                        let tex = ctx.load_texture(
                            format!("keypad-{}", digit),
                            color_image,
                            egui::TextureOptions::LINEAR,
                        );
                        self.keypad_textures.push((digit.clone(), tex));
                    }
                }
                self.keypad_textures.sort_by(|a, b| a.0.cmp(&b.0));

                self.add_log(format!(
                    "Passcode theme loaded: '{}' (telephony: {}, lang: {}, bold: {}, {} assets)",
                    theme.name,
                    theme.detected_version,
                    self.keypad_language,
                    self.passcode_bold,
                    theme.items.len()
                ));
                self.status_msg = format!(
                    "Loaded '{}' with {} assets (target: {}, lang: {}, bold: {})",
                    theme.name,
                    theme.items.len(),
                    theme.detected_version,
                    self.keypad_language,
                    if self.passcode_bold { "ON" } else { "OFF" }
                );
                self.theme_path = Some(path.to_path_buf());
                self.loaded_theme = Some(theme);
            }
            Err(err) => {
                self.add_log(format!("Failed to parse theme: {err:#}"));
                self.status_msg = format!("Failed to parse theme: {err:#}");
            }
        }
    }

    fn flash_theme(&mut self) {
        let Some(udid) = self.selected_udid.clone() else {
            self.add_log("Theme flash failed: No connected iPhone selected.");
            self.status_msg = self.lang.status_need_device().to_string();
            return;
        };
        let Some(theme) = self.loaded_theme.as_ref() else {
            self.add_log("Theme flash failed: No .passthm theme loaded.");
            self.status_msg = self.lang.status_need_theme().to_string();
            return;
        };

        let items = theme.items.clone();
        self.is_busy = true;
        self.progress_step = 0;
        self.progress_total = items.len();
        self.progress_msg = self.lang.progress_init_flash().to_string();
        self.status_msg = self.lang.status_writing_theme().to_string();
        self.add_log(format!("Flashing passcode theme '{}' ({} button assets) to device {}", theme.name, items.len(), udid));

        let (tx, rx) = channel();
        self.task_rx = Some(rx);

        let success_msg = self.lang.flash_theme_success().to_string();
        thread::spawn(move || {
            let tx_progress = tx.clone();
            let tx_log = tx.clone();
            let res = flash_passcode_theme(
                &udid,
                &items,
                move |step, total, msg| {
                    let _ = tx_progress.send(BackgroundTaskMessage::Progress {
                        step,
                        total,
                        message: msg.to_string(),
                    });
                },
                move |msg| {
                    let _ = tx_log.send(BackgroundTaskMessage::Log(msg.to_string()));
                },
            );

            match res {
                Ok(()) => {
                    let _ = tx.send(BackgroundTaskMessage::Done(Ok(success_msg)));
                }
                Err(e) => {
                    let _ = tx.send(BackgroundTaskMessage::Done(Err(format!("{:#}", e))));
                }
            }
        });
    }

    fn handle_messages(&mut self) {
        let mut messages = Vec::new();
        if let Some(ref rx) = self.task_rx {
            while let Ok(msg) = rx.try_recv() {
                messages.push(msg);
            }
        }

        let mut finished = false;
        for msg in messages {
            match msg {
                BackgroundTaskMessage::Progress { step, total, message } => {
                    self.progress_step = step;
                    self.progress_total = total;
                    self.progress_msg = message.clone();
                    let msg_str = format!("[{}/{}] {}", step, total, message);
                    self.add_log(&msg_str);
                    self.status_msg = msg_str;
                }
                BackgroundTaskMessage::Log(log_line) => {
                    self.add_log(log_line);
                }
                BackgroundTaskMessage::CardFound { hash, name } => {
                    self.card_hash = hash.clone();
                    self.saved_cards = load_saved_cards();
                    let msg_str = format!("Card captured: {} ({})", name, hash);
                    self.add_log(&msg_str);
                    self.status_msg = msg_str;
                }
                BackgroundTaskMessage::Done(res) => {
                    self.is_busy = false;
                    self.scanning_syslog = false;
                    finished = true;
                    match res {
                        Ok(ok_msg) => {
                            self.add_log(format!("Operation completed: {}", ok_msg));
                            self.status_msg = ok_msg;
                        }
                        Err(err_msg) => {
                            self.add_log(format!("Operation failed: {}", err_msg));
                            self.status_msg = format!("Error: {}", err_msg);
                        }
                    }
                }
            }
        }
        if finished {
            self.task_rx = None;
        }
    }
}

pub mod md3 {
    use eframe::egui::Color32;

    // M3 Dark scheme
    pub const SURFACE: Color32 = Color32::from_rgb(18, 18, 20);
    pub const SURFACE_CONTAINER: Color32 = Color32::from_rgb(33, 31, 36);
    pub const SURFACE_CONTAINER_HIGH: Color32 = Color32::from_rgb(43, 41, 48);
    pub const SURFACE_CONTAINER_HIGHEST: Color32 = Color32::from_rgb(54, 52, 59);
    pub const ON_SURFACE: Color32 = Color32::from_rgb(230, 225, 229);
    pub const ON_SURFACE_VARIANT: Color32 = Color32::from_rgb(196, 199, 197);
    pub const OUTLINE: Color32 = Color32::from_rgb(147, 143, 153);
    pub const OUTLINE_VARIANT: Color32 = Color32::from_rgb(73, 69, 79);

    // Primary
    pub const PRIMARY: Color32 = Color32::from_rgb(208, 188, 255);
    pub const ON_PRIMARY: Color32 = Color32::from_rgb(56, 30, 114);
    pub const PRIMARY_CONTAINER: Color32 = Color32::from_rgb(79, 55, 139);
    pub const ON_PRIMARY_CONTAINER: Color32 = Color32::from_rgb(234, 221, 255);

    // Secondary
    pub const SECONDARY_CONTAINER: Color32 = Color32::from_rgb(74, 68, 88);
    pub const ON_SECONDARY_CONTAINER: Color32 = Color32::from_rgb(232, 222, 248);

    // Tertiary
    pub const TERTIARY_CONTAINER: Color32 = Color32::from_rgb(99, 59, 72);
    pub const ON_TERTIARY_CONTAINER: Color32 = Color32::from_rgb(255, 216, 228);

    // Error
    pub const ERROR: Color32 = Color32::from_rgb(242, 184, 181);
    pub const ERROR_CONTAINER: Color32 = Color32::from_rgb(140, 29, 24);

    // Extra
    pub const SUCCESS: Color32 = Color32::from_rgb(120, 220, 120);
}

fn draw_status_dot(ui: &mut egui::Ui, color: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
    ui.painter().circle_filled(rect.center(), 4.0, color);
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // 静态内嵌开源霞鹜新晰黑 (LXGW Neo XiHei) 中文字体
    fonts.font_data.insert(
        "lxgw_neo_xihei".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
            "../assets/fonts/LXGWNeoXiHei.ttf"
        ))),
    );

    // 将中文字体加入比例字体与等宽字体的候选链中
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .push("lxgw_neo_xihei".to_owned());

    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("lxgw_neo_xihei".to_owned());

    ctx.set_fonts(fonts);
}

fn setup_custom_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    visuals.panel_fill = md3::SURFACE;
    visuals.window_fill = md3::SURFACE;
    visuals.extreme_bg_color = md3::SURFACE_CONTAINER;
    visuals.faint_bg_color = md3::SURFACE_CONTAINER;

    visuals.window_corner_radius = 16.into();
    visuals.menu_corner_radius = 12.into();

    visuals.widgets.noninteractive.corner_radius = 12.into();
    visuals.widgets.noninteractive.bg_fill = md3::SURFACE_CONTAINER;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::NONE;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0_f32, md3::ON_SURFACE);

    visuals.widgets.inactive.bg_fill = md3::SURFACE_CONTAINER_HIGH;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, md3::ON_SURFACE_VARIANT);
    visuals.widgets.inactive.corner_radius = 12.into();

    visuals.widgets.hovered.bg_fill = md3::SURFACE_CONTAINER_HIGHEST;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0_f32, md3::ON_SURFACE);
    visuals.widgets.hovered.corner_radius = 12.into();

    visuals.widgets.active.bg_fill = md3::PRIMARY_CONTAINER;
    visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0_f32, md3::ON_PRIMARY_CONTAINER);
    visuals.widgets.active.corner_radius = 12.into();

    visuals.widgets.open.bg_fill = md3::SURFACE_CONTAINER_HIGHEST;
    visuals.widgets.open.corner_radius = 12.into();
    visuals.widgets.open.bg_stroke = egui::Stroke::NONE;

    visuals.selection.bg_fill = md3::PRIMARY_CONTAINER;
    visuals.selection.stroke = egui::Stroke::new(1.0_f32, md3::PRIMARY);

    ctx.set_visuals(visuals);

    ctx.style_mut(|style| {
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(16.0, 8.0);
    });
}

fn m3_card<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::new()
        .fill(md3::SURFACE_CONTAINER)
        .corner_radius(16)
        .inner_margin(egui::Margin::same(20))
        .show(ui, |ui| ui.vertical(add_contents).inner)
        .inner
}

fn m3_button_filled(ui: &mut egui::Ui, label: &str) -> bool {
    let btn = egui::Button::new(
        egui::RichText::new(label).size(13.0).color(md3::ON_PRIMARY),
    )
    .fill(md3::PRIMARY)
    .corner_radius(20)
    .stroke(egui::Stroke::NONE);
    ui.add(btn).clicked()
}

fn m3_button_tonal(ui: &mut egui::Ui, label: &str) -> bool {
    let btn = egui::Button::new(
        egui::RichText::new(label).size(13.0).color(md3::ON_SECONDARY_CONTAINER),
    )
    .fill(md3::SECONDARY_CONTAINER)
    .corner_radius(20)
    .stroke(egui::Stroke::NONE);
    ui.add(btn).clicked()
}

fn m3_button_outlined(ui: &mut egui::Ui, label: &str) -> bool {
    let btn = egui::Button::new(
        egui::RichText::new(label).size(13.0).color(md3::PRIMARY),
    )
    .fill(egui::Color32::TRANSPARENT)
    .corner_radius(20)
    .stroke(egui::Stroke::new(1.0_f32, md3::OUTLINE));
    ui.add(btn).clicked()
}

fn m3_tab(ui: &mut egui::Ui, current: &mut AppTab, target: AppTab, label: &str) {
    let selected = *current == target;
    let (bg, fg) = if selected {
        (md3::SECONDARY_CONTAINER, md3::ON_SECONDARY_CONTAINER)
    } else {
        (egui::Color32::TRANSPARENT, md3::ON_SURFACE_VARIANT)
    };
    let btn = egui::Button::new(
        egui::RichText::new(label).size(12.5).color(fg),
    )
    .fill(bg)
    .corner_radius(20)
    .stroke(egui::Stroke::NONE);
    if ui.add(btn).clicked() {
        *current = target;
    }
}

impl eframe::App for AirCardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_messages();

        if self.is_busy || self.scanning_syslog {
            ctx.request_repaint();
        }

        // Top bar
        egui::TopBottomPanel::top("header")
            .frame(
                egui::Frame::new()
                    .fill(md3::SURFACE)
                    .inner_margin(egui::Margin::symmetric(20, 10)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("AirCard")
                            .strong()
                            .size(18.0)
                            .color(md3::ON_SURFACE),
                    );
                    ui.label(
                        egui::RichText::new("v1.2.2")
                            .size(11.0)
                            .color(md3::ON_SURFACE_VARIANT),
                    );

                    ui.add_space(20.0);
                    m3_tab(ui, &mut self.current_tab, AppTab::Wallet, self.lang.tab_wallet());
                    m3_tab(ui, &mut self.current_tab, AppTab::Passcode, self.lang.tab_passcode());
                    m3_tab(ui, &mut self.current_tab, AppTab::Help, self.lang.tab_help());

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // 语言切换按钮
                        if m3_button_outlined(ui, self.lang.button_label()) {
                            self.lang.toggle();
                        }
                        ui.add_space(6.0);
                        if m3_button_outlined(ui, self.lang.refresh()) {
                            self.refresh_devices();
                        }
                        ui.add_space(4.0);
                        let has_device = !self.devices.is_empty();
                        draw_status_dot(ui, if has_device { md3::SUCCESS } else { md3::ERROR });
                        if has_device {
                            let name = self.devices.iter()
                                .find(|d| Some(&d.udid) == self.selected_udid.as_ref())
                                .map(|d| d.name.clone())
                                .unwrap_or_else(|| "iPhone".into());
                            ui.label(egui::RichText::new(name).size(12.0).color(md3::ON_SURFACE))
                                .on_hover_text(&self.apple_status);
                        } else {
                            ui.label(egui::RichText::new(self.lang.no_device()).size(12.0).color(md3::ON_SURFACE_VARIANT))
                                .on_hover_text(&self.apple_status);
                        }
                    });
                });
            });

        // Status bar
        egui::TopBottomPanel::bottom("status_bar")
            .frame(
                egui::Frame::new()
                    .fill(md3::SURFACE)
                    .inner_margin(egui::Margin::symmetric(20, 8)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let dot_col = if self.is_busy || self.scanning_syslog {
                        md3::PRIMARY
                    } else if self.status_msg.starts_with("Error") || self.status_msg.starts_with("Failed") || self.status_msg.starts_with("无法") || self.status_msg.starts_with("错误") {
                        md3::ERROR
                    } else {
                        md3::SUCCESS
                    };
                    draw_status_dot(ui, dot_col);
                    if self.is_busy || self.scanning_syslog { ui.spinner(); }
                    ui.label(egui::RichText::new(&self.status_msg).size(11.5).color(md3::ON_SURFACE_VARIANT));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let btn_text = self.lang.btn_logs(self.show_logs_window);
                        let btn = egui::Button::new(
                            egui::RichText::new(btn_text).size(11.0).color(
                                if self.show_logs_window { md3::ON_PRIMARY_CONTAINER } else { md3::ON_SURFACE_VARIANT }
                            ),
                        )
                        .fill(if self.show_logs_window { md3::PRIMARY_CONTAINER } else { egui::Color32::TRANSPARENT })
                        .corner_radius(20)
                        .stroke(egui::Stroke::new(1.0_f32, if self.show_logs_window { md3::PRIMARY } else { md3::OUTLINE_VARIANT }));
                        if ui.add(btn).clicked() {
                            self.show_logs_window = !self.show_logs_window;
                        }
                    });
                });
            });

        // Central - same SURFACE fill as header/status for flat look
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(md3::SURFACE)
                    .inner_margin(egui::Margin::same(16)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    match self.current_tab {
                        AppTab::Wallet => self.show_wallet_tab(ctx, ui),
                        AppTab::Passcode => self.show_passcode_tab(ctx, ui),
                        AppTab::Help => self.show_help_tab(ui),
                    }
                });
            });

        let mut show_logs = self.show_logs_window;
        let mut file_saved_msg: Option<String> = None;
        if show_logs {
            egui::Window::new(self.lang.window_logs_title())
                .open(&mut show_logs)
                .default_size([540.0, 300.0])
                .min_size([360.0, 180.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if m3_button_tonal(ui, self.lang.btn_copy_logs()) {
                            ctx.copy_text(self.logs.join("\n"));
                        }
                        if m3_button_outlined(ui, self.lang.btn_save_logs()) {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_file_name("aircard-diagnostics.log")
                                .add_filter("Log files", &["log", "txt"])
                                .save_file()
                            {
                                let content = self.logs.join("\r\n");
                                let _ = std::fs::write(&path, content);
                                file_saved_msg = Some(format!("Saved log file to {}", path.display()));
                            }
                        }
                        if m3_button_outlined(ui, self.lang.btn_clear()) {
                            self.logs.clear();
                        }
                        ui.label(
                            egui::RichText::new(self.lang.logs_entries_count(self.logs.len()))
                                .size(11.0)
                                .color(md3::ON_SURFACE_VARIANT),
                        );
                    });
                    ui.add_space(8.0);
                    egui::Frame::new()
                        .fill(md3::SURFACE_CONTAINER_HIGH)
                        .corner_radius(12)
                        .inner_margin(egui::Margin::same(10))
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .stick_to_bottom(true)
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    if self.logs.is_empty() {
                                        ui.label(egui::RichText::new(self.lang.logs_empty()).size(11.0).color(md3::ON_SURFACE_VARIANT));
                                    } else {
                                        for line in &self.logs {
                                            ui.label(
                                                egui::RichText::new(line)
                                                    .size(10.5)
                                                    .monospace()
                                                    .color(md3::ON_SURFACE),
                                            );
                                        }
                                    }
                                });
                        });
                });
            self.show_logs_window = show_logs;
            if let Some(msg) = file_saved_msg {
                self.add_log(msg);
            }
        }
    }
}

impl AirCardApp {
    fn show_wallet_tab(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if self.scanning_syslog {
            egui::Frame::new()
                .fill(md3::TERTIARY_CONTAINER)
                .corner_radius(16)
                .inner_margin(egui::Margin::same(16))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(self.lang.syslog_scanning_title()).strong().size(13.0).color(md3::ON_TERTIARY_CONTAINER));
                            ui.label(egui::RichText::new(self.lang.syslog_scanning_hint()).size(11.5).color(md3::ON_TERTIARY_CONTAINER));
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let btn = egui::Button::new(egui::RichText::new(self.lang.btn_stop()).size(12.0).color(md3::ON_SURFACE))
                                .fill(md3::ERROR_CONTAINER).corner_radius(20).stroke(egui::Stroke::NONE);
                            if ui.add(btn).clicked() { self.toggle_syslog_scan(); }
                        });
                    });
                });
            ui.add_space(8.0);
        }

        ui.columns(2, |cols| {
            let left = &mut cols[0];
            m3_card(left, |ui| {
                ui.label(egui::RichText::new(self.lang.wallet_config_title()).strong().size(16.0).color(md3::ON_SURFACE));
                ui.add_space(4.0);
                ui.label(egui::RichText::new(self.lang.wallet_config_subtitle()).size(12.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(16.0);

                // Target Card Hash
                ui.label(egui::RichText::new(self.lang.label_target_card_hash()).strong().size(12.0).color(md3::ON_SURFACE));
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    let btn_w = 90.0;
                    let text_w = (ui.available_width() - btn_w - 12.0).max(150.0);
                    ui.add(egui::TextEdit::singleline(&mut self.card_hash).hint_text(self.lang.hint_card_hash()).desired_width(text_w));

                    let scan_label = if self.scanning_syslog { self.lang.btn_stop() } else { self.lang.btn_scan() };
                    let scan_bg = if self.scanning_syslog { md3::ERROR_CONTAINER } else { md3::PRIMARY_CONTAINER };
                    let scan_fg = if self.scanning_syslog { md3::ERROR } else { md3::ON_PRIMARY_CONTAINER };
                    let scan_btn = egui::Button::new(egui::RichText::new(scan_label).size(12.0).color(scan_fg))
                        .fill(scan_bg).corner_radius(20).stroke(egui::Stroke::NONE);
                    if ui.add(scan_btn).clicked() { self.toggle_syslog_scan(); }
                });

                if !self.saved_cards.is_empty() {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(self.lang.label_saved_cards()).size(11.0).color(md3::ON_SURFACE_VARIANT));
                    ui.add_space(2.0);
                    let combo_w = (ui.available_width() - 4.0).max(150.0);
                    let sel_label = self.saved_cards.iter()
                        .find(|c| c.hash == self.card_hash)
                        .map(|c| format!("{} ({})", c.name, &c.hash[..8.min(c.hash.len())]))
                        .unwrap_or_else(|| self.lang.label_select_card().into());

                    egui::ComboBox::from_id_salt("saved_cards_box")
                        .width(combo_w)
                        .selected_text(egui::RichText::new(sel_label).color(md3::ON_SURFACE))
                        .show_ui(ui, |ui| {
                            for card in &self.saved_cards {
                                let is_selected = self.card_hash == card.hash;
                                let label = format!("{} ({}...)", card.name, &card.hash[..8.min(card.hash.len())]);
                                let text = egui::RichText::new(label)
                                    .color(if is_selected { md3::ON_PRIMARY_CONTAINER } else { md3::ON_SURFACE })
                                    .strong();
                                if ui.selectable_label(is_selected, text).clicked() {
                                    self.card_hash = card.hash.clone();
                                }
                            }
                        });
                }

                ui.add_space(16.0);

                // Card Skin
                ui.label(egui::RichText::new(self.lang.label_card_skin_artwork()).strong().size(12.0).color(md3::ON_SURFACE));
                ui.label(egui::RichText::new(self.lang.label_skin_artwork_desc()).size(11.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if m3_button_filled(ui, self.lang.btn_choose_image()) { self.select_skin(ctx); }
                    if self.skin.is_some() {
                        if m3_button_tonal(ui, self.lang.btn_export_png()) { self.save_prepared_png(); }
                    }
                });

                if let Some(skin) = &self.skin {
                    ui.add_space(4.0);
                    let fname = self.source_path.as_ref()
                        .and_then(|p| p.file_name()).and_then(|n| n.to_str()).unwrap_or("image");
                    ui.label(egui::RichText::new(format!("{} - 1536x969 - {:.0} KB", fname, skin.png.len() as f32 / 1024.0)).size(11.0).color(md3::PRIMARY));
                }

                ui.add_space(16.0);

                // Apply
                ui.label(egui::RichText::new(self.lang.label_write_to_iphone()).strong().size(12.0).color(md3::ON_SURFACE));
                ui.add_space(4.0);

                let can_flash = !self.is_busy && self.selected_udid.is_some() && !self.card_hash.trim().is_empty() && self.skin.is_some();
                let flash_btn = egui::Button::new(
                    egui::RichText::new(self.lang.btn_apply_card_skin()).strong().size(14.0)
                        .color(if can_flash { md3::ON_PRIMARY } else { md3::ON_SURFACE_VARIANT }),
                )
                .fill(if can_flash { md3::PRIMARY } else { md3::SURFACE_CONTAINER_HIGH })
                .corner_radius(20).stroke(egui::Stroke::NONE)
                .min_size(egui::vec2(ui.available_width(), 40.0));

                let resp = ui.add_enabled(can_flash, flash_btn);
                if resp.clicked() { self.flash_card(); }
                if !can_flash {
                    let mut r = Vec::new();
                    if self.selected_udid.is_none() { r.push(self.lang.need_connect_iphone()); }
                    if self.card_hash.trim().is_empty() { r.push(self.lang.need_enter_hash()); }
                    if self.skin.is_none() { r.push(self.lang.need_choose_image()); }
                    if !r.is_empty() { resp.on_disabled_hover_text(format!("{}{}", self.lang.need_prefix(), r.join(", "))); }
                }

                if self.is_busy {
                    ui.add_space(8.0);
                    if self.progress_total > 0 {
                        ui.add(egui::ProgressBar::new(self.progress_step as f32 / self.progress_total as f32).animate(true));
                    }
                    ui.label(egui::RichText::new(&self.progress_msg).size(11.0).color(md3::PRIMARY));
                }
            });

            // Right: preview
            let right = &mut cols[1];
            m3_card(right, |ui| {
                ui.label(egui::RichText::new(self.lang.preview_wallet_title()).strong().size(16.0).color(md3::ON_SURFACE));
                ui.add_space(4.0);
                ui.label(egui::RichText::new(self.lang.preview_wallet_subtitle()).size(12.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(12.0);

                let pass_w = (ui.available_width() - 8.0).clamp(250.0, 400.0);
                let pass_h = pass_w * (969.0 / 1536.0);
                ui.vertical_centered(|ui| {
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(pass_w, pass_h), egui::Sense::hover());
                    let painter = ui.painter();
                    if let Some(tex) = self.skin_texture.as_ref() {
                        painter.image(tex.id(), rect,
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            egui::Color32::WHITE);
                        painter.rect_stroke(rect, 16.0,
                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgba_premultiplied(255, 255, 255, 30)),
                            egui::StrokeKind::Inside);
                    } else {
                        painter.rect_filled(rect, 16.0, md3::SURFACE_CONTAINER_HIGH);
                        painter.text(rect.center(), egui::Align2::CENTER_CENTER,
                            self.lang.preview_no_artwork(), egui::FontId::proportional(14.0), md3::ON_SURFACE_VARIANT);
                    }
                });

                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("1536x969").size(11.0).color(md3::ON_SURFACE_VARIANT));
                    ui.label(egui::RichText::new("|").size(11.0).color(md3::OUTLINE_VARIANT));
                    ui.label(egui::RichText::new("1.585 ratio").size(11.0).color(md3::ON_SURFACE_VARIANT));
                    ui.label(egui::RichText::new("|").size(11.0).color(md3::OUTLINE_VARIANT));
                    if self.skin.is_some() {
                        ui.label(egui::RichText::new(self.lang.preview_ready()).size(11.0).color(md3::SUCCESS));
                    } else {
                        ui.label(egui::RichText::new(self.lang.preview_no_image()).size(11.0).color(md3::ON_SURFACE_VARIANT));
                    }
                });
                ui.add_space(8.0);
                ui.label(egui::RichText::new(self.lang.preview_post_apply_wallet_hint()).size(11.0).color(md3::ON_SURFACE_VARIANT));
            });
        });
    }

    fn show_passcode_tab(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.columns(2, |cols| {
            // Left: config
            let left = &mut cols[0];
            m3_card(left, |ui| {
                ui.label(egui::RichText::new(self.lang.passcode_theme_title()).strong().size(16.0).color(md3::ON_SURFACE));
                ui.add_space(4.0);
                ui.label(egui::RichText::new(self.lang.passcode_theme_subtitle()).size(12.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(16.0);

                // Theme file
                ui.label(egui::RichText::new(self.lang.label_theme_package()).strong().size(12.0).color(md3::ON_SURFACE));
                ui.label(egui::RichText::new(self.lang.label_theme_package_desc()).size(11.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(4.0);
                if m3_button_filled(ui, self.lang.btn_choose_passthm()) { self.select_theme_file(ctx); }

                if let Some(theme) = &self.loaded_theme {
                    let fname = self.theme_path.as_ref()
                        .and_then(|p| p.file_name()).and_then(|n| n.to_str()).unwrap_or("theme.passthm");
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(self.lang.theme_assets_summary(fname, theme.items.len())).size(11.0).color(md3::PRIMARY));
                }

                ui.add_space(16.0);

                // iOS version
                ui.label(egui::RichText::new(self.lang.label_target_ios_cache()).strong().size(12.0).color(md3::ON_SURFACE));
                ui.label(egui::RichText::new(self.lang.label_target_ios_cache_desc()).size(11.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(4.0);
                let combo_w = (ui.available_width() - 4.0).max(150.0);
                let mut ver_changed = false;
                let sel_ver_text = match self.forced_telephony_ver.as_str() {
                    "Auto (TelephonyUI-10)" => self.lang.telephony_auto(),
                    "TelephonyUI-10" => self.lang.telephony_10(),
                    "TelephonyUI-9" => self.lang.telephony_9(),
                    "TelephonyUI-8" => self.lang.telephony_8(),
                    _ => &self.forced_telephony_ver,
                };
                egui::ComboBox::from_id_salt("telephony_combo")
                    .width(combo_w)
                    .selected_text(sel_ver_text)
                    .show_ui(ui, |ui| {
                        ver_changed |= ui.selectable_value(&mut self.forced_telephony_ver, "Auto (TelephonyUI-10)".into(), self.lang.telephony_auto()).clicked();
                        ver_changed |= ui.selectable_value(&mut self.forced_telephony_ver, "TelephonyUI-10".into(), self.lang.telephony_10()).clicked();
                        ver_changed |= ui.selectable_value(&mut self.forced_telephony_ver, "TelephonyUI-9".into(), self.lang.telephony_9()).clicked();
                        ver_changed |= ui.selectable_value(&mut self.forced_telephony_ver, "TelephonyUI-8".into(), self.lang.telephony_8()).clicked();
                    });

                if ver_changed {
                    if let Some(path) = self.theme_path.clone() {
                        self.load_theme_from_path(ctx, &path);
                    }
                }

                ui.add_space(16.0);

                // Keypad Language
                ui.label(egui::RichText::new(self.lang.label_keypad_language()).strong().size(12.0).color(md3::ON_SURFACE));
                ui.label(egui::RichText::new(self.lang.label_keypad_language_desc()).size(11.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(4.0);
                let mut lang_changed = false;
                let sel_keypad_text = match self.keypad_language.as_str() {
                    "English" => self.lang.keypad_lang_english(),
                    "Russian" => self.lang.keypad_lang_russian(),
                    "Ukrainian" => self.lang.keypad_lang_ukrainian(),
                    "Japanese" => self.lang.keypad_lang_japanese(),
                    "All Languages (Universal)" => self.lang.keypad_lang_universal(),
                    _ => &self.keypad_language,
                };
                egui::ComboBox::from_id_salt("keypad_lang_combo")
                    .width(combo_w)
                    .selected_text(egui::RichText::new(sel_keypad_text).color(md3::ON_SURFACE))
                    .show_ui(ui, |ui| {
                        lang_changed |= ui.selectable_value(&mut self.keypad_language, "English".into(), self.lang.keypad_lang_english()).clicked();
                        lang_changed |= ui.selectable_value(&mut self.keypad_language, "Russian".into(), self.lang.keypad_lang_russian()).clicked();
                        lang_changed |= ui.selectable_value(&mut self.keypad_language, "Ukrainian".into(), self.lang.keypad_lang_ukrainian()).clicked();
                        lang_changed |= ui.selectable_value(&mut self.keypad_language, "Japanese".into(), self.lang.keypad_lang_japanese()).clicked();
                        lang_changed |= ui.selectable_value(&mut self.keypad_language, "All Languages (Universal)".into(), self.lang.keypad_lang_universal()).clicked();
                    });

                if lang_changed {
                    if let Some(path) = self.theme_path.clone() {
                        self.load_theme_from_path(ctx, &path);
                    }
                }

                ui.add_space(10.0);

                // Bold Font Toggle
                let mut bold_changed = false;
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut self.passcode_bold, egui::RichText::new(self.lang.checkbox_bold_text()).strong().size(12.0).color(md3::ON_SURFACE)).changed() {
                        bold_changed = true;
                    }
                });
                ui.label(
                    egui::RichText::new(self.lang.bold_text_desc())
                        .size(11.0)
                        .color(md3::ON_SURFACE_VARIANT),
                );

                if bold_changed {
                    if let Some(path) = self.theme_path.clone() {
                        self.load_theme_from_path(ctx, &path);
                    }
                }

                ui.add_space(16.0);

                // Apply
                ui.label(egui::RichText::new(self.lang.label_write_to_iphone()).strong().size(12.0).color(md3::ON_SURFACE));
                ui.add_space(4.0);

                let can_flash = !self.is_busy && self.selected_udid.is_some() && self.loaded_theme.is_some();
                let flash_btn = egui::Button::new(
                    egui::RichText::new(self.lang.btn_apply_passcode_theme()).strong().size(14.0)
                        .color(if can_flash { md3::ON_PRIMARY } else { md3::ON_SURFACE_VARIANT }),
                )
                .fill(if can_flash { md3::PRIMARY } else { md3::SURFACE_CONTAINER_HIGH })
                .corner_radius(20).stroke(egui::Stroke::NONE)
                .min_size(egui::vec2(ui.available_width(), 40.0));

                let resp = ui.add_enabled(can_flash, flash_btn);
                if resp.clicked() { self.flash_theme(); }
                if !can_flash {
                    let mut r = Vec::new();
                    if self.selected_udid.is_none() { r.push(self.lang.need_connect_iphone()); }
                    if self.loaded_theme.is_none() { r.push(self.lang.need_select_theme()); }
                    if !r.is_empty() { resp.on_disabled_hover_text(format!("{}{}", self.lang.need_prefix(), r.join(", "))); }
                }

                if self.is_busy {
                    ui.add_space(8.0);
                    if self.progress_total > 0 {
                        ui.add(egui::ProgressBar::new(self.progress_step as f32 / self.progress_total as f32).animate(true));
                    }
                    ui.label(egui::RichText::new(&self.progress_msg).size(11.0).color(md3::PRIMARY));
                }
            });

            // Right: preview
            let right = &mut cols[1];
            m3_card(right, |ui| {
                ui.label(egui::RichText::new(self.lang.preview_keypad_title()).strong().size(16.0).color(md3::ON_SURFACE));
                ui.add_space(4.0);
                ui.label(egui::RichText::new(self.lang.preview_keypad_subtitle()).size(12.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(12.0);

                let pass_w = (ui.available_width() - 8.0).clamp(240.0, 360.0);
                let pass_h = 265.0;

                ui.vertical_centered(|ui| {
                    if self.keypad_textures.is_empty() {
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(pass_w, pass_h), egui::Sense::hover());
                        let painter = ui.painter();
                        painter.rect_filled(rect, 16.0, md3::SURFACE_CONTAINER_HIGH);
                        painter.text(rect.center(), egui::Align2::CENTER_CENTER,
                            self.lang.preview_no_theme(), egui::FontId::proportional(14.0), md3::ON_SURFACE_VARIANT);
                    } else {
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(pass_w, pass_h), egui::Sense::hover());
                        let painter = ui.painter();
                        painter.rect_filled(rect, 16.0, md3::SURFACE_CONTAINER_HIGH);
                        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                            ui.vertical_centered(|ui| {
                                ui.add_space(10.0);
                                const DIALER_LAYOUT: &[&[&str]] = &[
                                    &["1", "2", "3"],
                                    &["4", "5", "6"],
                                    &["7", "8", "9"],
                                    &["", "0", ""],
                                ];
                                egui::Grid::new("keypad_grid")
                                    .spacing([18.0, 6.0])
                                    .show(ui, |ui| {
                                        for row in DIALER_LAYOUT {
                                            for &d in *row {
                                                if d.is_empty() {
                                                    ui.allocate_exact_size(egui::vec2(44.0, 50.0), egui::Sense::hover());
                                                } else if let Some((_, tex)) = self.keypad_textures.iter().find(|(k, _)| k == d) {
                                                    ui.vertical_centered(|ui| {
                                                        egui::Frame::new()
                                                            .fill(md3::SURFACE)
                                                            .corner_radius(12)
                                                            .inner_margin(3)
                                                            .show(ui, |ui| { ui.image((tex.id(), egui::vec2(40.0, 40.0))); });
                                                        ui.label(egui::RichText::new(d).size(9.5).color(md3::ON_SURFACE_VARIANT));
                                                    });
                                                } else {
                                                    ui.vertical_centered(|ui| {
                                                        egui::Frame::new()
                                                            .fill(md3::SURFACE)
                                                            .corner_radius(12)
                                                            .inner_margin(3)
                                                            .show(ui, |ui| {
                                                                let (btn_rect, _) = ui.allocate_exact_size(egui::vec2(40.0, 40.0), egui::Sense::hover());
                                                                ui.painter().rect_filled(btn_rect, 8.0, md3::SURFACE_CONTAINER);
                                                                ui.painter().text(btn_rect.center(), egui::Align2::CENTER_CENTER, d, egui::FontId::proportional(14.0), md3::ON_SURFACE_VARIANT);
                                                            });
                                                        ui.label(egui::RichText::new(d).size(9.5).color(md3::ON_SURFACE_VARIANT));
                                                    });
                                                }
                                            }
                                            ui.end_row();
                                        }
                                    });
                            });
                        });
                    }
                });

                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(self.lang.preview_keypad_layout_label()).size(11.0).color(md3::ON_SURFACE_VARIANT));
                    ui.label(egui::RichText::new("|").size(11.0).color(md3::OUTLINE_VARIANT));
                    ui.label(egui::RichText::new("TelephonyUI").size(11.0).color(md3::ON_SURFACE_VARIANT));
                    ui.label(egui::RichText::new("|").size(11.0).color(md3::OUTLINE_VARIANT));
                    if self.loaded_theme.is_some() {
                        ui.label(egui::RichText::new(self.lang.preview_ready()).size(11.0).color(md3::SUCCESS));
                    } else {
                        ui.label(egui::RichText::new(self.lang.preview_no_theme()).size(11.0).color(md3::ON_SURFACE_VARIANT));
                    }
                });
                ui.add_space(8.0);
                ui.label(egui::RichText::new(self.lang.preview_post_apply_keypad_hint()).size(11.0).color(md3::ON_SURFACE_VARIANT));
            });
        });
    }

    fn show_help_tab(&mut self, ui: &mut egui::Ui) {
        ui.columns(2, |cols| {
            let left = &mut cols[0];
            m3_card(left, |ui| {
                ui.label(egui::RichText::new(self.lang.help_setup_title()).strong().size(16.0).color(md3::ON_SURFACE));
                ui.add_space(4.0);
                ui.label(egui::RichText::new(self.lang.help_setup_subtitle()).size(12.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(16.0);

                ui.label(egui::RichText::new(self.lang.help_prerequisites_title()).strong().size(12.0).color(md3::ON_SURFACE));
                ui.add_space(6.0);
                ui.label(egui::RichText::new(self.lang.help_prereq_1()).size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new(self.lang.help_prereq_2()).size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new(self.lang.help_prereq_3()).size(11.5).color(md3::ON_SURFACE_VARIANT));

                ui.add_space(18.0);

                ui.label(egui::RichText::new(self.lang.help_find_hash_title()).strong().size(12.0).color(md3::ON_SURFACE));
                ui.add_space(6.0);
                ui.label(egui::RichText::new(self.lang.help_hash_step_1()).size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new(self.lang.help_hash_step_2()).size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new(self.lang.help_hash_step_3()).size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new(self.lang.help_hash_step_4()).size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new(self.lang.help_hash_step_5()).size(11.5).color(md3::ON_SURFACE_VARIANT));
            });

            let right = &mut cols[1];
            m3_card(right, |ui| {
                ui.label(egui::RichText::new(self.lang.help_theme_title()).strong().size(16.0).color(md3::ON_SURFACE));
                ui.add_space(4.0);
                ui.label(egui::RichText::new(self.lang.help_theme_subtitle()).size(12.0).color(md3::ON_SURFACE_VARIANT));
                ui.add_space(16.0);

                ui.label(egui::RichText::new(self.lang.help_activate_wallet_title()).strong().size(12.0).color(md3::ON_SURFACE));
                ui.add_space(6.0);
                ui.label(egui::RichText::new(self.lang.help_wallet_step_1()).size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new(self.lang.help_wallet_step_2()).size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new(self.lang.help_wallet_step_3()).size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new(self.lang.help_wallet_step_4()).size(11.5).color(md3::ON_SURFACE_VARIANT));

                ui.add_space(18.0);

                ui.label(egui::RichText::new(self.lang.help_passthm_title()).strong().size(12.0).color(md3::ON_SURFACE));
                ui.add_space(6.0);
                ui.label(egui::RichText::new(self.lang.help_passthm_desc_1()).size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new(self.lang.help_passthm_desc_2()).size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new(self.lang.help_passthm_desc_3()).size(11.5).color(md3::ON_SURFACE_VARIANT));
                ui.label(egui::RichText::new(self.lang.help_passthm_desc_4()).size(11.5).color(md3::ON_SURFACE_VARIANT));
            });
        });
    }
}
