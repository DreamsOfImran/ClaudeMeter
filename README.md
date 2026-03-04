# ClaudeMeter

A macOS menu bar app that displays your Claude.ai usage in real-time.

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
- No dock icon — pure menu bar app

Unlike other usage trackers, ClaudeMeter loads the actual Claude usage settings page under the hood to get accurate data. Your credentials are handled by the browser and are never stored or transmitted by the app.

## Installation

Download the latest `.dmg` from the [Releases](../../releases) page.

1. Open the `.dmg` and drag **ClaudeMeter** to your Applications folder
2. On first launch, right-click the app and select **Open** to bypass Gatekeeper

## Building from source

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://rustup.rs/) (stable)
- Xcode Command Line Tools
- Platform-specific dependencies: see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

### Development

```bash
npm install
npm run tauri:dev
```

### Production build

```bash
npm run tauri:build
```

The built app and `.dmg` will be in `src-tauri/target/release/bundle/`.

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
│       └── release.yml        # Build + publish DMG on git tag push
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
