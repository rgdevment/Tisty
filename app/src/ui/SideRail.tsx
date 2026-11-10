import { useRef } from "react";
import { fold } from "../breadth";
import type { List, Papers } from "../core";
import { fill, t } from "../locales";
import type { Chosen, Named } from "../views";
import Glyph from "./Glyph";
import { painted } from "./Hue";
import { fitting, useRoom } from "./room";
import { FoldButton } from "./SideEdge";

const NAMED: { key: Named; icon: string }[] = [
  { key: "search", icon: "search" },
  { key: "tasks", icon: "sun" },
  { key: "quadrants", icon: "grid" },
  { key: "spread", icon: "calendar" },
  { key: "lists", icon: "rows" },
  { key: "tags", icon: "tag" },
  { key: "archive", icon: "archive" },
];

const STEP = 40;

interface Props {
  lists: List[];
  papers: Papers;
  counts: Record<string, number>;
  chosen: Chosen;
  here?: string | null;
  onChoose: (chosen: Chosen) => void;
  onReveal: (folder?: string) => void;
}

export default function SideRail({
  lists,
  papers,
  counts,
  chosen,
  here,
  onChoose,
  onReveal,
}: Props) {
  const shelf = useRef<HTMLDivElement>(null);
  const room = useRoom(shelf, "height");
  const top = papers.folders.filter((one) => one.parent === null && !one.away);
  const fits = fitting(top.length, room, STEP);
  const hidden = top.length - fits;
  const plain = !chosen.list && !chosen.tags?.length;
  const unfoldTo = (folder?: string) => {
    fold(false);
    onReveal(folder);
  };

  return (
    <div className="flex min-h-0 flex-1 flex-col items-center gap-1 pb-1.5">
      <FoldButton folded />
      {NAMED.map(({ key, icon }) => {
        const many = key === "lists" ? lists.length || undefined : counts[key];
        const said = many ? `${t(key)} · ${many}` : t(key);
        const on = plain && chosen.named === key;
        const late = key === "tasks" ? counts.overdue : 0;
        return (
          <button
            key={key}
            type="button"
            onClick={() => onChoose({ named: key })}
            aria-label={said}
            aria-current={on ? "page" : undefined}
            title={late ? `${said} · ${late} ${t("pulseOverdue")}` : said}
            className={`relative grid size-9 shrink-0 place-items-center rounded-[10px] transition-colors duration-150 hover:bg-hover ${
              on ? "bg-active text-accent" : "text-soft hover:text-ink"
            }`}
          >
            <Glyph name={icon} />
            {late ? (
              <span
                aria-hidden="true"
                className="absolute top-1 right-0.5 grid h-3.5 min-w-3.5 place-items-center rounded-full bg-urgent px-1 text-[9px] leading-none font-semibold text-bg"
              >
                {late}
              </span>
            ) : null}
          </button>
        );
      })}
      <span aria-hidden="true" className="my-1.5 h-px w-6 shrink-0 bg-hair" />
      <button
        type="button"
        onClick={() => unfoldTo()}
        aria-label={t("library")}
        title={t("library")}
        className={`grid size-9 shrink-0 place-items-center rounded-[10px] transition-colors duration-150 hover:bg-hover ${
          chosen.named === "docs" ? "bg-active text-hue-amber" : "text-soft hover:text-ink"
        }`}
      >
        <Glyph name="library" />
      </button>
      <div ref={shelf} className="flex min-h-0 flex-1 flex-col items-center gap-1 overflow-hidden">
        {top.slice(0, fits).map((folder) => (
          <button
            key={folder.id}
            type="button"
            onClick={() => unfoldTo(folder.id)}
            aria-label={folder.name}
            title={folder.name}
            className={`grid size-9 shrink-0 place-items-center rounded-[10px] hover:bg-hover ${
              here === folder.id && chosen.named === "docs" ? "bg-active" : ""
            }`}
          >
            <span
              className={`grid size-6 place-items-center rounded-md bg-current/10 ${painted(folder.color)}`}
            >
              <Glyph name={folder.icon ?? "folder"} className="h-[13px] w-[13px]" />
            </span>
          </button>
        ))}
        {hidden > 0 && (
          <button
            type="button"
            onClick={() => unfoldTo()}
            aria-label={fill("foldersMore", String(hidden))}
            title={fill("foldersMore", String(hidden))}
            className="grid size-9 shrink-0 place-items-center rounded-[10px] text-[11.5px] font-semibold text-faint hover:bg-hover hover:text-ink"
          >
            +{hidden}
          </button>
        )}
      </div>
    </div>
  );
}
