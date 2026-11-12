import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import React from "react";
import ReactDOM from "react-dom/client";
import { broke, thrown } from "./broke";
import { drawsHeicHere } from "./core";
import { locale } from "./locales";
import { drawsHeic } from "./previews";
import Standing, { fell } from "./Standing";
import "tisty-editor/tokens.css";
import "tisty-editor/side.css";
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

window.addEventListener("error", thrown);
window.addEventListener("unhandledrejection", (e) =>
  broke(e.reason?.name ?? "Rejection", e.reason?.message, e.reason?.stack),
);

const label = getCurrentWindow().label;
const quick = label === "quick";
if (quick) document.documentElement.classList.add("quick");

const drawn = () =>
  drawsHeicHere()
    .then(drawsHeic)
    .catch(() => undefined);
// Whether a HEIC is a picture is known before anything is drawn, though never at the cost of starting.
const READY_WITHIN = 300;

const windows = {
  moving: () => import("./Moving"),
  viewing: () => import("./Viewing"),
  quick: () => import("./Quick"),
  main: () => import("./App"),
};
const own =
  label === "moving"
    ? windows.moving
    : label.startsWith("view-")
      ? windows.viewing
      : quick
        ? windows.quick
        : windows.main;

const shown = () =>
  Promise.all([
    own().catch(fell),
    Promise.race([drawn(), new Promise((ready) => setTimeout(ready, READY_WITHIN))]),
  ]).then(([{ default: Own }]) =>
    ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
      <React.StrictMode>
        <Standing>
          <Own />
        </Standing>
      </React.StrictMode>,
    ),
  );

// A window nobody sees draws nothing of its own: it is there to finish a document.
if (label.startsWith("finisher-"))
  import("./finishing")
    .then((one) => one.finishWhatIsAsked())
    .catch(() => invoke("finishing_answered", { answers: null }));
else void shown();
