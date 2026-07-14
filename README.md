# Friday — Cursor Agent Portal

Friday is a desktop companion and control portal for Cursor agents.

- **Observe** external Cursor CLI sessions running on your machine
- **Launch** Friday-owned local Cursor CLI sessions (PTY + NDJSON stream)
- **Monitor** via pet + Agent Control Panel
- **Remote** approve sessions from the iOS companion app (local HTTP bridge)

## Monorepo Structure

```
Friday/
├── apps/
│   ├── desktop/           # Tauri + React (pet, panel, command center)
│   └── mobile/            # Expo iOS companion (bridge client)
├── packages/
│   ├── agent-core/        # Shared types, events, capabilities
│   ├── bridge-client/     # HTTP + WebSocket client for desktop bridge
│   └── shared/            # Shared utilities
├── services/
│   └── sdk-worker/        # v2 stub for Cursor SDK / Cloud
└── docs/
```

## Development

Prerequisites: Node.js and Rust (for Tauri). See [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

For **Local CLI** mode, install the Cursor Agent CLI and ensure `cursor-agent` is on your `PATH` (or set the path in Settings → Advanced: Cursor CLI). Onboarding probes for it automatically.

For **Cursor API / Cloud** mode, add a Cursor dashboard API key (`crsr_…`) in onboarding or Settings.

```bash
npm install

# Desktop (Tauri + React)
npm run tauri:dev          # Dev desktop app with hot reload
npm run dev                # Vite frontend only (no Rust rebuild)

# Production builds
npm run build              # Typecheck/build agent-core, bridge-client, desktop, mobile
npm run tauri:build        # Production Tauri bundle

# Mobile (Expo)
npm run start -w @friday/mobile
npm run typecheck:mobile
```

## Architecture

- [ARCHITECTURE.md](docs/ARCHITECTURE.md) — layers, adapters, Rust modules
- [SESSION_MODEL.md](docs/SESSION_MODEL.md) — session lifecycle and events
- [MOBILE_REMOTE.md](docs/MOBILE_REMOTE.md) — pairing the iOS app with the desktop bridge
- [ROADMAP.md](docs/ROADMAP.md)