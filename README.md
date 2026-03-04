# CClineBar

> Forked from [Haleclipse/CCometixLine](https://github.com/Haleclipse/CCometixLine) — thank you for the excellent foundation.

[English](README.md) | [中文](README.zh.md)

A high-performance Claude Code statusline tool written in Rust with Git integration, usage tracking, interactive TUI configuration, and Claude Code enhancement utilities.

![Language:Rust](https://img.shields.io/static/v1?label=Language&message=Rust&color=orange&style=flat-square)
![License:MIT](https://img.shields.io/static/v1?label=License&message=MIT&color=blue&style=flat-square)

## Screenshot

![CClineBar](assets/img1.png)

## What This Fork Adds

### Cost Tracking — `$session / $monthly`

As a Max plan subscriber, I was curious: how much would I actually be paying if I used a pay-as-you-go API key instead? This segment answers that question — it shows the API-equivalent cost of your usage, so you can see the real dollar value of what your subscription covers.

- **Session cost**: Read from Claude Code's `total_cost_usd` field, based on actual model and API pricing
- **Monthly total**: Accumulated across all sessions on this machine, tracked in `~/.claude/ccline/monthly_cost.json`, resets each month
- **Always tracking**: Cost data is recorded on every render regardless of whether the cost segment is displayed
- **No double-counting**: Each session is identified by its transcript path — repeated renders overwrite, not accumulate

### Model Working Time

If you think of the LLM as a copilot, it's interesting to know how long your copilot has actually been working. This segment shows the total model generation time (`total_api_duration_ms`) for the current session — pure thinking time, not wall clock time.

### Other Additions

- **`show_icons` toggle** — hide all segment icons to save terminal space
- **Auto-patch context warnings** — automatically removes "Context low" messages on first render, no manual `--patch` needed
- **Accurate context %** — shows context remaining (not used), matching Claude Code's own formula
- **Cost always tracked** — monthly cost accumulates even when the cost segment is hidden

## Features

### Core Functionality
- **Git integration** with branch, status, and tracking info
- **Model display** with simplified Claude model names
- **Usage tracking** based on transcript analysis
- **Directory display** showing current workspace
- **Minimal design** using Nerd Font icons

### Interactive TUI Features
- **Interactive main menu** when executed without input
- **TUI configuration interface** with real-time preview
- **Theme system** with multiple built-in presets
- **Segment customization** with granular control
- **Configuration management** (init, check, edit)

### Claude Code Enhancement
- **Context warning disabler** — remove "Context low" messages (auto-applied on first render)
- **Verbose mode enabler** — enhanced output detail
- **Robust patcher** — survives Claude Code version updates
- **Automatic backups** — safe modification with easy recovery

## Installation

### Quick Install (Recommended)

Install via npm (works on all platforms):

```bash
npm install -g @cometix/ccline
```

After installation:
- Global command `ccline` is available everywhere
- Follow the configuration steps below to integrate with Claude Code
- Run `ccline -c` to open configuration panel for theme selection

### Claude Code Configuration

Add to your Claude Code `settings.json`:

**Linux/macOS:**
```json
{
  "statusLine": {
    "type": "command",
    "command": "~/.claude/ccline/ccline",
    "padding": 0
  }
}
```

**Windows:**
```json
{
  "statusLine": {
    "type": "command",
    "command": "%USERPROFILE%\\.claude\\ccline\\ccline.exe",
    "padding": 0
  }
}
```

### Update

```bash
npm update -g @cometix/ccline
```

<details>
<summary>Manual Installation (Click to expand)</summary>

Download from [Releases](https://github.com/Haleclipse/CCometixLine/releases):

#### Linux (Dynamic — Recommended)
```bash
mkdir -p ~/.claude/ccline
wget https://github.com/Haleclipse/CCometixLine/releases/latest/download/ccline-linux-x64.tar.gz
tar -xzf ccline-linux-x64.tar.gz
cp ccline ~/.claude/ccline/
chmod +x ~/.claude/ccline/ccline
```

#### Linux (Static — Universal)
```bash
mkdir -p ~/.claude/ccline
wget https://github.com/Haleclipse/CCometixLine/releases/latest/download/ccline-linux-x64-static.tar.gz
tar -xzf ccline-linux-x64-static.tar.gz
cp ccline ~/.claude/ccline/
chmod +x ~/.claude/ccline/ccline
```

#### macOS (Apple Silicon)
```bash
mkdir -p ~/.claude/ccline
wget https://github.com/Haleclipse/CCometixLine/releases/latest/download/ccline-macos-arm64.tar.gz
tar -xzf ccline-macos-arm64.tar.gz
cp ccline ~/.claude/ccline/
chmod +x ~/.claude/ccline/ccline
```

#### macOS (Intel)
```bash
mkdir -p ~/.claude/ccline
wget https://github.com/Haleclipse/CCometixLine/releases/latest/download/ccline-macos-x64.tar.gz
tar -xzf ccline-macos-x64.tar.gz
cp ccline ~/.claude/ccline/
chmod +x ~/.claude/ccline/ccline
```

#### Windows
```powershell
New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\.claude\ccline"
Invoke-WebRequest -Uri "https://github.com/Haleclipse/CCometixLine/releases/latest/download/ccline-windows-x64.zip" -OutFile "ccline-windows-x64.zip"
Expand-Archive -Path "ccline-windows-x64.zip" -DestinationPath "."
Move-Item "ccline.exe" "$env:USERPROFILE\.claude\ccline\"
```

</details>

### Build from Source

```bash
git clone https://github.com/leo919cc/CClineBar.git
cd CClineBar
cargo build --release

mkdir -p ~/.claude/ccline
cp target/release/ccometixline ~/.claude/ccline/ccline
chmod +x ~/.claude/ccline/ccline
```

## Usage

### Configuration Management

```bash
ccline --init      # Initialize configuration file
ccline --check     # Check configuration validity
ccline --print     # Print current configuration
ccline --config    # Enter TUI configuration mode
```

### Theme Override

```bash
ccline --theme cometix
ccline --theme minimal
ccline --theme gruvbox
ccline --theme nord
ccline --theme powerline-dark
```

Or use custom theme files from `~/.claude/ccline/themes/`.

## Configuration

CClineBar supports full configuration via TOML files and interactive TUI:

- **Configuration file**: `~/.claude/ccline/config.toml`
- **Interactive TUI**: `ccline --config` for real-time editing with preview
- **Theme files**: `~/.claude/ccline/themes/*.toml` for custom themes
- **Automatic initialization**: `ccline --init` creates default configuration

### Style Options

```toml
[style]
mode = "nerd_font"    # "plain", "nerd_font", or "powerline"
separator = " | "
show_icons = true     # Set to false to hide all segment icons and save space
```

### Available Segments

All segments are configurable with enable/disable toggle, custom separators, icons, colors, and format options.

Supported segments: Directory, Git, Model, Context Window, Model Time, Usage, Cost, Session, OutputStyle

## Requirements

- **Claude Code**: Required — CClineBar is a statusline extension for Claude Code
- **Git**: Version 1.5+ (Git 2.22+ recommended)
- **Nerd Font** (recommended): Required for NerdFont and Powerline themes. The `plain` theme uses emoji icons and works without Nerd Fonts
- **Rust** (build from source only): Install via [rustup](https://rustup.rs/)

## Development

```bash
cargo build            # Development build
cargo test             # Run tests
cargo build --release  # Optimized release build
```

## Related Projects

- [CCometixLine](https://github.com/Haleclipse/CCometixLine) — the original project this fork is based on
- [tweakcc](https://github.com/Piebald-AI/tweakcc) — command-line tool to customize Claude Code themes, thinking verbs, and more

## License

[MIT License](LICENSE)
