import { openLink } from "./host";

// A link left to the browser engine would take this window away from the document.
export const linkPressed = (e: MouseEvent, fail: (problem: unknown) => void): void => {
  const link = (e.target as Element).closest<HTMLAnchorElement>("a[href]");
  if (!link) return;
  e.preventDefault();
  const said = link.getAttribute("href") ?? "";
  if (said.startsWith("#")) {
    document.getElementById(decodeURIComponent(said.slice(1)))?.scrollIntoView({ block: "start" });
    return;
  }
  if (/^(https?|mailto|tel):/i.test(said)) void openLink(said).catch(fail);
};
