/** @friday/sdk-worker — v2 stub for Cursor SDK / Cloud Agent IPC */

export type WorkerMessage =
  | { type: "ping" }
  | { type: "pong" }
  | { type: "create_cloud_session"; prompt: string; repo: string }
  | { type: "error"; message: string };

console.log(JSON.stringify({ type: "ready", version: "0.0.1-stub" }));

process.stdin.on("data", (chunk) => {
  const lines = chunk.toString().split("\n").filter(Boolean);
  for (const line of lines) {
    try {
      const msg = JSON.parse(line) as WorkerMessage;
      if (msg.type === "ping") {
        console.log(JSON.stringify({ type: "pong" }));
      } else if (msg.type === "create_cloud_session") {
        console.log(
          JSON.stringify({
            type: "error",
            message: "Cloud Agent not available in v1 stub",
          }),
        );
      }
    } catch {
      console.log(JSON.stringify({ type: "error", message: "invalid json" }));
    }
  }
});
