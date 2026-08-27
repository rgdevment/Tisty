import { ask } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import { type List, listAdd, listDrop, listLook, listRename } from "../core";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import Glyph from "./Glyph";
import { painted } from "./Hue";

import Naming from "./Naming";
import Pick from "./Pick";

interface Props {
  lists: List[];
  counts: Record<string, number>;
  onOpen: (id: string) => void;
  onChanged: () => void;
  onError: (problem: unknown) => void;
}

export default function Lists({ lists, counts, onOpen, onChanged, onError }: Props) {
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

  /// One sheet for the lot: name, icon and colour, the same one a folder opens.
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
      <div className="mx-auto w-full max-w-[760px] px-6 pb-8">
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

        <div className="grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] items-start gap-2.5">
          {lists.map((list) => (
            <div key={list.id} className="rounded-xl border border-hair bg-panel">
              <div className="flex items-start gap-2 px-3.5 py-3">
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
                  className="min-w-0 flex-1 text-left"
                >
                  <span className="block truncate text-[13.5px] font-medium text-ink">
                    {list.name}
                  </span>
                  <span className="mt-0.5 block text-[11.5px] text-faint">
                    {counts[list.id]
                      ? fill("openTasks", String(counts[list.id]))
                      : t("listSettled")}
                  </span>
                </button>
              </div>
            </div>
          ))}
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
