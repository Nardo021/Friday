# Intent Router

Friday routes Quick Bubble input through a hybrid classifier:

1. **Rules** — fast, offline patterns (stop, status, save idea, follow-up, new task)
2. **LLM fallback** — OpenAI `gpt-4o-mini` JSON classification when rules are ambiguous (uses STT or Cursor API key from keyring)
3. **Clarify** — returns action chips when confidence is low

## IPC

- `submit_quick_input` — route + execute
- `execute_quick_intent` — run a resolved intent (after Clarify UI)
- `route_quick_input` — classify only

## Rule examples

| Input | Intent |
|-------|--------|
| 暂停 / stop | Control::Stop |
| 现在做到哪 / status | QueryStatus |
| 记一下 … | SaveIdea |
| Active running session + text | FollowUp |
| 帮我 … / help me … | NewTask |

Implementation: `apps/desktop/src-tauri/src/core/intent_router.rs`
