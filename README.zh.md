# cclinepro

[English](README.md) | [中文](README.zh.md)

cclinepro 是基于 Rust 的高性能 Claude Code 状态栏工具，通过 `ccline` 命令提供 Git 状态、模型/上下文展示、API 使用量与会话指标、交互式 TUI 配置、主题管理和 Claude Code 增强工具。

![Language:Rust](https://img.shields.io/static/v1?label=Language&message=Rust&color=orange&style=flat-square)
![License:MIT](https://img.shields.io/static/v1?label=License&message=MIT&color=blue&style=flat-square)

## 截图

![cclinepro](assets/img1.png)

状态栏显示：模型 | 目录 | Git 分支状态 | 上下文窗口信息

## 功能升级

相比上游 `master` 分支，当前分支的功能升级重点是让 Context Window 段落更准确：

- **优先读取官方 Context Window 输入**：当 Claude Code 的 statusline payload 中包含 `context_window` / `contextWindow` 时，优先使用官方数据，不再只依赖 transcript 解析。
- **更准确的上下文窗口上限**：支持读取 `context_window_size` / `contextWindowSize`，因此 1M 等大上下文窗口会按真实上限显示。
- **更准确的已用 token 计算**：优先使用 `total_input_tokens`；否则汇总当前 input、cache creation input、cache read input。output tokens 不计入已用上下文。
- **百分比回退计算**：当只提供 `used_percentage` / `usedPercentage` 时，可结合官方上下文窗口大小反推已用 token。
- **保留兼容回退**：官方 context 数据缺失或不完整时，继续使用原有 transcript 解析逻辑。
- **字段格式兼容**：新增 Context Window 数据同时支持 snake_case 和 camelCase 字段。
- **针对性测试覆盖**：新增官方数据优先、1M 上下文上限、排除 output tokens、transcript 回退、camelCase 反序列化等测试。

## 特性

### 核心功能
- **Git 集成** 显示分支、状态和跟踪信息
- **模型与上下文展示** 简化 Claude 模型名称并跟踪上下文窗口
- **API 使用量、成本和会话指标** 在 Claude Code 提供相关数据时展示
- **目录显示** 显示当前工作空间
- **简洁设计** 使用 Nerd Font 图标

### 交互式 TUI 功能
- **交互式主菜单** 无输入时直接执行显示菜单
- **TUI 配置界面** 实时预览配置效果
- **主题系统** 多种内置预设主题
- **段落自定义** 精细化控制各段落
- **配置管理** 通过交互式主菜单完成初始化、检查和编辑

### Claude Code 增强
- **禁用上下文警告** 移除烦人的"Context low"消息
- **启用详细模式** 增强输出详细信息
- **稳定补丁器** 适应 Claude Code 版本更新
- **自动备份** 安全修改，支持轻松恢复

## 安装

### 快速安装（推荐）

通过 npm 安装（适用于所有平台）：

```bash
# 全局安装
npm install -g @panden/cclinepro

# 或使用 yarn
yarn global add @panden/cclinepro

# 或使用 pnpm
pnpm add -g @panden/cclinepro
```

使用镜像源加速下载：
```bash
npm install -g @panden/cclinepro --registry https://registry.npmmirror.com
```

如果你正在从旧包名切换，建议先卸载旧包，避免全局 `ccline` 命令和 Claude Code 安装目标冲突：

```bash
npm uninstall -g @panden/ccline @cometix/ccline
```

安装后：
- ✅ 全局命令 `ccline` 可在任何地方使用
- ⚙️ 按照下方提示进行配置以集成到 Claude Code
- 🎨 运行 `ccline -c` 打开配置面板进行主题选择

### Claude Code 配置

添加到 Claude Code `settings.json`：

**跨平台通用（推荐）**
```json
{
  "statusLine": {
    "type": "command",
    "command": "~/.claude/ccline/ccline",
    "padding": 0
  }
}
```

> **Windows 用户注意：** 从 Claude Code v2.1.47+ 开始，Windows 上支持 Unix 风格路径解析。`~` 符号会自动展开为您的用户主目录。**请勿使用 `%USERPROFILE%`** — 它在 v2.1.47+ 版本中不再可靠。
> - 推荐：`~/.claude/ccline/ccline`（跨平台通用）
> - 备选：`"ccline"`（需要 npm 全局安装）

**后备方案 (npm 安装):**
```json
{
  "statusLine": {
    "type": "command",
    "command": "ccline",
    "padding": 0
  }
}
```
*如果 npm 全局安装已在 PATH 中可用，则使用此配置*

### 更新

```bash
npm update -g @panden/cclinepro
```

<details>
<summary>手动安装（点击展开）</summary>

或者从 [Releases](https://github.com/dengshuit/cclinepro/releases) 手动下载：

#### Linux

#### 选项 1: 动态链接版本（推荐）
```bash
mkdir -p ~/.claude/ccline
wget https://github.com/dengshuit/cclinepro/releases/latest/download/ccline-linux-x64.tar.gz
tar -xzf ccline-linux-x64.tar.gz
cp ccline ~/.claude/ccline/
chmod +x ~/.claude/ccline/ccline
```
*系统要求: Ubuntu 22.04+, CentOS 9+, Debian 11+, RHEL 9+ (glibc 2.35+)*

#### 选项 2: 静态链接版本（通用兼容）
```bash
mkdir -p ~/.claude/ccline
wget https://github.com/dengshuit/cclinepro/releases/latest/download/ccline-linux-x64-static.tar.gz
tar -xzf ccline-linux-x64-static.tar.gz
cp ccline ~/.claude/ccline/
chmod +x ~/.claude/ccline/ccline
```
*适用于任何 Linux 发行版（静态链接，无依赖）*

#### macOS (Intel)

```bash  
mkdir -p ~/.claude/ccline
wget https://github.com/dengshuit/cclinepro/releases/latest/download/ccline-macos-x64.tar.gz
tar -xzf ccline-macos-x64.tar.gz
cp ccline ~/.claude/ccline/
chmod +x ~/.claude/ccline/ccline
```

#### macOS (Apple Silicon)

```bash
mkdir -p ~/.claude/ccline  
wget https://github.com/dengshuit/cclinepro/releases/latest/download/ccline-macos-arm64.tar.gz
tar -xzf ccline-macos-arm64.tar.gz
cp ccline ~/.claude/ccline/
chmod +x ~/.claude/ccline/ccline
```

#### Windows

```powershell
# 创建目录并下载
New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\.claude\ccline"
Invoke-WebRequest -Uri "https://github.com/dengshuit/cclinepro/releases/latest/download/ccline-windows-x64.zip" -OutFile "ccline-windows-x64.zip"
Expand-Archive -Path "ccline-windows-x64.zip" -DestinationPath "."
Move-Item "ccline.exe" "$env:USERPROFILE\.claude\ccline\"
```

</details>

### 从源码构建

```bash
git clone https://github.com/dengshuit/cclinepro.git
cd cclinepro
cargo build --release
cp target/release/ccometixline ~/.claude/ccline/ccline
```

## 使用

### 主题覆盖

```bash
# 临时使用指定主题（覆盖配置文件设置）
ccline --theme default
ccline --theme minimal
ccline --theme gruvbox
ccline --theme nord
ccline --theme powerline-dark

# 或使用 ~/.claude/ccline/themes/ 目录下的自定义主题
ccline --theme my-custom-theme
```

### Claude Code 增强

```bash
# 禁用上下文警告并启用详细模式
ccline --patch /path/to/claude-code/cli.js

# 常见安装路径示例
ccline --patch ~/.local/share/fnm/node-versions/v24.4.1/installation/lib/node_modules/@anthropic-ai/claude-code/cli.js
```

## 默认段落

显示：`目录 | Git 分支状态 | 模型 | 上下文窗口`

### Git 状态指示器

- 带 Nerd Font 图标的分支名
- 状态：`✓` 清洁，`●` 有更改，`⚠` 冲突
- 远程跟踪：`↑n` 领先，`↓n` 落后

### 模型显示

显示简化的 Claude 模型名称：
- `claude-3-5-sonnet` → `Sonnet 3.5`
- `claude-4-sonnet` → `Sonnet 4`

### 上下文窗口显示

基于转录文件分析的令牌使用百分比，包含上下文限制跟踪。

## 配置

cclinepro 支持通过 TOML 文件和交互式 TUI 进行完整配置：

- **配置文件**: `~/.claude/ccline/config.toml`
- **交互式 TUI**: `ccline --config` 实时编辑配置并预览效果
- **主题文件**: `~/.claude/ccline/themes/*.toml` 自定义主题文件
- **自动初始化**: 无 stdin 运行 `ccline` 后，可在主菜单中创建或检查配置

### 可用段落

所有段落都支持配置：
- 启用/禁用切换
- 自定义分隔符和图标
- 颜色自定义
- 格式选项

支持的段落：目录、Git、模型、上下文窗口、API 使用量、成本、会话、输出样式、更新提示

### 模型配置 (`models.toml`)

文件位置：`~/.claude/ccline/models.toml`（首次运行时自动创建）

此文件配置模型 ID 的显示名称及其上下文窗口限制。Claude 模型（Sonnet、Opus、Haiku）会自动识别并提取版本号，此文件仅用于覆盖默认行为或添加第三方模型支持。

```toml
# 模型条目：基于模型 ID 的子字符串匹配
# 优先级高于内置 Claude 模型识别
[[models]]
pattern = "glm-4.5"
display_name = "GLM-4.5"
context_limit = 128000

[[models]]
pattern = "kimi-k2"
display_name = "Kimi K2"
context_limit = 128000

# 上下文修饰符：独立匹配，可与模型条目组合使用
# 覆盖 context_limit 并将 display_suffix 追加到显示名称
# 例如：模型 "Opus 4" + 修饰符 " 1M" = "Opus 4 1M"
[[context_modifiers]]
pattern = "[1m]"
display_suffix = " 1M"
context_limit = 1000000
```


## 系统要求

- **Git**: 版本 1.5+ (推荐 Git 2.22+ 以获得更好的分支检测)
- **终端**: 必须支持 Nerd Font 图标正常显示
  - 安装 [Nerd Font](https://www.nerdfonts.com/) 字体
  - 中文用户推荐: [Maple Font](https://github.com/subframe7536/maple-font) (支持中文的 Nerd Font)
  - 在终端中配置使用该字体
- **Claude Code**: 用于状态栏集成

## 开发

```bash
# 构建开发版本
cargo build

# 运行测试
cargo test

# 构建优化版本
cargo build --release
```

## 路线图

- [x] TOML 配置文件支持
- [x] TUI 配置界面
- [x] 自定义主题
- [x] 交互式主菜单
- [x] Claude Code 增强工具

## 贡献

欢迎贡献！请随时提交 issue 或 pull request。

## 上游致谢

cclinepro 由 panden 维护，基于[上游项目](https://github.com/Haleclipse/CCometixLine)。感谢原项目及其贡献者。

## 许可证

本项目采用 [MIT 许可证](LICENSE)。

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=dengshuit/cclinepro&type=Date)](https://star-history.com/#dengshuit/cclinepro&Date)
