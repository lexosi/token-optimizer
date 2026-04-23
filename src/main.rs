#![windows_subsystem = "windows"]

mod ai_client;
mod clipboard;
mod config;
mod hotkeys;

use config::Config;
use hotkeys::{HotkeyEvent, HotkeySpec};
use rdev::{simulate, EventType, Key};
use std::sync::mpsc;
use std::time::Duration;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    TrayIconBuilder,
};

fn main() {
    // Install a panic hook that writes to stderr AND to crash.log next to the
    // binary, so the cause is captured even if the console window closes first.
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("[PANIC] {}", info);
        eprintln!("{}", msg);
        let log_path = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("crash.log")))
            .unwrap_or_else(|| std::path::PathBuf::from("crash.log"));
        let _ = std::fs::write(&log_path, &msg);
        eprintln!("[diag] crash.log written to {:?}", log_path);
    }));

    eprintln!("[diag] main() entered — panic hook installed");

    let result = std::panic::catch_unwind(|| {
        run();
    });

    if let Err(e) = result {
        let msg = if let Some(s) = e.downcast_ref::<&str>() {
            format!("panic: {}", s)
        } else if let Some(s) = e.downcast_ref::<String>() {
            format!("panic: {}", s)
        } else {
            "panic: (non-string payload)".to_string()
        };
        eprintln!("[diag] caught panic — {}", msg);
        std::process::exit(101);
    }
}

fn run() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
    eprintln!("[diag] logger initialised");

    let cfg = Config::load().unwrap_or_else(|e| {
        log::error!("Failed to load config: {} — using defaults", e);
        Config::default()
    });
    eprintln!("[diag] config loaded — hotkey={} auto_trigger={}", cfg.hotkey, cfg.auto_trigger);

    let spec = match HotkeySpec::parse(&cfg.hotkey) {
        Some(s) => s,
        None => {
            eprintln!("[diag] FATAL: unrecognised hotkey '{}'", cfg.hotkey);
            std::process::exit(1);
        }
    };
    eprintln!("[diag] hotkey spec parsed");

    // Hotkey listener and compression run on a background thread so the main
    // thread is free to drive the Win32 message queue for the tray icon.
    std::thread::spawn(move || {
        let (tx, rx) = mpsc::channel::<HotkeyEvent>();
        hotkeys::spawn_listener(spec, tx);
        eprintln!("[diag] hotkey listener spawned");
        loop {
            match rx.recv() {
                Ok(HotkeyEvent::Triggered) => {
                    let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
                    log::info!("[{}] Hotkey triggered", ts);
                    run_compression(&cfg);
                }
                Err(e) => {
                    log::error!("Hotkey channel error: {} — exiting thread", e);
                    break;
                }
            }
        }
    });

    eprintln!("[diag] loading PNG icon...");
    let icon = load_icon();
    eprintln!("[diag] icon ready");

    eprintln!("[diag] building menu...");
    let label = MenuItem::new("Token Optimizer \u{2014} Running", false, None);
    let quit = MenuItem::new("Quit", true, None);
    let quit_id = quit.id().clone();
    let menu = Menu::new();
    menu.append(&label).unwrap();
    menu.append(&quit).unwrap();
    eprintln!("[diag] menu built");

    eprintln!("[diag] calling TrayIconBuilder::build()...");
    let _tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Token Optimizer")
        .with_icon(icon)
        .build()
        .unwrap_or_else(|e| {
            eprintln!("[diag] FATAL: TrayIconBuilder::build() failed: {:?}", e);
            std::process::exit(1);
        });
    eprintln!("[diag] tray icon created — entering message loop");

    // Pump the Win32 message queue on the main thread.
    // tray-icon creates a message-only HWND on the calling thread; messages are
    // only delivered when GetMessage/DispatchMessage runs on that same thread.
    // After each message is dispatched, drain the muda MenuEvent channel.
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, TranslateMessage, MSG,
        };
        let mut msg: MSG = std::mem::zeroed();
        eprintln!("[diag] GetMessageW loop starting");
        loop {
            let ret = GetMessageW(&mut msg, 0, 0, 0);
            if ret == 0 {
                eprintln!("[diag] WM_QUIT received — exiting message loop");
                break;
            }
            if ret != -1 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            // muda sends MenuEvent to a crossbeam channel inside WndProc;
            // drain it immediately after dispatch so we never miss a click.
            while let Ok(ev) = MenuEvent::receiver().try_recv() {
                if ev.id == quit_id {
                    eprintln!("[diag] Quit clicked — exiting");
                    std::process::exit(0);
                }
            }
        }
    }
    eprintln!("[diag] run() returning");
}

/// Load the tray icon from `Token Optimizer.png`.
/// Searches next to the binary first (deployed layout), then the working
/// directory (development / `cargo run`). Falls back to a 16×16 blue square
/// so the app always starts even if the PNG is missing.
fn load_icon() -> tray_icon::Icon {
    let paths = [
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("Token Optimizer.png")))
            .unwrap_or_default(),
        std::path::PathBuf::from("Token Optimizer.png"),
    ];

    for path in &paths {
        if path.exists() {
            if let Ok(img) = image::open(path) {
                let rgba = img.into_rgba8();
                let (w, h) = rgba.dimensions();
                if let Ok(icon) = tray_icon::Icon::from_rgba(rgba.into_raw(), w, h) {
                    return icon;
                }
            }
        }
    }

    log::warn!("Token Optimizer.png not found — using fallback icon");
    let size = 16u32;
    let pixels: Vec<u8> = (0..size * size)
        .flat_map(|_| [0u8, 120u8, 215u8, 255u8])
        .collect();
    tray_icon::Icon::from_rgba(pixels, size, size).expect("Failed to create fallback icon")
}

fn simulate_copy() {
    let sequence = [
        EventType::KeyPress(Key::ControlLeft),
        EventType::KeyPress(Key::KeyC),
        EventType::KeyRelease(Key::KeyC),
        EventType::KeyRelease(Key::ControlLeft),
    ];
    for event in &sequence {
        if let Err(e) = simulate(event) {
            log::warn!("simulate Ctrl+C: {:?}", e);
        }
    }
    std::thread::sleep(Duration::from_millis(200));
}

fn run_compression(cfg: &Config) {
    simulate_copy();

    let text = match clipboard::read_text() {
        Ok(t) if t.is_empty() => {
            log::warn!("Clipboard is empty — skipping");
            return;
        }
        Ok(t) => t,
        Err(e) => {
            log::error!("Failed to read clipboard: {}", e);
            return;
        }
    };

    log::info!("Compressing {} chars...", text.len());

    match ai_client::compress(
        &cfg.lm_studio_url,
        &cfg.model,
        &cfg.system_prompt,
        cfg.max_tokens,
        &text,
    ) {
        Ok(compressed) => match clipboard::write_text(&compressed) {
            Ok(()) => {
                let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
                log::info!(
                    "[{}] Compression done: {} \u{2192} {} chars",
                    ts,
                    text.len(),
                    compressed.len()
                );
            }
            Err(e) => log::error!("Failed to write clipboard: {}", e),
        },
        Err(e) => log::error!("AI compression failed: {}", e),
    }
}
