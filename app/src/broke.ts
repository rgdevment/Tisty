import { noteBreak } from "./core";

export const framesOf = (stack?: string): string =>
  (stack ?? "")
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.startsWith("at ") || /@.*:\d+/.test(line))
    .slice(0, 4)
    .join(" | ");

const seen = new Set<string>();

const noted = (stopped: boolean, kind: string, said?: string, stack?: string) => {
  const frames = framesOf(stack);
  const once = `${stopped} ${kind} ${said ?? ""} ${frames}`;
  if (seen.has(once)) return;
  seen.add(once);
  void noteBreak(kind, said ?? null, frames, stopped).catch(() => {});
};

export const broke = (kind: string, said?: string, stack?: string) =>
  noted(true, kind, said, stack);

interface Described {
  kind?: string;
  said?: string;
  stack?: string;
}

const text = (value: unknown) => (typeof value === "string" ? value : undefined);

// What a thrown string or object says may be the person's own words, so only its shape is kept.
const described = (value: unknown): Described => {
  if (value === null || value === undefined) return {};
  if (typeof value !== "object") return { said: `a ${typeof value} without a message` };
  const fault = value as Record<string, unknown>;
  const said = text(fault.message);
  if (said !== undefined) return { kind: text(fault.name), said, stack: text(fault.stack) };
  const code = text(fault.code);
  if (code !== undefined) return { kind: "Refusal", said: code };
  return { said: "an object without a message" };
};

// Chromium tells of a resize it settles on the next frame as an error, though nothing broke.
const SETTLES_NEXT_FRAME = /^ResizeObserver loop/;

export const thrown = (event: ErrorEvent) => {
  const message = event.message ?? "";
  if (SETTLES_NEXT_FRAME.test(message)) return;
  const fault = described(event.error);
  const at = event.filename ? `at ${event.filename}:${event.lineno}:${event.colno}` : undefined;
  noted(
    false,
    fault.kind ?? "Error",
    event.error == null ? message : fault.said,
    framesOf(fault.stack) ? fault.stack : at,
  );
};

export const rejected = (event: PromiseRejectionEvent) => {
  const fault = described(event.reason);
  noted(false, fault.kind ?? "Rejection", fault.said, fault.stack);
};
