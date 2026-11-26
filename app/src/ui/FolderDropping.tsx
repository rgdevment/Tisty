import { useEffect, useId, useState } from "react";
import { agentsTree } from "../agentsSync";
import type { Folded } from "../core";
import {
  asksFolderDrop,
  type Contents,
  type FolderAnswer,
  type FolderAsked,
} from "../folderDropping";
import { trail } from "../folders";
import { fill, t } from "../locales";
import { button } from "./AgentsReview";
import Modal from "./Modal";

interface Open extends FolderAsked {
  done: (answer: FolderAnswer | null) => void;
}

const danger = "rounded-[10px] bg-urgent px-3.5 py-1.5 font-semibold text-bg disabled:opacity-60";

export default function FolderDropping({ folders }: Readonly<{ folders: readonly Folded[] }>) {
  const [open, setOpen] = useState<Open | null>(null);
  useEffect(() => {
    asksFolderDrop((asked) => new Promise((done) => setOpen({ ...asked, done })));
    return () => asksFolderDrop(null);
  }, []);
  if (!open) return null;
  const answer = (said: FolderAnswer | null) => {
    setOpen(null);
    open.done(said);
  };
  return <Asking key={open.folder.id} open={open} folders={folders} onAnswer={answer} />;
}

function Asking({
  open,
  folders,
  onAnswer,
}: Readonly<{
  open: Open;
  folders: readonly Folded[];
  onAnswer: (answer: FolderAnswer | null) => void;
}>) {
  const [contents, setContents] = useState<Contents>("up");
  const [into, setInto] = useState("");
  const name = useId();
  const { folder, up } = open;
  const below = (at: string | null): boolean =>
    at === folder.id || (!!at && below(folders.find((one) => one.id === at)?.parent ?? null));
  const agents = agentsTree(folders);
  const elsewhere = folders.filter(
    (one) => !one.away && !below(one.id) && one.id !== up?.id && !agents.has(one.id),
  );
  const held = fill("folderHolds", String(open.docs), String(open.folders));
  const choices: { key: Contents; title: string; said: string }[] = [
    {
      key: "up",
      title: up ? fill("folderDropUp", up.name) : t("folderDropUnfiled"),
      said: t("folderDropNothingLost"),
    },
    ...(open.handsOnly
      ? []
      : [
          { key: "into" as const, title: t("folderDropInto"), said: t("folderDropIntoSaid") },
          { key: "drop" as const, title: t("folderDropAll"), said: t("folderDropAllSaid") },
        ]),
  ];
  const ready = contents !== "into" || into !== "";
  return (
    <Modal title={fill("folderDropTitle", folder.name)} wide onClose={() => onAnswer(null)}>
      <p className="text-[13px] text-soft">{held}</p>
      <fieldset className="mt-3 flex flex-col gap-2">
        <legend className="sr-only">{t("folderDropWhat")}</legend>
        {choices.map((one) => (
          <label
            key={one.key}
            className={`flex cursor-pointer items-start gap-2.5 rounded-md border px-2.5 py-2 text-[12.5px] ${
              contents === one.key ? "border-accent bg-accent-soft" : "border-hair"
            }`}
          >
            <input
              type="radio"
              name={name}
              checked={contents === one.key}
              onChange={() => setContents(one.key)}
              className="mt-0.5 accent-accent"
            />
            <span>
              <span className="block font-semibold text-ink">{one.title}</span>
              <span className="block text-[11.5px] text-faint">{one.said}</span>
            </span>
          </label>
        ))}
      </fieldset>
      {contents === "into" && (
        <select
          value={into}
          onChange={(e) => setInto(e.target.value)}
          aria-label={t("folderDropInto")}
          className="mt-2 w-full rounded-md border border-hair bg-bg px-2 py-1.5 text-[12.5px]"
        >
          <option value="">{t("folderDropPick")}</option>
          <option value="~root">{t("unfiled")}</option>
          {elsewhere.map((one) => (
            <option key={one.id} value={one.id}>
              {trail(folders as Folded[], one.id)}
            </option>
          ))}
        </select>
      )}
      <div className="mt-5 flex flex-wrap justify-end gap-2 text-[12.5px]">
        <button type="button" onClick={() => onAnswer(null)} className={button}>
          {t("cancel")}
        </button>
        <button
          type="button"
          disabled={!ready}
          onClick={() =>
            onAnswer(
              contents === "into"
                ? { contents, into: into === "~root" ? undefined : into }
                : { contents },
            )
          }
          className={danger}
        >
          {t("folderDropYes")}
        </button>
      </div>
    </Modal>
  );
}
