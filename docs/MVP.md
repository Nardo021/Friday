# Friday MVP Scope

## v0.1 — Minimum Viable

- [x] Tauri desktop app
- [x] Transparent pet window
- [x] Click pet → open chat
- [x] Add project directory
- [x] Input prompt → start Cursor CLI
- [x] Read stdout/stderr, show real-time status
- [x] Conversation timeline
- [x] Stop current task
- [x] Save session history

**Not in v0.1:** multi-agent, cloud sync, plugin market, remote run.

## v0.2 — Usable Daily

- Quick Bubble
- Right-click menu
- Session list
- File changes + command records
- High-risk command approval
- Secret redaction
- Project allowlist
- System tray + launch at startup

## v0.3 — Product Ready

- Command Center (full)
- Adapter Registry (Cursor live, others stub)
- Pet personality settings
- Session summary, log search, export
- Update system, error recovery

## Development Phases

| Phase | Focus | Success Criteria |
|---|---|---|
| 1 | Core pipeline | Prompt → Cursor CLI → Chat timeline |
| 2 | State + Pet | Pet reflects thinking/editing/command/done |
| 3 | Storage | History survives app restart |
| 4 | Safety | No silent high-risk commands |
| 5 | Polish | Tray, bubble, command center |
