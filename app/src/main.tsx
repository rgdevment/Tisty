import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App";
import Quick from "./Quick";
import { noteBreak } from "./core";
import { locale } from "./locales";
import "./index.css";

/// A native app has no Back, no Reload and no page source. Only a field where
/// you type keeps the menu, for paste; on a selection Chromium adds Print and
/// Inspect, which is worse than losing right-click copy.
document.addEventListener("contextmenu", (e) => {
  const writes = (e.target as HTMLElement).closest("input, textarea, [contenteditable]");
  if (!writes) e.preventDefault();
});

// A screen reader picks its voice and its pronunciation from this. Left at
// «en», Spanish came out read with English phonetics.
document.documentElement.lang = locale();

/// The stack without its first line: in V8 that line is «Kind: message», and
/// the message is the part that can be carrying somebody's task title.
const framesOf = (stack?: string): string =>
  (stack ?? "")
    .split("\n")
    .filter((line) => line.trimStart().startsWith("at "))
    .slice(0, 4)
    .map((line) => line.trim())
    .join(" | ");

/// React retries a failed render, so one broken component can raise the same
/// error dozens of times a second. Unchecked it would roll the log past its own
/// 256 kB and throw away the history that explains how it got there.
const seen = new Set<string>();

const broke = (kind: string, stack?: string) => {
  const frames = framesOf(stack);
  const once = `${kind} ${frames}`;
  if (seen.has(once)) return;
  seen.add(once);
  void noteBreak(kind, frames).catch(() => {});
};

window.addEventListener("error", (e) => broke(e.error?.name ?? "Error", e.error?.stack));
window.addEventListener("unhandledrejection", (e) =>
  broke(e.reason?.name ?? "Rejection", e.reason?.stack),
);

// One bundle, two windows: the label says which one this is.
const quick = getCurrentWindow().label === "quick";
if (quick) document.documentElement.classList.add("quick");

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>{quick ? <Quick /> : <App />}</React.StrictMode>,
);
