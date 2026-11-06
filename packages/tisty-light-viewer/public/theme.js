// Runs before the first paint, so whoever chose a theme never sees the other one flash.
(() => {
  const root = document.documentElement;
  let chosen = null;
  try {
    chosen = localStorage.getItem("tisty.viewer.theme");
  } catch {}
  const dark = chosen ? chosen === "dark" : matchMedia("(prefers-color-scheme: dark)").matches;
  root.dataset.theme = dark ? "dark" : "light";
  root.classList.toggle("dark", dark);
})();
