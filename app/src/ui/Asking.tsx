import { useId, useState } from "react";
import { t } from "../locales";

export default function Asking({
  onName,
  leaf,
}: {
  onName: (name: string) => void;
  leaf?: boolean;
}) {
  const [name, setName] = useState("");
  const field = useId();

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        const said = name.trim();
        if (said) onName(said);
      }}
    >
      <label
        htmlFor={field}
        className="block px-2.5 pt-1 text-[11px] tracking-[0.04em] text-faint uppercase"
      >
        {t(leaf ? "pageName" : "docName")}
      </label>
      <input
        id={field}
        autoFocus
        value={name}
        onChange={(e) => setName(e.target.value)}
        className="mt-0.5 w-full rounded-md bg-hover px-2.5 py-1.5 outline-none"
      />
      <button type="submit" className="sr-only">
        {t(leaf ? "insertNewPage" : "insertNewDoc")}
      </button>
    </form>
  );
}
