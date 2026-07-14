# Agent Adapter Specification (v2)

## Adapter IDs

| ID | Session Type | Status |
|---|---|---|
| `external-cursor-observer` | external_cli | Live (observe / attach by PID) |
| `cursor-cli-local` | friday_owned_cli | Live (PTY + stream-json) |
| `cursor-sdk-local` | cursor_sdk_local | Stub |
| `cursor-cloud-agent` | cursor_cloud | Live (`api.cursor.com`) |

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
resolve_executable → build_args (from settings.argTemplates)
→ start_pty | pipe_fallback → register_session → stream_events
```

Default headless template:

```
--print --output-format {outputFormat} --stream-partial-output {prompt}
```

If templates omit `{prompt}`, Friday appends the prompt so the task is never dropped.

Stop behavior (PTY): Ctrl+C → 3s grace → kill. Child PID is recorded and excluded from external discovery.

## Capabilities

See `packages/agent-core/src/capabilities.ts` for `AgentCapabilities` matrix per adapter.

## Events

All adapters map output to `AgentEvent` in `packages/agent-core/src/events.ts`.

`cursor-cli-local` parses Cursor `stream-json` NDJSON (`system`, `assistant`, `tool_call`, `result`) into normalized events. UI and storage only consume those events.

## Approvals

High-risk shell commands observed from the CLI stream surface an approval card. This is **observe-and-stop**, not a Cursor permission gate: Acknowledge continues watching; Reject stops the Friday-owned session.
