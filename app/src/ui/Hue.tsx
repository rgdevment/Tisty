import { t } from "../locales";

export const HUES = [
  "red",
  "orange",
  "amber",
  "green",
  "teal",
  "blue",
  "indigo",
  "purple",
  "pink",
  "brown",
  "gray",
] as const;

export type Hue = (typeof HUES)[number];

/// Tailwind only ships classes it can see written out, so a built name would arrive unstyled.
const CLASS: Record<Hue, string> = {
  red: "text-hue-red",
  orange: "text-hue-orange",
  amber: "text-hue-amber",
  green: "text-hue-green",
  teal: "text-hue-teal",
  blue: "text-hue-blue",
  indigo: "text-hue-indigo",
  purple: "text-hue-purple",
  pink: "text-hue-pink",
  brown: "text-hue-brown",
  gray: "text-hue-gray",
};

/// Stored as a key, like the icon, so the palette can be retuned without touching anyone's lists.
export const painted = (hue?: string | null): string =>
  hue && hue in CLASS ? CLASS[hue as Hue] : "text-soft";

interface Props {
  chosen?: string | null;
  onPick: (hue: string | undefined) => void;
  onHold?: (e: React.MouseEvent) => void;
}

export default function Hue({ chosen, onPick, onHold }: Props) {
  return (
    <fieldset className="flex flex-wrap gap-1.5">
      <legend className="sr-only">{t("pickAColour")}</legend>
      <button
        type="button"
        onMouseDown={onHold}
        onClick={() => onPick(undefined)}
        aria-pressed={!chosen}
        aria-label={t("noColour")}
        title={t("noColour")}
        className={`grid h-6 w-6 place-items-center rounded-full border text-[10px] text-faint ${
          chosen ? "border-line hover:bg-hover" : "border-accent bg-accent-soft"
        }`}
      >
        ○
      </button>
      {HUES.map((hue) => (
        <button
          key={hue}
          type="button"
          onMouseDown={onHold}
          onClick={() => onPick(hue)}
          aria-pressed={chosen === hue}
          aria-label={t(`hue_${hue}`)}
          title={t(`hue_${hue}`)}
          className={`h-6 w-6 rounded-full bg-current ring-offset-2 ring-offset-bg ${painted(hue)} ${
            chosen === hue ? "ring-2 ring-current" : ""
          }`}
        />
      ))}
    </fieldset>
  );
}
