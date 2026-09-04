import { ask } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import { type List, listAdd, listDrop, listLook, listRename, type Task } from "../core";
import { isOverdue, isToday, whenLabel } from "../format";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import Glyph from "./Glyph";
import { painted } from "./Hue";

import Naming from "./Naming";
import Pick from "./Pick";

const AHEAD = 3;

const whenOf = (task: Task) => task.date ?? task.deadline;

const soonest = (all: Task[]): Task[] =>
  [...all]
    .sort((a, b) => {
      const at = whenOf(a)?.at;
      const bt = whenOf(b)?.at;
      if (at && bt) return at.localeCompare(bt);
      if (at) return -1;
      if (bt) return 1;
      return a.order.localeCompare(b.order);
    })
    .slice(0, AHEAD);

interface Props {
  lists: List[];
  counts: Record<string, number>;
  tasks: Task[];
  onOpen: (id: string) => void;
  onChanged: () => void;
  onError: (problem: unknown) => void;
}

export default function Lists({ lists, counts, tasks, onOpen, onChanged, onError }: Props) {
  const [making, setMaking] = useState(false);
  const [name, setName] = useState("");
  const [icon, setIcon] = useState<string>();
  const [hue, setHue] = useState<string>();
  const [editing, setEditing] = useState<List | null>(null);

  const held = lists.reduce((sum, list) => sum + (counts[list.id] ?? 0), 0);

  const make = () => {
    const wanted = name.trim();
    if (!wanted) return;
    listAdd(wanted, icon, hue)
      .then(() => {
        setName("");
        setIcon(undefined);
        setHue(undefined);
        setMaking(false);
        onChanged();
      })
      .catch((e) => onError(saidPlainly(e)));
  };

  const settle = (list: List, called: string, drawn?: string, colour?: string) => {
    const wanted = called.trim();
    if (!wanted) return;
    Promise.all([
      wanted === list.name ? Promise.resolve() : listRename(list.id, wanted),
      drawn === (list.icon ?? undefined) && colour === (list.color ?? undefined)
        ? Promise.resolve()
        : listLook(list.id, drawn, colour),
    ])
      .then(() => {
        setEditing(null);
        onChanged();
      })
      .catch((e) => onError(saidPlainly(e)));
  };

  const drop = (list: List) => {
    ask(fill("listDropSure", list.name), { kind: "warning" })
      .then((sure) => {
        if (!sure) return undefined;
        setEditing(null);
        return listDrop(list.id).then(() => {
          onChanged();
        });
      })
      .catch((e) => onError(saidPlainly(e)));
  };

  return (
    <main className="scroller flex min-w-0 flex-1 flex-col">
      <div data-tauri-drag-region className="h-9 shrink-0" />
      <div className="@container w-full px-6 pb-8">
        <div className="flex items-baseline justify-between">
          <h2 className="text-[21px] font-semibold">{t("lists")}</h2>
          <button
            type="button"
            onClick={() => setMaking((open) => !open)}
            className="rounded-lg px-2.5 py-1 text-[12.5px] text-soft hover:bg-hover"
          >
            {t("newList")}
          </button>
        </div>
        <p className="mt-0.5 mb-4 text-[12.5px] text-faint">
          {lists.length === 0
            ? t("noListsYet")
            : `${fill("listsAre", String(lists.length))} · ${fill("openTasks", String(held))}`}
        </p>

        {making && (
          <div className="mb-4 rounded-xl border border-hair bg-panel p-3.5">
            <input
              autoFocus
              value={name}
              onChange={(e) => setName(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && make()}
              placeholder={t("listName")}
              aria-label={t("listName")}
              className="w-full rounded-lg bg-hover px-3 py-2 text-[13.5px] outline-none placeholder:text-faint"
            />
            <div className="mt-2.5">
              <Pick icon={icon} colour={hue} onIcon={setIcon} onColour={setHue} />
            </div>
            <div className="mt-3 flex gap-2">
              <button
                type="button"
                onClick={make}
                className="rounded-lg bg-accent px-3 py-1.5 text-[12.5px] text-bg"
              >
                {t("create")}
              </button>
              <button
                type="button"
                onClick={() => setMaking(false)}
                className="rounded-lg px-3 py-1.5 text-[12.5px] text-soft hover:bg-hover"
              >
                {t("cancel")}
              </button>
            </div>
          </div>
        )}

        <div className="grid grid-cols-1 items-start gap-2.5 @min-[380px]:grid-cols-2 @min-[760px]:grid-cols-3 @min-[1180px]:grid-cols-4">
          {lists.map((list) => {
            const next = soonest(tasks.filter((one) => one.list === list.id));
            return (
              <div key={list.id} className="rounded-xl border border-hair bg-panel px-3.5 py-3">
                <div className="flex items-center gap-2">
                  <button
                    type="button"
                    onClick={() => setEditing(list)}
                    aria-label={fill("iconOf", list.name)}
                    className={`grid h-7 w-7 shrink-0 place-items-center rounded-lg text-[15px] hover:bg-hover ${painted(list.color)}`}
                  >
                    {list.icon ? (
                      <Glyph name={list.icon ?? ""} />
                    ) : (
                      <span className="text-[12px] text-faint">○</span>
                    )}
                  </button>
                  <button
                    type="button"
                    onClick={() => onOpen(list.id)}
                    className="min-w-0 flex-1 truncate text-left text-[13.5px] font-medium text-ink"
                  >
                    {list.name}
                  </button>
                  <span className="shrink-0 text-[11.5px] tabular-nums text-faint">
                    {counts[list.id] ?? 0}
                  </span>
                </div>

                <div className="mt-2 border-t border-hair pt-1.5">
                  {next.length === 0 ? (
                    <p className="text-[11.5px] text-faint italic">{t("listSettled")}</p>
                  ) : (
                    next.map((task) => {
                      const when = whenOf(task);
                      return (
                        <button
                          key={task.id}
                          type="button"
                          onClick={() => onOpen(list.id)}
                          className="flex w-full items-baseline gap-2 py-px text-left text-[11.5px] text-soft"
                        >
                          <span className="min-w-0 truncate">{task.title}</span>
                          <span
                            className={`ml-auto shrink-0 tabular-nums ${
                              when
                                ? isOverdue(when)
                                  ? "text-urgent"
                                  : isToday(when)
                                    ? "text-accent"
                                    : "text-faint"
                                : "text-faint"
                            }`}
                          >
                            {when ? whenLabel(when) : "—"}
                          </span>
                        </button>
                      );
                    })
                  )}
                </div>
              </div>
            );
          })}
        </div>
      </div>
      {editing && (
        <Naming
          key={editing.id}
          title={t("editList")}
          invite={t("listName")}
          called={editing.name}
          drawn={editing.icon ?? undefined}
          painted={editing.color ?? undefined}
          action={t("saveIt")}
          dropWord={t("listDrop")}
          onName={(called, drawn, colour) => settle(editing, called, drawn, colour)}
          onDrop={() => drop(editing)}
          onClose={() => setEditing(null)}
        />
      )}
    </main>
  );
}
