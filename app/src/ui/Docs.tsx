import { Suspense, lazy, useCallback, useEffect, useRef, useState } from "react";
import { docNew, docRead, docWrite, docs as readDocs, type Doc } from "../core";
import { t } from "../locales";
import { saidPlainly } from "../refusal";

const Editor = lazy(() => import("./Editor"));

const SETTLES = 700;

interface Props {
  open?: string;
  onKnown: (docs: Doc[]) => void;
  onError: (problem: unknown) => void;
}

export default function Docs({ open: asked, onKnown, onError }: Props) {
  const [all, setAll] = useState<Doc[]>([]);
  const [open, setOpen] = useState<Doc | null>(null);
  const [body, setBody] = useState("");
  const [saving, setSaving] = useState(false);
  const [taking, setTaking] = useState(false);
  const settling = useRef<ReturnType<typeof setTimeout>>(null);
  const held = useRef<{ id: string; body: string } | null>(null);

  const look = useCallback(() => {
    readDocs()
      .then((found) => {
        setAll(found);
        onKnown(found);
      })
      .catch((e) => onError(saidPlainly(e)));
  }, [onError, onKnown]);

  useEffect(look, [look]);

  useEffect(() => {
    if (!asked || asked === open?.id) return;
    const wanted = all.find((one) => one.id === asked);
    if (!wanted) return;
    docRead(wanted.id)
      .then((text) => {
        setOpen(wanted);
        setBody(text);
        setTaking(true);
      })
      .catch((e) => onError(saidPlainly(e)));
  }, [asked, all, open, onError]);

  const keep = useCallback(
    (id: string, text: string) => {
      setSaving(true);
      docWrite(id, text)
        .then((fresh) => {
          setAll((were) => {
            const now = were.map((one) => (one.id === fresh.id ? fresh : one));
            onKnown(now);
            return now;
          });
          held.current = null;
        })
        .catch((e) => onError(saidPlainly(e)))
        .finally(() => setSaving(false));
    },
    [onError, onKnown],
  );

  const flush = useCallback(() => {
    if (settling.current) clearTimeout(settling.current);
    const waiting = held.current;
    if (waiting) keep(waiting.id, waiting.body);
  }, [keep]);

  useEffect(() => flush, [flush]);

  const wrote = (text: string) => {
    if (!open) return;
    setBody(text);
    held.current = { id: open.id, body: text };
    if (settling.current) clearTimeout(settling.current);
    settling.current = setTimeout(() => keep(open.id, text), SETTLES);
  };

  const show = (doc: Doc) => {
    flush();
    docRead(doc.id)
      .then((text) => {
        setOpen(doc);
        setBody(text);
        setTaking(true);
      })
      .catch((e) => onError(saidPlainly(e)));
  };

  const make = () => {
    flush();
    docNew()
      .then((made) => {
        setAll((were) => {
          const now = [...were, made];
          onKnown(now);
          return now;
        });
        setOpen(made);
        setBody("");
        setTaking(true);
      })
      .catch((e) => onError(saidPlainly(e)));
  };

  return (
    <main className="flex min-h-0 flex-1">
      <nav className="flex w-60 shrink-0 flex-col border-r border-hair">
        <div data-tauri-drag-region className="h-9 shrink-0" />
        <div className="flex items-center justify-between px-4 pb-2">
          <h2 className="text-[13px] font-semibold text-soft">{t("docs")}</h2>
          <button
            type="button"
            onClick={make}
            aria-label={t("newDoc")}
            title={t("newDoc")}
            className="grid h-6 w-6 place-items-center rounded-md text-soft hover:bg-hover"
          >
            +
          </button>
        </div>
        <ul className="min-h-0 flex-1 overflow-y-auto px-2 pb-3">
          {all.length === 0 && (
            <li className="px-2 py-1.5 text-[12.5px] text-faint">{t("noDocsYet")}</li>
          )}
          {all.map((doc) => (
            <li key={doc.id}>
              <button
                type="button"
                onClick={() => show(doc)}
                aria-current={open?.id === doc.id ? "true" : undefined}
                className={`w-full truncate rounded-md px-2 py-1.5 text-left text-[13px] ${
                  open?.id === doc.id ? "bg-hover text-ink" : "text-soft hover:bg-hover"
                }`}
              >
                {doc.title || t("untitledDoc")}
              </button>
            </li>
          ))}
        </ul>
      </nav>

      <section className="flex min-w-0 flex-1 flex-col">
        <div data-tauri-drag-region className="h-9 shrink-0" />
        {open ? (
          <div className="min-h-0 flex-1 overflow-hidden px-10">
            <Suspense fallback={<p className="text-[12.5px] text-faint">{t("opening")}</p>}>
              <Editor key={open.id} value={body} taking={taking} onWrite={wrote} />
            </Suspense>
          </div>
        ) : (
          <p className="px-10 text-[12.5px] text-faint">{t("pickADoc")}</p>
        )}
        <div className="h-6 px-10 text-[11.5px] text-faint">{saving ? t("saving") : ""}</div>
      </section>
    </main>
  );
}
