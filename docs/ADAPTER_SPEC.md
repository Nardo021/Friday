# Agent Adapter Specification (v2)

## Adapter IDs

| ID | Session Type | v1 Status |
|---|---|---|
| `external-cursor-observer` | external_cli | Live |
| `cursor-cli-local` | friday_owned_cli | Live |
| `cursor-sdk-local` | cursor_sdk_local | Stub |
| `cursor-cloud-agent` | cursor_cloud | Stub |

## AgentAdapter (Rust trait / TS contract)

```typescript
createSession(input: CreateSessionInput): Promise<FridaySession>
attachSession?(input: AttachSessionInput): Promise<FridaySession>
sendMessage?(sessionId, message): Promise<void>
stopSession?(sessionId): Promise<void>
onEvent(callback): void
```

## CursorCliLauncher

```
resolve_executable → validate_repo_path → build_args (from settings.argTemplates)
→ inject_safe_env → start_pty | pipe_fallback → register_session → stream_events
```

CLI flags must **not** be hardcoded — use `FridaySettings.cursor.argTemplates.headlessStream`.

## Capabilities

See `packages/agent-core/src/capabilities.ts` for `AgentCapabilities` matrix per adapter.

## Events

All adapters map output to `AgentEvent` in `packages/agent-core/src/events.ts`.

UI and storage only consume normalized events.
