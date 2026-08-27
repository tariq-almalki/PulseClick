#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{
    self, Align, Align2, Button, Color32, Frame, Layout, RichText, Rounding, Stroke,
};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const MIN_CUSTOM_BURST_CLICKS: u32 = 5;
const MAX_BURST_CLICKS: u32 = 1_000;
const CONTENT_MAX_WIDTH: f32 = 960.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ClickPattern {
    Single,
    Double,
    Triple,
    Quadruple,
    Custom,
}

impl ClickPattern {
    fn count(self) -> usize {
        match self {
            Self::Single => 1,
            Self::Double => 2,
            Self::Triple => 3,
            Self::Quadruple => 4,
            Self::Custom => MIN_CUSTOM_BURST_CLICKS as usize,
        }
    }

    fn resolved_count(self, custom_clicks: u32) -> usize {
        match self {
            Self::Custom => custom_clicks.clamp(MIN_CUSTOM_BURST_CLICKS, MAX_BURST_CLICKS) as usize,
            _ => self.count(),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Single => "Single",
            Self::Double => "Double",
            Self::Triple => "Triple",
            Self::Quadruple => "Quadruple",
            Self::Custom => "Custom",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MouseButton {
    Left,
    Right,
    Middle,
}

impl MouseButton {
    fn label(self) -> &'static str {
        match self {
            Self::Left => "Left button",
            Self::Right => "Right button",
            Self::Middle => "Middle button",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RepeatMode {
    UntilStopped,
    FixedCount,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TargetMode {
    CurrentCursor,
    FixedPosition,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ThemeMode {
    Dark,
    Light,
}

impl ThemeMode {
    fn label(self) -> &'static str {
        match self {
            Self::Dark => "Black",
            Self::Light => "Light",
        }
    }
}

#[derive(Clone, Copy)]
struct ThemeColors {
    background: Color32,
    surface: Color32,
    surface_alt: Color32,
    surface_soft: Color32,
    border: Color32,
    text: Color32,
    muted: Color32,
    accent: Color32,
    accent_fill: Color32,
    success: Color32,
    warning: Color32,
    danger: Color32,
    input: Color32,
}

fn theme_colors(theme: ThemeMode) -> ThemeColors {
    match theme {
        ThemeMode::Dark => ThemeColors {
            background: Color32::from_rgb(10, 10, 11),
            surface: Color32::from_rgb(20, 20, 21),
            surface_alt: Color32::from_rgb(30, 30, 31),
            surface_soft: Color32::from_rgb(15, 15, 16),
            border: Color32::from_rgb(57, 57, 59),
            text: Color32::from_rgb(245, 245, 241),
            muted: Color32::from_rgb(166, 166, 161),
            accent: Color32::from_rgb(235, 235, 229),
            accent_fill: Color32::from_rgb(71, 71, 73),
            success: Color32::from_rgb(151, 215, 170),
            warning: Color32::from_rgb(226, 195, 126),
            danger: Color32::from_rgb(232, 139, 149),
            input: Color32::from_rgb(12, 12, 13),
        },
        ThemeMode::Light => ThemeColors {
            background: Color32::from_rgb(246, 246, 243),
            surface: Color32::from_rgb(255, 255, 252),
            surface_alt: Color32::from_rgb(237, 237, 233),
            surface_soft: Color32::from_rgb(242, 242, 238),
            border: Color32::from_rgb(211, 211, 204),
            text: Color32::from_rgb(27, 27, 26),
            muted: Color32::from_rgb(102, 102, 96),
            accent: Color32::from_rgb(32, 32, 31),
            accent_fill: Color32::from_rgb(56, 56, 54),
            success: Color32::from_rgb(28, 119, 72),
            warning: Color32::from_rgb(133, 91, 17),
            danger: Color32::from_rgb(177, 63, 76),
            input: Color32::from_rgb(251, 251, 248),
        },
    }
}

fn apply_theme(ctx: &egui::Context, theme: ThemeMode) {
    let colors = theme_colors(theme);
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(12.0, 8.0);
    style.spacing.button_padding = egui::vec2(16.0, 8.0);
    // Keep the scrollbar in its own layout lane so it never paints over a card.
    // The default egui scrollbar is floating, which is a poor fit for this dense
    // settings surface because its handle can cover the right edge of controls.
    style.spacing.scroll = egui::style::ScrollStyle::solid();
    style.spacing.scroll.bar_width = 10.0;
    style.spacing.scroll.bar_inner_margin = 4.0;
    style.spacing.scroll.bar_outer_margin = 4.0;

    let mut visuals = match theme {
        ThemeMode::Dark => egui::Visuals::dark(),
        ThemeMode::Light => egui::Visuals::light(),
    };
    visuals.window_fill = colors.surface_alt;
    visuals.panel_fill = colors.background;
    visuals.extreme_bg_color = colors.input;
    visuals.faint_bg_color = colors.surface_soft;
    visuals.override_text_color = Some(colors.text);
    visuals.selection.bg_fill = colors.accent_fill;
    visuals.selection.stroke = Stroke::new(1.0_f32, colors.accent);

    ctx.set_style(style);
    ctx.set_visuals(visuals);
}

fn hotkey_virtual_key(key: egui::Key) -> Option<u32> {
    Some(match key {
        egui::Key::ArrowDown => 0x28,
        egui::Key::ArrowLeft => 0x25,
        egui::Key::ArrowRight => 0x27,
        egui::Key::ArrowUp => 0x26,
        egui::Key::Escape => 0x1B,
        egui::Key::Tab => 0x09,
        egui::Key::Backspace => 0x08,
        egui::Key::Enter => 0x0D,
        egui::Key::Space => 0x20,
        egui::Key::Insert => 0x2D,
        egui::Key::Delete => 0x2E,
        egui::Key::Home => 0x24,
        egui::Key::End => 0x23,
        egui::Key::PageUp => 0x21,
        egui::Key::PageDown => 0x22,
        egui::Key::Colon | egui::Key::Semicolon => 0xBA,
        egui::Key::Comma => 0xBC,
        egui::Key::Minus => 0xBD,
        egui::Key::Period => 0xBE,
        egui::Key::Plus | egui::Key::Equals => 0xBB,
        egui::Key::Slash | egui::Key::Questionmark => 0xBF,
        egui::Key::OpenBracket => 0xDB,
        egui::Key::Backslash | egui::Key::Pipe => 0xDC,
        egui::Key::CloseBracket => 0xDD,
        egui::Key::Backtick => 0xC0,
        egui::Key::Quote => 0xDE,
        egui::Key::Num0 => 0x30,
        egui::Key::Num1 => 0x31,
        egui::Key::Num2 => 0x32,
        egui::Key::Num3 => 0x33,
        egui::Key::Num4 => 0x34,
        egui::Key::Num5 => 0x35,
        egui::Key::Num6 => 0x36,
        egui::Key::Num7 => 0x37,
        egui::Key::Num8 => 0x38,
        egui::Key::Num9 => 0x39,
        egui::Key::A => 0x41,
        egui::Key::B => 0x42,
        egui::Key::C => 0x43,
        egui::Key::D => 0x44,
        egui::Key::E => 0x45,
        egui::Key::F => 0x46,
        egui::Key::G => 0x47,
        egui::Key::H => 0x48,
        egui::Key::I => 0x49,
        egui::Key::J => 0x4A,
        egui::Key::K => 0x4B,
        egui::Key::L => 0x4C,
        egui::Key::M => 0x4D,
        egui::Key::N => 0x4E,
        egui::Key::O => 0x4F,
        egui::Key::P => 0x50,
        egui::Key::Q => 0x51,
        egui::Key::R => 0x52,
        egui::Key::S => 0x53,
        egui::Key::T => 0x54,
        egui::Key::U => 0x55,
        egui::Key::V => 0x56,
        egui::Key::W => 0x57,
        egui::Key::X => 0x58,
        egui::Key::Y => 0x59,
        egui::Key::Z => 0x5A,
        egui::Key::F1 => 0x70,
        egui::Key::F2 => 0x71,
        egui::Key::F3 => 0x72,
        egui::Key::F4 => 0x73,
        egui::Key::F5 => 0x74,
        egui::Key::F6 => 0x75,
        egui::Key::F7 => 0x76,
        egui::Key::F8 => 0x77,
        egui::Key::F9 => 0x78,
        egui::Key::F10 => 0x79,
        egui::Key::F11 => 0x7A,
        egui::Key::F12 => 0x7B,
        egui::Key::F13 => 0x7C,
        egui::Key::F14 => 0x7D,
        egui::Key::F15 => 0x7E,
        egui::Key::F16 => 0x7F,
        egui::Key::F17 => 0x80,
        egui::Key::F18 => 0x81,
        egui::Key::F19 => 0x82,
        egui::Key::F20 => 0x83,
        egui::Key::F21 => 0x84,
        egui::Key::F22 => 0x85,
        egui::Key::F23 => 0x86,
        egui::Key::F24 => 0x87,
        _ => return None,
    })
}

fn hotkey_is_reserved(key: egui::Key) -> bool {
    matches!(key, egui::Key::F8 | egui::Key::F9)
}

#[derive(Clone, Copy, Debug)]
struct ClickSettings {
    start_delay: Duration,
    interval: Duration,
    burst_interval: Duration,
    button: MouseButton,
    click_count: usize,
    repeat_mode: RepeatMode,
    repeat_count: u64,
    target_mode: TargetMode,
    fixed_x: i32,
    fixed_y: i32,
}

#[derive(Clone, Copy, Debug, Default)]
struct CursorPosition {
    x: i32,
    y: i32,
}

enum WorkerEvent {
    Completed(u64),
    Stopped,
    InputError,
    Click { x: i32, y: i32, button: MouseButton },
}

enum WorkerOutcome {
    Completed(u64),
    Stopped,
    InputError,
}

#[cfg(target_os = "windows")]
mod win32 {
    use super::MouseButton;
    use std::ffi::c_void;
    use std::mem::size_of;
    use std::ptr::null_mut;
    use std::slice;
    use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, AtomicUsize, Ordering};
    use std::sync::mpsc::{self, Receiver, Sender};
    use std::sync::Arc;
    use std::sync::OnceLock;
    use std::thread;
    use std::time::{Duration, Instant};

    type Hwnd = *mut c_void;
    type Hdc = *mut c_void;
    type Hbitmap = *mut c_void;

    const INPUT_MOUSE: u32 = 0;
    const MOUSEEVENTF_LEFTDOWN: u32 = 0x0002;
    const MOUSEEVENTF_LEFTUP: u32 = 0x0004;
    const MOUSEEVENTF_RIGHTDOWN: u32 = 0x0008;
    const MOUSEEVENTF_RIGHTUP: u32 = 0x0010;
    const MOUSEEVENTF_MIDDLEDOWN: u32 = 0x0020;
    const MOUSEEVENTF_MIDDLEUP: u32 = 0x0040;

    const WM_HOTKEY: u32 = 0x0312;
    const PM_REMOVE: u32 = 0x0001;
    const MOD_NOREPEAT: u32 = 0x4000;
    const VK_F8: u32 = 0x77;
    const VK_F9: u32 = 0x78;

    const WM_NCHITTEST: u32 = 0x0084;
    const WM_MOUSEACTIVATE: u32 = 0x0021;
    const HTTRANSPARENT: isize = -1;
    const MA_NOACTIVATE: isize = 3;
    const WS_POPUP: u32 = 0x8000_0000;
    const WS_EX_LAYERED: u32 = 0x0008_0000;
    const WS_EX_TRANSPARENT: u32 = 0x0000_0020;
    const WS_EX_TOOLWINDOW: u32 = 0x0000_0080;
    const WS_EX_NOACTIVATE: u32 = 0x0800_0000;
    const WS_EX_TOPMOST: u32 = 0x0000_0008;
    const SW_HIDE: i32 = 0;
    const SW_SHOWNOACTIVATE: i32 = 4;
    const SWP_NOACTIVATE: u32 = 0x0010;
    const ULW_ALPHA: u32 = 0x0000_0002;
    const AC_SRC_OVER: u8 = 0;
    const AC_SRC_ALPHA: u8 = 1;
    const DIB_RGB_COLORS: u32 = 0;
    const BI_RGB: u32 = 0;
    const OVERLAY_SIZE: i32 = 176;
    const EFFECT_LIFETIME: Duration = Duration::from_millis(900);

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct MouseInput {
        dx: i32,
        dy: i32,
        mouse_data: u32,
        flags: u32,
        time: u32,
        extra_info: usize,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    union InputData {
        mouse: MouseInput,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Input {
        input_type: u32,
        data: InputData,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct Msg {
        hwnd: Hwnd,
        message: u32,
        w_param: usize,
        l_param: isize,
        time: u32,
        point: Point,
        private: u32,
    }

    #[repr(C)]
    struct Size {
        cx: i32,
        cy: i32,
    }

    #[repr(C)]
    struct BlendFunction {
        blend_op: u8,
        blend_flags: u8,
        source_constant_alpha: u8,
        alpha_format: u8,
    }

    #[repr(C)]
    struct WndClassW {
        style: u32,
        lpfn_wnd_proc: Option<unsafe extern "system" fn(Hwnd, u32, usize, isize) -> isize>,
        cb_cls_extra: i32,
        cb_wnd_extra: i32,
        h_instance: Hwnd,
        h_icon: Hwnd,
        h_cursor: Hwnd,
        hbr_background: Hwnd,
        lpsz_menu_name: *const u16,
        lpsz_class_name: *const u16,
    }

    #[repr(C)]
    struct BitmapInfoHeader {
        bi_size: u32,
        bi_width: i32,
        bi_height: i32,
        bi_planes: u16,
        bi_bit_count: u16,
        bi_compression: u32,
        bi_size_image: u32,
        bi_x_pels_per_meter: i32,
        bi_y_pels_per_meter: i32,
        bi_clr_used: u32,
        bi_clr_important: u32,
    }

    #[repr(C)]
    struct BitmapInfo {
        header: BitmapInfoHeader,
        colors: [u32; 1],
    }

    #[link(name = "user32")]
    extern "system" {
        fn GetCursorPos(point: *mut Point) -> i32;
        fn SetCursorPos(x: i32, y: i32) -> i32;
        fn SendInput(count: u32, inputs: *const Input, input_size: i32) -> u32;
        fn RegisterHotKey(hwnd: Hwnd, id: i32, modifiers: u32, virtual_key: u32) -> i32;
        fn UnregisterHotKey(hwnd: Hwnd, id: i32) -> i32;
        fn GetModuleHandleW(module_name: *const u16) -> Hwnd;
        fn RegisterClassW(window_class: *const WndClassW) -> u16;
        fn CreateWindowExW(
            extended_style: u32,
            class_name: *const u16,
            window_name: *const u16,
            style: u32,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
            parent: Hwnd,
            menu: Hwnd,
            instance: Hwnd,
            parameter: *mut c_void,
        ) -> Hwnd;
        fn DefWindowProcW(hwnd: Hwnd, message: u32, w_param: usize, l_param: isize) -> isize;
        fn ShowWindow(hwnd: Hwnd, command: i32) -> i32;
        fn SetWindowPos(
            hwnd: Hwnd,
            insert_after: Hwnd,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
            flags: u32,
        ) -> i32;
        fn UpdateLayeredWindow(
            hwnd: Hwnd,
            destination_dc: Hdc,
            destination_point: *const Point,
            size: *const Size,
            source_dc: Hdc,
            source_point: *const Point,
            color_key: u32,
            blend: *const BlendFunction,
            flags: u32,
        ) -> i32;
        fn GetDC(hwnd: Hwnd) -> Hdc;
        fn ReleaseDC(hwnd: Hwnd, dc: Hdc) -> i32;
        fn CreateCompatibleDC(dc: Hdc) -> Hdc;
        fn CreateDIBSection(
            dc: Hdc,
            bitmap_info: *const BitmapInfo,
            usage: u32,
            bits: *mut *mut c_void,
            section: Hwnd,
            offset: u32,
        ) -> Hbitmap;
        fn SelectObject(dc: Hdc, object: Hbitmap) -> Hbitmap;
        fn DeleteObject(object: Hbitmap) -> i32;
        fn DeleteDC(dc: Hdc) -> i32;
        fn TranslateMessage(message: *const Msg) -> i32;
        fn DispatchMessageW(message: *const Msg) -> isize;
        fn PeekMessageW(
            msg: *mut Msg,
            hwnd: Hwnd,
            min_filter: u32,
            max_filter: u32,
            remove_message: u32,
        ) -> i32;
    }

    pub fn cursor_position() -> Option<(i32, i32)> {
        let mut point = Point::default();
        let result = unsafe { GetCursorPos(&mut point) };
        (result != 0).then_some((point.x, point.y))
    }

    pub fn set_cursor_position(x: i32, y: i32) -> bool {
        unsafe { SetCursorPos(x, y) != 0 }
    }

    pub fn send_click(button: MouseButton) -> bool {
        send_clicks(button, 1)
    }

    pub fn send_clicks(button: MouseButton, click_count: usize) -> bool {
        if click_count == 0 {
            return true;
        }

        let (down, up) = match button {
            MouseButton::Left => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
            MouseButton::Right => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
            MouseButton::Middle => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
        };

        let mut inputs = Vec::with_capacity(click_count.saturating_mul(2));
        for _ in 0..click_count {
            for flags in [down, up] {
                inputs.push(Input {
                    input_type: INPUT_MOUSE,
                    data: InputData {
                        mouse: MouseInput {
                            dx: 0,
                            dy: 0,
                            mouse_data: 0,
                            flags,
                            time: 0,
                            extra_info: 0,
                        },
                    },
                });
            }
        }

        unsafe {
            SendInput(
                inputs.len() as u32,
                inputs.as_ptr(),
                size_of::<Input>() as i32,
            ) == inputs.len() as u32
        }
    }

    enum OverlayCommand {
        Show { x: i32, y: i32, color: [u8; 3] },
        Hide,
    }

    struct NativeClickEffect {
        x: i32,
        y: i32,
        started: Instant,
        color: [u8; 3],
    }

    struct OverlayBitmap {
        dc: Hdc,
        bitmap: Hbitmap,
        previous_bitmap: Hbitmap,
        bits: *mut u8,
    }

    static OVERLAY_SENDER: OnceLock<Sender<OverlayCommand>> = OnceLock::new();
    static OVERLAY_STATE: AtomicU8 = AtomicU8::new(0);

    pub fn show_click_indicator(x: i32, y: i32, color: (u8, u8, u8)) {
        let sender = OVERLAY_SENDER.get_or_init(|| {
            let (sender, receiver) = mpsc::channel();
            thread::spawn(move || overlay_loop(receiver));
            sender
        });
        let _ = sender.send(OverlayCommand::Show {
            x,
            y,
            color: [color.0, color.1, color.2],
        });
    }

    pub fn click_indicator_status() -> u8 {
        OVERLAY_STATE.load(Ordering::Acquire)
    }

    pub fn hide_click_indicator() {
        if let Some(sender) = OVERLAY_SENDER.get() {
            let _ = sender.send(OverlayCommand::Hide);
        }
    }

    fn overlay_loop(receiver: Receiver<OverlayCommand>) {
        let hwnd = unsafe { create_overlay_window() };
        if hwnd.is_null() {
            OVERLAY_STATE.store(3, Ordering::Release);
            return;
        }
        OVERLAY_STATE.store(1, Ordering::Release);
        let Some(mut bitmap) = (unsafe { OverlayBitmap::new() }) else {
            OVERLAY_STATE.store(4, Ordering::Release);
            unsafe {
                ShowWindow(hwnd, SW_HIDE);
            }
            return;
        };

        let mut effects = Vec::with_capacity(8);
        let mut message = Msg::default();
        loop {
            while let Ok(command) = receiver.try_recv() {
                match command {
                    OverlayCommand::Show { x, y, color } => {
                        if effects.len() >= 12 {
                            effects.remove(0);
                        }
                        effects.push(NativeClickEffect {
                            x,
                            y,
                            started: Instant::now(),
                            color,
                        });
                    }
                    OverlayCommand::Hide => {
                        effects.clear();
                        unsafe {
                            ShowWindow(hwnd, SW_HIDE);
                        }
                    }
                }
            }

            unsafe {
                while PeekMessageW(&mut message, null_mut(), 0, 0, PM_REMOVE) > 0 {
                    if message.message == 0x0012 {
                        return;
                    }
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }

            let now = Instant::now();
            effects.retain(|effect| now.duration_since(effect.started) < EFFECT_LIFETIME);
            if let Some(anchor) = effects.last().map(|effect| (effect.x, effect.y)) {
                unsafe {
                    render_overlay(hwnd, &mut bitmap, &effects, anchor, now);
                }
            } else {
                unsafe {
                    ShowWindow(hwnd, SW_HIDE);
                }
            }
            thread::sleep(Duration::from_millis(8));
        }
    }

    unsafe extern "system" fn overlay_window_proc(
        hwnd: Hwnd,
        message: u32,
        w_param: usize,
        l_param: isize,
    ) -> isize {
        match message {
            WM_NCHITTEST => HTTRANSPARENT,
            WM_MOUSEACTIVATE => MA_NOACTIVATE,
            _ => DefWindowProcW(hwnd, message, w_param, l_param),
        }
    }

    unsafe fn create_overlay_window() -> Hwnd {
        let class_name: Vec<u16> = "PulseClickClickOverlay"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let instance = GetModuleHandleW(null_mut());
        let window_class = WndClassW {
            style: 0,
            lpfn_wnd_proc: Some(overlay_window_proc),
            cb_cls_extra: 0,
            cb_wnd_extra: 0,
            h_instance: instance,
            h_icon: null_mut(),
            h_cursor: null_mut(),
            hbr_background: null_mut(),
            lpsz_menu_name: null_mut(),
            lpsz_class_name: class_name.as_ptr(),
        };
        RegisterClassW(&window_class);

        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TOPMOST,
            class_name.as_ptr(),
            class_name.as_ptr(),
            WS_POPUP,
            0,
            0,
            OVERLAY_SIZE,
            OVERLAY_SIZE,
            null_mut(),
            null_mut(),
            instance,
            null_mut(),
        );
        if !hwnd.is_null() {
            ShowWindow(hwnd, SW_HIDE);
        }
        hwnd
    }

    impl OverlayBitmap {
        unsafe fn new() -> Option<Self> {
            let dc = CreateCompatibleDC(null_mut());
            if dc.is_null() {
                return None;
            }
            let info = BitmapInfo {
                header: BitmapInfoHeader {
                    bi_size: size_of::<BitmapInfoHeader>() as u32,
                    bi_width: OVERLAY_SIZE,
                    bi_height: -OVERLAY_SIZE,
                    bi_planes: 1,
                    bi_bit_count: 32,
                    bi_compression: BI_RGB,
                    bi_size_image: 0,
                    bi_x_pels_per_meter: 0,
                    bi_y_pels_per_meter: 0,
                    bi_clr_used: 0,
                    bi_clr_important: 0,
                },
                colors: [0],
            };
            let mut bits: *mut c_void = null_mut();
            let bitmap = CreateDIBSection(dc, &info, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
            if bitmap.is_null() || bits.is_null() {
                DeleteDC(dc);
                return None;
            }
            let previous_bitmap = SelectObject(dc, bitmap);
            Some(Self {
                dc,
                bitmap,
                previous_bitmap,
                bits: bits.cast(),
            })
        }

        fn pixels_mut(&mut self) -> &mut [u8] {
            let byte_count = (OVERLAY_SIZE * OVERLAY_SIZE * 4) as usize;
            unsafe { slice::from_raw_parts_mut(self.bits, byte_count) }
        }
    }

    impl Drop for OverlayBitmap {
        fn drop(&mut self) {
            unsafe {
                if !self.dc.is_null() {
                    SelectObject(self.dc, self.previous_bitmap);
                    DeleteObject(self.bitmap);
                    DeleteDC(self.dc);
                }
            }
        }
    }

    unsafe fn render_overlay(
        hwnd: Hwnd,
        bitmap: &mut OverlayBitmap,
        effects: &[NativeClickEffect],
        anchor: (i32, i32),
        now: Instant,
    ) {
        let dc = bitmap.dc;
        let pixels = bitmap.pixels_mut();
        pixels.fill(0);
        let base_center = (OVERLAY_SIZE as f32 / 2.0, OVERLAY_SIZE as f32 / 2.0);
        for effect in effects {
            let center = (
                base_center.0 + (effect.x - anchor.0) as f32,
                base_center.1 + (effect.y - anchor.1) as f32,
            );
            draw_native_effect(pixels, center, effect, now);
        }

        let destination = Point {
            x: anchor.0 - OVERLAY_SIZE / 2,
            y: anchor.1 - OVERLAY_SIZE / 2,
        };
        let size = Size {
            cx: OVERLAY_SIZE,
            cy: OVERLAY_SIZE,
        };
        let source = Point { x: 0, y: 0 };
        let blend = BlendFunction {
            blend_op: AC_SRC_OVER,
            blend_flags: 0,
            source_constant_alpha: 255,
            alpha_format: AC_SRC_ALPHA,
        };

        SetWindowPos(
            hwnd,
            (-1isize) as Hwnd,
            destination.x,
            destination.y,
            OVERLAY_SIZE,
            OVERLAY_SIZE,
            SWP_NOACTIVATE,
        );

        let screen_dc = GetDC(null_mut());
        if !screen_dc.is_null() {
            let updated = UpdateLayeredWindow(
                hwnd,
                screen_dc,
                &destination,
                &size,
                dc,
                &source,
                0,
                &blend,
                ULW_ALPHA,
            ) != 0;
            OVERLAY_STATE.store(if updated { 2 } else { 5 }, Ordering::Release);
            ReleaseDC(null_mut(), screen_dc);
            ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        } else {
            OVERLAY_STATE.store(5, Ordering::Release);
        }
    }

    fn draw_native_effect(
        pixels: &mut [u8],
        center: (f32, f32),
        effect: &NativeClickEffect,
        now: Instant,
    ) {
        let progress = (now.duration_since(effect.started).as_secs_f32() / 0.9).clamp(0.0, 1.0);
        let eased = 1.0 - (1.0 - progress).powi(3);
        let fade = (1.0 - progress).powf(1.22);
        let flash = (1.0 - progress / 0.2).clamp(0.0, 1.0);
        let rotation = -0.75 + progress * 1.75;
        let color = effect.color;
        let luminance =
            0.2126 * color[0] as f32 + 0.7152 * color[1] as f32 + 0.0722 * color[2] as f32;
        let shadow = if luminance > 150.0 {
            [8, 8, 9]
        } else {
            [248, 248, 244]
        };

        let outer_radius = 23.0 + 47.0 * eased;
        let inner_radius = 8.0 + 22.0 * eased;
        let bracket_radius = 34.0 + 5.0 * (1.0 - eased);

        draw_ring(pixels, center, outer_radius + 2.0, 5.5, shadow, fade * 0.25);
        draw_ring(pixels, center, outer_radius, 2.4, color, fade * 0.92);
        draw_ring(pixels, center, inner_radius, 3.0, shadow, fade * 0.35);
        draw_ring(pixels, center, inner_radius, 2.0, color, fade);

        for segment in 0..4 {
            let start = rotation + segment as f32 * std::f32::consts::TAU / 4.0 + 0.16;
            draw_arc(
                pixels,
                center,
                bracket_radius,
                start,
                std::f32::consts::TAU * 0.13,
                3.4,
                shadow,
                fade * 0.34,
            );
            draw_arc(
                pixels,
                center,
                bracket_radius,
                start,
                std::f32::consts::TAU * 0.13,
                2.2,
                color,
                fade,
            );
        }

        let orbit_angle = rotation + std::f32::consts::TAU * 0.13;
        let orbit_point = (
            center.0 + orbit_angle.cos() * outer_radius,
            center.1 + orbit_angle.sin() * outer_radius,
        );
        draw_disc(pixels, orbit_point, 4.2, shadow, fade * 0.45);
        draw_disc(pixels, orbit_point, 2.8, color, fade);

        let ray_length = 11.0 + 9.0 * eased;
        for ray in 0..8 {
            let angle = ray as f32 * std::f32::consts::TAU / 8.0 + rotation * 0.35;
            let start = (center.0 + angle.cos() * 10.0, center.1 + angle.sin() * 10.0);
            let end = (
                center.0 + angle.cos() * (10.0 + ray_length),
                center.1 + angle.sin() * (10.0 + ray_length),
            );
            draw_line(pixels, start, end, 3.0, shadow, flash * 0.35);
            draw_line(pixels, start, end, 1.8, color, flash * 0.9);
        }

        draw_disc(pixels, center, 5.0 + 5.0 * flash, shadow, fade * 0.32);
        draw_disc(
            pixels,
            center,
            3.6 + 3.5 * flash,
            color,
            (0.55 + 0.45 * fade) * flash,
        );
        draw_ring(pixels, center, 6.0 + 9.0 * eased, 1.6, color, fade * flash);
    }

    fn draw_disc(pixels: &mut [u8], center: (f32, f32), radius: f32, color: [u8; 3], alpha: f32) {
        if radius <= 0.0 || alpha <= 0.0 {
            return;
        }
        let min_x = (center.0 - radius - 1.5).floor().max(0.0) as i32;
        let max_x = (center.0 + radius + 1.5)
            .ceil()
            .min((OVERLAY_SIZE - 1) as f32) as i32;
        let min_y = (center.1 - radius - 1.5).floor().max(0.0) as i32;
        let max_y = (center.1 + radius + 1.5)
            .ceil()
            .min((OVERLAY_SIZE - 1) as f32) as i32;
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let dx = x as f32 + 0.5 - center.0;
                let dy = y as f32 + 0.5 - center.1;
                let distance = (dx * dx + dy * dy).sqrt();
                let coverage = (radius + 1.0 - distance).clamp(0.0, 1.0);
                blend_pixel(pixels, x, y, color, alpha * coverage);
            }
        }
    }

    fn draw_ring(
        pixels: &mut [u8],
        center: (f32, f32),
        radius: f32,
        width: f32,
        color: [u8; 3],
        alpha: f32,
    ) {
        if radius <= 0.0 || width <= 0.0 || alpha <= 0.0 {
            return;
        }
        let padding = width / 2.0 + 1.5;
        let min_x = (center.0 - radius - padding).floor().max(0.0) as i32;
        let max_x = (center.0 + radius + padding)
            .ceil()
            .min((OVERLAY_SIZE - 1) as f32) as i32;
        let min_y = (center.1 - radius - padding).floor().max(0.0) as i32;
        let max_y = (center.1 + radius + padding)
            .ceil()
            .min((OVERLAY_SIZE - 1) as f32) as i32;
        let half_width = width / 2.0;
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let dx = x as f32 + 0.5 - center.0;
                let dy = y as f32 + 0.5 - center.1;
                let distance = (dx * dx + dy * dy).sqrt();
                let edge = (half_width + 1.0 - (distance - radius).abs()).clamp(0.0, 1.0);
                blend_pixel(pixels, x, y, color, alpha * edge);
            }
        }
    }

    fn draw_arc(
        pixels: &mut [u8],
        center: (f32, f32),
        radius: f32,
        start: f32,
        sweep: f32,
        width: f32,
        color: [u8; 3],
        alpha: f32,
    ) {
        let steps = ((sweep.abs() * radius) / 2.0).ceil().max(8.0) as usize;
        for index in 0..=steps {
            let progress = index as f32 / steps as f32;
            let angle = start + sweep * progress;
            draw_disc(
                pixels,
                (
                    center.0 + angle.cos() * radius,
                    center.1 + angle.sin() * radius,
                ),
                width / 2.0,
                color,
                alpha,
            );
        }
    }

    fn draw_line(
        pixels: &mut [u8],
        start: (f32, f32),
        end: (f32, f32),
        width: f32,
        color: [u8; 3],
        alpha: f32,
    ) {
        let dx = end.0 - start.0;
        let dy = end.1 - start.1;
        let steps = (dx * dx + dy * dy).sqrt().ceil().max(1.0) as usize;
        for index in 0..=steps {
            let progress = index as f32 / steps as f32;
            draw_disc(
                pixels,
                (start.0 + dx * progress, start.1 + dy * progress),
                width / 2.0,
                color,
                alpha,
            );
        }
    }

    fn blend_pixel(pixels: &mut [u8], x: i32, y: i32, color: [u8; 3], alpha: f32) {
        if !(0..OVERLAY_SIZE).contains(&x) || !(0..OVERLAY_SIZE).contains(&y) || alpha <= 0.0 {
            return;
        }
        let source_alpha = (alpha.clamp(0.0, 1.0) * 255.0).round() as u16;
        if source_alpha == 0 {
            return;
        }
        let index = ((y * OVERLAY_SIZE + x) * 4) as usize;
        let inverse_alpha = 255_u16 - source_alpha;
        let destination_alpha = pixels[index + 3] as u16;
        pixels[index] = ((u16::from(color[2]) * source_alpha / 255
            + u16::from(pixels[index]) * inverse_alpha / 255)
            .min(255)) as u8;
        pixels[index + 1] = ((u16::from(color[1]) * source_alpha / 255
            + u16::from(pixels[index + 1]) * inverse_alpha / 255)
            .min(255)) as u8;
        pixels[index + 2] = ((u16::from(color[0]) * source_alpha / 255
            + u16::from(pixels[index + 2]) * inverse_alpha / 255)
            .min(255)) as u8;
        pixels[index + 3] = (source_alpha + destination_alpha * inverse_alpha / 255).min(255) as u8;
    }

    pub fn spawn_hotkey_listener(
        toggle_request: Arc<AtomicUsize>,
        stop_request: Arc<AtomicBool>,
        capture_request: Arc<AtomicBool>,
        hotkeys_available: Arc<AtomicBool>,
        toggle_hotkey: Arc<AtomicU32>,
    ) {
        thread::spawn(move || unsafe {
            let hwnd: Hwnd = null_mut();
            let stop_registered = RegisterHotKey(hwnd, 2, MOD_NOREPEAT, VK_F8) != 0;
            let capture_registered = RegisterHotKey(hwnd, 3, MOD_NOREPEAT, VK_F9) != 0;
            let mut toggle_registered = false;
            let mut registered_toggle_key = 0_u32;
            let mut msg = Msg::default();
            loop {
                let requested_toggle_key = toggle_hotkey.load(Ordering::Acquire);
                if requested_toggle_key != registered_toggle_key {
                    if toggle_registered {
                        UnregisterHotKey(hwnd, 1);
                    }
                    toggle_registered = requested_toggle_key != 0
                        && RegisterHotKey(hwnd, 1, MOD_NOREPEAT, requested_toggle_key) != 0;
                    registered_toggle_key = requested_toggle_key;
                }

                hotkeys_available.store(
                    toggle_registered && stop_registered && capture_registered,
                    Ordering::Release,
                );

                while PeekMessageW(&mut msg, hwnd, 0, 0, PM_REMOVE) > 0 {
                    if msg.message != WM_HOTKEY {
                        continue;
                    }

                    match msg.w_param as i32 {
                        1 => {
                            toggle_request.fetch_add(1, Ordering::AcqRel);
                        }
                        2 => {
                            stop_request.store(true, Ordering::Release);
                        }
                        3 => {
                            capture_request.store(true, Ordering::Release);
                        }
                        _ => {}
                    }
                }

                thread::sleep(Duration::from_millis(8));
            }
        });
    }
}

#[cfg(not(target_os = "windows"))]
mod win32 {
    use super::MouseButton;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Arc;

    pub fn cursor_position() -> Option<(i32, i32)> {
        None
    }

    pub fn set_cursor_position(_x: i32, _y: i32) -> bool {
        false
    }

    pub fn send_click(_button: MouseButton) -> bool {
        false
    }

    pub fn send_clicks(_button: MouseButton, _click_count: usize) -> bool {
        false
    }

    pub fn show_click_indicator(_x: i32, _y: i32, _color: (u8, u8, u8)) {}

    pub fn hide_click_indicator() {}

    pub fn click_indicator_status() -> u8 {
        0
    }

    pub fn spawn_hotkey_listener(
        _toggle_request: Arc<AtomicUsize>,
        _stop_request: Arc<AtomicBool>,
        _capture_request: Arc<AtomicBool>,
        hotkeys_available: Arc<AtomicBool>,
        _toggle_hotkey: Arc<AtomicU32>,
    ) {
        hotkeys_available.store(false, Ordering::Release);
    }
}

struct PulseClickApp {
    pattern: ClickPattern,
    custom_clicks: u32,
    button: MouseButton,
    repeat_mode: RepeatMode,
    repeat_count: u64,
    target_mode: TargetMode,
    target: CursorPosition,
    theme: ThemeMode,
    applied_theme: ThemeMode,
    toggle_hotkey: egui::Key,
    toggle_hotkey_code: Arc<AtomicU32>,
    recording_hotkey: bool,
    start_delay_seconds: u32,
    interval_hours: u32,
    interval_minutes: u32,
    interval_seconds: u32,
    interval_millis: u32,
    burst_interval_millis: u32,
    always_on_top: bool,
    show_click_animation: bool,
    applied_always_on_top: bool,
    last_status: String,
    toggle_request: Arc<AtomicUsize>,
    stop_request: Arc<AtomicBool>,
    capture_request: Arc<AtomicBool>,
    hotkeys_available: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
    starting: Arc<AtomicBool>,
    worker_stop: Option<Arc<AtomicBool>>,
    worker_handle: Option<JoinHandle<()>>,
    worker_events: Receiver<WorkerEvent>,
    worker_event_sender: Sender<WorkerEvent>,
    last_effect_at: Option<Instant>,
    preview_animation_started: Option<Instant>,
}

impl PulseClickApp {
    fn new(
        cc: &eframe::CreationContext<'_>,
        toggle_request: Arc<AtomicUsize>,
        stop_request: Arc<AtomicBool>,
        capture_request: Arc<AtomicBool>,
        hotkeys_available: Arc<AtomicBool>,
        toggle_hotkey_code: Arc<AtomicU32>,
    ) -> Self {
        apply_theme(&cc.egui_ctx, ThemeMode::Dark);

        let (worker_event_sender, worker_events) = mpsc::channel();

        Self {
            pattern: ClickPattern::Single,
            custom_clicks: MIN_CUSTOM_BURST_CLICKS,
            button: MouseButton::Left,
            repeat_mode: RepeatMode::UntilStopped,
            repeat_count: 100,
            target_mode: TargetMode::CurrentCursor,
            target: CursorPosition::default(),
            theme: ThemeMode::Dark,
            applied_theme: ThemeMode::Dark,
            toggle_hotkey: egui::Key::F6,
            toggle_hotkey_code,
            recording_hotkey: false,
            start_delay_seconds: 2,
            interval_hours: 0,
            interval_minutes: 0,
            interval_seconds: 0,
            interval_millis: 100,
            burst_interval_millis: 35,
            always_on_top: false,
            show_click_animation: true,
            applied_always_on_top: false,
            last_status: "Choose your settings, then start clicking.".to_string(),
            toggle_request,
            stop_request,
            capture_request,
            hotkeys_available,
            running: Arc::new(AtomicBool::new(false)),
            starting: Arc::new(AtomicBool::new(false)),
            worker_stop: None,
            worker_handle: None,
            worker_events,
            worker_event_sender,
            last_effect_at: None,
            preview_animation_started: None,
        }
    }

    fn interval_millis(&self) -> u64 {
        let total = u64::from(self.interval_hours) * 3_600_000
            + u64::from(self.interval_minutes) * 60_000
            + u64::from(self.interval_seconds) * 1_000
            + u64::from(self.interval_millis);
        total.max(1)
    }

    fn burst_interval_millis(&self) -> u64 {
        u64::from(self.burst_interval_millis.min(500))
    }

    fn click_count(&self) -> usize {
        self.pattern.resolved_count(self.custom_clicks)
    }

    fn effective_cycle_millis(&self) -> u64 {
        let gaps_inside_burst = self.click_count().saturating_sub(1) as u64;
        self.interval_millis().saturating_add(
            self.burst_interval_millis()
                .saturating_mul(gaps_inside_burst),
        )
    }

    fn settings(&self) -> ClickSettings {
        ClickSettings {
            start_delay: Duration::from_secs(u64::from(self.start_delay_seconds)),
            interval: Duration::from_millis(self.interval_millis()),
            burst_interval: Duration::from_millis(self.burst_interval_millis()),
            button: self.button,
            click_count: self.click_count(),
            repeat_mode: self.repeat_mode,
            repeat_count: self.repeat_count.max(1),
            target_mode: self.target_mode,
            fixed_x: self.target.x,
            fixed_y: self.target.y,
        }
    }

    fn process_hotkeys(&mut self) {
        if self.capture_request.swap(false, Ordering::AcqRel) {
            self.capture_position();
        }

        if self.stop_request.swap(false, Ordering::AcqRel) {
            self.stop_clicking();
        }

        let toggles = self.toggle_request.swap(0, Ordering::AcqRel);
        for _ in 0..toggles {
            self.toggle_clicking();
        }
    }

    fn process_recorded_hotkey(&mut self, ctx: &egui::Context) {
        if !self.recording_hotkey {
            return;
        }

        let captured = ctx.input(|input| {
            input.events.iter().find_map(|event| match event {
                egui::Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    ..
                } => Some(*key),
                _ => None,
            })
        });

        let Some(key) = captured else {
            return;
        };

        if hotkey_is_reserved(key) {
            self.recording_hotkey = false;
            self.last_status = format!(
                "{} is reserved for safety. Choose another start/stop key.",
                key.name()
            );
        } else if hotkey_virtual_key(key).is_some() {
            self.set_toggle_hotkey(key);
            self.recording_hotkey = false;
        }
    }

    fn set_toggle_hotkey(&mut self, key: egui::Key) {
        let Some(code) = hotkey_virtual_key(key) else {
            return;
        };
        self.toggle_hotkey = key;
        self.toggle_hotkey_code.store(code, Ordering::Release);
        self.last_status = format!("Start/stop hotkey set to {}.", key.name());
    }

    fn process_worker_events(&mut self) {
        while let Ok(event) = self.worker_events.try_recv() {
            match event {
                WorkerEvent::Click { x, y, button } => self.add_click_effect(x, y, button),
                WorkerEvent::Completed(cycles) => {
                    self.last_status = format!("Finished {cycles} cycle(s).");
                }
                WorkerEvent::Stopped => {
                    self.last_status = "Stopped safely.".to_string();
                }
                WorkerEvent::InputError => {
                    self.last_status =
                        "Windows rejected the mouse input. Try matching the target app's privilege level."
                            .to_string();
                }
            }
        }
    }

    fn add_click_effect(&mut self, x: i32, y: i32, button: MouseButton) {
        if !self.show_click_animation {
            return;
        }
        let now = Instant::now();
        if let Some(last) = self.last_effect_at {
            if now.duration_since(last) < Duration::from_millis(16) {
                return;
            }
        }
        self.last_effect_at = Some(now);

        let color = click_indicator_color(self.theme, button);
        self.preview_animation_started = Some(now);
        win32::show_click_indicator(x, y, (color.r(), color.g(), color.b()));
    }

    fn preview_click_indicator(&mut self) {
        if !self.show_click_animation {
            self.last_status = "Turn on Show click indicator to preview the effect.".to_string();
            return;
        }

        let Some((x, y)) = win32::cursor_position() else {
            self.last_status = "Could not read the current cursor position.".to_string();
            return;
        };

        // A preview is an explicit user action, so it should not be suppressed
        // by the worker-event throttle used for very fast click bursts.
        self.last_effect_at = None;
        self.add_click_effect(x, y, self.button);
        self.last_status = format!("Previewing the click indicator at ({x}, {y}).");
    }

    fn capture_position(&mut self) {
        if let Some((x, y)) = win32::cursor_position() {
            self.target = CursorPosition { x, y };
            self.target_mode = TargetMode::FixedPosition;
            self.last_status = format!("Target captured at ({x}, {y}).");
        } else {
            self.last_status = "Could not read the current cursor position.".to_string();
        }
    }

    fn cleanup_finished_worker(&mut self) {
        if self.worker_handle.is_some() && !self.running.load(Ordering::Acquire) {
            if let Some(handle) = self.worker_handle.take() {
                let _ = handle.join();
            }
            self.worker_stop = None;
        }
    }

    fn start_clicking(&mut self) {
        self.cleanup_finished_worker();
        if self.running.load(Ordering::Acquire) {
            return;
        }

        let settings = self.settings();
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let running = Arc::clone(&self.running);
        let starting = Arc::clone(&self.starting);
        let events = self.worker_event_sender.clone();

        running.store(true, Ordering::Release);
        starting.store(true, Ordering::Release);
        self.worker_stop = Some(stop);
        self.last_status = if settings.start_delay.is_zero() {
            "Starting now.".to_string()
        } else {
            format!(
                "Starting in {} second(s) — move the cursor to your target.",
                settings.start_delay.as_secs()
            )
        };

        self.worker_handle = Some(thread::spawn(move || {
            let outcome = run_clicker(settings, worker_stop, Arc::clone(&starting), &events);
            let event = match outcome {
                WorkerOutcome::Completed(cycles) => WorkerEvent::Completed(cycles),
                WorkerOutcome::Stopped => WorkerEvent::Stopped,
                WorkerOutcome::InputError => WorkerEvent::InputError,
            };
            let _ = events.send(event);
            starting.store(false, Ordering::Release);
            running.store(false, Ordering::Release);
        }));
    }

    fn stop_clicking(&mut self) {
        if let Some(stop) = &self.worker_stop {
            stop.store(true, Ordering::Release);
        }

        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
        self.worker_stop = None;
        self.starting.store(false, Ordering::Release);
        self.running.store(false, Ordering::Release);
        self.last_status = "Stopped safely.".to_string();
    }

    fn toggle_clicking(&mut self) {
        if self.running.load(Ordering::Acquire) {
            self.stop_clicking();
        } else {
            self.start_clicking();
        }
    }

    fn card<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
        let colors = Self::ui_theme_colors(ui);
        Frame::none()
            .fill(colors.surface)
            .stroke(Stroke::new(1.0_f32, colors.border))
            .rounding(Rounding::same(12.0))
            .inner_margin(egui::Margin::symmetric(24.0, 20.0))
            .show(ui, |ui| {
                let content_width = ui.available_width();
                ui.set_min_width(content_width);
                add_contents(ui)
            })
            .inner
    }

    fn section_header(ui: &mut egui::Ui, title: &str, caption: &str) {
        let colors = Self::ui_theme_colors(ui);
        ui.label(RichText::new(title).size(16.0).strong().color(colors.text));
        ui.label(RichText::new(caption).size(11.5).color(colors.muted));
        ui.add_space(12.0);
    }

    fn ui_theme_colors(ui: &egui::Ui) -> ThemeColors {
        theme_colors(if ui.visuals().dark_mode {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        })
    }

    fn render_overview_strip(&self, ui: &mut egui::Ui) {
        let colors = theme_colors(self.theme);
        let interval = format_interval(self.interval_millis());
        let target = match self.target_mode {
            TargetMode::CurrentCursor => "Live cursor".to_string(),
            TargetMode::FixedPosition => format!("({}, {})", self.target.x, self.target.y),
        };
        let plan = match self.repeat_mode {
            RepeatMode::UntilStopped => "Until stopped".to_string(),
            RepeatMode::FixedCount => format!("{} cycles", self.repeat_count.max(1)),
        };

        Frame::none()
            .fill(colors.surface_soft)
            .stroke(Stroke::new(1.0_f32, colors.border))
            .rounding(Rounding::same(10.0))
            .inner_margin(egui::Margin::symmetric(20.0, 16.0))
            .show(ui, |ui| {
                let content_width = ui.available_width();
                ui.set_min_width(content_width);
                ui.columns(4, |columns| {
                    Self::summary_item(
                        &mut columns[0],
                        colors,
                        "ACTION",
                        &format!(
                            "{} · {}",
                            self.pattern.label(),
                            self.button.label().replace(" button", "")
                        ),
                    );
                    Self::summary_item(
                        &mut columns[1],
                        colors,
                        "PACE",
                        &format!("Every {interval}"),
                    );
                    Self::summary_item(&mut columns[2], colors, "TARGET", &target);
                    Self::summary_item(&mut columns[3], colors, "PLAN", &plan);
                });
            });
    }

    fn summary_item(ui: &mut egui::Ui, colors: ThemeColors, label: &str, value: &str) {
        ui.vertical(|ui| {
            ui.label(RichText::new(label).size(9.5).strong().color(colors.muted));
            ui.label(RichText::new(value).size(12.0).strong().color(colors.text));
        });
    }

    fn render_control_card(&mut self, ui: &mut egui::Ui) {
        let colors = theme_colors(self.theme);
        let running = self.running.load(Ordering::Acquire);
        let starting = self.starting.load(Ordering::Acquire);
        let status_color = if starting {
            colors.warning
        } else if running {
            colors.success
        } else {
            colors.accent
        };
        let status_label = if starting {
            "STARTING"
        } else if running {
            "RUNNING"
        } else {
            "IDLE"
        };

        Frame::none()
            .fill(colors.surface_alt)
            .stroke(Stroke::new(1.0_f32, colors.border))
            .rounding(Rounding::same(14.0))
            .inner_margin(egui::Margin::same(24.0))
            .show(ui, |ui| {
                let content_width = ui.available_width();
                ui.set_min_width(content_width);
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("CONTROL CENTER")
                                .size(10.5)
                                .strong()
                                .color(colors.muted),
                        );
                        ui.add_space(3.0);
                        ui.horizontal_top(|ui| {
                            Frame::none()
                                .fill(effect_color(status_color, 0.16))
                                .rounding(Rounding::same(7.0))
                                .inner_margin(egui::Margin::symmetric(12.0, 4.0))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new(status_label)
                                            .size(11.0)
                                            .strong()
                                            .color(status_color),
                                    );
                                });
                            ui.label(
                                RichText::new(if running {
                                    "Automation is active"
                                } else {
                                    "Ready when you are"
                                })
                                .size(15.0)
                                .strong()
                                .color(colors.text),
                            );
                        });
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(&self.last_status)
                                .size(11.5)
                                .color(colors.muted),
                        );
                    });

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let (label, fill) = if running {
                            (
                                format!("Stop clicking  ·  {}", self.toggle_hotkey.name()),
                                colors.danger,
                            )
                        } else {
                            (
                                format!("Start clicking  ·  {}", self.toggle_hotkey.name()),
                                colors.accent_fill,
                            )
                        };
                        let button = Button::new(RichText::new(label).size(15.0).strong())
                            .fill(fill)
                            .min_size(egui::vec2(218.0, 52.0));
                        if ui.add(button).clicked() {
                            self.toggle_clicking();
                        }
                    });
                });
            });
    }

    fn render_pattern_card(&mut self, ui: &mut egui::Ui) {
        let colors = theme_colors(self.theme);
        Self::card(ui, |ui| {
            Self::section_header(
                ui,
                "Click action",
                "Choose the burst size and the mouse button.",
            );
            ui.horizontal_wrapped(|ui| {
                for pattern in [
                    ClickPattern::Single,
                    ClickPattern::Double,
                    ClickPattern::Triple,
                    ClickPattern::Quadruple,
                    ClickPattern::Custom,
                ] {
                    let selected = self.pattern == pattern;
                    let fill = if selected {
                        colors.accent_fill
                    } else {
                        colors.surface_alt
                    };
                    let text_color = if selected { colors.text } else { colors.muted };
                    let count = pattern.resolved_count(self.custom_clicks);
                    let button = Button::new(
                        RichText::new(format!(
                            "{}\n{} click{}",
                            pattern.label(),
                            count,
                            if count == 1 { "" } else { "s" }
                        ))
                        .size(12.0)
                        .color(text_color)
                        .strong(),
                    )
                    .fill(fill)
                    .min_size(egui::vec2(104.0, 48.0));
                    if ui.add(button).clicked() {
                        self.pattern = pattern;
                    }
                }
            });

            if self.pattern == ClickPattern::Custom {
                self.custom_clicks = self
                    .custom_clicks
                    .clamp(MIN_CUSTOM_BURST_CLICKS, MAX_BURST_CLICKS);
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Clicks per burst")
                            .color(colors.text)
                            .strong(),
                    );
                    ui.add(
                        egui::DragValue::new(&mut self.custom_clicks)
                            .range(MIN_CUSTOM_BURST_CLICKS..=MAX_BURST_CLICKS)
                            .speed(1)
                            .suffix(" clicks"),
                    );
                });
                ui.label(
                    RichText::new(format!(
                        "Choose any burst from {} to {} physical clicks.",
                        MIN_CUSTOM_BURST_CLICKS, MAX_BURST_CLICKS
                    ))
                    .size(11.0)
                    .color(colors.muted),
                );
            }

            ui.add_space(14.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Mouse button").color(colors.muted));
                egui::ComboBox::from_id_salt("mouse_button")
                    .selected_text(self.button.label())
                    .width(150.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.button, MouseButton::Left, "Left button");
                        ui.selectable_value(&mut self.button, MouseButton::Right, "Right button");
                        ui.selectable_value(&mut self.button, MouseButton::Middle, "Middle button");
                    });
            });

            ui.add_space(11.0);
            let click_count = self.click_count();
            ui.label(
                RichText::new(format!(
                    "A {} cycle sends {} physical click{} as one burst.",
                    self.pattern.label().to_lowercase(),
                    click_count,
                    if click_count == 1 { "" } else { "s" }
                ))
                .size(11.5)
                .color(colors.muted),
            );
        });
    }

    fn render_timing_card(&mut self, ui: &mut egui::Ui) {
        let colors = theme_colors(self.theme);
        Self::card(ui, |ui| {
            Self::section_header(
                ui,
                "Timing",
                "Separate the burst rhythm from the repeat cadence.",
            );
            ui.horizontal(|ui| {
                ui.label(RichText::new("Start delay").color(colors.muted));
                ui.add(
                    egui::DragValue::new(&mut self.start_delay_seconds)
                        .range(0..=60)
                        .speed(1),
                );
                ui.label(RichText::new("seconds").color(colors.muted));
            });
            ui.label(
                RichText::new("Gives you time to move the cursor after pressing Start.")
                    .size(11.0)
                    .color(colors.muted),
            );
            ui.add_space(12.0);
            ui.label(RichText::new("Repeat interval").color(colors.text).strong());
            ui.label(
                RichText::new("Time from one click group to the next.")
                    .size(11.0)
                    .color(colors.muted),
            );
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                timing_field(ui, &mut self.interval_hours, "h", 23);
                timing_field(ui, &mut self.interval_minutes, "min", 59);
                timing_field(ui, &mut self.interval_seconds, "sec", 59);
                timing_field(ui, &mut self.interval_millis, "ms", 999);
            });
            let click_count = self.click_count();
            if click_count > 1 {
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Burst gap").color(colors.text).strong());
                    ui.add(
                        egui::DragValue::new(&mut self.burst_interval_millis)
                            .range(0..=500)
                            .speed(1)
                            .suffix(" ms"),
                    );
                });
                ui.label(
                    RichText::new(if self.burst_interval_millis() == 0 {
                        "Turbo mode: the full burst is submitted in one Windows input batch."
                    } else {
                        "Short gap between physical clicks inside the selected burst."
                    })
                    .size(11.0)
                    .color(colors.muted),
                );
            }
            ui.add_space(9.0);
            let interval = self.effective_cycle_millis();
            let groups_per_second = 1_000.0 / interval as f64;
            let clicks_per_second = groups_per_second * click_count as f64;
            ui.label(
                RichText::new(format!(
                    "Estimated pace: {groups_per_second:.2} cycle(s)/s · {click_count} click{} / cycle · {clicks_per_second:.0} click{} / s",
                    if click_count == 1 { "" } else { "s" },
                    if clicks_per_second.round() == 1.0 { "" } else { "s" },
                ))
                .size(11.0)
                .color(colors.muted),
            );
        });
    }

    fn render_target_card(&mut self, ui: &mut egui::Ui) {
        let colors = theme_colors(self.theme);
        Self::card(ui, |ui| {
            Self::section_header(
                ui,
                "Target",
                "Use the cursor or lock clicking to a screen position.",
            );
            ui.radio_value(
                &mut self.target_mode,
                TargetMode::CurrentCursor,
                "Use current cursor position",
            );
            ui.radio_value(
                &mut self.target_mode,
                TargetMode::FixedPosition,
                "Use a fixed screen position",
            );
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("X").color(colors.muted));
                ui.add(
                    egui::DragValue::new(&mut self.target.x)
                        .range(-32_000..=32_000)
                        .speed(1),
                );
                ui.label(RichText::new("Y").color(colors.muted));
                ui.add(
                    egui::DragValue::new(&mut self.target.y)
                        .range(-32_000..=32_000)
                        .speed(1),
                );
                if ui.button("Capture now (F9)").clicked() {
                    self.capture_position();
                }
            });
            ui.label(
                RichText::new("Best workflow: move to the target, then press F9 to lock it.")
                    .size(11.0)
                    .color(colors.muted),
            );
        });
    }

    fn render_repeat_card(&mut self, ui: &mut egui::Ui) {
        let colors = theme_colors(self.theme);
        Self::card(ui, |ui| {
            Self::section_header(
                ui,
                "Repeat",
                "Run continuously or stop after a set number of cycles.",
            );
            ui.radio_value(
                &mut self.repeat_mode,
                RepeatMode::UntilStopped,
                "Until I stop it",
            );
            ui.radio_value(
                &mut self.repeat_mode,
                RepeatMode::FixedCount,
                "Stop after a fixed number of cycles",
            );
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Cycles").color(colors.muted));
                ui.add_enabled(
                    self.repeat_mode == RepeatMode::FixedCount,
                    egui::DragValue::new(&mut self.repeat_count)
                        .range(1..=10_000_000)
                        .speed(1),
                );
                if self.repeat_mode == RepeatMode::FixedCount {
                    ui.label(
                        RichText::new(format!(
                            "{} total physical clicks",
                            self.repeat_count.max(1) * self.click_count() as u64
                        ))
                        .size(11.0)
                        .color(colors.muted),
                    );
                }
            });
        });
    }

    fn render_options_card(&mut self, ui: &mut egui::Ui) {
        let colors = theme_colors(self.theme);
        Self::card(ui, |ui| {
            Self::section_header(
                ui,
                "Preferences",
                "Personalize the appearance and keyboard controls.",
            );
            ui.horizontal_wrapped(|ui| {
                ui.checkbox(&mut self.always_on_top, "Keep PulseClick on top");
                ui.checkbox(&mut self.show_click_animation, "Show click indicator");
            });
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Click feedback")
                        .size(11.0)
                        .color(colors.muted),
                );
                if ui
                    .add_enabled(
                        self.show_click_animation,
                        Button::new("Preview indicator").min_size(egui::vec2(132.0, 28.0)),
                    )
                    .clicked()
                {
                    self.preview_click_indicator();
                }
                ui.label(
                    RichText::new("A live marker appears at the cursor after each click.")
                        .size(11.0)
                        .color(colors.muted),
                );
            });
            self.render_indicator_preview(ui);
            let (indicator_status, indicator_color) = match win32::click_indicator_status() {
                1 => ("Desktop marker: starting…", colors.warning),
                2 => ("Desktop marker: ready", colors.success),
                3..=5 => (
                    "Desktop marker unavailable · in-app preview still works",
                    colors.warning,
                ),
                _ => ("Desktop marker: ready on first click", colors.muted),
            };
            ui.label(
                RichText::new(indicator_status)
                    .size(10.5)
                    .color(indicator_color),
            );

            ui.add_space(10.0);
            ui.label(
                RichText::new("Start / stop hotkey")
                    .color(colors.text)
                    .strong(),
            );
            ui.label(
                RichText::new("Press the same key to start and stop. F6 is the default.")
                    .size(11.0)
                    .color(colors.muted),
            );
            ui.add_space(3.0);
            ui.horizontal(|ui| {
                let mut selected_key = self.toggle_hotkey;
                egui::ComboBox::from_id_salt("toggle_hotkey")
                    .selected_text(self.toggle_hotkey.name())
                    .width(132.0)
                    .show_ui(ui, |ui| {
                        for key in egui::Key::ALL.iter().copied().filter(|key| {
                            hotkey_virtual_key(*key).is_some() && !hotkey_is_reserved(*key)
                        }) {
                            ui.selectable_value(&mut selected_key, key, key.name());
                        }
                    });
                if selected_key != self.toggle_hotkey {
                    self.set_toggle_hotkey(selected_key);
                }

                let record_label = if self.recording_hotkey {
                    "Press a key…"
                } else {
                    "Record key"
                };
                if ui
                    .add_enabled(
                        !self.recording_hotkey,
                        Button::new(record_label).min_size(egui::vec2(104.0, 30.0)),
                    )
                    .clicked()
                {
                    self.recording_hotkey = true;
                    self.last_status = "Press any supported key to assign start/stop.".to_string();
                }
                if self.recording_hotkey && ui.button("Cancel").clicked() {
                    self.recording_hotkey = false;
                }
            });
            ui.label(
                RichText::new(
                    "F8 stays reserved for emergency stop · F9 stays reserved for target capture.",
                )
                .size(11.0)
                .color(colors.muted),
            );

            ui.add_space(10.0);
            ui.label(RichText::new("Theme").color(colors.text).strong());
            ui.horizontal(|ui| {
                for theme in [ThemeMode::Dark, ThemeMode::Light] {
                    let selected = self.theme == theme;
                    let fill = if selected {
                        colors.accent_fill
                    } else {
                        colors.surface_alt
                    };
                    let text_color = if selected { colors.text } else { colors.muted };
                    if ui
                        .add(
                            Button::new(RichText::new(theme.label()).color(text_color).strong())
                                .fill(fill)
                                .min_size(egui::vec2(84.0, 30.0)),
                        )
                        .clicked()
                    {
                        self.theme = theme;
                    }
                }
            });

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                let hotkeys_active = self.hotkeys_available.load(Ordering::Acquire);
                let (hotkey_color, hotkey_text) = if hotkeys_active {
                    (colors.success, "Global hotkeys active")
                } else {
                    (colors.warning, "One or more global hotkeys are unavailable")
                };
                Frame::none()
                    .fill(effect_color(hotkey_color, 0.14))
                    .rounding(Rounding::same(6.0))
                    .inner_margin(egui::Margin::symmetric(9.0, 4.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(hotkey_text)
                                .size(11.0)
                                .color(hotkey_color)
                                .strong(),
                        );
                    });
            });
            ui.add_space(7.0);
            ui.label(
                RichText::new(format!(
                    "{} start/stop   ·   F8 emergency stop   ·   F9 capture target",
                    self.toggle_hotkey.name()
                ))
                .size(11.0)
                .color(colors.muted),
            );
        });
    }

    fn render_indicator_preview(&mut self, ui: &mut egui::Ui) {
        const PREVIEW_LIFETIME_SECONDS: f32 = 2.0;
        let colors = theme_colors(self.theme);
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 112.0),
            egui::Sense::hover(),
        );
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, Rounding::same(9.0), colors.surface_soft);

        let Some(started) = self.preview_animation_started else {
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                "Preview the click indicator",
                egui::FontId::proportional(12.0),
                colors.muted,
            );
            return;
        };
        let progress = (started.elapsed().as_secs_f32() / PREVIEW_LIFETIME_SECONDS).clamp(0.0, 1.0);
        let finished = progress >= 1.0;
        if !finished {
            ui.ctx().request_repaint_after(Duration::from_millis(16));
        }

        let center = rect.center();
        let scale = (rect.width().min(rect.height()) / 176.0).max(0.5);
        let eased = 1.0 - (1.0 - progress).powi(3);
        let fade = (1.0 - progress).powf(1.22);
        let flash = (1.0 - progress / 0.2).clamp(0.0, 1.0);
        let rotation = -0.75 + progress * 1.75;
        let outer_radius = (23.0 + 47.0 * eased) * scale;
        let inner_radius = (8.0 + 22.0 * eased) * scale;
        let bracket_radius = (34.0 + 5.0 * (1.0 - eased)) * scale;
        let color = click_indicator_color(self.theme, self.button);

        painter.circle_filled(
            center,
            outer_radius + 5.0 * scale,
            effect_color(color, fade * 0.05),
        );
        painter.circle_stroke(
            center,
            outer_radius,
            Stroke::new(2.0 * scale, effect_color(color, fade * 0.9)),
        );
        painter.circle_stroke(
            center,
            inner_radius,
            Stroke::new(3.0 * scale, effect_color(color, fade)),
        );
        for segment in 0..4 {
            let start = rotation + segment as f32 * std::f32::consts::TAU / 4.0 + 0.16;
            painter.add(egui::Shape::line(
                arc_points(
                    center,
                    bracket_radius,
                    start,
                    std::f32::consts::TAU * 0.13,
                    12,
                ),
                Stroke::new(3.0 * scale, effect_color(color, fade)),
            ));
        }
        let orbit_angle = rotation + std::f32::consts::TAU * 0.13;
        let orbit_point = center + egui::vec2(orbit_angle.cos(), orbit_angle.sin()) * outer_radius;
        painter.circle_filled(orbit_point, 3.5 * scale, effect_color(color, fade));
        painter.circle_filled(
            center,
            (4.0 + 5.0 * flash) * scale,
            effect_color(color, (0.55 + 0.45 * fade) * flash),
        );
        painter.circle_filled(
            center,
            (1.5 + 1.5 * flash) * scale,
            effect_color(Color32::WHITE, 0.95 * fade),
        );

        if finished {
            self.preview_animation_started = None;
        }
    }

    fn render_click_effect(&mut self, _ctx: &egui::Context) {
        if !self.show_click_animation {
            win32::hide_click_indicator();
        }
    }
}

impl eframe::App for PulseClickApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_recorded_hotkey(ctx);
        self.process_hotkeys();
        self.process_worker_events();
        self.cleanup_finished_worker();

        if self.theme != self.applied_theme {
            apply_theme(ctx, self.theme);
            self.applied_theme = self.theme;
        }

        if self.always_on_top != self.applied_always_on_top {
            let window_level = if self.always_on_top {
                egui::WindowLevel::AlwaysOnTop
            } else {
                egui::WindowLevel::Normal
            };
            ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(window_level));
            self.applied_always_on_top = self.always_on_top;
        }

        ctx.request_repaint_after(Duration::from_millis(50));
        self.render_click_effect(ctx);
        let colors = theme_colors(self.theme);

        egui::CentralPanel::default()
            .frame(
                Frame::none()
                    .fill(colors.background)
                    .inner_margin(egui::Margin::symmetric(0.0, 24.0)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let available_width = ui.available_width();
                        let content_width = available_width.min(CONTENT_MAX_WIDTH);
                        let side_gutter = ((available_width - content_width) * 0.5).max(0.0);

                        ui.allocate_ui_with_layout(
                            egui::vec2(available_width, 0.0),
                            Layout::left_to_right(Align::TOP),
                            |ui| {
                                ui.spacing_mut().item_spacing.x = 0.0;
                                ui.add_space(side_gutter);
                                ui.allocate_ui_with_layout(
                                    egui::vec2(content_width, 0.0),
                                    Layout::top_down(Align::Min),
                                    |ui| {
                        ui.horizontal_top(|ui| {
                            let (logo_rect, _) = ui.allocate_exact_size(
                                egui::vec2(42.0, 42.0),
                                egui::Sense::hover(),
                            );
                            ui.painter().rect_filled(
                                logo_rect,
                                Rounding::same(11.0),
                                Color32::from_rgb(3, 3, 3),
                            );
                            let logo_center = logo_rect.center();
                            ui.painter().circle_stroke(
                                logo_center,
                                13.0,
                                Stroke::new(1.2_f32, Color32::from_rgb(232, 232, 226)),
                            );
                            ui.painter().circle_stroke(
                                logo_center,
                                7.0,
                                Stroke::new(1.2_f32, Color32::from_rgb(172, 172, 166)),
                            );
                            ui.painter().circle_filled(
                                logo_center,
                                2.8,
                                Color32::from_rgb(248, 248, 242),
                            );
                            ui.painter().line_segment(
                                [
                                    logo_center + egui::vec2(9.5, -9.5),
                                    logo_center + egui::vec2(14.0, -14.0),
                                ],
                                Stroke::new(1.6_f32, Color32::from_rgb(248, 248, 242)),
                            );
                            ui.add_space(2.0);
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new("PulseClick")
                                        .size(28.0)
                                        .strong()
                                        .color(colors.text),
                                );
                                ui.label(
                                    RichText::new("Precision automation for Windows")
                                        .size(12.0)
                                        .color(colors.muted),
                                );
                            });
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(
                                    RichText::new(format!(
                                        "Desktop edition  ·  {} control  ·  F8 stop",
                                        self.toggle_hotkey.name()
                                    ))
                                        .size(11.0)
                                        .color(colors.muted),
                                );
                            });
                        });

                        ui.add_space(20.0);
                        self.render_control_card(ui);
                        ui.add_space(16.0);
                        self.render_overview_strip(ui);
                        ui.add_space(16.0);
                        let column_gap = 16.0;
                        let column_width = ((ui.available_width() - column_gap) * 0.5).max(0.0);
                        ui.horizontal_top(|ui| {
                            ui.spacing_mut().item_spacing.x = column_gap;
                            ui.allocate_ui_with_layout(
                                egui::vec2(column_width, 0.0),
                                Layout::top_down(Align::Min),
                                |ui| self.render_pattern_card(ui),
                            );
                            ui.allocate_ui_with_layout(
                                egui::vec2(column_width, 0.0),
                                Layout::top_down(Align::Min),
                                |ui| self.render_timing_card(ui),
                            );
                        });
                        ui.add_space(16.0);
                        let column_gap = 16.0;
                        let column_width = ((ui.available_width() - column_gap) * 0.5).max(0.0);
                        ui.horizontal_top(|ui| {
                            ui.spacing_mut().item_spacing.x = column_gap;
                            ui.allocate_ui_with_layout(
                                egui::vec2(column_width, 0.0),
                                Layout::top_down(Align::Min),
                                |ui| self.render_target_card(ui),
                            );
                            ui.allocate_ui_with_layout(
                                egui::vec2(column_width, 0.0),
                                Layout::top_down(Align::Min),
                                |ui| self.render_repeat_card(ui),
                            );
                        });
                        ui.add_space(16.0);
                        self.render_options_card(ui);
                        ui.add_space(16.0);
                        ui.label(
                            RichText::new(
                                "Tip: a burst is the selected click count. Set Burst gap to 0 ms for the fastest batch mode; the repeat interval starts after the burst finishes.",
                            )
                            .size(10.5)
                            .color(colors.muted),
                        );
                                    },
                                );
                            },
                        );
                    });
            });
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.stop_clicking();
    }
}

fn click_indicator_color(theme: ThemeMode, button: MouseButton) -> Color32 {
    match theme {
        ThemeMode::Dark => match button {
            MouseButton::Left => Color32::from_rgb(248, 248, 242),
            MouseButton::Right => Color32::from_rgb(194, 198, 202),
            MouseButton::Middle => Color32::from_rgb(145, 150, 154),
        },
        ThemeMode::Light => match button {
            MouseButton::Left => Color32::from_rgb(25, 25, 24),
            MouseButton::Right => Color32::from_rgb(78, 78, 76),
            MouseButton::Middle => Color32::from_rgb(124, 124, 119),
        },
    }
}

fn arc_points(
    center: egui::Pos2,
    radius: f32,
    start_angle: f32,
    sweep_angle: f32,
    segments: usize,
) -> Vec<egui::Pos2> {
    (0..=segments)
        .map(|index| {
            let progress = index as f32 / segments.max(1) as f32;
            let angle = start_angle + sweep_angle * progress;
            center + egui::vec2(angle.cos(), angle.sin()) * radius
        })
        .collect()
}

fn effect_color(color: Color32, alpha: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(
        color.r(),
        color.g(),
        color.b(),
        (alpha.clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}

fn timing_field(ui: &mut egui::Ui, value: &mut u32, label: &str, max: u32) {
    let colors = theme_colors(if ui.visuals().dark_mode {
        ThemeMode::Dark
    } else {
        ThemeMode::Light
    });
    ui.vertical(|ui| {
        ui.add(
            egui::DragValue::new(value)
                .range(0..=max)
                .speed(1)
                .fixed_decimals(0),
        );
        ui.label(RichText::new(label).size(10.5).color(colors.muted));
    });
}

fn format_interval(milliseconds: u64) -> String {
    if milliseconds >= 3_600_000 {
        format!(
            "{}h {}m",
            milliseconds / 3_600_000,
            (milliseconds / 60_000) % 60
        )
    } else if milliseconds >= 60_000 {
        format!(
            "{}m {}s",
            milliseconds / 60_000,
            (milliseconds / 1_000) % 60
        )
    } else if milliseconds >= 1_000 {
        format!("{}.{:03}s", milliseconds / 1_000, milliseconds % 1_000)
    } else {
        format!("{milliseconds}ms")
    }
}

fn run_clicker(
    settings: ClickSettings,
    stop: Arc<AtomicBool>,
    starting: Arc<AtomicBool>,
    events: &Sender<WorkerEvent>,
) -> WorkerOutcome {
    if !wait_interruptible(settings.start_delay, &stop) {
        starting.store(false, Ordering::Release);
        return WorkerOutcome::Stopped;
    }
    starting.store(false, Ordering::Release);

    let mut completed_cycles = 0_u64;
    let mut last_visual_at: Option<Instant> = None;
    loop {
        if stop.load(Ordering::Acquire) {
            return WorkerOutcome::Stopped;
        }

        if settings.target_mode == TargetMode::FixedPosition
            && !win32::set_cursor_position(settings.fixed_x, settings.fixed_y)
        {
            return WorkerOutcome::InputError;
        }

        let click_position = match settings.target_mode {
            TargetMode::FixedPosition => Some((settings.fixed_x, settings.fixed_y)),
            TargetMode::CurrentCursor => win32::cursor_position(),
        };

        if settings.burst_interval.is_zero() {
            // With no gap requested, submit the whole burst in one Windows
            // input batch. This is substantially faster than making one API
            // call per physical click while preserving the down/up ordering.
            if !win32::send_clicks(settings.button, settings.click_count) {
                return WorkerOutcome::InputError;
            }
            emit_click_visual(click_position, settings.button, &mut last_visual_at, events);
        } else {
            for click_index in 0..settings.click_count {
                if stop.load(Ordering::Acquire) {
                    return WorkerOutcome::Stopped;
                }
                if !win32::send_click(settings.button) {
                    return WorkerOutcome::InputError;
                }
                emit_click_visual(click_position, settings.button, &mut last_visual_at, events);

                if click_index + 1 < settings.click_count
                    && !wait_interruptible(settings.burst_interval, &stop)
                {
                    return WorkerOutcome::Stopped;
                }
            }
        }

        completed_cycles = completed_cycles.saturating_add(1);
        if settings.repeat_mode == RepeatMode::FixedCount
            && completed_cycles >= settings.repeat_count
        {
            return WorkerOutcome::Completed(completed_cycles);
        }

        if !wait_interruptible(settings.interval, &stop) {
            return WorkerOutcome::Stopped;
        }
    }
}

fn emit_click_visual(
    click_position: Option<(i32, i32)>,
    button: MouseButton,
    last_visual_at: &mut Option<Instant>,
    events: &Sender<WorkerEvent>,
) {
    if last_visual_at
        .map(|last| last.elapsed() >= Duration::from_millis(16))
        .unwrap_or(true)
    {
        if let Some((x, y)) = click_position {
            let _ = events.send(WorkerEvent::Click { x, y, button });
        }
        *last_visual_at = Some(Instant::now());
    }
}

fn wait_interruptible(duration: Duration, stop: &AtomicBool) -> bool {
    let deadline = Instant::now() + duration;
    while Instant::now() < deadline {
        if stop.load(Ordering::Acquire) {
            return false;
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        thread::sleep(remaining.min(Duration::from_millis(4)));
    }
    !stop.load(Ordering::Acquire)
}

fn app_icon() -> egui::IconData {
    let image = image::load_from_memory(include_bytes!("../assets/pulseclick-256.png"))
        .expect("embedded PulseClick icon should be a valid PNG")
        .to_rgba8();
    egui::IconData {
        width: image.width(),
        height: image.height(),
        rgba: image.into_raw(),
    }
}

fn main() -> eframe::Result<()> {
    let toggle_request = Arc::new(AtomicUsize::new(0));
    let stop_request = Arc::new(AtomicBool::new(false));
    let capture_request = Arc::new(AtomicBool::new(false));
    let hotkeys_available = Arc::new(AtomicBool::new(false));
    let toggle_hotkey_code = Arc::new(AtomicU32::new(
        hotkey_virtual_key(egui::Key::F6).expect("F6 must have a Windows virtual-key code"),
    ));

    win32::spawn_hotkey_listener(
        Arc::clone(&toggle_request),
        Arc::clone(&stop_request),
        Arc::clone(&capture_request),
        Arc::clone(&hotkeys_available),
        Arc::clone(&toggle_hotkey_code),
    );

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("PulseClick")
            .with_icon(app_icon())
            .with_inner_size([1120.0, 880.0])
            .with_min_inner_size([980.0, 760.0]),
        ..Default::default()
    };

    eframe::run_native(
        "PulseClick",
        options,
        Box::new(move |cc| {
            Ok(Box::new(PulseClickApp::new(
                cc,
                toggle_request,
                stop_request,
                capture_request,
                hotkeys_available,
                toggle_hotkey_code,
            )))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn click_patterns_map_to_their_physical_click_counts() {
        assert_eq!(ClickPattern::Single.count(), 1);
        assert_eq!(ClickPattern::Double.count(), 2);
        assert_eq!(ClickPattern::Triple.count(), 3);
        assert_eq!(ClickPattern::Quadruple.count(), 4);
        assert_eq!(ClickPattern::Custom.resolved_count(5), 5);
        assert_eq!(ClickPattern::Custom.resolved_count(6), 6);
        assert_eq!(
            ClickPattern::Custom.resolved_count(0),
            MIN_CUSTOM_BURST_CLICKS as usize
        );
        assert_eq!(
            ClickPattern::Custom.resolved_count(MAX_BURST_CLICKS + 1),
            MAX_BURST_CLICKS as usize
        );
    }

    #[test]
    fn f6_is_the_default_toggle_and_f8_f9_are_reserved() {
        assert_eq!(hotkey_virtual_key(egui::Key::F6), Some(0x75));
        assert!(hotkey_is_reserved(egui::Key::F8));
        assert!(hotkey_is_reserved(egui::Key::F9));
        assert!(!hotkey_is_reserved(egui::Key::F7));
    }

    #[test]
    fn interval_summary_stays_compact() {
        assert_eq!(format_interval(35), "35ms");
        assert_eq!(format_interval(1_250), "1.250s");
        assert_eq!(format_interval(65_000), "1m 5s");
    }
}
