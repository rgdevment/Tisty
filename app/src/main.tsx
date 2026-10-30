import { getCurrentWindow } from "@tauri-apps/api/window";
import React, { lazy, Suspense } from "react";
import ReactDOM from "react-dom/client";
import { broke } from "./broke";
import { drawsHeicHere } from "./core";
import { locale } from "./locales";
import { drawsHeic } from "./previews";
import Standing from "./Standing";
import "tisty-editor/tokens.css";
import "./index.css";

const App = lazy(() => import("./App"));
const Moving = lazy(() => import("./Moving"));
const Quick = lazy(() => import("./Quick"));
const Viewing = lazy(() => import("./Viewing"));

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

const label = getCurrentWindow().label;
const quick = label === "quick";
if (quick) document.documentElement.classList.add("quick");

const drawn = drawsHeicHere()
  .then(drawsHeic)
  .catch(() => undefined);
// Whether a HEIC is a picture is known before anything is drawn, though never at the cost of starting.
const READY_WITHIN = 300;

void Promise.race([drawn, new Promise((ready) => setTimeout(ready, READY_WITHIN))]).then(() =>
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <Standing>
        <Suspense fallback={null}>
          {label === "moving" ? (
            <Moving />
          ) : label.startsWith("view-") ? (
            <Viewing />
          ) : quick ? (
            <Quick />
          ) : (
            <App />
          )}
        </Suspense>
      </Standing>
    </React.StrictMode>,
  ),
);
