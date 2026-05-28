# Friday Architecture

Friday is a **Cursor Agent Portal** — a desktop control layer for local and cloud coding agents.

## Core Principle

```
Agent Session → Events → State Machine → UI
```

UI never parses Cursor CLI stdout directly.

## Four Layers

1. **Pet Surface** — moving native window + status bubble + entry point
2. **Agent Control Panel** — sessions, chat portal, actions
3. **Friday Local Bridge** — PTY, discovery, SQLite, security, git
4. **Adapter Runtime** — external observer, local CLI, SDK/cloud stubs

## Monorepo Layout

```
apps/desktop/     Tauri + React
packages/agent-core/   Shared types & events
packages/shared/       Utils
services/sdk-worker/   v2 Node worker stub
```

## Adapters (v1)

| Adapter | Type | Control |
|---|---|---|
| external-cursor-observer | external_cli | observe |
| cursor-cli-local | friday_owned_cli | full |
| cursor-sdk-local | cursor_sdk_local | stub |
| cursor-cloud-agent | cursor_cloud | stub |

## Backend Modules (Rust)

- `core/` — AgentCore, SessionManager, events
- `adapters/` — trait + registry + implementations
- `pty/` — portable-pty manager (create, resize, close)
- `discovery/` — sysinfo process scan
- `process/` — pipe fallback supervisor
- `security/` — allowlist, risk, approval, redaction
- `storage/` — SQLite v2

- `system/` — tray, window manager, screen manager (monitor bounds, position clamp)

## Pet Engine

```
Agent Session → AgentMoodMapper → PetEngine → WindowController (Rust)
                      ↓
              BehaviorStateMachine + MotionController (30fps)
                      ↓
              set_pet_position → native window moves on screen
```

Frontend modules (`apps/desktop/src/pet-engine/`):

| Module | Role |
|---|---|
| `PetActor` | Screen position, mood, velocity |
| `BehaviorStateMachine` | idle / walk / dragged / thinking / editing / … |
| `MotionController` | Patrol along work area bottom, edge clamp |
| `BubbleController` | Status bubble show/hide + anchor |
| `HitTestEngine` | Circular alpha hit-test → click-through |
| `PetEngine` | Tick loop, position persist, drag sync |

Rust modules:

| Module | Role |
|---|---|
| `screen_manager` | Monitor work area, clamp position, default spawn |
| `window_manager` | set/get position, anchor bubble, click-through |

Position persisted in `settings.pet.lastX / lastY`.

## Frontend

- `windows/panel/` — Agent Control Panel
- `windows/pet/` — Pet sprite (animation only)
- `windows/status-bubble/` — Ephemeral status line
- `pet-engine/` — Motion + behavior orchestration
- `packages/agent-core/` — TypeScript contract

See also: [SESSION_MODEL.md](SESSION_MODEL.md), [UI_SPEC.md](UI_SPEC.md), [ADAPTER_SPEC.md](ADAPTER_SPEC.md), [ROADMAP.md](ROADMAP.md).
