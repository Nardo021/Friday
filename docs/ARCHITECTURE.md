# Friday Architecture

Friday is a desktop control layer for local coding agents — not a UI skin for Cursor CLI.

## Core Principles

1. **UI never talks to Cursor CLI directly** — UI consumes `AgentEvent` via Zustand stores.
2. **Adapter First** — Cursor CLI is the first adapter; future agents plug in via `AgentAdapter`.
3. **Local First** — no cloud dependency in v1.
4. **Pet is an entry point** — full UI lives in Chat Panel and Command Center.

## Layers

```
Pet / Chat / Command Center (React)
        ↓ Tauri IPC
Agent Core (state machine, session manager, event bus)
        ↓
Adapter Layer (Cursor CLI, future agents)
        ↓
Process Supervisor + Security + Storage
```

## Unified Events

All adapters emit normalized events: `agent.status`, `agent.message`, `tool.*`, `file.changed`, `command.*`, `approval.required`, `session.*`.

## Windows

| ID | Purpose |
|---|---|
| `pet` | Transparent floating pet, status animation |
| `quick-bubble` | Quick status + approvals |
| `chat` | Conversation + controls |
| `command-center` | Dashboard, sessions, projects, settings |

## Module Map

### Frontend (`src/`)

- `app/` — routing by window label
- `windows/` — pet, chat, command-center
- `state/` — Zustand stores
- `agent/` — shared types, events, mood-map
- `components/` — shadcn-style UI + Friday components

### Backend (`src-tauri/src/`)

- `core/` — agent_core, event_bus, state_machine, session_manager
- `adapters/` — adapter trait + cursor implementation
- `process/` — process supervisor
- `security/` — allowlist, risk, approval, redaction
- `storage/` — SQLite repos
- `system/` — tray, windows, autostart
- `commands/` — Tauri IPC handlers

## Data Flow

User prompt → `start_agent_session` → Agent Core → Cursor Adapter → Process Supervisor → stdout NDJSON → Parser → Mapper → AgentEvent → Event Bus → Frontend stores → Pet + Chat UI.

## Version Milestones

- **v0.1** — Core pipeline + pet + basic storage
- **v0.2** — Quick bubble, tray, security, approvals
- **v0.3** — Command Center, adapter registry, settings

See [MVP.md](MVP.md) for scope details.
