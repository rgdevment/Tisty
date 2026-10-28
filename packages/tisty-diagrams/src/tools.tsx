import { useReactFlow, useViewport } from "@xyflow/react";
import { type ReactNode, useEffect, useRef, useState } from "react";
import { FILLS, LINES, type Look } from "./flowstyles";
import { ALIGNMENTS, type Alignment } from "./flowtools";
import { useSay } from "./handing";
import type { Word } from "./words";

const TOOL =
  "flex h-8 w-8 items-center justify-center rounded-md text-soft hover:bg-hover hover:text-ink disabled:opacity-35 disabled:hover:bg-transparent aria-pressed:bg-accent-soft aria-pressed:text-accent";

// Each icon is drawn in a 24 box with the line of the words around it.
const ICONS: Record<string, string> = {
  undo: "M9 14 L4 9 L9 4 M4 9 H15 a5 5 0 0 1 0 10 H11",
  redo: "M15 14 L20 9 L15 4 M20 9 H9 a5 5 0 0 0 0 10 H13",
  remove: "M4 7 H20 M10 11 V17 M14 11 V17 M6 7 L7 20 H17 L18 7 M9 7 V4 H15 V7",
  duplicate: "M8 8 h12 v12 h-12 Z M16 8 V4 H4 V16 H8",
  fill: "M12 3 C12 3 6 10 6 14 a6 6 0 0 0 12 0 C18 10 12 3 12 3 Z",
  line: "M4 4 h16 v16 h-16 Z",
  "-->": "M3 12 H19 M14 7 L19 12 L14 17",
  "---": "M3 12 H21",
  "-.->": "M3 12 H6 M9 12 H12 M15 12 H19 M14 7 L19 12 L14 17",
  "==>": "M3 10 H18 M3 14 H18 M14 6 L20 12 L14 18",
  left: "M4 3 V21 M8 7 H18 M8 12 H14 M8 17 H20",
  center: "M12 3 V21 M6 7 H18 M8 12 H16 M5 17 H19",
  right: "M20 3 V21 M6 7 H16 M10 12 H16 M4 17 H16",
  top: "M3 4 H21 M7 8 V18 M12 8 V14 M17 8 V20",
  middle: "M3 12 H21 M7 6 V18 M12 8 V16 M17 5 V19",
  bottom: "M3 20 H21 M7 6 V16 M12 10 V16 M17 4 V16",
  relaid: "M4 4 h6 v6 h-6 Z M14 14 h6 v6 h-6 Z M10 7 H17 V14",
  auto: "M4 4 h6 v6 h-6 Z M14 14 h6 v6 h-6 Z M10 7 H17 V14 M3 17 a4 4 0 0 0 7 3 M3 21 V17 H7",
  zoomOut: "M6 12 H18",
  zoomIn: "M12 6 V18 M6 12 H18",
  fit: "M4 9 V4 H9 M15 4 H20 V9 M20 15 V20 H15 M9 20 H4 V15",
  inherits: "M3 12 H14 M14 6 L21 12 L14 18 Z",
  realizes: "M3 12 H5 M8 12 H10 M13 12 H14 M14 6 L21 12 L14 18 Z",
  composes: "M3 12 L7 8 L11 12 L7 16 Z M5 12 H9 M11 12 H21",
  aggregates: "M3 12 L7 8 L11 12 L7 16 Z M11 12 H21",
  associates: "M3 12 H20 M15 7 L20 12 L15 17",
  depends: "M3 12 H5 M8 12 H10 M13 12 H15 M18 12 H20 M15 7 L20 12 L15 17",
  links: "M3 12 H21",
  child: "M3 5 h8 v5 h-8 Z M13 14 h8 v5 h-8 Z M7 10 V16.5 H13",
  sibling: "M3 4 h8 v5 h-8 Z M3 15 h8 v5 h-8 Z M7 9 V15 M15 12 H21 M18 9 V15",
};

const ALIGN_WORDS: Record<Alignment, Word> = {
  left: "flowAlignLeft",
  center: "flowAlignCenter",
  right: "flowAlignRight",
  top: "flowAlignTop",
  middle: "flowAlignMiddle",
  bottom: "flowAlignBottom",
};

export function Icon({ name }: Readonly<{ name: string }>) {
  return (
    <svg
      aria-hidden="true"
      width="18"
      height="18"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.75"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d={ICONS[name]} />
    </svg>
  );
}

export function Tool({
  label,
  icon,
  onClick,
  disabled = false,
  pressed,
  hint,
}: Readonly<{
  label: string;
  icon: string;
  onClick: () => void;
  disabled?: boolean;
  pressed?: boolean;
  hint?: string;
}>) {
  return (
    <button
      type="button"
      aria-label={label}
      title={hint ?? label}
      aria-pressed={pressed}
      disabled={disabled}
      onClick={onClick}
      className={TOOL}
    >
      <Icon name={icon} />
    </button>
  );
}

export const Gap = () => <span aria-hidden="true" className="mx-1 h-5 w-px bg-line" />;

/** A tool that opens a small panel under it, closed by a choice, Escape or a press elsewhere. */
function Popover({
  label,
  icon,
  disabled,
  look,
  children,
}: Readonly<{
  label: string;
  icon: string;
  disabled: boolean;
  look: string;
  children: (close: () => void) => ReactNode;
}>) {
  const [open, setOpen] = useState(false);
  const holder = useRef<HTMLDivElement>(null);
  // A press anywhere else closes it; on a Mac a button takes no focus, so leaving focus is not enough.
  useEffect(() => {
    if (!open) return;
    const away = (e: PointerEvent) => {
      if (!holder.current?.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("pointerdown", away);
    return () => document.removeEventListener("pointerdown", away);
  }, [open]);
  return (
    // biome-ignore lint/a11y/noStaticElementInteractions: Escape closes the panel inside
    <div
      ref={holder}
      className="relative"
      onKeyDown={(e) => {
        if (e.key === "Escape") setOpen(false);
      }}
    >
      <button
        type="button"
        aria-label={label}
        title={label}
        aria-expanded={open}
        disabled={disabled}
        onClick={() => setOpen(!open)}
        className={TOOL}
      >
        <Icon name={icon} />
      </button>
      {open && !disabled && (
        <div
          role="group"
          aria-label={label}
          className={`absolute top-full z-20 mt-1 rounded-md border border-line bg-panel p-2 shadow ${look}`}
        >
          {children(() => setOpen(false))}
        </div>
      )}
    </div>
  );
}

/** A colour to choose from a row of swatches, or none. */
export function Swatches({
  label,
  icon,
  disabled,
  choices,
  onChoose,
}: Readonly<{
  label: string;
  icon: string;
  disabled: boolean;
  choices: readonly { key: string; label: string; color: string }[];
  onChoose: (key: string | null) => void;
}>) {
  const t = useSay();
  return (
    <Popover label={label} icon={icon} disabled={disabled} look="left-0 flex gap-1.5">
      {(close) => (
        <>
          <button
            type="button"
            aria-label={t("flowNoColor")}
            title={t("flowNoColor")}
            onClick={() => {
              close();
              onChoose(null);
            }}
            className="h-6 w-6 rounded-full border border-dashed border-faint"
          />
          {choices.map((one) => (
            <button
              key={one.key}
              type="button"
              aria-label={one.label}
              title={one.label}
              onClick={() => {
                close();
                onChoose(one.key);
              }}
              className="h-6 w-6 rounded-full border border-line"
              style={{ background: one.color }}
            />
          ))}
        </>
      )}
    </Popover>
  );
}

/** The bar of tools over a canvas, told apart from it by its own name. */
export function ToolBar({ children }: Readonly<{ children: ReactNode }>) {
  const t = useSay();
  return (
    <div
      role="toolbar"
      aria-label={t("flowTools")}
      className="flex shrink-0 flex-wrap items-center gap-0.5 border-b border-hair bg-panel px-2 py-1"
    >
      {children}
    </div>
  );
}

export function UndoTools({
  undo,
  redo,
}: Readonly<{ undo: (() => void) | null; redo: (() => void) | null }>) {
  const t = useSay();
  return (
    <>
      <Tool label={t("flowUndo")} icon="undo" disabled={!undo} onClick={() => undo?.()} />
      <Tool label={t("flowRedo")} icon="redo" disabled={!redo} onClick={() => redo?.()} />
    </>
  );
}

export function RemoveTools({
  removable,
  refused,
  onRemove,
  onDuplicate,
}: Readonly<{
  removable: boolean;
  refused: string | undefined;
  onRemove: () => void;
  onDuplicate: (() => void) | null;
}>) {
  const t = useSay();
  return (
    <>
      <Tool
        label={t("flowDelete")}
        icon="remove"
        disabled={!removable}
        hint={refused}
        onClick={onRemove}
      />
      <Tool
        label={t("flowDuplicate")}
        icon="duplicate"
        disabled={!onDuplicate}
        onClick={() => onDuplicate?.()}
      />
    </>
  );
}

/** A fill and a line colour for what is chosen, written in its `style` line. */
export function PaintTools({
  disabled,
  onStyle,
}: Readonly<{
  disabled: boolean;
  onStyle: (look: { readonly [key in keyof Look]?: string | null }) => void;
}>) {
  const t = useSay();
  return (
    <>
      <Swatches
        label={t("flowFill")}
        icon="fill"
        disabled={disabled}
        choices={FILLS.map((one) => ({ key: one.fill, label: t(one.word), color: one.fill }))}
        onChoose={(key) =>
          onStyle({ fill: key, color: FILLS.find((one) => one.fill === key)?.color ?? null })
        }
      />
      <Swatches
        label={t("flowLine")}
        icon="line"
        disabled={disabled}
        choices={LINES.map((one) => ({ key: one.stroke, label: t(one.word), color: one.stroke }))}
        onChoose={(key) => onStyle({ stroke: key })}
      />
    </>
  );
}

export function AlignTools({
  disabled,
  onAlign,
}: Readonly<{ disabled: boolean; onAlign: (how: Alignment) => void }>) {
  const t = useSay();
  return (
    <>
      {ALIGNMENTS.map((how) => (
        <Tool
          key={how}
          label={t(ALIGN_WORDS[how])}
          icon={how}
          disabled={disabled}
          onClick={() => onAlign(how)}
        />
      ))}
    </>
  );
}

/** Laying out again and zooming, pushed to the far end of the bar. */
export function ViewTools({
  laidOut,
  onRelay,
}: Readonly<{
  laidOut: boolean;
  // One way to lay out, or several to choose from, such as the directions a flowchart runs.
  onRelay: (() => void) | readonly { readonly label: string; readonly onPick: () => void }[];
}>) {
  const t = useSay();
  const { zoomIn, zoomOut, fitView } = useReactFlow();
  const { zoom } = useViewport();
  return (
    <>
      {typeof onRelay === "function" ? (
        <Tool label={t("flowRelaid")} icon="relaid" disabled={!laidOut} onClick={onRelay} />
      ) : (
        <Popover
          label={t("flowRelaid")}
          icon="relaid"
          disabled={!laidOut}
          look="left-0 flex flex-col"
        >
          {(close) =>
            onRelay.map((one) => (
              <button
                key={one.label}
                type="button"
                onClick={() => {
                  close();
                  one.onPick();
                }}
                className="rounded px-2 py-1 text-left text-[13px] whitespace-nowrap text-ink hover:bg-hover"
              >
                {one.label}
              </button>
            ))
          }
        </Popover>
      )}
      <ZoomTools
        scale={zoom}
        onOut={() => zoomOut()}
        onIn={() => zoomIn()}
        onFit={() => fitView()}
      />
    </>
  );
}

/** How a canvas zooms, whatever draws it. */
export interface Zooming {
  readonly scale: number;
  readonly onOut: () => void;
  readonly onIn: () => void;
  readonly onFit: () => void;
}

/** Zooming, pushed to the far end of the bar. */
export function ZoomTools({ scale, onOut, onIn, onFit }: Readonly<Zooming>) {
  const t = useSay();
  return (
    <>
      <span className="flex-1" />
      <Tool label={t("flowZoomOut")} icon="zoomOut" onClick={onOut} />
      <span className="w-12 text-center text-[12px] text-soft tabular-nums">
        {Math.round(scale * 100)} %
      </span>
      <Tool label={t("flowZoomIn")} icon="zoomIn" onClick={onIn} />
      <Tool label={t("flowFit")} icon="fit" onClick={onFit} />
    </>
  );
}
