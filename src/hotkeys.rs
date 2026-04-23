use rdev::{listen, Event, EventType, Key};
use std::sync::mpsc::Sender;

#[derive(Debug, Clone)]
pub enum HotkeyEvent {
    Triggered,
}

/// Parse a hotkey string such as "F14", "Alt+C", "Ctrl+F13" into an
/// `rdev::Key` and optional modifier requirement.
#[derive(Debug, Clone)]
pub struct HotkeySpec {
    pub key: Key,
    pub require_alt: bool,
    pub require_ctrl: bool,
}

impl HotkeySpec {
    pub fn parse(spec: &str) -> Option<Self> {
        let parts: Vec<&str> = spec.split('+').collect();
        let (modifiers, key_str) = parts.split_at(parts.len().saturating_sub(1));

        let key_name = key_str.first().copied()?.trim();
        let key = parse_key(key_name)?;

        let require_alt = modifiers.iter().any(|m| m.eq_ignore_ascii_case("alt"));
        let require_ctrl = modifiers.iter().any(|m| m.eq_ignore_ascii_case("ctrl"));

        Some(HotkeySpec {
            key,
            require_alt,
            require_ctrl,
        })
    }
}

fn parse_key(name: &str) -> Option<Key> {
    match name {
        "F1" => Some(Key::F1),
        "F2" => Some(Key::F2),
        "F3" => Some(Key::F3),
        "F4" => Some(Key::F4),
        "F5" => Some(Key::F5),
        "F6" => Some(Key::F6),
        "F7" => Some(Key::F7),
        "F8" => Some(Key::F8),
        "F9" => Some(Key::F9),
        "F10" => Some(Key::F10),
        "F11" => Some(Key::F11),
        "F12" => Some(Key::F12),
        // F13-F24 require rdev's Unknown scancode mapping on Windows.
        // Virtual key codes: F13=124, F14=125, ... F24=135.
        "F13" => Some(Key::Unknown(124)),
        "F14" => Some(Key::Unknown(125)),
        "F15" => Some(Key::Unknown(126)),
        "F16" => Some(Key::Unknown(127)),
        "F17" => Some(Key::Unknown(128)),
        "F18" => Some(Key::Unknown(129)),
        "F19" => Some(Key::Unknown(130)),
        "F20" => Some(Key::Unknown(131)),
        "F21" => Some(Key::Unknown(132)),
        "F22" => Some(Key::Unknown(133)),
        "F23" => Some(Key::Unknown(134)),
        "F24" => Some(Key::Unknown(135)),
        // Single character keys
        k if k.len() == 1 => {
            let c = k.chars().next()?;
            match c {
                'a'..='z' | 'A'..='Z' => Some(Key::Unknown(c.to_ascii_uppercase() as u32)),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Spawn a background thread that listens for global keyboard events.
/// Sends `HotkeyEvent::Triggered` over `tx` whenever the configured hotkey fires.
/// The thread runs for the lifetime of the process — there is no shutdown signal.
pub fn spawn_listener(spec: HotkeySpec, tx: Sender<HotkeyEvent>) {
    std::thread::spawn(move || {
        let mut alt_down = false;
        let mut ctrl_down = false;

        let callback = move |event: Event| {
            match &event.event_type {
                EventType::KeyPress(key) => {
                    match key {
                        Key::Alt | Key::AltGr => alt_down = true,
                        Key::ControlLeft | Key::ControlRight => ctrl_down = true,
                        _ => {}
                    }

                    let modifiers_ok = (!spec.require_alt || alt_down)
                        && (!spec.require_ctrl || ctrl_down);

                    if modifiers_ok && keys_match(key, &spec.key) {
                        if let Err(e) = tx.send(HotkeyEvent::Triggered) {
                            log::error!("Failed to send hotkey event: {}", e);
                        }
                    }
                }
                EventType::KeyRelease(key) => match key {
                    Key::Alt | Key::AltGr => alt_down = false,
                    Key::ControlLeft | Key::ControlRight => ctrl_down = false,
                    _ => {}
                },
                _ => {}
            }
        };

        if let Err(e) = listen(callback) {
            log::error!("rdev listen error: {:?}", e);
        }
    });
}

fn keys_match(pressed: &Key, target: &Key) -> bool {
    match (pressed, target) {
        (Key::Unknown(a), Key::Unknown(b)) => a == b,
        _ => pressed == target,
    }
}
