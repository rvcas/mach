---
title: Installation
description: How to install mach on your system.
---

## Homebrew

The easiest way to install on macOS or Linux:

```sh
brew install rvcas/tap/mach
```

## Shell (Linux/macOS)

Download and run the installer script:

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/rvcas/mach/releases/latest/download/machich-installer.sh | sh
```

## PowerShell (Windows)

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/rvcas/mach/releases/latest/download/machich-installer.ps1 | iex"
```

## npm / pnpm / bun

Install globally via your preferred package manager:

```sh
npm install -g @rvcas/mach
```

```sh
pnpm install -g @rvcas/mach
```

```sh
bun install -g @rvcas/mach
```

## Cargo

If you have Rust installed:

```sh
cargo install machich
```

## From Source

Clone and build from source:

```sh
git clone https://github.com/rvcas/mach
cd mach
cargo install --path crates/mach
```
