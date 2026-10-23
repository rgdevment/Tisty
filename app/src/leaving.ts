import { t } from "./locales";
import type { Choice } from "./ui/Menu";

const asked = (name: string) => () => window.dispatchEvent(new CustomEvent(name));

// One list of the ways a document leaves, for its menu and its column alike; Docs answers each.
export const waysOut = (off: boolean, making = false): Choice[] => [
  {
    key: "seePdf",
    icon: "▤",
    label: making ? t("makingPdf") : t("previewPdf"),
    off,
    onPick: making ? undefined : asked("tisty:see-pdf"),
  },
  {
    key: "asPdf",
    icon: "⇩",
    label: t("toPdf"),
    off,
    onPick: making ? undefined : asked("tisty:to-pdf"),
  },
  { key: "asHtml", icon: "⇩", label: t("toHtml"), off, onPick: asked("tisty:to-html") },
  { key: "asWord", icon: "⇩", label: t("toWord"), off, onPick: asked("tisty:to-word") },
];
