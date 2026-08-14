import { Suspense, lazy, useCallback, useEffect, useRef, useState } from "react";
import { open as pick } from "@tauri-apps/plugin-dialog";
import { attach, convertPaper, docRead, docWrite, opened, roomy, type Filed } from "../core";
import { busy, holds, queued } from "../saving";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { frail } from "../frail";
import { MANY, crowd, weighed } from "../previews";

const Editor = lazy(() => import("./Editor"));

const SETTLES = 700;

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

  const keep = useCallback(
    (id: string, text: string) => {
      setSaving(true);
      const mine = queued(id, () => docWrite(id, text))
        .then((fresh) => {
          if (held.current?.id === id && held.current.body === text) held.current = null;
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

  return (
    <main className="flex min-w-0 flex-1 flex-col overflow-hidden">
      <div data-tauri-drag-region className="h-9 shrink-0" />
      {open ? (
        <div className="mx-auto flex min-h-0 w-full max-w-[820px] flex-1 flex-col px-10">
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
              onShaped={(text) => {
                shaped.current = text;
              }}
            />
          </Suspense>
        </div>
      ) : (
        <p className="mx-auto w-full max-w-[820px] px-10 text-[12.5px] text-faint">
          {t("pickADoc")}
        </p>
      )}
      <div
        aria-live="polite"
        className="mx-auto h-5 w-full max-w-[820px] px-10 text-[11.5px] text-faint"
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
        <div className="mx-auto mb-2 flex w-full max-w-[820px] flex-wrap items-center gap-x-3 gap-y-1 px-10 text-[11.5px]">
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
    </main>
  );
}
