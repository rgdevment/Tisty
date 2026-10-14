import { getCurrentWindow } from "@tauri-apps/api/window";
import React, { lazy, Suspense } from "react";
import ReactDOM from "react-dom/client";
import { broke } from "./broke";
import { locale } from "./locales";
import Standing from "./Standing";
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
);
