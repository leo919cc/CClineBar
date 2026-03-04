# CClineBar

> 基于 [Haleclipse/CCometixLine](https://github.com/Haleclipse/CCometixLine) 开发 — 感谢原作者的优秀项目。

[English](README.md) | [中文](README.zh.md)

基于 Rust 的高性能 Claude Code 状态栏工具，集成 Git 信息、使用量跟踪、交互式 TUI 配置和 Claude Code 补丁工具。

![Language:Rust](https://img.shields.io/static/v1?label=Language&message=Rust&color=orange&style=flat-square)
![License:MIT](https://img.shields.io/static/v1?label=License&message=MIT&color=blue&style=flat-square)

## 截图

![CClineBar](assets/img1.png)

## Fork 新增功能

### 费用追踪 — `$当前会话 / $当月累计`

作为 Max 订阅用户，一直很好奇：如果用按量付费的 API Key，实际要花多少钱？这个模块就是为了回答这个问题 — 显示你的使用量对应的 API 等价费用，让你直观看到订阅到底帮你省了多少。

- **会话费用**：读取 Claude Code 的 `total_cost_usd` 字段，基于实际模型和 API 定价计算
- **月度累计**：记录在 `~/.claude/ccline/monthly_cost.json`，跨会话累计，每月自动重置
- **始终记录**：无论费用模块是否显示，每次渲染都会记录费用数据
- **不重复计算**：每个会话通过 transcript 路径标识，重复渲染只覆盖不累加

### 模型工作时长

如果把 LLM 看作你的编程搭档，知道它实际工作了多久是件很有趣的事。这个模块显示当前会话的模型生成时间（`total_api_duration_ms`）— 纯推理时间，不是挂钟时间。

### 其他新增

- **`show_icons` 开关** — 隐藏所有图标以节省终端空间
- **自动补丁上下文警告** — 首次渲染时自动移除 "Context low" 提示，无需手动 `--patch`
- **准确的上下文百分比** — 显示剩余比例（非已用），与 Claude Code 自身公式一致
- **费用始终追踪** — 即使隐藏费用模块，月度费用仍会累计

## 功能特性

### 核心功能
- **Git 集成** 显示分支、状态和跟踪信息
- **模型显示** 简化的 Claude 模型名称
- **使用量跟踪** 基于 transcript 文件分析
- **目录显示** 显示当前工作空间
- **简洁设计** 使用 Nerd Font 图标

### 交互式 TUI 功能
- **交互式主菜单** 无输入时直接显示菜单
- **TUI 配置界面** 实时预览配置效果
- **主题系统** 多种内置预设主题
- **段落自定义** 精细化控制各段落
- **配置管理** 初始化、检查、编辑配置

### Claude Code 增强
- **禁用上下文警告** — 首次渲染时自动移除 "Context low" 消息
- **启用详细模式** — 增强输出详细信息
- **稳定补丁器** — 适应 Claude Code 版本更新
- **自动备份** — 安全修改，支持轻松恢复

## 安装

### 快速安装（推荐）

通过 npm 安装（适用于所有平台）：

```bash
npm install -g @cometix/ccline
```

安装后：
- 全局命令 `ccline` 可在任何地方使用
- 按照下方提示进行配置以集成到 Claude Code
- 运行 `ccline -c` 打开配置面板进行主题选择

### Claude Code 配置

添加到 Claude Code `settings.json`：

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

### 更新

```bash
npm update -g @cometix/ccline
```

<details>
<summary>手动安装（点击展开）</summary>

从 [Releases](https://github.com/Haleclipse/CCometixLine/releases) 下载：

#### Linux（动态链接 — 推荐）
```bash
mkdir -p ~/.claude/ccline
wget https://github.com/Haleclipse/CCometixLine/releases/latest/download/ccline-linux-x64.tar.gz
tar -xzf ccline-linux-x64.tar.gz
cp ccline ~/.claude/ccline/
chmod +x ~/.claude/ccline/ccline
```

#### Linux（静态链接 — 通用兼容）
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

### 从源码构建

```bash
git clone https://github.com/leo919cc/CClineBar.git
cd CClineBar
cargo build --release

mkdir -p ~/.claude/ccline
cp target/release/ccometixline ~/.claude/ccline/ccline
chmod +x ~/.claude/ccline/ccline
```

## 使用

### 配置管理

```bash
ccline --init      # 初始化配置文件
ccline --check     # 检查配置有效性
ccline --print     # 打印当前配置
ccline --config    # 进入 TUI 配置模式
```

### 主题覆盖

```bash
ccline --theme cometix
ccline --theme minimal
ccline --theme gruvbox
ccline --theme nord
ccline --theme powerline-dark
```

也可以使用 `~/.claude/ccline/themes/` 目录下的自定义主题。

## 配置

CClineBar 支持通过 TOML 文件和交互式 TUI 进行完整配置：

- **配置文件**: `~/.claude/ccline/config.toml`
- **交互式 TUI**: `ccline --config` 实时编辑配置并预览效果
- **主题文件**: `~/.claude/ccline/themes/*.toml` 自定义主题文件
- **自动初始化**: `ccline --init` 创建默认配置

### 样式选项

```toml
[style]
mode = "nerd_font"    # "plain", "nerd_font", 或 "powerline"
separator = " | "
show_icons = true     # 设为 false 隐藏所有图标以节省空间
```

### 可用段落

所有段落都支持启用/禁用、自定义分隔符、图标、颜色和格式选项。

支持的段落：目录、Git、模型、上下文窗口、模型时间、使用量、费用、会话、输出样式

## 系统要求

- **Claude Code**：必需 — CClineBar 是 Claude Code 的状态栏扩展
- **Git**：版本 1.5+（推荐 Git 2.22+）
- **Nerd Font**（推荐）：NerdFont 和 Powerline 主题需要。`plain` 主题使用 emoji 图标，无需 Nerd Font
- **Rust**（仅从源码构建时需要）：通过 [rustup](https://rustup.rs/) 安装

## 开发

```bash
cargo build            # 开发版本
cargo test             # 运行测试
cargo build --release  # 优化发布版本
```

## 相关项目

- [CCometixLine](https://github.com/Haleclipse/CCometixLine) — 本 fork 的上游原始项目
- [tweakcc](https://github.com/Piebald-AI/tweakcc) — 自定义 Claude Code 主题、思考动词等的命令行工具

## 许可证

[MIT 许可证](LICENSE)
