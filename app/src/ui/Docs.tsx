import { ask, save as intoFile, open as pick } from "@tauri-apps/plugin-dialog";
import { lazy, Suspense, useCallback, useEffect, useRef, useState } from "react";
import { asPlain } from "../copying";
import {
  attach,
  attachExport,
  attached,
  convertPaper,
  docExport,
  docRead,
  docWrite,
  type Filed,
  keepPdf,
  opened,
  type Paper,
  roomy,
} from "../core";
import { frail } from "../frail";
import { fill, t } from "../locales";
import { crowd, ending, MANY, weighed } from "../previews";
import { saidPlainly } from "../refusal";
import { busy, holds, queued } from "../saving";
import Beside from "./Beside";
import type { Block } from "./Slash";
import { clearOfChrome } from "./WindowChrome";
import type { Head } from "./writing";

const Editor = lazy(() => import("./Editor"));

const SETTLES = 700;

const WIDE = 1440;

const RAIL = 284;
const SHEET = 820;

const PAPER: Record<Paper, number> = { a4: 820, letter: 843, tabloid: 1090 };
const leaves = (): Record<string, Paper> => {
  try {
    const said: unknown = JSON.parse(localStorage.getItem("tisty.paper") ?? "{}");
    if (!said || typeof said !== "object") return {};
    return Object.fromEntries(
      Object.entries(said as Record<string, string>).filter(([, leaf]) => leaf in PAPER),
    ) as Record<string, Paper>;
  } catch {
    return {};
  }
};
const PAGE: Record<Paper, string> = { a4: "A4", letter: "Letter", tabloid: "11in 17in" };

const ASIDE = 344;

interface Props {
  open?: string;
  known: Filed[];
  onKept: (doc: { id: string; title: string }) => void;
  onError: (problem: unknown) => void;
  onDoc?: (id: string) => void;
  onShown?: (file: string | null) => void;
  fresh?: number;
}

export default function Docs({
  open: asked,
  known,
  onKept,
  onError,
  onDoc,
  onShown,
  fresh = 0,
}: Props) {
  const [open, setOpen] = useState<Filed | null>(null);
  const [body, setBody] = useState("");
  const [saving, setSaving] = useState(false);
  const [packed, setPacked] = useState(0);
  const shaped = useRef("");
  const seen = useRef(0);
  const [stuck, setStuck] = useState(false);
  const [clashed, setClashed] = useState(false);
  const settling = useRef<ReturnType<typeof setTimeout>>(null);
  const held = useRef<{ id: string; body: string } | null>(null);
  const turn = useRef(0);
  const [room, setRoom] = useState(() => window.innerWidth);
  const [shown, setShown] = useState<boolean | null>(null);
  const wide = room >= WIDE;
  const [blocks, setBlocks] = useState<Block[]>([]);
  const [heads, setHeads] = useState<Head[]>([]);
  const [saved, setSaved] = useState(0);
  const [sized, setSized] = useState<Record<string, Paper>>(leaves);
  const [making, setMaking] = useState(false);
  const [seeing, setSeeing] = useState<string | null>(null);
  const giving = useRef<(() => unknown) | null>(null);
  const handed = useCallback((read: () => unknown) => {
    giving.current = read;
  }, []);
  const crossed = useRef(wide);

  useEffect(() => {
    const look = () => setRoom(window.innerWidth);
    window.addEventListener("resize", look);
    return () => window.removeEventListener("resize", look);
  }, []);

  useEffect(() => {
    if (crossed.current === wide) return;
    crossed.current = wide;
    setShown(null);
  }, [wide]);

  const keep = useCallback(
    (id: string, text: string, anyway?: boolean) => {
      setSaving(true);
      const mine = queued(id, () => docWrite(id, text, anyway))
        .then((fresh) => {
          if (held.current?.id === id && held.current.body === text) held.current = null;
          setClashed(false);
          setSaved((many) => many + 1);
          onKept(fresh);
        })
        .catch((e) => {
          if ((e as { code?: string } | null)?.code === "documentMoved") setClashed(true);
          else onError(saidPlainly(e));
        })
        .finally(() => setSaving(false));
      return mine;
    },
    [onError, onKept],
  );

  const flush = useCallback(() => {
    if (settling.current) clearTimeout(settling.current);
    const waiting = held.current;
    if (!waiting) return;
    setPacked(crowd(waiting.body));
    keep(waiting.id, waiting.body);
  }, [keep]);

  const drop = useCallback(() => {
    if (settling.current) clearTimeout(settling.current);
    held.current = null;
  }, []);

  const mineStands = useCallback(() => {
    if (!open) return;
    if (settling.current) clearTimeout(settling.current);
    const waiting = held.current;
    keep(open.file, waiting?.id === open.file ? waiting.body : shaped.current, true);
  }, [keep, open]);

  const theirsStands = useCallback(async () => {
    if (!open) return;
    if (!(await ask(t("clashSure"), { kind: "warning" }))) return;
    drop();
    docRead(open.file)
      .then((text) => {
        setBody(text);
        setPacked(crowd(text));
        const brittle = frail(text);
        setWarned(brittle.length ? brittle : null);
        setReading(brittle.length > 0);
        setClashed(false);
      })
      .catch((e) => onError(saidPlainly(e)));
  }, [drop, onError, open]);

  const leaving = useRef(flush);
  leaving.current = flush;

  useEffect(() => {
    const now = () => leaving.current();
    window.addEventListener("blur", now);
    return () => {
      window.removeEventListener("blur", now);
      now();
    };
  }, []);

  useEffect(() => holds(() => leaving.current()), []);

  useEffect(() => {
    if (!open) return;
    const still = known.some((one) => one.file === open.file);
    if (!still || !asked) {
      drop();
      turn.current += 1;
      setOpen(null);
      setBody("");
    }
  }, [asked, known, open, drop]);

  useEffect(() => {
    if (!asked) {
      turn.current += 1;
      return;
    }
    if (asked === open?.file && (fresh === seen.current || held.current)) {
      turn.current += 1;
      return;
    }
    seen.current = fresh;
    const wanted = known.find((one) => one.file === asked);
    if (!wanted) return;
    flush();
    const mine = ++turn.current;
    (busy(wanted.file) ?? Promise.resolve())
      .catch(() => {})
      .then(() => docRead(wanted.file))
      .then((text) => {
        if (turn.current !== mine) return;
        setOpen(wanted);
        setBody(text);
        setPacked(crowd(text));
        const brittle = frail(text);
        setWarned(brittle.length ? brittle : null);
        setReading(brittle.length > 0);
        setStuck(false);
        setClashed(false);
      })
      .catch((e) => onError(saidPlainly(e)));
  }, [asked, known, open, flush, onError, fresh]);

  useEffect(() => {
    onShown?.(open?.file ?? null);
  }, [open, onShown]);

  const [warned, setWarned] = useState<string[] | null>(null);
  const [reading, setReading] = useState(false);

  const [ceiling, setCeiling] = useState(0);
  useEffect(() => {
    roomy()
      .then(setCeiling)
      .catch(() => {});
  }, []);
  const brimming = ceiling > 0 && body.length > ceiling;

  const convert = (file: string) => {
    const body = shaped.current;
    if (!body) return;
    const left = frail(body);
    convertPaper(file, body)
      .then(() => {
        setBody(body);
        setWarned(left.length ? left : null);
        setStuck(left.length > 0);
        setReading(left.length > 0);
      })
      .catch((e) => onError(saidPlainly(e)));
  };

  const wrote = (text: string) => {
    if (!open || reading) return;
    setBody(text);
    held.current = { id: open.file, body: text };
    if (settling.current) clearTimeout(settling.current);
    settling.current = setTimeout(flush, SETTLES);
  };

  const beside = Boolean(open) && (shown ?? wide);
  const leaf = (open && sized[open.file]) || "a4";
  const wall = { maxWidth: `${PAPER[leaf]}px` };

  useEffect(() => {
    const id = "tisty-page";
    let tag = document.getElementById(id) as HTMLStyleElement | null;
    if (!tag) {
      tag = document.createElement("style");
      tag.id = id;
      document.head.append(tag);
    }
    tag.textContent = `@page { size: ${PAGE[leaf]}; margin: 22mm 20mm; }`;
  }, [leaf]);

  const blobOf = async (): Promise<Blob | null> => {
    const read = giving.current;
    if (!open || !read) return null;
    const [{ pdf }, { Papered, registered }, { fetched, shapesOf }] = await Promise.all([
      import("@react-pdf/renderer"),
      import("./paper"),
      import("./shaping"),
    ]);
    registered();
    const shapes = await fetched(shapesOf(read()), attached);
    return pdf(<Papered shapes={shapes} leaf={leaf} />).toBlob();
  };

  const preview = async () => {
    if (making) return;
    setMaking(true);
    try {
      const blob = await blobOf();
      if (blob) setSeeing(URL.createObjectURL(blob));
    } catch (e) {
      onError(saidPlainly(e));
    } finally {
      setMaking(false);
    }
  };

  const asking = useRef({ preview: () => {}, toPdf: () => {} });

  useEffect(() => {
    const see = () => asking.current.preview();
    const keep = () => asking.current.toPdf();
    window.addEventListener("tisty:see-pdf", see);
    window.addEventListener("tisty:to-pdf", keep);
    return () => {
      window.removeEventListener("tisty:see-pdf", see);
      window.removeEventListener("tisty:to-pdf", keep);
    };
  }, []);

  const shut = () => {
    if (seeing) URL.revokeObjectURL(seeing);
    setSeeing(null);
  };

  const toPdf = async () => {
    if (!open || making) return;
    setMaking(true);
    try {
      const blob = await blobOf();
      if (!blob) return;
      const where = await intoFile({
        defaultPath: `${open.title || t("untitledDoc")}.pdf`,
        filters: [{ name: "PDF", extensions: ["pdf"] }],
      });
      if (!where) return;
      const bytes = new Uint8Array(await blob.arrayBuffer());
      await keepPdf(where, Array.from(bytes));
    } catch (e) {
      onError(saidPlainly(e));
    } finally {
      setMaking(false);
    }
  };

  const kept = (reference: string, name: string) => {
    const kind = ending(reference);
    intoFile({
      defaultPath: name,
      filters: kind ? [{ name: kind.toUpperCase(), extensions: [kind] }] : undefined,
    })
      .then((where) => (where ? attachExport(reference, where) : undefined))
      .catch((e) => onError(saidPlainly(e)));
  };

  asking.current = { preview, toPdf };

  const showing = seeing;
  useEffect(() => {
    return () => {
      if (showing) URL.revokeObjectURL(showing);
    };
  }, [showing]);

  const nowAt = open?.file;
  const wasAt = useRef(nowAt);
  useEffect(() => {
    if (wasAt.current === nowAt) return;
    wasAt.current = nowAt;
    setSeeing((was) => {
      if (was) URL.revokeObjectURL(was);
      return null;
    });
  }, [nowAt]);

  const resize = (paper: Paper) => {
    if (!open) return;
    const now = { ...sized, [open.file]: paper };
    setSized(now);
    window.localStorage.setItem("tisty.paper", JSON.stringify(now));
  };
  const spare = room - RAIL;
  const reserve = beside && spare - ASIDE >= SHEET ? ASIDE : 0;
  return (
    <main className="relative flex min-w-0 flex-1 flex-col overflow-hidden bg-desk">
      <div data-tauri-drag-region className="h-9 shrink-0" />
      {open && !beside && (
        <button
          type="button"
          onClick={() => setShown(true)}
          title={t("beside")}
          aria-label={t("beside")}
          className="absolute top-11 right-3 z-10 grid h-6 w-6 place-items-center rounded-md text-[13px] text-faint hover:bg-hover hover:text-ink"
        >
          <span aria-hidden="true">◨</span>
        </button>
      )}
      <div
        className="flex min-h-0 flex-1 flex-col motion-safe:transition-[padding] motion-safe:duration-150"
        style={{ paddingRight: reserve }}
      >
        {open ? (
          <div style={wall} className="relative mx-auto flex min-h-0 w-full flex-1 flex-col">
            <Suspense fallback={<p className="text-[12.5px] text-faint">{t("opening")}</p>}>
              <Editor
                key={`${open.file}${reading ? ":read" : ""}`}
                value={body}
                taking={!reading}
                reading={reading}
                label={open.title || t("untitledDoc")}
                papers={known}
                folder={open.folder}
                paper={open.file}
                onMade={(id, name) => onKept({ id, title: name })}
                onDoc={onDoc}
                onAttach={() =>
                  pick({ multiple: false })
                    .then((at) => (typeof at === "string" ? attach(at, undefined, true) : null))
                    .catch((e) => {
                      onError(saidPlainly(e));
                      return null;
                    })
                }
                onOpen={(reference) => opened(reference).catch((e) => onError(saidPlainly(e)))}
                onKeep={kept}
                onWrite={wrote}
                onBlocks={setBlocks}
                onOutline={setHeads}
                onReady={handed}
                onShaped={(text) => {
                  shaped.current = text;
                }}
              />
            </Suspense>
          </div>
        ) : (
          <p style={wall} className="mx-auto w-full px-10 text-[12.5px] text-faint">
            {t("pickADoc")}
          </p>
        )}
        <div
          aria-live="polite"
          style={wall}
          className="mx-auto h-5 w-full px-10 text-[11.5px] text-faint"
        >
          {saving
            ? t("saving")
            : brimming
              ? fill("docBrimming", weighed(body.length))
              : packed > MANY
                ? fill("docCrowded", String(packed))
                : ""}
        </div>
        {clashed && open && (
          <div
            style={wall}
            className="mx-auto mb-2 flex w-full flex-wrap items-center gap-x-3 gap-y-1 px-10 text-[11.5px]"
          >
            <span className="text-soft">{t("documentMoved")}</span>
            <button
              type="button"
              onClick={mineStands}
              className="rounded-[7px] border border-line px-2 py-0.5 text-[11.5px] hover:bg-hover"
            >
              {t("clashSave")}
            </button>
            <button
              type="button"
              onClick={theirsStands}
              className="rounded-[7px] border border-line px-2 py-0.5 text-[11.5px] hover:bg-hover"
            >
              {t("clashLook")}
            </button>
          </div>
        )}
        {reading && warned && open && (
          <div
            style={wall}
            className="mx-auto mb-2 flex w-full flex-wrap items-center gap-x-3 gap-y-1 px-10 text-[11.5px]"
          >
            <span className="text-soft">{t(stuck ? "frailStuck" : "frailNeeds")}</span>
            {!stuck && (
              <button
                type="button"
                onClick={() => convert(open.file)}
                className="rounded-[7px] border border-line px-2 py-0.5 text-[11.5px] hover:bg-hover"
              >
                {t("frailConvert")}
              </button>
            )}
          </div>
        )}
      </div>
      {seeing && (
        <div className="absolute inset-0 z-40 flex flex-col bg-veil">
          <div className={`flex h-9 shrink-0 items-center justify-end gap-2 pl-3 ${clearOfChrome}`}>
            <button
              type="button"
              onClick={shut}
              aria-label={t("leaveIt")}
              className="rounded-md bg-bg px-2.5 py-1 text-[11.5px] text-soft hover:text-ink"
            >
              {t("leaveIt")}
            </button>
          </div>
          <iframe src={seeing} title={t("seePdf")} className="min-h-0 flex-1 border-0 bg-bg" />
        </div>
      )}
      {beside && open && (
        <Beside
          title={open.title}
          paper={open.file}
          body={body}
          kept={saved}
          blocks={blocks}
          heads={heads}
          leaf={leaf}
          onLeaf={resize}
          making={making}
          onPdf={toPdf}
          onSee={preview}
          onCopy={() => {
            asPlain(open.file)
              .then(() => onShown?.(open.file))
              .catch((e) => onError(saidPlainly(e)));
          }}
          onTakeOut={() => {
            pick({ directory: true })
              .then((at) => (typeof at === "string" ? docExport(open.file, at) : null))
              .catch((e) => onError(saidPlainly(e)));
          }}
          onShut={() => setShown(false)}
        />
      )}
    </main>
  );
}
