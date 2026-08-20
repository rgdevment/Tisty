import { open as pick } from "@tauri-apps/plugin-dialog";
import { lazy, Suspense, useCallback, useEffect, useRef, useState } from "react";
import {
  attach,
  convertPaper,
  docRead,
  docWrite,
  type Filed,
  keepSettings,
  opened,
  roomy,
  type Settings,
  settings,
} from "../core";
import { frail } from "../frail";
import { fill, t } from "../locales";
import { crowd, MANY, weighed } from "../previews";
import { saidPlainly } from "../refusal";
import { busy, holds, queued } from "../saving";
import Beside from "./Beside";
import type { Block } from "./Slash";
import type { Head } from "./writing";

const Editor = lazy(() => import("./Editor"));

const SETTLES = 700;

const WIDE = 1440;

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
  const settling = useRef<ReturnType<typeof setTimeout>>(null);
  const held = useRef<{ id: string; body: string } | null>(null);
  const turn = useRef(0);
  const [minded, setMinded] = useState<Settings | null>(null);
  const [wide, setWide] = useState(() => window.innerWidth >= WIDE);
  const [blocks, setBlocks] = useState<Block[]>([]);
  const [heads, setHeads] = useState<Head[]>([]);
  const [saved, setSaved] = useState(0);

  useEffect(() => {
    settings()
      .then(setMinded)
      .catch(() => {});
  }, []);

  useEffect(() => {
    const look = () => setWide(window.innerWidth >= WIDE);
    window.addEventListener("resize", look);
    return () => window.removeEventListener("resize", look);
  }, []);

  const keep = useCallback(
    (id: string, text: string) => {
      setSaving(true);
      const mine = queued(id, () => docWrite(id, text))
        .then((fresh) => {
          if (held.current?.id === id && held.current.body === text) held.current = null;
          setSaved((many) => many + 1);
          onKept(fresh);
        })
        .catch((e) => onError(saidPlainly(e)))
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

  const wanted = minded?.beside;
  const beside = Boolean(open) && (wanted ?? wide);
  const sheet = beside ? "mr-auto" : "mx-auto";
  const wall = { maxWidth: beside ? "min(820px, 100% - 344px)" : "820px" };

  const decide = (yes: boolean) => {
    setMinded((was) => (was ? { ...was, beside: yes } : was));
    settings()
      .then((now) => keepSettings({ ...now, beside: yes }))
      .then(setMinded)
      .catch((e) => onError(saidPlainly(e)));
  };

  return (
    <main className="relative flex min-w-0 flex-1 flex-col overflow-hidden">
      <div data-tauri-drag-region className="flex h-9 shrink-0 items-center justify-end px-2.5">
        {open && !beside && (
          <button
            type="button"
            onClick={() => decide(true)}
            title={t("beside")}
            className="flex h-6 items-center gap-1.5 rounded-md px-2 text-[11.5px] text-faint hover:bg-hover hover:text-ink"
          >
            <span aria-hidden="true">▤</span>
            {t("beside")}
          </button>
        )}
      </div>
      {open ? (
        <div className={`${sheet} flex min-h-0 w-full flex-1 flex-col px-10`} style={wall}>
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
              onWrite={wrote}
              onBlocks={setBlocks}
              onOutline={setHeads}
              onShaped={(text) => {
                shaped.current = text;
              }}
            />
          </Suspense>
        </div>
      ) : (
        <p className={`${sheet} w-full px-10 text-[12.5px] text-faint`} style={wall}>
          {t("pickADoc")}
        </p>
      )}
      <div
        aria-live="polite"
        className={`${sheet} h-5 w-full px-10 text-[11.5px] text-faint`}
        style={wall}
      >
        {saving
          ? t("saving")
          : brimming
            ? fill("docBrimming", weighed(body.length))
            : packed > MANY
              ? fill("docCrowded", String(packed))
              : ""}
      </div>
      {reading && warned && open && (
        <div
          className={`${sheet} mb-2 flex w-full flex-wrap items-center gap-x-3 gap-y-1 px-10 text-[11.5px]`}
          style={wall}
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
      {beside && open && (
        <Beside
          title={open.title}
          paper={open.file}
          body={body}
          kept={saved}
          blocks={blocks}
          heads={heads}
          onShut={() => decide(false)}
        />
      )}
    </main>
  );
}
