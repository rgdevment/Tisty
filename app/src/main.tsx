import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";

/// A native app has no Back, no Reload and no page source. Only a field where
/// you type keeps the menu, for paste; on a selection Chromium adds Print and
/// Inspect, which is worse than losing right-click copy.
document.addEventListener("contextmenu", (e) => {
  const writes = (e.target as HTMLElement).closest("input, textarea, [contenteditable]");
  if (!writes) e.preventDefault();
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
