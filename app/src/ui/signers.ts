import type { Signing } from "../crediting";
import { fill, t } from "../locales";

const shownAs = (signing: Signing, by: string): string => {
  if (signing === "mark") return "◆";
  return signing === "agent" ? t("anAgent") : by;
};

// The core draws who signed each assistant's note; the person's choice says how much of the name shows.
export const signers = (page: Document, signing: Signing = "named") => {
  for (const one of page.querySelectorAll<HTMLElement>(".page .signer[data-by]")) {
    const by = one.dataset.by ?? "";
    const said = shownAs(signing, by);
    if (one.textContent !== said) one.textContent = said;
    const wrote = fill("agentWroteIt", by);
    const where = signing === "named" ? null : wrote.charAt(0).toUpperCase() + wrote.slice(1);
    if (where === null) one.removeAttribute("title");
    else if (one.title !== where) one.title = where;
    if (signing === "mark") {
      if (one.getAttribute("aria-label") !== where) one.setAttribute("aria-label", where ?? "");
    } else one.removeAttribute("aria-label");
  }
};
