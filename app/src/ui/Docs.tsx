import { Suspense, lazy, useCallback, useEffect, useRef, useState } from "react";
import { docRead, docWrite, type Doc } from "../core";
import { t } from "../locales";
import { saidPlainly } from "../refusal";

const Editor = lazy(() => import("./Editor"));

const SETTLES = 700;

interface Props {
  open?: string;
  known: Doc[];
  onKept: (doc: Doc) => void;
  onError: (problem: unknown) => void;
}

export default function Docs({ open: asked, known, onKept, onError }: Props) {
  const [open, setOpen] = useState<Doc | null>(null);
  const [body, setBody] = useState("");
  const [saving, setSaving] = useState(false);
  const settling = useRef<ReturnType<typeof setTimeout>>(null);
  const held = useRef<{ id: string; body: string } | null>(null);

  const keep = useCallback(
    (id: string, text: string) => {
      setSaving(true);
      docWrite(id, text)
        .then((fresh) => {
          onKept(fresh);
          held.current = null;
        })
        .catch((e) => onError(saidPlainly(e)))
        .finally(() => setSaving(false));
    },
    [onError, onKept],
  );

  const flush = useCallback(() => {
    if (settling.current) clearTimeout(settling.current);
    const waiting = held.current;
    if (waiting) keep(waiting.id, waiting.body);
  }, [keep]);

  useEffect(() => flush, [flush]);

  useEffect(() => {
    if (!asked || asked === open?.id) return;
    const wanted = known.find((one) => one.id === asked);
    if (!wanted) return;
    flush();
    docRead(wanted.id)
      .then((text) => {
        setOpen(wanted);
        setBody(text);
      })
      .catch((e) => onError(saidPlainly(e)));
  }, [asked, known, open, flush, onError]);

  const wrote = (text: string) => {
    if (!open) return;
    setBody(text);
    held.current = { id: open.id, body: text };
    if (settling.current) clearTimeout(settling.current);
    settling.current = setTimeout(() => keep(open.id, text), SETTLES);
  };

  return (
    <main className="flex min-w-0 flex-1 flex-col">
      <div data-tauri-drag-region className="h-9 shrink-0" />
      {open ? (
        <div className="mx-auto min-h-0 w-full max-w-[820px] flex-1 px-10">
          <Suspense fallback={<p className="text-[12.5px] text-faint">{t("opening")}</p>}>
            <Editor key={open.id} value={body} taking onWrite={wrote} />
          </Suspense>
        </div>
      ) : (
        <p className="mx-auto w-full max-w-[820px] px-10 text-[12.5px] text-faint">
          {t("pickADoc")}
        </p>
      )}
      <div className="mx-auto h-5 w-full max-w-[820px] px-10 text-[11.5px] text-faint">
        {saving ? t("saving") : ""}
      </div>
    </main>
  );
}
