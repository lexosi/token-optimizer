# token-optimizer

A lightweight Windows background daemon that compresses clipboard text through a local LLM before you paste it into an AI assistant — reducing token consumption without losing technical meaning.

Press a hotkey (default: **F14**), and whatever text you have selected or in your clipboard gets compressed and replaced in-place. Paste as normal.

---

## How it works

1. You press the configured hotkey (e.g. F14 on a gaming mouse macro button).
2. The daemon simulates **Ctrl+C** to copy your current selection.
3. It waits 200 ms for the OS to update the clipboard.
4. The clipboard text is sent to a locally-running LLM via the LM Studio API.
5. The LLM returns a compressed version — all technical terms, variable names, error codes, file paths and line numbers preserved; decorative formatting stripped.
6. The compressed text is written back to the clipboard.
7. You paste normally. The AI assistant receives fewer tokens.

The daemon runs silently in the Windows system tray. Right-click the tray icon to quit.

---

## Requirements

- Windows 10/11 x64
- [LM Studio](https://lmstudio.ai/) running locally with the server enabled on port 1234
- A loaded model — recommended: **Phi-4-mini-instruct** (fast, high quality for compression tasks)
- Rust toolchain with MSVC backend (Visual Studio 2022 Build Tools)

---

## Dependencies

| Crate | Purpose |
|-------|---------|
| `rdev 0.5` | Global hotkey listener (F1–F24, Alt+X, Ctrl+X) |
| `arboard 3` | Clipboard read/write |
| `ureq 2` | Blocking HTTP client for LM Studio API |
| `serde / serde_json / toml` | Config parsing and JSON serialisation |
| `tray-icon 0.14` | Windows system tray icon and menu |
| `image 0.25` | PNG loading for the tray icon |
| `windows-sys 0.52` | Win32 message loop (drives tray events) |
| `log / env_logger / chrono` | Structured logging with timestamps |

---

## Build

> **Must use CMD, not PowerShell.** `vcvars64.bat` sets up the MSVC linker environment; it does not work correctly from PowerShell.

```cmd
REM Open Command Prompt

REM 1. Activate MSVC 64-bit toolchain
"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"

REM 2. Set CMake generator (required by some C dependencies)
set CMAKE_GENERATOR=Visual Studio 17 2022

REM 3. Build release binary
cd F:\proyectosprog\token-optimizer
cargo build --release
```

Output: `target\release\token-optimizer.exe`

---

## Setup

1. Copy `config.toml.example` to `config.toml` in the same directory as the binary.
2. Edit `config.toml` — at minimum set `model` to match the model loaded in LM Studio.
3. Place `Token Optimizer.png` (32×32 or 64×64 RGBA PNG) next to the binary for the tray icon. A plain blue square is used as fallback if the file is missing.
4. Start LM Studio and enable the local server (default port 1234).
5. Load your chosen model in LM Studio.
6. Run `token-optimizer.exe`. The tray icon appears in the system tray.

---

## Configuration

Copy `config.toml.example` to `config.toml` next to the binary and adjust as needed.

| Field | Default | Description |
|-------|---------|-------------|
| `lm_studio_url` | `http://localhost:1234` | LM Studio local server URL |
| `model` | `Phi-4-mini-instruct` | Model ID as shown in LM Studio |
| `hotkey` | `F14` | Trigger key — F1–F24, Alt+X, Ctrl+X |
| `auto_trigger` | `false` | Auto-compress when clipboard exceeds threshold |
| `auto_threshold_chars` | `2000` | Character count threshold for auto mode |
| `system_prompt` | (see below) | System instruction sent to the LLM |
| `max_tokens` | `512` | Maximum tokens in the LLM response |
| `log_to_file` | `false` | Also write logs to `token-optimizer.log` |

### Default system prompt

```
You are a lossless text compressor for developer content. Output ONLY the
compressed text, nothing else. Rules: 1) Keep all error codes, file paths,
line numbers and column numbers. 2) Keep all variable names, type names and
function names exactly. 3) Remove decorative characters like arrows, pipes,
carets and repeated dashes used only for visual formatting. 4) Remove lines
that only contain whitespace or visual separators. 5) Keep the error message,
the affected lines of code, and the location references. No explanations, no
preamble.
```

---

## Supported hotkeys

| Format | Example | Notes |
|--------|---------|-------|
| Function key | `F14` | F1–F24; F13–F24 are common on gaming mice |
| Alt combo | `Alt+C` | |
| Ctrl combo | `Ctrl+F13` | |

---

## Tray icon

Right-click the tray icon to see:
- **Token Optimizer — Running** (greyed out status label)
- **Quit** — exits the daemon cleanly

---

## Project structure

```
token-optimizer/
├── Cargo.toml
├── config.toml.example
├── src/
│   ├── main.rs          tray icon, Win32 message loop, panic hook
│   ├── config.rs        Config struct + TOML loading
│   ├── hotkeys.rs       global hotkey listener
│   ├── clipboard.rs     clipboard read/write
│   └── ai_client.rs     LM Studio HTTP client
└── CLAUDE.md            developer context for Claude Code
```

---

## License

MIT
