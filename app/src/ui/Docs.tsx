import { Suspense, lazy, useCallback, useEffect, useRef, useState } from "react";
import { open as pick } from "@tauri-apps/plugin-dialog";
import { attach, docRead, docWrite, type Filed } from "../core";
import { t } from "../locales";
import { saidPlainly } from "../refusal";

const Editor = lazy(() => import("./Editor"));

const SETTLES = 700;

interface Props {
  open?: string;
  known: Filed[];
  onKept: (doc: { id: string; title: string }) => void;
  onError: (problem: unknown) => void;
}

export default function Docs({ open: asked, known, onKept, onError }: Props) {
  const [open, setOpen] = useState<Filed | null>(null);
  const [body, setBody] = useState("");
  const [saving, setSaving] = useState(false);
  const settling = useRef<ReturnType<typeof setTimeout>>(null);
  const held = useRef<{ id: string; body: string } | null>(null);
  const queues = useRef(new Map<string, Promise<unknown>>());
  const turn = useRef(0);

  const keep = useCallback(
    (id: string, text: string) => {
      setSaving(true);
      const before = queues.current.get(id) ?? Promise.resolve();
      const mine = before
        .catch(() => {})
        .then(() => docWrite(id, text))
        .then((fresh) => {
          if (held.current?.id === id && held.current.body === text) held.current = null;
          onKept(fresh);
        })
        .catch((e) => onError(saidPlainly(e)))
        .finally(() => {
          if (queues.current.get(id) === mine) setSaving(false);
        });
      queues.current.set(id, mine);
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
    window.addEventListener("beforeunload", now);
    return () => {
      window.removeEventListener("blur", now);
      window.removeEventListener("beforeunload", now);
      now();
    };
  }, []);

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
    (queues.current.get(wanted.file) ?? Promise.resolve())
      .catch(() => {})
      .then(() => docRead(wanted.file))
      .then((text) => {
        if (turn.current !== mine) return;
        setOpen(wanted);
        setBody(text);
      })
      .catch((e) => onError(saidPlainly(e)));
  }, [asked, known, open, flush, onError]);

  const wrote = (text: string) => {
    if (!open) return;
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
              key={open.file}
              value={body}
              taking
              label={open.title || t("untitledDoc")}
              onAttach={() =>
                pick({ multiple: false })
                  .then((at) => (typeof at === "string" ? attach(at) : null))
                  .catch((e) => {
                    onError(saidPlainly(e));
                    return null;
                  })
              }
              onRuin={() => onError(t("wouldRuin"))}
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
        {saving ? t("saving") : ""}
      </div>
    </main>
  );
}
