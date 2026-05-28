# Friday Mobile Remote

Connect the iOS companion app to your desktop Friday instance over the local network.

## Enable on desktop

1. Open **Command Center → Settings → Mobile Remote**
2. Enable **Local HTTP bridge**
3. Note the **URL** (e.g. `http://192.168.1.10:8787`) and **auth token**
4. Keep Friday running on the same Wi‑Fi as your phone

## Pair iOS app

1. Install/run `@friday/mobile` (Expo)
2. Enter bridge URL + token
3. Tap **Pair**

## Capabilities

| Action | Supported |
|--------|-----------|
| List active sessions | Yes |
| Watch session events (WebSocket) | Yes |
| Approve / reject commands | Yes (via REST + push on `approval.required`) |
| Stop session | Yes |

## API (v1)

| Method | Path | Auth |
|--------|------|------|
| GET | `/health` | No |
| GET | `/v1/info` | Bearer |
| GET | `/v1/sessions` | Bearer |
| GET | `/v1/sessions/:id` | Bearer |
| GET | `/v1/sessions/:id/events?limit=100` | Bearer |
| GET | `/v1/approvals/pending` | Bearer |
| POST | `/v1/approvals/:id/approve` | Bearer |
| POST | `/v1/approvals/:id/reject` | Bearer |
| POST | `/v1/sessions/:id/stop` | Bearer |
| WS | `/v1/ws?token=…` | Query token |

TypeScript client: `packages/bridge-client`.

## Security

- Bridge is **disabled by default**
- Token is required for all `/v1/*` routes
- Intended for **LAN only** — do not expose port 8787 to the public internet
- Events are redacted with the same secret redactor as the desktop app

## Development

```bash
npm install
npm run typecheck -w @friday/mobile   # TypeScript only
npm run start -w @friday/mobile         # Expo dev server (requires Expo CLI)
```

True APNs remote push (when phone is off-LAN) is future work; the companion uses local notifications when connected via WebSocket.
