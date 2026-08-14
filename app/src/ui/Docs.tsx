import { Suspense, lazy, useCallback, useEffect, useRef, useState } from "react";
import { open as pick } from "@tauri-apps/plugin-dialog";
import { attach, docRead, docWrite, opened, roomy, type Filed } from "../core";
import { busy, holds, queued } from "../saving";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { frail } from "../frail";
import Modal from "./Modal";
import { weighed } from "../previews";

const Editor = lazy(() => import("./Editor"));

const SETTLES = 700;

interface Props {
  open?: string;
  known: Filed[];
  onKept: (doc: { id: string; title: string }) => void;
  onError: (problem: unknown) => void;
  onDoc?: (id: string) => void;
  onShown?: (file: string | null) => void;
}

export default function Docs({ open: asked, known, onKept, onError, onDoc, onShown }: Props) {
  const [open, setOpen] = useState<Filed | null>(null);
  const [body, setBody] = useState("");
  const [saving, setSaving] = useState(false);
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
    if (waiting) keep(waiting.id, waiting.body);
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
    if (!asked || asked === open?.file) {
      turn.current += 1;
      return;
    }
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
        const brittle = frail(text);
        setWarned(brittle.length ? brittle : null);
        setReading(brittle.length > 0);
      })
      .catch((e) => onError(saidPlainly(e)));
  }, [asked, known, open, flush, onError]);

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

  const wrote = (text: string) => {
    if (!open || reading) return;
    setBody(text);
    held.current = { id: open.file, body: text };
    if (settling.current) clearTimeout(settling.current);
    settling.current = setTimeout(flush, SETTLES);
  };

  return (
    <main className="flex min-w-0 flex-1 flex-col overflow-hidden">
      {warned && (
        <Modal title={t("frailTitle")} onClose={() => setWarned(null)}>
          <p className="text-[13px] text-soft">{t("frailWhy")}</p>
          <ul className="mt-2 mb-4 list-disc pl-5 text-[13px] text-ink">
            {warned.map((one) => (
              <li key={one}>{t(one as Parameters<typeof t>[0])}</li>
            ))}
          </ul>
          <div className="flex justify-end gap-2">
            <button
              type="button"
              onClick={() => setWarned(null)}
              className="rounded-md px-3 py-1.5 text-[12.5px] text-soft hover:bg-hover"
            >
              {t("frailRead")}
            </button>
            <button
              type="button"
              autoFocus
              onClick={() => {
                setWarned(null);
                setReading(false);
              }}
              className="rounded-md bg-accent px-3 py-1.5 text-[12.5px] text-bg"
            >
              {t("frailEdit")}
            </button>
          </div>
        </Modal>
      )}
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
        {reading
          ? t("frailReading")
          : saving
            ? t("saving")
            : brimming
              ? fill("docBrimming", weighed(body.length))
              : ""}
      </div>
    </main>
  );
}
