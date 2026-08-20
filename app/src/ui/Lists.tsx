import { ask } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import { type List, listAdd, listDrop, listLook, listRename } from "../core";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import Icons, { drawn, useIcons } from "./Icons";

interface Props {
  lists: List[];
  counts: Record<string, number>;
  onOpen: (id: string) => void;
  onChanged: () => void;
  onError: (problem: unknown) => void;
}

type Pane = "icon" | "more" | "name";

export default function Lists({ lists, counts, onOpen, onChanged, onError }: Props) {
  const all = useIcons();
  const [making, setMaking] = useState(false);
  const [name, setName] = useState("");
  const [icon, setIcon] = useState<string>();
  const [open, setOpen] = useState<{ id: string; pane: Pane }>();
  const [fresh, setFresh] = useState("");

  const held = lists.reduce((sum, list) => sum + (counts[list.id] ?? 0), 0);

  const shows = (id: string, pane: Pane) => open?.id === id && open.pane === pane;
  const show = (id: string, pane: Pane) => setOpen(shows(id, pane) ? undefined : { id, pane });

  const make = () => {
    const wanted = name.trim();
    if (!wanted) return;
    listAdd(wanted, icon)
      .then(() => {
        setName("");
        setIcon(undefined);
        setMaking(false);
        onChanged();
      })
      .catch((e) => onError(saidPlainly(e)));
  };

  const dress = (id: string, key: string | undefined) => {
    listLook(id, key)
      .then(() => {
        setOpen(undefined);
        onChanged();
      })
      .catch((e) => onError(saidPlainly(e)));
  };

  const rename = (list: List) => {
    const wanted = fresh.trim();
    if (!wanted || wanted === list.name) {
      setOpen(undefined);
      return;
    }
    listRename(list.id, wanted)
      .then(() => {
        setOpen(undefined);
        onChanged();
      })
      .catch((e) => onError(saidPlainly(e)));
  };

  const drop = (list: List) => {
    ask(fill("listDropSure", list.name), { kind: "warning" })
      .then((sure) => {
        if (!sure) return undefined;
        return listDrop(list.id).then(() => {
          setOpen(undefined);
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
            <div className="scroller mt-2.5 max-h-52">
              <Icons chosen={icon} onPick={setIcon} />
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
                  onClick={() => show(list.id, "icon")}
                  aria-label={fill("iconOf", list.name)}
                  className="grid h-7 w-7 shrink-0 place-items-center rounded-lg text-[15px] hover:bg-hover"
                >
                  {drawn(all, list.icon) ?? <span className="text-[12px] text-faint">○</span>}
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
                <button
                  type="button"
                  onClick={() => show(list.id, "more")}
                  aria-label={fill("listMore", list.name)}
                  aria-expanded={shows(list.id, "more") || shows(list.id, "name")}
                  className="grid h-7 w-7 shrink-0 place-items-center rounded-lg text-[13px] text-faint hover:bg-hover"
                >
                  ⋯
                </button>
              </div>
              {shows(list.id, "icon") && (
                <div className="scroller max-h-52 border-t border-hair px-3 py-2.5">
                  <Icons chosen={list.icon} onPick={(key) => dress(list.id, key)} />
                </div>
              )}
              {shows(list.id, "more") && (
                <div className="flex gap-2 border-t border-hair px-3 py-2.5">
                  <button
                    type="button"
                    onClick={() => {
                      setFresh(list.name);
                      setOpen({ id: list.id, pane: "name" });
                    }}
                    className="rounded-lg px-2.5 py-1 text-[12.5px] text-soft hover:bg-hover"
                  >
                    {t("rename")}
                  </button>
                  <button
                    type="button"
                    onClick={() => drop(list)}
                    className="rounded-lg px-2.5 py-1 text-[12.5px] text-soft hover:bg-hover"
                  >
                    {t("listDrop")}
                  </button>
                </div>
              )}
              {shows(list.id, "name") && (
                <div className="border-t border-hair px-3 py-2.5">
                  <input
                    autoFocus
                    value={fresh}
                    onChange={(e) => setFresh(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") rename(list);
                      if (e.key === "Escape") setOpen(undefined);
                    }}
                    aria-label={t("listName")}
                    className="w-full rounded-lg bg-hover px-3 py-2 text-[13.5px] outline-none"
                  />
                  <div className="mt-2 flex gap-2">
                    <button
                      type="button"
                      onClick={() => rename(list)}
                      className="rounded-lg bg-accent px-3 py-1.5 text-[12.5px] text-bg"
                    >
                      {t("rename")}
                    </button>
                    <button
                      type="button"
                      onClick={() => setOpen(undefined)}
                      className="rounded-lg px-3 py-1.5 text-[12.5px] text-soft hover:bg-hover"
                    >
                      {t("cancel")}
                    </button>
                  </div>
                </div>
              )}
            </div>
          ))}
        </div>
      </div>
    </main>
  );
}
