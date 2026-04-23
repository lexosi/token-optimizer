# token-optimizer — Claude Code Context

## Project purpose

A lightweight Rust daemon that monitors the clipboard and uses a local LLM (LM Studio) to compress/distill text before it gets pasted into an AI assistant — reducing token consumption without losing meaning. Triggered by hotkey (F14 by default) or automatically when clipboard content exceeds a configurable size threshold.

---

## Repository

- **Main repo:** https://github.com/lexosi/token-optimizer
- **Project path:** `F:\proyectosprog\token-optimizer\`

---

## Project structure

```
token-optimizer/
├── Cargo.toml              single crate binary
├── Cargo.lock
├── CLAUDE.md               this file
├── README.md               user-facing documentation
├── .gitignore
├── config.toml.example     template — copy to config.toml and fill in
└── src/
    ├── main.rs             entry point — panic hook, tray icon, Win32 message loop
    ├── config.rs           Config struct, TOML loading, defaults
    ├── hotkeys.rs          global hotkey listener (rdev) — F13-F24, Alt+X, Ctrl+X
    ├── clipboard.rs        read/write clipboard text (arboard)
    └── ai_client.rs        HTTP POST to LM Studio local API (OpenAI-compatible)
```

---

## Implementation status

All modules are implemented and compiling cleanly.

| Module | Status | Notes |
|--------|--------|-------|
| `config.rs` | Done | Config struct, TOML load from binary dir, defaults, unit tests |
| `hotkeys.rs` | Done | rdev listen, F1–F24 (F13–F24 via Unknown VK codes), mpsc channel |
| `clipboard.rs` | Done | arboard read/write text |
| `ai_client.rs` | Done | ureq POST, temperature=0, OpenAI-compatible |
| `main.rs` | Done | System tray (tray-icon), Win32 message loop, background hotkey thread |

---

## Module responsibilities

### `config.rs`
- `Config` struct with all fields
- Load from `config.toml` next to the binary (not the working directory)
- Provide sensible defaults for all optional fields
- Unit tests for every default value

### `hotkeys.rs`
- Global hotkey listener using `rdev` crate (non-blocking, runs in its own thread)
- Supports: standard combos (`Alt+C`, `Ctrl+X`), function keys F1–F24
- F13–F24 mapped via `Key::Unknown(VK_code)` — these are the Windows virtual key codes used by gaming mice/keyboards
- Sends `HotkeyEvent::Triggered` through a `std::sync::mpsc` channel to the background thread
- Does NOT intercept or consume keypresses (rdev `listen` mode only)

### `clipboard.rs`
- `read_text()` — reads current clipboard text via arboard
- `write_text(s)` — writes compressed result back to clipboard

### `ai_client.rs`
- `compress(base_url, model, system_prompt, max_tokens, text)` → `Result<String, AiError>`
- POST to `{lm_studio_url}/v1/chat/completions`
- Request includes `temperature: 0` for deterministic output
- Serialises body with `serde_json`, sends with `ureq::send_string`
- Extracts `choices[0].message.content`

### `main.rs`
- Installs a panic hook that writes `crash.log` next to the binary
- Wraps `run()` in `std::panic::catch_unwind`
- Builds system tray icon from `Token Optimizer.png` (fallback: 16×16 blue square)
- Right-click menu: "Token Optimizer — Running" (disabled label) + "Quit"
- Spawns background thread: hotkey listener → `simulate_copy()` → clipboard read → AI → clipboard write
- Main thread runs Win32 `GetMessageW` loop to drive tray icon events

### `simulate_copy()`
- Fires synthetic `Ctrl+C` via `rdev::simulate` before reading the clipboard
- Sleeps 200 ms to let the OS update the clipboard from the active selection
- Errors are logged as warnings but do not abort

---

## Config fields (`config.toml`)

| Field | Type | Default | Notes |
|-------|------|---------|-------|
| `lm_studio_url` | string | `"http://localhost:1234"` | LM Studio local server base URL |
| `model` | string | `"Phi-4-mini-instruct"` | Recommended: Phi-4-mini-instruct for speed and quality |
| `hotkey` | string | `"F14"` | Trigger key. Supports F1-F24, Alt+X, Ctrl+X |
| `auto_trigger` | bool | `false` | Auto-trigger when clipboard exceeds threshold |
| `auto_threshold_chars` | usize | `2000` | Char count that triggers auto mode |
| `system_prompt` | string | see below | Instruction sent to the LLM |
| `max_tokens` | u32 | `512` | Max tokens in LLM response |
| `log_to_file` | bool | `false` | Write logs to `token-optimizer.log` next to binary |

Default system prompt:
```
You are a lossless text compressor for developer content. Output ONLY the compressed text, nothing else. Rules: 1) Keep all error codes, file paths, line numbers and column numbers. 2) Keep all variable names, type names and function names exactly. 3) Remove decorative characters like arrows, pipes, carets and repeated dashes used only for visual formatting. 4) Remove lines that only contain whitespace or visual separators. 5) Keep the error message, the affected lines of code, and the location references. No explanations, no preamble.
```

---

## Activation modes

### Manual (hotkey)
- User presses configured hotkey (e.g. F14 on gaming mouse macro button)
- Daemon simulates Ctrl+C to copy active selection, waits 200 ms
- Reads clipboard, sends to LM Studio, writes compressed result back
- User can now paste the optimized version

### Smart Auto
- If `auto_trigger = true`, daemon watches clipboard changes
- If new clipboard content exceeds `auto_threshold_chars`, triggers automatically
- Not yet implemented — groundwork is in config

---

## LM Studio API (local, OpenAI-compatible)

Server running at: `http://127.0.0.1:1234`
Recommended model: `Phi-4-mini-instruct`

Example request:
```json
POST http://localhost:1234/v1/chat/completions
{
  "model": "Phi-4-mini-instruct",
  "messages": [
    { "role": "system", "content": "<system_prompt>" },
    { "role": "user", "content": "<clipboard_text>" }
  ],
  "max_tokens": 512,
  "temperature": 0,
  "stream": false
}
```

Response: standard OpenAI chat completion JSON. Extract `choices[0].message.content`.

---

## Dependencies (Cargo.toml)

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
toml = "0.8"
rdev = "0.5"           # global hotkey listener, supports F13-F24
arboard = "3"          # cross-platform clipboard read/write
ureq = "2"             # lightweight blocking HTTP client
serde_json = "1"
log = "0.4"
env_logger = "0.11"
chrono = "0.4"         # timestamps in logs
tray-icon = "0.14"     # Windows system tray icon
image = "0.25"         # PNG loading for tray icon
windows-sys = { version = "0.52", features = ["Win32_UI_WindowsAndMessaging", "Win32_Foundation"] }
```

---

## Build instructions (Windows — CMD only, never PowerShell)

```cmd
REM 1. Open Command Prompt (cmd.exe)

REM 2. Activate the MSVC 64-bit toolchain
"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"

REM 3. Set CMake generator
set CMAKE_GENERATOR=Visual Studio 17 2022

REM 4. Build
cd F:\proyectosprog\token-optimizer
cargo build --release
```

Binary: `target\release\token-optimizer.exe`

**Why CMD?** `vcvars64.bat` modifies PATH/LIB/INCLUDE for the current session; PowerShell does not propagate these correctly.

---

## Runtime requirements

- LM Studio must be running with local server started on port 1234
- `Phi-4-mini-instruct` (or your chosen model) must be loaded in LM Studio
- `config.toml` must be in the same directory as the binary
- `Token Optimizer.png` should be in the same directory as the binary (fallback icon used if missing)

---

## Guidelines

- All code comments and documentation in English.
- Never commit `config.toml` or `Token Optimizer.png`.
- Minimal memory footprint — this runs in the background at all times.
- Never intercept or consume keypresses — listen only (rdev `listen` mode).
- Log every trigger event with timestamp.
- Unit tests for config defaults in `config.rs` under `#[cfg(test)]`.
