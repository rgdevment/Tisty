import { getCurrentWindow } from "@tauri-apps/api/window";
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { broke } from "./broke";
import { locale } from "./locales";
import Quick from "./Quick";
import Standing from "./Standing";
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

window.addEventListener("error", (e) =>
  broke(e.error?.name ?? "Error", e.error?.message, e.error?.stack),
);
window.addEventListener("unhandledrejection", (e) =>
  broke(e.reason?.name ?? "Rejection", e.reason?.message, e.reason?.stack),
);

const quick = getCurrentWindow().label === "quick";
if (quick) document.documentElement.classList.add("quick");

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Standing>{quick ? <Quick /> : <App />}</Standing>
  </React.StrictMode>,
);
