const PATTERNS: RegExp[] = [
  /OPENAI_API_KEY=\S+/gi,
  /ANTHROPIC_API_KEY=\S+/gi,
  /CURSOR_TOKEN=\S+/gi,
  /GITHUB_TOKEN=\S+/gi,
  /DATABASE_URL=\S+/gi,
  /sk-[a-zA-Z0-9]{20,}/g,
  /ghp_[a-zA-Z0-9]{20,}/g,
];

export function redactSecrets(input: string): string {
  let result = input;
  for (const pattern of PATTERNS) {
    result = result.replace(pattern, "[REDACTED]");
  }
  return result;
}

export function cn(...classes: (string | false | undefined | null)[]): string {
  return classes.filter(Boolean).join(" ");
}
