import { useRef } from "react";
import type { Showing } from "../../core";
import { fill, t } from "../../locales";
import { clearOfChrome, onMac } from "../WindowChrome";
import { button, Glyph, Segments, tinted, tool } from "./Glyphs";
import { PRESETS, type Preset } from "./Stage";

interface Props {
  shown: Showing | null;
  side: boolean;
  raw: boolean;
  busy: boolean;
  preset: Preset;
  menu: boolean;
  onSide: () => void;
  onRaw: (raw: boolean) => void;
  onEdit: () => void;
  onPreset: (one: Preset) => void;
  onAgain: () => void;
  onMore: (at: { x: number; y: number }) => void;
}

export const titled = (shown: Showing | null): { title: string; under: string } => {
  switch (shown?.kind) {
    case "file":
      return { title: shown.name, under: `${shown.place} · ${t("viewOnlyReading")}` };
    case "missed":
      return { title: shown.name, under: shown.place };
    case "doc":
      return {
        title: shown.paper.title || t("untitledDoc"),
        under: `${shown.paper.place || "Tisty"} · ${t("viewOnlyReading")}`,
      };
    case "widget":
      return shown.from
        ? { title: shown.from.title, under: fill("viewWidgetOf", shown.from.title) }
        : { title: t("viewWidgetTitle"), under: t("viewWidgetTitle") };
    default:
      return { title: "", under: "" };
  }
};

export default function Header(props: Props) {
  const { shown, side, raw, busy, preset, menu } = props;
  const more = useRef<HTMLButtonElement>(null);
  const { title, under } = titled(shown);
  const reading = shown?.kind === "file" || shown?.kind === "doc";
  const kind = shown?.kind === "doc" ? "doc" : shown?.kind === "widget" ? "widget" : null;

  return (
    <header
      data-tauri-drag-region
      className={`flex h-11 shrink-0 items-center gap-2 border-b border-hair bg-panel ${onMac ? "pl-20" : "pl-3.5"} ${clearOfChrome}`}
    >
      <div
        data-tauri-drag-region
        className="grid h-[22px] w-[22px] shrink-0 place-items-center rounded-md bg-accent-soft text-[9px] font-bold text-accent"
      >
        {kind ? <Glyph name={kind} className="h-3.5 w-3.5" /> : "MD"}
      </div>
      <div data-tauri-drag-region className="mr-1.5 flex min-w-0 flex-1 flex-col leading-tight">
        <b data-tauri-drag-region className="truncate text-[13px] font-semibold">
          {title}
        </b>
        <small data-tauri-drag-region className="mt-0.5 truncate text-[11.5px] text-faint">
          {under}
        </small>
      </div>
      {reading && (
        <>
          <button
            type="button"
            className={`${tool} ${side ? "bg-hover text-ink" : ""}`}
            title={`${t("viewIndex")} (Ctrl+\\)`}
            aria-label={t("viewIndex")}
            aria-pressed={side}
            onClick={props.onSide}
          >
            <Glyph name="list" />
          </button>
          <Segments
            value={raw ? "raw" : "read"}
            options={[
              { key: "read", label: t("viewRead") },
              { key: "raw", label: t("viewSource") },
            ]}
            onPick={(one) => props.onRaw(one === "raw")}
          />
          <button
            type="button"
            disabled={busy}
            className={shown.kind === "doc" ? button : tinted}
            onClick={props.onEdit}
          >
            {t("viewEditIn")}
          </button>
        </>
      )}
      {shown?.kind === "widget" && (
        <>
          <Segments
            value={preset}
            options={PRESETS.map((one) => ({ key: one.key, label: t(one.word) }))}
            onPick={props.onPreset}
          />
          <button
            type="button"
            className={tool}
            title={`${t("viewReload")} (Ctrl+R)`}
            aria-label={t("viewReload")}
            onClick={props.onAgain}
          >
            <Glyph name="reload" />
          </button>
        </>
      )}
      {shown && (
        <button
          ref={more}
          type="button"
          className={`${tool} ${menu ? "bg-hover text-ink" : ""}`}
          title={t("viewMore")}
          aria-label={t("viewMore")}
          onClick={() => {
            const box = more.current?.getBoundingClientRect();
            if (box) props.onMore({ x: box.right - 236, y: box.bottom + 4 });
          }}
        >
          <Glyph name="more" />
        </button>
      )}
    </header>
  );
}
