import { useState } from "react";
import { docAway, docFile, type Filed, folderAdd, type Papers } from "../core";
import { trail } from "../folders";
import { fill, t } from "../locales";
import Modal from "./Modal";

interface Props {
  doc: Filed;
  papers: Papers;
  onClose: () => void;
  onDone: () => void;
  fail: (problem: unknown) => void;
}

export default function BringingBack({ doc, papers, onClose, onDone, fail }: Props) {
  const [backTo, setBackTo] = useState<string>("same");

  const backHome = (doc: Filed): string | null => {
    if (doc.folder && papers.folders.some((one) => one.id === doc.folder)) return doc.folder;
    const was = (doc.folderWas ?? []).join(" / ");
    const again = was
      ? papers.folders.find((one) => !one.away && trail(papers.folders, one.id) === was)
      : undefined;
    return again?.id ?? null;
  };

  const backFrom = (doc: Filed): string | null => {
    const home = backHome(doc);
    if (home) return trail(papers.folders, home);
    const was = doc.folderWas ?? [];
    return was.length ? was.join(" / ") : null;
  };

  const madeAgain = async (way: string[]): Promise<string | null> => {
    let parent: string | null = null;
    for (const name of way) {
      const here = papers.folders.find(
        (one) => !one.away && one.name === name && (one.parent ?? null) === parent,
      );
      parent = here ? here.id : await folderAdd(name, parent ?? undefined);
    }
    return parent;
  };

  const putBack = () => {
    onClose();
    const home = backHome(doc);
    const lands = (): Promise<string | null> =>
      backTo === "none"
        ? Promise.resolve(null)
        : backTo !== "same"
          ? Promise.resolve(backTo)
          : home
            ? Promise.resolve(home)
            : doc.folderWas?.length
              ? madeAgain(doc.folderWas)
              : Promise.resolve(null);
    docAway(doc.id, false)
      .then(lands)
      .then((folder) =>
        folder === (doc.folder ?? null) ? undefined : docFile(doc.id, folder ?? undefined),
      )
      .then(onDone)
      .catch((e) => fail(e));
  };

  return (
    <Modal title={fill("backWhere", doc.title || t("untitledDoc"))} onClose={onClose}>
      <p id="back-why" className="mt-3 text-[12.5px] leading-relaxed text-soft">
        {backFrom(doc) === null
          ? t("backFromNowhere")
          : fill(backHome(doc) ? "backFrom" : "backFromGone", backFrom(doc) as string)}
      </p>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          putBack();
        }}
      >
        <fieldset
          aria-describedby="back-why"
          className="scroller mt-4 flex max-h-[248px] flex-col gap-0.5"
        >
          <legend className="sr-only">{t("backWhere").replace("{name}", "")}</legend>
          <Where
            name="where-back"
            value="same"
            chosen={backTo}
            onPick={setBackTo}
            label={
              backFrom(doc) === null
                ? t("backToNone")
                : backHome(doc)
                  ? fill("backToSame", backFrom(doc) as string)
                  : fill("backToMade", (doc.folderWas ?? []).join(" / "))
            }
            hint={backFrom(doc) === null ? t("backWasHere") : undefined}
          />
          {backFrom(doc) !== null && (
            <Where
              name="where-back"
              value="none"
              chosen={backTo}
              onPick={setBackTo}
              label={t("backToNone")}
              hint={t("backAtRoot")}
            />
          )}
          {papers.folders.some((one) => !one.away && one.id !== backHome(doc)) && (
            <label className="flex cursor-pointer items-center gap-2.5 rounded-[10px] px-2 py-1.5 text-[12.5px] hover:bg-hover">
              <input
                type="radio"
                name="where-back"
                value="other"
                checked={backTo !== "same" && backTo !== "none"}
                onChange={() => {
                  const first = papers.folders.find((one) => !one.away && one.id !== backHome(doc));
                  if (first) setBackTo(first.id);
                }}
                className="accent-accent"
              />
              <span className="shrink-0">{t("backToOther")}</span>
              <select
                value={backTo !== "same" && backTo !== "none" ? backTo : ""}
                onChange={(e) => setBackTo(e.target.value)}
                className="ml-auto min-w-0 max-w-[60%] truncate rounded-md border border-line bg-bg px-2 py-1 text-[12.5px] text-ink"
              >
                <option value="" disabled>
                  {t("backToOther")}
                </option>
                {papers.folders
                  .filter((one) => !one.away && one.id !== backHome(doc))
                  .map((one) => (
                    <option key={one.id} value={one.id}>
                      {trail(papers.folders, one.id)}
                    </option>
                  ))}
              </select>
            </label>
          )}
        </fieldset>
        <div className="mt-5 flex flex-wrap items-center justify-end gap-2 text-[12.5px]">
          <button
            type="button"
            onClick={onClose}
            className="cursor-pointer rounded-[10px] px-3 py-1.5 text-faint hover:text-ink"
          >
            {t("cancel")}
          </button>
          <button
            type="submit"
            className="cursor-pointer rounded-[10px] border border-line px-3 py-1.5 text-ink hover:bg-line/40"
          >
            {t("bringBack")}
          </button>
        </div>
      </form>
    </Modal>
  );
}

function Where({
  name,
  value,
  chosen,
  label,
  hint,
  onPick,
}: {
  name: string;
  value: string;
  chosen: string;
  label: string;
  hint?: string;
  onPick: (value: string) => void;
}) {
  return (
    <label className="flex cursor-pointer items-center gap-2.5 rounded-[10px] px-2 py-1.5 text-[12.5px] hover:bg-hover">
      <input
        type="radio"
        name={name}
        value={value}
        checked={chosen === value}
        onChange={() => onPick(value)}
        className="accent-accent"
      />
      <span className="min-w-0 truncate">{label}</span>
      {hint && <span className="ml-auto shrink-0 text-[11.5px] text-faint">{hint}</span>}
    </label>
  );
}
