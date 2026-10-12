import { getCurrentWindow } from "@tauri-apps/api/window";
import { save as intoFile } from "@tauri-apps/plugin-dialog";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { useCallback, useEffect, useRef, useState } from "react";
import { docCopy, keepPdf, type Paper, type Showing, viewGo, viewImport, viewShown } from "./core";
import { adopt, fill, t } from "./locales";
import { saidPlainly } from "./refusal";
import Menu, { type Choice } from "./ui/Menu";
import { pdfOfText, signature } from "./ui/printing";
import { pageFor } from "./ui/sheets";
import { button, Glyph, Segments, tinted, tool } from "./ui/viewer/Glyphs";
import Preview from "./ui/viewer/Preview";
import Problem, { type Trouble } from "./ui/viewer/Problem";
import Reader from "./ui/viewer/Reader";
import Stage, { PRESETS, type Preset } from "./ui/viewer/Stage";
import WindowChrome, { clearOfChrome, onMac } from "./ui/WindowChrome";

const KEPT = "tisty.viewer";
const ZOOMS = [0.8, 0.9, 1, 1.1, 1.25, 1.4, 1.6];

const remembered = (): { side: boolean; leaf: Paper } => {
  try {
    const said = JSON.parse(localStorage.getItem(KEPT) ?? "{}") as Record<string, unknown>;
    const leaf = ["a4", "letter", "tabloid"].includes(String(said.leaf)) ? said.leaf : "a4";
    return { side: said.side !== false, leaf: leaf as Paper };
  } catch {
    return { side: true, leaf: "a4" };
  }
};

const troubleOf = (refusal: { code: string; name?: string }): Trouble =>
  refusal.code === "documentTooBig"
    ? { kind: "big", limit: refusal.name ?? "" }
    : { kind: "unreadable" };

export default function Viewing() {
  useState(() => adopt(new URLSearchParams(window.location.search).get("locale") ?? undefined));
  const [shown, setShown] = useState<Showing | null>(null);
  const [failed, setFailed] = useState<string | null>(null);
  const [note, setNote] = useState<string | null>(null);
  const [band, setBand] = useState<string | null>(null);
  const [{ side, leaf }, setKept] = useState(remembered);
  const [raw, setRaw] = useState(false);
  const [zoom, setZoom] = useState(2);
  const [finding, setFinding] = useState(false);
  const [menu, setMenu] = useState<{ x: number; y: number } | null>(null);
  const [preset, setPreset] = useState<Preset>("fit");
  const [actual, setActual] = useState(false);
  const [dark, setDark] = useState<boolean | null>(null);
  const [again, setAgain] = useState(0);
  const [code, setCode] = useState(false);
  const [seeing, setSeeing] = useState<string | null>(null);
  const [signing, setSigning] = useState(false);
  const [making, setMaking] = useState(false);
  const more = useRef<HTMLButtonElement>(null);

  const look = useCallback(
    () =>
      viewShown()
        .then((one) => {
          setShown(one);
          setFailed(null);
        })
        .catch((e) => setFailed(saidPlainly(e))),
    [],
  );

  useEffect(() => {
    void look();
  }, [look]);

  useEffect(() => {
    try {
      localStorage.setItem(KEPT, JSON.stringify({ side, leaf }));
    } catch {
      void 0;
    }
    pageFor(leaf);
  }, [side, leaf]);

  const reading = shown?.kind === "file" || shown?.kind === "doc";
  const body = shown?.kind === "file" || shown?.kind === "doc" ? shown.body : "";
  const paper = shown?.kind === "doc" ? shown.paper : null;

  useEffect(() => {
    if (shown?.kind !== "doc") return;
    const fresh = () => void look();
    window.addEventListener("focus", fresh);
    return () => window.removeEventListener("focus", fresh);
  }, [shown?.kind, look]);

  const failing = (e: unknown) => setNote(saidPlainly(e));

  const editIn = (open: boolean) => {
    if (paper) return void viewGo(paper.file).catch(failing);
    const place = shown?.kind === "file" ? shown.place : "";
    viewImport(open)
      .then(() => look())
      .then(() =>
        setBand(`${t(open ? "viewNowIn" : "viewSavedIn")} · ${fill("viewOriginal", place)}`),
      )
      .catch(failing);
  };

  const madePdf = async () => {
    const signed = paper && signing ? await signature(paper.file) : undefined;
    return pdfOfText(body, leaf, signed);
  };

  const preview = async () => {
    if (!reading || making) return;
    setMaking(true);
    try {
      const blob = await madePdf();
      setSeeing((was) => {
        if (was) URL.revokeObjectURL(was);
        return URL.createObjectURL(blob);
      });
    } catch (e) {
      failing(e);
    } finally {
      setMaking(false);
    }
  };

  const keepIt = async () => {
    if (!reading || making) return;
    setMaking(true);
    try {
      const named = shown?.kind === "file" ? shown.name.replace(/\.[^.]+$/, "") : paper?.title;
      const where = await intoFile({
        defaultPath: `${named || t("untitledDoc")}.pdf`,
        filters: [{ name: "PDF", extensions: ["pdf"] }],
      });
      if (!where) return;
      const blob = await madePdf();
      await keepPdf(where, Array.from(new Uint8Array(await blob.arrayBuffer())));
    } catch (e) {
      failing(e);
    } finally {
      setMaking(false);
    }
  };

  // biome-ignore lint/correctness/useExhaustiveDependencies: only the paper and the signature redraw an open preview
  useEffect(() => {
    if (seeing) void preview();
  }, [leaf, signing]);

  const shut = () => {
    if (seeing) URL.revokeObjectURL(seeing);
    setSeeing(null);
  };

  const keys = useRef<(e: KeyboardEvent) => void>(() => {});
  keys.current = (e: KeyboardEvent) => {
    const held = e.ctrlKey || e.metaKey;
    if (e.key === "Escape") {
      if (seeing) shut();
      else if (code) setCode(false);
      else setFinding(false);
      return;
    }
    if (!held) return;
    const key = e.key.toLowerCase();
    const doing: Record<string, () => void> = {
      w: () => void getCurrentWindow().close(),
      r: () => setAgain((n) => n + 1),
    };
    if (reading) {
      Object.assign(doing, {
        f: () => setFinding(true),
        "\\": () => setKept((was) => ({ ...was, side: !was.side })),
        p: () => void preview(),
        "=": () => setZoom((at) => Math.min(ZOOMS.length - 1, at + 1)),
        "+": () => setZoom((at) => Math.min(ZOOMS.length - 1, at + 1)),
        "-": () => setZoom((at) => Math.max(0, at - 1)),
        "0": () => setZoom(2),
      });
      if (e.shiftKey && key === "m") doing.m = () => setRaw((was) => !was);
    }
    if (shown?.kind === "widget") {
      PRESETS.forEach((one, at) => {
        doing[String(at + 1)] = () => setPreset(one.key);
      });
      doing["0"] = () => setActual((was) => !was);
    }
    const act = doing[key];
    if (!act) return;
    e.preventDefault();
    act();
  };
  useEffect(() => {
    const heard = (e: KeyboardEvent) => keys.current(e);
    window.addEventListener("keydown", heard);
    return () => window.removeEventListener("keydown", heard);
  }, []);

  const choices = (): Choice[] => {
    if (shown?.kind === "widget") {
      const from = shown.from;
      return [
        {
          key: "again",
          label: t("viewReload"),
          hint: "Ctrl+R",
          onPick: () => setAgain((n) => n + 1),
        },
        {
          key: "code",
          label: t("viewCode"),
          off: !("source" in shown.lent),
          onPick: () => setCode(true),
        },
        {
          key: "doc",
          label: t("viewInDocGo"),
          off: !from,
          onPick: () => from && void viewGo(from.file).catch(failing),
        },
        {
          key: "dark",
          label: `${dark ? "✓ " : ""}${t("viewDark")}`,
          apart: true,
          onPick: () => setDark((was) => !was),
        },
        {
          key: "actual",
          label: `${actual ? "✓ " : ""}${t("viewActual")}`,
          hint: "Ctrl+0",
          onPick: () => setActual((was) => !was),
        },
      ];
    }
    const path = shown?.kind === "file" || shown?.kind === "missed" ? shown.path : null;
    const reveal = {
      key: "reveal",
      label: t("viewReveal"),
      off: !path,
      onPick: () => path && void revealItemInDir(path).catch(failing),
    };
    if (shown?.kind === "missed")
      return [reveal, { key: "retry", label: t("viewRetry"), onPick: () => void look() }];
    return [
      { key: "find", label: t("viewFind"), hint: "Ctrl+F", onPick: () => setFinding(true) },
      {
        key: "bigger",
        label: t("viewBigger"),
        hint: "Ctrl+=",
        onPick: () => setZoom((at) => Math.min(ZOOMS.length - 1, at + 1)),
      },
      {
        key: "smaller",
        label: t("viewSmaller"),
        hint: "Ctrl+−",
        onPick: () => setZoom((at) => Math.max(0, at - 1)),
      },
      {
        key: "copy",
        label: t("viewCopyMd"),
        onPick: () => void navigator.clipboard.writeText(body).catch(failing),
      },
      { key: "see", label: t("seePdf"), hint: "Ctrl+P", apart: true, onPick: () => void preview() },
      { key: "pdf", label: t("toPdf"), onPick: () => void keepIt() },
      ...(paper
        ? [
            {
              key: "copyDoc",
              label: t("viewDuplicate"),
              apart: true,
              onPick: () =>
                void docCopy(paper.id)
                  .then((made) => viewGo(made.id))
                  .catch(failing),
            },
          ]
        : [
            { key: "quiet", label: t("viewImportQuiet"), apart: true, onPick: () => editIn(false) },
            { ...reveal },
          ]),
    ];
  };

  const title =
    shown?.kind === "file" || shown?.kind === "missed"
      ? shown.name
      : shown?.kind === "doc"
        ? shown.paper.title || t("untitledDoc")
        : shown?.kind === "widget" && shown.from
          ? shown.from.title
          : t("viewWidgetTitle");
  const under =
    shown?.kind === "file"
      ? `${shown.place} · ${t("viewOnlyReading")}`
      : shown?.kind === "missed"
        ? shown.place
        : shown?.kind === "doc"
          ? `${shown.paper.place || "Tisty"} · ${t("viewOnlyReading")}`
          : shown?.kind === "widget"
            ? shown.from
              ? fill("viewWidgetOf", shown.from.title)
              : t("viewWidgetTitle")
            : "";
  const kind = shown?.kind === "doc" ? "doc" : shown?.kind === "widget" ? "widget" : null;

  return (
    <main className="relative flex h-full flex-col bg-desk font-sans">
      <WindowChrome />
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
              onClick={() => setKept((was) => ({ ...was, side: !was.side }))}
            >
              <Glyph name="list" />
            </button>
            <Segments
              value={raw ? "raw" : "read"}
              options={[
                { key: "read", label: t("viewRead") },
                { key: "raw", label: t("viewSource") },
              ]}
              onPick={(one) => setRaw(one === "raw")}
            />
            <button type="button" className={paper ? button : tinted} onClick={() => editIn(true)}>
              {t("viewEditIn")}
            </button>
          </>
        )}
        {shown?.kind === "widget" && (
          <>
            <Segments
              value={preset}
              options={PRESETS.map((one) => ({ key: one.key, label: t(one.word) }))}
              onPick={setPreset}
            />
            <button
              type="button"
              className={tool}
              title={`${t("viewReload")} (Ctrl+R)`}
              aria-label={t("viewReload")}
              onClick={() => setAgain((n) => n + 1)}
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
              setMenu(box ? { x: box.right - 236, y: box.bottom + 4 } : null);
            }}
          >
            <Glyph name="more" />
          </button>
        )}
      </header>
      {band && (
        <div className="flex shrink-0 items-center gap-2.5 border-b border-hair bg-accent-soft px-4 py-[7px] text-[12.5px] text-soft">
          <span>{band}</span>
          <button
            type="button"
            className="ml-auto text-[12.5px] font-semibold text-accent"
            onClick={() => setBand(null)}
          >
            {t("viewGotIt")}
          </button>
        </div>
      )}
      {(note || failed) && (
        <p className="shrink-0 px-4 py-1.5 text-[12.5px] text-urgent" role="alert">
          {note ?? failed}
        </p>
      )}
      {reading && (shown.kind === "doc" || body.trim()) && (
        <Reader
          key={shown.kind === "doc" ? shown.paper.file : shown.path}
          body={body}
          title={title}
          stranger={shown.kind === "file"}
          side={side}
          raw={raw}
          zoom={ZOOMS[zoom]}
          leaf={leaf}
          finding={finding}
          onLeaf={(one) => setKept((was) => ({ ...was, leaf: one }))}
          onRaw={setRaw}
          onFound={() => setFinding(false)}
          onDoc={(file) => void viewGo(file).catch(failing)}
        />
      )}
      {shown?.kind === "file" && !body.trim() && (
        <Problem
          trouble={{ kind: "empty" }}
          name={shown.name}
          place={shown.place}
          onRetry={look}
          onEdit={() => editIn(true)}
        />
      )}
      {shown?.kind === "missed" && (
        <Problem
          trouble={troubleOf(shown.refusal)}
          name={shown.name}
          place={shown.place}
          onReveal={() => void revealItemInDir(shown.path).catch(failing)}
          onRetry={look}
        />
      )}
      {shown?.kind === "widget" && (
        <Stage lent={shown.lent} preset={preset} actual={actual} dark={dark} again={again} />
      )}
      {code && shown?.kind === "widget" && "source" in shown.lent && (
        <div className="absolute inset-x-0 top-11 bottom-0 z-20 flex flex-col bg-desk">
          <div className="flex h-10 shrink-0 items-center border-b border-hair bg-panel pr-2.5 pl-4 text-[12.5px]">
            <b className="font-semibold">{t("viewCode")}</b>
            <button
              type="button"
              className={`ml-auto ${button} border-transparent text-soft`}
              onClick={() => setCode(false)}
            >
              {t("viewClose")}
            </button>
          </div>
          <pre className="scroller m-0 min-h-0 flex-1 whitespace-pre-wrap break-words px-6 py-5 font-mono text-[12.5px] leading-relaxed">
            {shown.lent.source}
          </pre>
        </div>
      )}
      {seeing && (
        <Preview
          shown={seeing}
          leaf={leaf}
          signing={paper ? signing : null}
          making={making}
          onSign={setSigning}
          onKeep={() => void keepIt()}
          onClose={shut}
        />
      )}
      {menu && (
        <Menu at={menu} choices={choices()} label={t("viewMore")} onClose={() => setMenu(null)} />
      )}
    </main>
  );
}
