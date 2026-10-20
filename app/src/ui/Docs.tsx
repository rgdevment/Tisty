import { listen } from "@tauri-apps/api/event";
import { save as intoFile, open as pick } from "@tauri-apps/plugin-dialog";
import { lazy, Suspense, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { asPlain } from "../copying";
import {
  attach,
  attachPasted,
  docAway,
  docBack,
  docBackable,
  docCompactedLast,
  docExport,
  docKeepCompacted,
  docLock,
  docMerge,
  docOpened,
  docOrder,
  docPage,
  docRead,
  docWrite,
  type Filed,
  type Folded,
  keepFile,
  opened,
  type Paper,
  roomy,
  type Task,
  viewDoc,
} from "../core";
import { laidIn, withLandscape, withPaper, withSheets } from "../front";
import { keptAll, PASTED_UP_TO } from "../kept";
import { fill, t } from "../locales";
import { filed, inTextOrder, type Moved, namesIn, pagesOf, under } from "../paging";
import { crowd, MANY, weighed } from "../previews";
import { saidPlainly } from "../refusal";
import { busy, holds, queued } from "../saving";
import { sure } from "../sure";
import { changedIn } from "./anchoring";
import Beside, { trailed } from "./Beside";
import Contents from "./Contents";
import { blurbing, cardsOf } from "./carding";
import { useEditWhole } from "./editWhole";
import Modal from "./Modal";
import Picking, { type Picked } from "./Picking";
import type { Head } from "./paging";
import { usePreviewing } from "./previewing";
import { pdfOf, signature } from "./printing";
import Ribbon, { Onward } from "./Ribbon";
import { acrossOf, PAPER, pageFor, sheetFor } from "./sheets";
import { type Asking, pressed } from "./slashing";
import { CompactBar, MarkBar } from "./Tidied";
import { useLeaving } from "./useLeaving";
import { clearOfChrome } from "./WindowChrome";

const Sheet = lazy(() => import("./Sheet"));

const SETTLES = 700;

const REMEMBERS = 12;

// `print` is of the body on disk the words were typed over; `unwoven` holds a woven body the screen has yet to show.
type Read = { text: string; print?: string; unwoven?: boolean };

const remembered = (held: Map<string, Read>, file: string, read: Read) => {
  held.delete(file);
  held.set(file, read);
  while (held.size > REMEMBERS) {
    const oldest = held.keys().next().value;
    if (oldest === undefined) break;
    held.delete(oldest);
  }
};

const WIDE = 1440;

const RAIL = 336;
const RAIL_WIDE = 380;
const GUTTER = 8;
const SHEET = 820;

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

const ASIDE = 344;

const NONE: string[] = [];

interface Props {
  open?: string;
  known: Filed[];
  folders?: Folded[];
  onFolder?: (id: string | null) => void;
  onKept: (doc: { id: string; title: string }) => void;
  onPaging?: (put: ((page: Filed) => boolean) | null) => void;
  onError: (problem: unknown) => void;
  onDoc?: (id: string) => void;
  onTag?: (tag: string) => void;
  tags?: readonly string[];
  onShown?: (file: string | null) => void;
  onDrop?: (doc: Filed) => void;
  onDropAll?: (docs: Filed[]) => void;
  onTask?: (task: Task) => void;
  onBack?: (doc: Filed) => void;
  fresh?: number;
}

const tailless = (said: string): string => said.replace(/\r\n/g, "\n").replace(/\n+$/, "");

const NO_TAGS: readonly string[] = [];

const listed = (chosen: string | string[] | null): string[] => {
  if (Array.isArray(chosen)) return chosen;
  return chosen ? [chosen] : [];
};

export default function Docs({
  open: asked,
  known,
  folders = [],
  onFolder,
  onKept,
  onPaging,
  onError,
  onDoc,
  onTag,
  tags = NO_TAGS,
  onShown,
  onDrop,
  onDropAll,
  onTask,
  onBack,
  fresh = 0,
}: Props) {
  const [open, setOpen] = useState<Filed | null>(null);
  const [body, setBody] = useState("");
  const [saving, setSaving] = useState(false);
  const [waiting, setWaiting] = useState(false);
  const [packed, setPacked] = useState(0);
  const shaped = useRef("");
  const typed = useRef(0);
  const seen = useRef(0);
  const [coming, setComing] = useState(false);
  const [clashed, setClashed] = useState(false);
  const [stirred, setStirred] = useState(false);
  const [lately, setLately] = useState<[number, number] | null>(null);
  const [whole, setWhole] = useState(false);
  const clickWhole = useEditWhole();
  const [looking, setLooking] = useState(0);
  const lastRead = useRef(new Map<string, Read>());
  const onScreen = useRef<string | null>(null);
  const settling = useRef<ReturnType<typeof setTimeout>>(null);
  const held = useRef<{ id: string; body: string } | null>(null);
  const turn = useRef(0);
  const [room, setRoom] = useState(() => window.innerWidth);
  const [shown, setShown] = useState<boolean | null>(null);
  const wide = room >= WIDE;
  const [heads, setHeads] = useState<Head[]>([]);
  const [saved, setSaved] = useState(0);
  // A round that came while a save was in flight is read once the save lands, not dropped.
  const missed = useRef(false);
  const [behind, setBehind] = useState(0);
  const [sized, setSized] = useState<Record<string, Paper>>(leaves);
  const [making, setMaking] = useState(false);
  const [seeing, setSeeing] = useState<string | null>(null);
  const [pdfAsked, setPdfAsked] = useState(false);
  const [signing, setSigning] = useState(true);
  const putting = useRef<((page: Filed) => boolean) | null>(null);
  const paged = useRef(onPaging);
  paged.current = onPaging;
  const ordering = useRef<((file: string, before: string | null) => Moved) | null>(null);
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
      const wrote_at = typed.current;
      const mine = queued(id, async () => {
        try {
          const fresh = await docWrite(id, text, anyway, lastRead.current.get(id)?.print);
          return { fresh, woven: null };
        } catch (e) {
          const base = lastRead.current.get(id)?.text;
          if ((e as { code?: string } | null)?.code !== "documentMoved" || base === undefined) {
            throw e;
          }
          const woven = await docMerge(id, base, text);
          if (!woven) throw e;
          return { fresh: null, woven };
        }
      })
        .then(({ fresh, woven }) => {
          if (held.current?.id === id && held.current.body === text) held.current = null;
          setClashed(false);
          setSaved((many) => many + 1);
          if (fresh) {
            remembered(lastRead.current, id, { text, print: fresh.print });
            onKept(fresh);
            return;
          }
          if (!woven) return;
          onKept({ id, title: woven.title });
          // Words typed since are woven by their own save, over what this one sent; the old print keeps it refused.
          if (typed.current !== wrote_at) {
            const was = lastRead.current.get(id)?.print;
            remembered(lastRead.current, id, { text, print: was, unwoven: true });
            return;
          }
          remembered(lastRead.current, id, { text: woven.body, print: woven.print });
          if (onScreen.current !== id) return;
          shaped.current = woven.body;
          setBody(woven.body);
          setPacked(crowd(woven.body));
          setLately(changedIn(text, woven.body));
          setStirred(true);
        })
        .catch((e) => {
          if ((e as { code?: string } | null)?.code === "documentMoved") setClashed(true);
          else onError(saidPlainly(e));
        })
        .finally(() => {
          setSaving(false);
          if (held.current) return;
          setWaiting(false);
          if (!missed.current) return;
          missed.current = false;
          setBehind((many) => many + 1);
        });
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
    setWaiting(false);
  }, []);

  const mineStands = useCallback(() => {
    if (!open) return;
    if (settling.current) clearTimeout(settling.current);
    const waiting = held.current;
    keep(open.file, waiting?.id === open.file ? waiting.body : shaped.current, true);
  }, [keep, open]);

  const [backable, setBackable] = useState(false);
  const marked = filed(known, open?.file)?.compacted;
  const [compacted, setCompacted] = useState(false);

  useEffect(() => {
    if (!(marked && open)) {
      setCompacted(false);
      return;
    }
    let mine = true;
    docCompactedLast(open.file)
      .then((last) => {
        if (mine) setCompacted(last);
      })
      .catch(() => {
        if (mine) setCompacted(false);
      });
    return () => {
      mine = false;
    };
  }, [open, marked]);

  useEffect(() => {
    if (!(stirred || compacted) || !open) {
      setBackable(false);
      return;
    }
    let mine = true;
    docBackable(open.file)
      .then((can) => {
        if (mine) setBackable(can);
      })
      .catch(() => {
        if (mine) setBackable(false);
      });
    return () => {
      mine = false;
    };
  }, [open, stirred, compacted]);

  const wentBack = useCallback(async () => {
    if (!open) return;
    if (!(await sure(t("backSure")))) return;
    drop();
    try {
      const said = await docBack(open.file);
      const { body: text, print } = await docOpened(open.file);
      remembered(lastRead.current, open.file, { text, print });
      shaped.current = text;
      setBody(text);
      setPacked(crowd(text));
      setStirred(false);
      setBackable(false);
      onKept(said);
    } catch (e) {
      onError(saidPlainly(e));
    }
  }, [drop, onError, onKept, open]);

  const theirsStands = useCallback(async () => {
    if (!open) return;
    if (!(await sure(t("clashSure")))) return;
    drop();
    docOpened(open.file)
      .then(({ body: text, print }) => {
        remembered(lastRead.current, open.file, { text, print });
        shaped.current = text;
        setBody(text);
        setPacked(crowd(text));
        setClashed(false);
        setStirred(false);
      })
      .catch((e) => onError(saidPlainly(e)));
  }, [drop, onError, open]);

  const leaving = useRef(flush);
  leaving.current = flush;
  const settled = useRef(onKept);
  settled.current = onKept;

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
      setComing(false);
      return;
    }
    if (asked === open?.file && (fresh === seen.current || held.current)) {
      if (fresh !== seen.current) missed.current = true;
      turn.current += 1;
      return;
    }
    seen.current = fresh;
    const wanted = known.find((one) => one.file === asked);
    if (!wanted) {
      setComing(false);
      return;
    }
    const before = open?.file;
    if (before && before !== asked) {
      const leaving = known.find((one) => one.file === before);
      if (wanted.pageOf && leaving && wanted.pageOf === leaving.id) {
        from.current = { doc: before, page: asked };
      } else if (from.current?.doc !== asked) {
        from.current = null;
      }
    }
    flush();
    const mine = ++turn.current;
    const wrote_at = typed.current;
    (busy(wanted.file) ?? Promise.resolve())
      .catch(() => {})
      .then(() => docOpened(wanted.file))
      .then(({ body: text, print }) => {
        if (turn.current !== mine) return;
        // Somebody typed while this was in flight: reading it back over them would lose the
        // words and drop the caret at the end.
        if (typed.current !== wrote_at) return;
        const last = lastRead.current.get(wanted.file)?.text;
        const astir = last !== undefined && tailless(last) !== tailless(text);
        // A round that brought other things reads this one again; the same words leave the block being written open.
        const same = before === wanted.file && !astir;
        remembered(lastRead.current, wanted.file, { text, print });
        shaped.current = text;
        // Words woven in by the window's own save are already told; reading them back is no reason to forget it.
        setStirred((was) => astir || (was && before === wanted.file));
        if (astir) setLately(changedIn(last, text));
        setOpen(wanted);
        setClashed(false);
        if (!same) {
          setBody(text);
          setPacked(crowd(text));
        }
        if (known.some((one) => one.pageOf === wanted.id)) {
          docOrder(wanted.file, text)
            .then((moved) => {
              if (moved) settled.current({ id: wanted.id, title: wanted.title });
            })
            .catch(() => {});
        }
        setComing(false);
      })
      .catch((e) => {
        if (turn.current !== mine) return;
        const coming = (e as { code?: string })?.code === "docComing";
        setComing(coming);
        if (coming) {
          setOpen(null);
          return;
        }
        onError(saidPlainly(e));
      });
  }, [asked, known, open, flush, onError, fresh, behind]);

  useEffect(() => {
    onScreen.current = open?.file ?? null;
    onShown?.(open?.file ?? null);
  }, [open, onShown]);

  const [ceiling, setCeiling] = useState(0);
  useEffect(() => {
    roomy()
      .then(setCeiling)
      .catch(() => {});
  }, []);
  const brimming = ceiling > 0 && body.length > ceiling;

  const wrote = (text: string) => {
    if (!open || bolted) return;
    typed.current = Date.now();
    setLately(null);
    // Once the person writes, going back would take their words, not the compaction.
    setCompacted(false);
    setBody(text);
    const last = lastRead.current.get(open.file);
    const back = last !== undefined && !last.unwoven && tailless(last.text) === tailless(text);
    if (back) {
      // A chapter moved and moved back inside the beat would otherwise be written in the order it passed through.
      setWaiting(false);
      if (held.current?.id === open.file) {
        held.current = null;
        if (settling.current) clearTimeout(settling.current);
        settling.current = null;
      }
      return;
    }
    held.current = { id: open.file, body: text };
    setWaiting(true);
    if (settling.current) clearTimeout(settling.current);
    settling.current = setTimeout(flush, SETTLES);
  };

  const own = filed(known, open?.file);
  const called = own?.title ?? open?.title ?? "";
  // What the archive holds is read-only, whether the document says so or the folder above it does.
  const shelved = Boolean(own?.away);
  const bolted = Boolean(own?.locked) || shelved;

  // The page hears every Ctrl key the block left alone, so the switch works from inside a block too.
  useEffect(() => {
    if (!open || bolted) return;
    const flip = (event: KeyboardEvent) => {
      if (!(event.ctrlKey || event.metaKey) || !event.shiftKey) return;
      if (event.key.toLowerCase() !== "e") return;
      event.preventDefault();
      setWhole((was) => !was);
    };
    window.addEventListener("keydown", flip);
    // On a Mac the menu holds the keys, so the switch arrives as its item.
    let drop: (() => void) | undefined;
    let gone = false;
    listen("whole", () => setWhole((was) => !was))
      .then((off) => {
        if (gone) off();
        else drop = off;
      })
      .catch(() => undefined);
    return () => {
      gone = true;
      drop?.();
      window.removeEventListener("keydown", flip);
    };
  }, [open, bolted]);
  const stood = useRef(new Map<string, number>());
  const from = useRef<{ doc: string; page: string } | null>(null);
  const seek = own?.file ? stood.current.get(own.file) : undefined;
  const anchor = own?.file && from.current?.doc === own.file ? from.current.page : undefined;
  const pages = pagesOf(known, open?.file);
  const [picked, setPicked] = useState<Picked | null>(null);
  const leafy = Boolean(own && !own.pageOf);
  const slashAsks = useMemo<Asking | undefined>(
    () =>
      bolted
        ? undefined
        : {
            kinds: leafy
              ? ["newPage", "page", "doc", "newDoc", "icon", "file"]
              : ["doc", "newDoc", "icon", "file"],
            tags,
            pasted: (files) =>
              keptAll(
                files.map((one) =>
                  one.size > PASTED_UP_TO
                    ? Promise.reject({ code: "attachmentTooBig", name: "650 MB" })
                    : one
                        .arrayBuffer()
                        .then((bytes) => attachPasted(one.name, one.type, new Uint8Array(bytes))),
                ),
                onError,
              ),
            lost: () => onError(t("attachmentLost")),
            ask: (kind, at, put, first) => {
              if (kind === "file") {
                // An attachment as a document's first words would become its title.
                if (first) return onError(t("fileNeedsTitle"));
                pick({ multiple: true })
                  .then((chosen) =>
                    keptAll(
                      listed(chosen).map((one) => attach(one)),
                      onError,
                    ),
                  )
                  .then((text) => {
                    if (text && !put(text)) onError(t("attachmentLost"));
                  })
                  .catch((error_) => onError(saidPlainly(error_)));
                return;
              }
              // A card as a document's first words would become its title, which the core refuses.
              if (kind !== "icon" && first) return onError(t("leafNeedsTitle"));
              setPicked({ kind, at, put, file: open?.file });
            },
          },
    [bolted, leafy, onError, open?.file, tags],
  );
  const above = under(known, own);
  const sisters = pagesOf(known, above?.file);
  const told = useMemo(() => (pages.length > 0 ? namesIn(body) : NONE), [body, pages.length]);
  const cards = useMemo(() => cardsOf(known, open?.file, told), [known, open?.file, told]);
  const [blurbs] = useState(blurbing);
  const named = useMemo(() => namesIn(body), [body]);
  // A card says what its document is now: a rename, an archive or a deletion draws the page again.
  const again = useMemo(
    () =>
      named
        .map((file) => {
          const one = known.find((doc) => doc.file === file);
          return one
            ? `${file}:${one.title}:${one.archived}:${one.away}:${one.gone}:${one.pageOf ?? ""}`
            : `${file}:-`;
        })
        .join("|"),
    [known, named],
  );
  const blurb = useCallback(
    (file: string) => blurbs(known.find((one) => one.file === file)),
    [known, blurbs],
  );

  const [aboveTold, setAboveTold] = useState<string[]>();
  const upstairs = above?.file;
  useEffect(() => {
    if (!upstairs) return setAboveTold(undefined);
    let gone = false;
    docRead(upstairs)
      .then((text) => {
        if (!gone) setAboveTold(namesIn(text));
      })
      .catch(() => {});
    return () => {
      gone = true;
    };
  }, [upstairs, fresh]);

  const inOrder = aboveTold
    ? aboveTold
        .map((file) => sisters.find((one) => one.file === file))
        .filter((one): one is Filed => Boolean(one))
    : [];
  const at = inOrder.findIndex((one) => one.file === own?.file);
  const next = at < 0 ? undefined : inOrder[at + 1];

  const beside = Boolean(open) && (shown ?? wide);
  const laid = useMemo(() => laidIn(body), [body]);
  const leaf =
    (open && (bolted ? (sized[open.file] ?? laid.paper) : (laid.paper ?? sized[open.file]))) ||
    "a4";
  const sheets = Boolean(open) && laid.sheets;
  const landscape = Boolean(open) && laid.landscape;
  const across = acrossOf(leaf, landscape);
  const wall = { maxWidth: `${across}px` };
  const sheetOf = useMemo(
    () => (sheets ? sheetFor(leaf, landscape, t("sheetsBreak")) : undefined),
    [leaf, sheets, landscape],
  );
  // The sheet keeps its paper's width; the room around it is the scroller's gutter and bar.
  const sheetRoom = { maxWidth: `${across + 48}px` };

  useEffect(() => pageFor(leaf, landscape), [leaf, landscape]);

  // What leaves is the book as it reads, not as the log last settled it.
  const book = async () => {
    if (!open) return null;
    const pages = inTextOrder(
      known.filter((one) => one.pageOf === open.id),
      told,
    );
    return { pages, written: await Promise.all(pages.map((one) => docRead(one.file))) };
  };

  useLeaving({
    book: () => book().then((read) => read && [body, ...read.written]),
    called: called || t("untitledDoc"),
    onError,
  });

  const blobOf = async (signed?: string): Promise<Blob | null> => {
    const read = await book();
    if (!read) return null;
    return pdfOf([body, ...read.written], leaf, signed, landscape);
  };

  const preview = async () => {
    if (!open || making) return;
    setMaking(true);
    try {
      const blob = await blobOf(signing ? await signature(open.file) : undefined);
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

  const toPdf = () => {
    if (!open || making) return;
    setSigning(true);
    setPdfAsked(true);
  };

  const madePdf = async (signed: boolean) => {
    if (!open || making) return;
    setPdfAsked(false);
    setMaking(true);
    try {
      const said = signed ? await signature(open.file) : undefined;
      const blob = await blobOf(said);
      if (!blob) return;
      const where = await intoFile({
        defaultPath: `${called || t("untitledDoc")}.pdf`,
        filters: [{ name: "PDF", extensions: ["pdf"] }],
      });
      if (!where) return;
      const bytes = new Uint8Array(await blob.arrayBuffer());
      await keepFile(where, Array.from(bytes));
    } catch (e) {
      onError(saidPlainly(e));
    } finally {
      setMaking(false);
    }
  };

  asking.current = { preview, toPdf };

  usePreviewing(Boolean(seeing));

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

  const reshape = (made: (now: string) => string) => {
    if (!open || bolted) return;
    wrote(made(held.current?.id === open.file ? held.current.body : body));
  };

  const resize = (paper: Paper) => {
    if (!open) return;
    const { [open.file]: _, ...others } = sized;
    const now = bolted ? { ...others, [open.file]: paper } : others;
    setSized(now);
    window.localStorage.setItem("tisty.paper", JSON.stringify(now));
    if (bolted) return;
    const latest = held.current?.id === open.file ? held.current.body : body;
    wrote(withPaper(latest, paper));
  };
  const spare = room - (wide ? RAIL_WIDE : RAIL) - GUTTER;
  const reserve = beside && spare - ASIDE >= SHEET ? ASIDE : 0;
  return (
    <main className="relative flex min-w-0 flex-1 flex-col overflow-hidden bg-desk">
      <div data-tauri-drag-region className="h-9 shrink-0" />
      {open && !bolted && (
        <fieldset
          aria-label={t("editingMode")}
          className={`absolute top-11 z-10 flex rounded-md border border-line text-[11.5px] ${beside ? "right-3" : "right-10"}`}
        >
          {[false, true].map((all) => (
            <button
              key={String(all)}
              type="button"
              aria-pressed={whole === all}
              title={all ? `${t("editWholeDoc")} · ${pressed("Shift E")}` : t("editBlock")}
              onClick={() => setWhole(all)}
              className={`px-2 py-0.5 first:rounded-l-md last:rounded-r-md ${whole === all ? "bg-hover text-ink" : "text-faint hover:text-ink"}`}
            >
              {t(all ? "editingWhole" : "editingBlock")}
            </button>
          ))}
        </fieldset>
      )}
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
          <div style={sheetRoom} className="relative mx-auto flex min-h-0 w-full flex-1 flex-col">
            <Suspense fallback={<div className="min-h-0 flex-1" />}>
              <Sheet
                sheet={sheetOf}
                file={open.file}
                whole={whole}
                onWhole={setWhole}
                clickWhole={clickWhole}
                lately={stirred ? lately : null}
                looking={looking}
                onOwn={(file) => {
                  const page = known.find((one) => one.file === file);
                  if (page)
                    docPage(page.id)
                      .then(() => onKept({ id: page.id, title: page.title }))
                      .catch((e) => onError(saidPlainly(e)));
                }}
                onFail={(e) => onError(saidPlainly(e))}
                card={cards}
                blurb={blurb}
                again={again}
                value={body}
                seek={seek}
                anchor={anchor}
                onSeen={(at) => {
                  if (own?.file) stood.current.set(own.file, at);
                }}
                reading={bolted}
                label={called || t("untitledDoc")}
                onDoc={onDoc}
                onTag={onTag}
                above={
                  above && (
                    <Ribbon
                      of={above}
                      sisters={sisters}
                      told={aboveTold}
                      here={open.file}
                      onOpen={(doc) => onDoc?.(doc.file)}
                    />
                  )
                }
                below={
                  above
                    ? next && <Onward next={next} onOpen={(doc) => onDoc?.(doc.file)} />
                    : pages.length > 0 && (
                        <Contents
                          pages={pages}
                          told={told}
                          onOpen={(page) => onDoc?.(page.file)}
                          onPut={
                            bolted
                              ? undefined
                              : (page) => {
                                  if (putting.current?.(page) === false) {
                                    onError(t("leafNeedsTitle"));
                                  }
                                }
                          }
                          onMove={
                            bolted
                              ? undefined
                              : (page, before) => {
                                  const how = ordering.current?.(page.file, before);
                                  const named = page.title || t("untitledDoc");
                                  if (how === "held") onError(fill("leafStaysPut", named));
                                  if (how === "unseen") onError(fill("leafInText", named));
                                  if (how === "titled") onError(fill("leafWouldName", named));
                                  return how;
                                }
                          }
                        />
                      )
                }
                onOpen={(reference) => opened(reference).catch((e) => onError(saidPlainly(e)))}
                onWrite={wrote}
                onOutline={setHeads}
                onInsert={(put) => {
                  const named = put && !bolted ? (page: Filed) => put(page.file, page.title) : null;
                  putting.current = named;
                  paged.current?.(named);
                }}
                onOrder={(move) => {
                  ordering.current = move;
                }}
                asking={slashAsks}
                kept={!saving && !waiting}
              />
            </Suspense>
          </div>
        ) : coming ? (
          <div role="status" aria-live="polite" style={wall} className="mx-auto w-full px-10">
            <p className="text-[12.5px] text-faint">{t("docComingHere")}</p>
            <span
              aria-hidden="true"
              className="mt-2.5 block h-0.5 w-32 overflow-hidden rounded-full bg-line"
            >
              <span className="sliding block h-full w-1/3 rounded-full bg-soft" />
            </span>
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
        {stirred && !clashed && !compacted && open && (
          <div
            style={wall}
            className="mx-auto mb-2 flex w-full flex-wrap items-center gap-x-3 gap-y-1 px-10 text-[11.5px]"
          >
            <span className="text-soft">{t("docStirred")}</span>
            {lately && (
              <button
                type="button"
                onClick={() => setLooking((many) => many + 1)}
                className="rounded-[10px] border border-line px-2 py-0.5 text-[11.5px] hover:bg-hover"
              >
                {t("docStirredSee")}
              </button>
            )}
            {backable && (
              <button
                type="button"
                onClick={() => wentBack()}
                className="rounded-[10px] border border-line px-2 py-0.5 text-[11.5px] hover:bg-hover"
              >
                {t("docStirredBack")}
              </button>
            )}
            <button
              type="button"
              onClick={() => setStirred(false)}
              className="rounded-[10px] border border-line px-2 py-0.5 text-[11.5px] hover:bg-hover"
            >
              {t("docStirredGone")}
            </button>
          </div>
        )}
        {clashed && open && (
          <div
            style={wall}
            className="mx-auto mb-2 flex w-full flex-wrap items-center gap-x-3 gap-y-1 px-10 text-[11.5px]"
          >
            <span className="text-soft">{t("documentMoved")}</span>
            <button
              type="button"
              onClick={mineStands}
              className="rounded-[10px] border border-line px-2 py-0.5 text-[11.5px] hover:bg-hover"
            >
              {t("clashSave")}
            </button>
            <button
              type="button"
              onClick={theirsStands}
              className="rounded-[10px] border border-line px-2 py-0.5 text-[11.5px] hover:bg-hover"
            >
              {t("clashLook")}
            </button>
          </div>
        )}
        {shelved && open && (
          <div
            style={wall}
            className="mx-auto mb-2 flex w-full flex-wrap items-center gap-x-3 gap-y-1 px-10 text-[11.5px]"
          >
            <span className="text-soft">
              {own?.archived ? t("docShelved") : own?.pageOf ? t("heldShelved") : t("heldByFolder")}
            </span>
            {(own?.archived || own?.pageOf) && (
              <button
                type="button"
                onClick={() => {
                  const which = own.archived ? own.id : (own.pageOf as string);
                  if (own.archived && !own.pageOf) {
                    if (onBack) return onBack(own);
                    return;
                  }
                  docAway(which, false)
                    .then(() => onKept({ id: open.id, title: called }))
                    .catch((e) => onError(saidPlainly(e)));
                }}
                className="rounded-[10px] border border-line px-2 py-0.5 text-[11.5px] hover:bg-hover"
              >
                {own.archived ? t("bringBack") : t("bringBackHolder")}
              </button>
            )}
          </div>
        )}
        {own?.compacted && open && !shelved && (
          <CompactBar
            own={own}
            style={wall}
            last={compacted}
            backable={backable}
            onUndo={() => void wentBack()}
            onKeep={() =>
              docKeepCompacted(own.id)
                .then(() => onKept({ id: open.id, title: called }))
                .catch((e) => onError(saidPlainly(e)))
            }
          />
        )}
        {own?.flagged && open && !shelved && (
          <MarkBar
            own={own}
            known={known}
            style={wall}
            onKept={() => onKept({ id: open.id, title: called })}
            onError={onError}
            onAway={() =>
              docAway(own.id, true)
                .then(() => onKept({ id: open.id, title: called }))
                .catch((e) => onError(saidPlainly(e)))
            }
            onDrop={onDrop}
            onDropAll={onDropAll}
            onDoc={onDoc}
            onTask={onTask}
          />
        )}
        {bolted && !shelved && open && (
          <div
            style={wall}
            className="mx-auto mb-2 flex w-full flex-wrap items-center gap-x-3 gap-y-1 px-10 text-[11.5px]"
          >
            <span className="text-soft">{t("docBolted")}</span>
            <button
              type="button"
              onClick={() =>
                Promise.all(
                  [own?.id, own?.pageOf]
                    .filter((one): one is string => typeof one === "string")
                    .map((one) => docLock(one, false)),
                )
                  .then(() => onKept({ id: open.id, title: called }))
                  .catch((e) => onError(saidPlainly(e)))
              }
              className="rounded-[10px] border border-line px-2 py-0.5 text-[11.5px] hover:bg-hover"
            >
              {t("unlockIt")}
            </button>
          </div>
        )}
      </div>
      {seeing && (
        <div className="absolute inset-0 z-40 flex flex-col bg-veil">
          <div className={`flex h-9 shrink-0 items-center gap-2 pl-3 ${clearOfChrome}`}>
            <button
              type="button"
              onClick={shut}
              className="rounded-md bg-bg px-2.5 py-1 text-[11.5px] text-soft hover:text-ink"
            >
              {t("previewBack")}
            </button>
          </div>
          <iframe src={seeing} title={t("seePdf")} className="min-h-0 flex-1 border-0 bg-bg" />
        </div>
      )}
      {pdfAsked && open && (
        <Modal
          title={t("toPdf")}
          onClose={() => {
            setPdfAsked(false);
            setSigning(true);
          }}
        >
          <p className="mt-3 text-[12.5px] leading-relaxed text-soft">{t("pdfSignWhy")}</p>
          <label className="mt-4 flex items-center gap-2.5 text-[13px]">
            <input
              type="checkbox"
              checked={signing}
              onChange={(e) => setSigning(e.target.checked)}
              className="h-3.5 w-3.5 accent-accent"
            />
            {t("pdfSign")}
          </label>
          <div className="mt-5 flex items-center justify-end gap-2 text-[12.5px]">
            <button
              type="button"
              onClick={() => {
                setPdfAsked(false);
                setSigning(true);
              }}
              className="rounded-[10px] px-3 py-1.5 text-faint hover:text-ink"
            >
              {t("cancel")}
            </button>
            <button
              type="button"
              onClick={() => madePdf(signing)}
              className="cursor-pointer rounded-[10px] bg-accent px-3.5 py-1.5 text-bg"
            >
              {t("toPdfDo")}
            </button>
          </div>
        </Modal>
      )}

      {picked && (
        <Picking
          picked={picked}
          own={own}
          pages={pages}
          others={known.filter((one) => one.file !== open?.file && !one.away)}
          onMade={onKept}
          onLost={(doc) => {
            if (picked.file !== open?.file) return;
            putting.current?.({ ...doc, id: doc.file, folder: null, archived: false, away: false });
          }}
          onError={(problem) => onError(saidPlainly(problem))}
          onDone={() => setPicked(null)}
        />
      )}

      {beside && open && (
        <Beside
          title={called}
          paper={open.file}
          body={body}
          tags={known.find((one) => one.file === open.file)?.tags ?? []}
          kept={saved}
          heads={heads}
          pages={above ? sisters : pages}
          onPage={(one) => onDoc?.(one.file)}
          leaf={leaf}
          onLeaf={resize}
          sheets={sheets}
          landscape={landscape}
          onSheets={
            bolted
              ? undefined
              : (on) => reshape((now) => withSheets(on ? now : withLandscape(now, false), on))
          }
          onLandscape={
            bolted ? undefined : (turned) => reshape((now) => withLandscape(now, turned))
          }
          making={making}
          onPdf={toPdf}
          onSee={preview}
          trail={trailed(folders, known.find((one) => one.file === open.file)?.folder ?? null)}
          onFolder={onFolder}
          onCopy={() => {
            asPlain(open.file)
              .then(() => onShown?.(open.file))
              .catch((e) => onError(saidPlainly(e)));
          }}
          onTakeOut={() => {
            pick({ directory: true })
              .then((at) => (typeof at === "string" ? docExport(open.file, at) : null))
              .then((took) => {
                if (took === null) return;
                if (took.missed > 0) {
                  onError(
                    took.missed === 1 ? t("takenShort") : fill("takenShorter", String(took.missed)),
                  );
                } else if (took.left > 0) {
                  onError(
                    took.left === 1 ? t("takenLess") : fill("takenLesser", String(took.left)),
                  );
                }
              })
              .catch((e) => onError(saidPlainly(e)));
          }}
          onShut={() => setShown(false)}
          onApart={() =>
            void viewDoc({
              id: open.id,
              file: open.file,
              title: called || t("untitledDoc"),
              place: trailed(folders, open.folder ?? null)
                .map((one) => one.name)
                .join(" / "),
            }).catch((e) => onError(saidPlainly(e)))
          }
        />
      )}
    </main>
  );
}
