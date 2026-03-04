# ClaudeMeter

A menu bar / system tray app that displays your Claude.ai usage in real-time. Available for macOS, Windows, and Linux.

<div align="center"><img src="public/logo.png" width="120" /></div>

## Features

- Designed for **Claude Pro and Max** usage tracking
- Shows current session usage % directly in the menu bar
- Displays weekly usage limits — all models and Sonnet-only
- Shows extra usage spending and remaining balance
- Color-coded tray icon: green (< 70%), orange (70–90%), red (> 90%)
- Configurable auto-refresh interval (1 / 5 / 15 / 30 / 60 min) with manual refresh
- Sign In / Sign Out from the right-click tray menu
- Dark / Light / System theme support
- No API key required — uses your existing Claude.ai browser session
- No dock icon — pure menu bar / system tray app

Unlike other usage trackers, ClaudeMeter loads the actual Claude usage settings page under the hood to get accurate data. Your credentials are handled by the browser and are never stored or transmitted by the app.

## Installation

Download the latest release for your platform from the [Releases](../../releases) page.

### macOS

1. Download `ClaudeMeter_x.x.x_universal.dmg` (works on both Apple Silicon and Intel)
2. Open the `.dmg` and drag **ClaudeMeter** to your Applications folder
3. On first launch, right-click the app and select **Open** to bypass Gatekeeper (unsigned build)

### Windows

1. Download `ClaudeMeter_x.x.x_x64-setup.msi`
2. Run the installer and follow the prompts
3. ClaudeMeter will appear in the system tray (notification area)

### Linux

**AppImage** (works on most distros):
1. Download `ClaudeMeter_x.x.x_amd64.AppImage`
2. Make it executable: `chmod +x ClaudeMeter_*.AppImage`
3. Run it: `./ClaudeMeter_*.AppImage`

**Debian / Ubuntu**:
1. Download `ClaudeMeter_x.x.x_amd64.deb`
2. Install: `sudo dpkg -i ClaudeMeter_*.deb`

> **Linux note:** requires `libappindicator3` for the system tray icon. Install it with `sudo apt install libayatana-appindicator3-1` if the tray icon does not appear.

## Building from source

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://rustup.rs/) (stable)
- Platform-specific dependencies: see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)
  - **macOS**: Xcode Command Line Tools
  - **Linux**: `sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`
  - **Windows**: Microsoft Visual Studio C++ build tools

### Development

```bash
npm install
npm run tauri:dev
```

### Production build

```bash
# Current platform
npm run tauri:build

# macOS universal binary (Apple Silicon + Intel)
npm run tauri:build -- --target universal-apple-darwin
```

The built app and installer will be in `src-tauri/target/release/bundle/`.

## How it works

1. On launch, a hidden WebView silently loads `claude.ai/settings/usage`
2. A JavaScript snippet extracts usage percentages and reset times from the page DOM
3. Data is passed back to Rust via a URL-hash trick and cached in app state
4. Rust emits a `usage-updated` event to the popover window on every refresh
5. The cycle repeats on the configured interval

Sign In / Sign Out navigates the same hidden WebView to `claude.ai/login` or `claude.ai/logout`.

## Project structure

```
ClaudeMeter/
├── .github/
│   └── workflows/
│       └── release.yml        # Build + publish for all platforms on git tag push
├── src-tauri/
│   ├── icons/                 # App + tray icons (PNG, ICNS)
│   └── src/
│       ├── lib.rs             # Tauri builder + plugin setup
│       ├── state.rs           # AppState, UsageData, PollingCommand
│       ├── commands/
│       │   └── usage.rs       # Tauri commands + WebView scraping logic
│       ├── polling/
│       │   └── mod.rs         # Background Tokio polling task
│       └── tray/
│           └── mod.rs         # Tray icon, context menu, window positioning
└── src/
    ├── components/            # Header, UsageCard, Footer, LoginSetup, …
    ├── stores/
    │   └── usageStore.tsx     # React context + reducer for usage state
    ├── hooks/
    │   ├── useTheme.ts
    │   └── useUsage.ts
    ├── lib/
    │   └── tauri.ts           # Typed invoke() wrappers
    └── App.tsx
```

## License

MIT
