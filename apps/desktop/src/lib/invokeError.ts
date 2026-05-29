/** Tauri invoke failures are often plain strings, not Error instances. */
export function invokeErrorMessage(error: unknown): string {
  if (typeof error === "string" && error.trim()) return error;
  if (error instanceof Error && error.message.trim()) return error.message;
  if (
    error &&
    typeof error === "object" &&
    "message" in error &&
    typeof (error as { message: unknown }).message === "string"
  ) {
    const msg = (error as { message: string }).message.trim();
    if (msg) return msg;
  }
  return "Something went wrong. Try again.";
}
