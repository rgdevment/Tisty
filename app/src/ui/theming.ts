export const darkNow = () => document.documentElement.getAttribute("data-theme") === "dark";

/** Calls `retheme` each time the window turns light or dark, until the returned stop is called. */
export const followTheme = (retheme: () => void): (() => void) => {
  const watching = new MutationObserver(retheme);
  watching.observe(document.documentElement, { attributes: true, attributeFilter: ["data-theme"] });
  return () => watching.disconnect();
};
