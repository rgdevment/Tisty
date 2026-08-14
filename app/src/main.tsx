import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App";
import Quick from "./Quick";
import { noteBreak } from "./core";
import { locale } from "./locales";
import "./index.css";

document.addEventListener("contextmenu", (e) => {
  const writes = (e.target as HTMLElement).closest("input, textarea, [contenteditable]");
  if (!writes) e.preventDefault();
});

document.documentElement.lang = locale();

{
  const dark = window.matchMedia("(prefers-color-scheme: dark)");
  const paint = () =>
    document.documentElement.setAttribute("data-theme", dark.matches ? "dark" : "light");
  paint();
  dark.addEventListener("change", paint);
}

export const framesOf = (stack?: string): string =>
  (stack ?? "")
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.startsWith("at ") || /@.*:\d+/.test(line))
    .slice(0, 4)
    .join(" | ");

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

const quick = getCurrentWindow().label === "quick";
if (quick) document.documentElement.classList.add("quick");

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>{quick ? <Quick /> : <App />}</React.StrictMode>,
);
