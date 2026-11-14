import { type ReactNode, type RefObject, useEffect, useState } from "react";
import { t } from "../locales";
import Menu from "./Menu";
import { onMac } from "./WindowChrome";

type Framing = {
  readonly lively: boolean;
  readonly frame: () => Window | null;
  readonly putting: RefObject<number>;
  readonly onStuck: (stuck: boolean) => void;
  readonly onTitleGone: (gone: boolean) => void;
  readonly onSeen: (at: number) => void;
  readonly ready: boolean;
  readonly turned: number;
};

// A page that opens with its title says what the ribbon would, so the ribbon names it only once it has scrolled away.
const titleOf = (page: Document) => {
  const first = Array.from(page.querySelector(".page")?.children ?? []).find(
    (one) => !one.matches(".front"),
  );
  return first?.tagName === "H1" ? first : null;
};

export const useFrameScroll = ({
  lively,
  frame,
  putting,
  onStuck,
  onTitleGone,
  onSeen,
  ready,
  turned,
}: Framing) => {
  // biome-ignore lint/correctness/useExhaustiveDependencies: each drawing may bring the frame anew, and the callbacks read refs.
  useEffect(() => {
    const inner = frame();
    inner?.document.documentElement.classList.toggle("lively", lively);
    if (!lively || !inner) return;
    const rolled = () => {
      const sheet = inner.document.scrollingElement;
      if (!sheet) return;
      onStuck(sheet.scrollTop > 8);
      const title = titleOf(inner.document);
      onTitleGone(title ? title.getBoundingClientRect().bottom < 0 : sheet.scrollTop > 8);
      if (putting.current >= 0 || sheet.scrollHeight <= sheet.clientHeight) return;
      onSeen(sheet.scrollTop);
    };
    inner.addEventListener("scroll", rolled, { passive: true });
    return () => inner.removeEventListener("scroll", rolled);
  }, [lively, ready, turned]);
};

type Below = {
  readonly children: ReactNode;
  readonly folds: boolean;
  readonly width: string;
};

export function LivelyBelow({ children, folds, width }: Below) {
  const [open, setOpen] = useState(false);
  if (!folds) return <div className={`mx-auto shrink-0 ${width}`}>{children}</div>;
  return (
    <div className={`mx-auto shrink-0 ${width}`}>
      <button
        type="button"
        aria-expanded={open}
        onClick={() => setOpen((was) => !was)}
        className="flex w-full items-center gap-1.5 px-2 py-1 text-left text-[11.5px] text-faint hover:text-ink"
      >
        <span aria-hidden>{open ? "▾" : "▸"}</span>
        <span>{t("theseLeaves")}</span>
      </button>
      {open && <div className="max-h-[45vh] overflow-y-auto">{children}</div>}
    </div>
  );
}

export type Viewing = {
  readonly markdown: boolean;
  readonly onTurn: (markdown: boolean) => void;
};

// The page's own choices wait behind one quiet button on the ribbon, first how its words are seen.
export function PageMore({ markdown, onTurn }: Viewing) {
  const [at, setAt] = useState<{ x: number; y: number } | null>(null);
  const toggled = (button: HTMLElement) => {
    const box = button.getBoundingClientRect();
    setAt(at ? null : { x: box.right - 210, y: box.bottom + 6 });
  };
  const seen = (asMarkdown: boolean) => () => {
    if (asMarkdown !== markdown) onTurn(asMarkdown);
  };
  return (
    <>
      <button
        type="button"
        aria-haspopup="menu"
        aria-expanded={at !== null}
        aria-label={t("pageMore")}
        title={t("pageMore")}
        // The menu shuts on any mousedown outside it, so a mouse toggles here and only a keyboard's click reaches onClick.
        onMouseDown={(event) => {
          event.preventDefault();
          toggled(event.currentTarget);
        }}
        onClick={(event) => {
          if (event.detail === 0) toggled(event.currentTarget);
        }}
        className="ribbon-more"
      >
        <span aria-hidden="true">⋯</span>
      </button>
      {at && (
        <Menu
          at={at}
          label={t("pageMore")}
          choices={[
            {
              key: "rich",
              glyph: "page",
              label: t("viewRich"),
              on: !markdown,
              onPick: seen(false),
            },
            {
              key: "markdown",
              glyph: "hash",
              label: t("viewMarkdown"),
              on: markdown,
              keys: markdown ? undefined : `${onMac ? "⌘" : "Ctrl"} E`,
              onPick: seen(true),
            },
          ]}
          onClose={() => setAt(null)}
        />
      )}
    </>
  );
}

// Beside a task's page one quiet word turns the view, so nothing stands over the words.
export function ViewTurn({ markdown, onTurn }: Viewing) {
  return (
    <button
      type="button"
      aria-pressed={markdown}
      onClick={() => onTurn(!markdown)}
      className="-mr-1.5 ml-auto flex items-center rounded-md px-1.5 py-0.5 text-[11.5px] text-faint transition-colors hover:bg-hover hover:text-ink"
    >
      {t(markdown ? "viewAsRich" : "viewAsMarkdown")}
    </button>
  );
}
