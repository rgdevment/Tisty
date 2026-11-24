import {
  DESIGNS,
  type Design,
  designNamed,
  FONTS,
  glimpseOf,
  HEADINGS,
  INKS,
  isOwn,
  OWN,
  type Own,
  ownIn,
  ownSaid,
  type Panel,
  SHEETS,
  SYSTEM,
} from "tisty-editor/read";
import { locale, t, type Word } from "../locales";

type Choosing = NonNullable<Extract<Panel, { kind: "paper" }>["design"]>;

const SAID: Record<(typeof DESIGNS)[number], Word> = {
  tisty: "designTisty",
  bosque: "designBosque",
  coral: "designCoral",
  fiordo: "designFiordo",
  violeta: "designVioleta",
};

const tones = (own: Own, part: "sheet" | "heading" | "ink", list: typeof SHEETS) =>
  Object.entries(list).map(([name, tone]) => ({
    key: ownSaid({ ...own, [part]: name }),
    label: locale().startsWith("es") ? tone.es : tone.en,
    on: own[part] === name,
    light: tone.light,
    dark: tone.dark,
  }));

const letters = (own: Own) =>
  [SYSTEM, ...Object.keys(FONTS)].map((name) => ({
    key: ownSaid({ ...own, font: name }),
    label: FONTS[name]?.name ?? t("designSystem"),
    on: own.font === name,
    face: FONTS[name] ? `"${FONTS[name].family}", ${FONTS[name].falls}` : "inherit",
  }));

// The last design of its own is offered back, so trying another one loses nothing.
export const designing = (
  now: Design,
  last: Design | null,
  onPick: (design: Design) => void,
): Choosing => {
  const own = isOwn(now) ? now : last && isOwn(last) ? last : ownSaid(OWN);
  const all = [...DESIGNS, own].map((one) => ({
    key: one,
    label: isOwn(one) ? t("designOwn") : t(SAID[one as keyof typeof SAID]),
    on: one === now,
    ...glimpseOf(one),
  }));
  const parts = ownIn(own);
  return {
    label: t("designIs"),
    all,
    own: isOwn(now)
      ? {
          colors: [
            { label: t("designSheet"), all: tones(parts, "sheet", SHEETS) },
            { label: t("designHeadings"), all: tones(parts, "heading", HEADINGS) },
            { label: t("designInk"), all: tones(parts, "ink", INKS) },
          ],
          font: { label: t("designFont"), all: letters(parts) },
        }
      : undefined,
    onPick: (key) => onPick(designNamed(key)),
  };
};
