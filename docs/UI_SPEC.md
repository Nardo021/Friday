# Friday UI Specification

## Layers

### Layer 1 — Pet Surface (Moving Native Window)

- **Pet window** `pet`: 160×160, transparent, always-on-top, no decorations
- **Screen coordinates**: pet position = native window `outer_position` (not CSS transform inside window)
- **Motion**: Rust `set_pet_position` at ~30fps patrol along taskbar; sprite animation at 60fps (Framer Motion, scale/rotate only)
- **Status bubble** `status-bubble`: 280×80, follows pet, auto-shows on agent status change
- **Quick bubble** `quick-bubble`: 320×220, anchored near pet on open
- **Interaction**: single-click → Quick Bubble; double-click → Panel; right-click → context menu
- **Click-through**: transparent pixels use `set_ignore_cursor_events` (circular hit region MVP)
- Pet window renders sprite only — no embedded chat or session UI

### Layer 2 — Agent Control Panel
- Window: `panel` (480×720)
- Components:
  - **CurrentStatusBar** — active session summary
  - **ActiveSessionsList** — SessionCard per session
  - **SessionTimeline** — events for active session
  - **ChatPortal** — repo + mode selector + prompt + start
  - **ActionsBar** — follow-up, stop, logs (capability-gated)

### Session Cards

| Card | Session type | Actions |
|---|---|---|
| FridaySessionCard | friday_owned_cli | Follow up, Stop, Logs |
| ExternalSessionCard | external_cli | View, Bind Repo |
| CloudSessionCard | cursor_cloud | Stub / Coming Soon |

### Layer 3 — Command Center
- Full history, projects, settings, adapters
- Settings: `usePty`, `argTemplates`, terminal cols/rows

## Mode Selector (v1)

| Mode | Enabled |
|---|---|
| Local CLI | Yes |
| SDK Local | Coming Soon |
| Cloud Agent | Coming Soon |

## Cross-Window Sync

Each Tauri webview has isolated Zustand state. Sync via:
1. Tauri `agent-event` broadcast
2. `list_sessions` hydrate on mount
3. `select_active_session` IPC
