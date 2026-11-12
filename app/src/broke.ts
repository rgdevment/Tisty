import { noteBreak } from "./core";

export const framesOf = (stack?: string): string =>
  (stack ?? "")
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.startsWith("at ") || /@.*:\d+/.test(line))
    .slice(0, 4)
    .join(" | ");

const seen = new Set<string>();

export const broke = (kind: string, said?: string, stack?: string) => {
  const frames = framesOf(stack);
  const once = `${kind} ${said ?? ""} ${frames}`;
  if (seen.has(once)) return;
  seen.add(once);
  void noteBreak(kind, said ?? null, frames).catch(() => {});
};

// Chromium tells of a resize it settles on the next frame as an error, though nothing broke.
const settlesNextFrame = (said: string) => said.includes("ResizeObserver loop");

export const thrown = (event: ErrorEvent) => {
  if (settlesNextFrame(event.message)) return;
  const at = event.filename ? `at ${event.filename}:${event.lineno}:${event.colno}` : undefined;
  broke(
    event.error?.name ?? "Error",
    event.error?.message ?? event.message,
    event.error?.stack ?? at,
  );
};
