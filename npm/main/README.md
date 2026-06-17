# @panden/cclinepro

cclinepro is a high-performance Claude Code statusline tool distributed as the `ccline` command.

## Installation

```bash
npm install -g @panden/cclinepro
```

## Features

- 🚀 **Fast**: Written in Rust for maximum performance
- 🌍 **Cross-platform**: Works on Windows, macOS, and Linux
- 📦 **Easy installation**: One command via npm
- 🎨 **Configurable**: Interactive TUI, built-in themes, configurable segments, and real-time preview
- 📊 **Rich statusline data**: Git, model, context window, API usage, cost, session, and output style segments
- 🔧 **Claude Code enhancements**: Optional patcher with automatic backups

## Upgrades From Upstream

Compared with the upstream `master` branch, this build improves Context Window rendering by preferring Claude Code's official `context_window` / `contextWindow` payload, using the official context window size when present, excluding output tokens from used-context calculations, and falling back to transcript parsing when official data is unavailable.

## Usage

After installation, ccline is automatically configured for Claude Code at `~/.claude/ccline/ccline`.

You can also use it directly:

```bash
ccline --help
ccline --version
```

## For Users in China

Use npm mirror for faster installation:

```bash
npm install -g @panden/cclinepro --registry https://registry.npmmirror.com
```

## More Information

- GitHub: https://github.com/dengshuit/cclinepro
- Issues: https://github.com/dengshuit/cclinepro/issues
- License: MIT

## Upstream / Acknowledgements

This npm distribution is maintained by panden and is based on the [upstream project](https://github.com/Haleclipse/CCometixLine).
