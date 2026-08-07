import { useEffect, useState } from "react";
import { search, type Scope, type Task } from "../core";
import { t } from "../locales";
import Field from "./Field";

interface Props {
  fixed?: Scope;
  onFound: (tasks: Task[] | null) => void;
}

const SETTLES = 150;
const SCOPES: Scope[] = ["open", "archived", "either"];
const LABEL: Record<Scope, "scopeEither" | "scopeOpen" | "scopeArchived"> = {
  either: "scopeEither",
  open: "scopeOpen",
  archived: "scopeArchived",
};

export default function Search({ fixed, onFound }: Props) {
  const [query, setQuery] = useState("");
  const [scope, setScope] = useState<Scope>(fixed ?? "open");

  useEffect(() => {
    if (!query.trim()) {
      onFound(null);
      return;
    }
    const timer = setTimeout(() => {
      search(query, scope)
        .then(onFound)
        .catch(() => onFound([]));
    }, SETTLES);
    return () => clearTimeout(timer);
  }, [query, scope, onFound]);

  return (
    <div className="w-full">
      <Field
        icon="⌕"
        value={query}
        hint={t(fixed === "archived" ? "searchArchive" : "searchEverywhere")}
        onChange={setQuery}
      />

      {!fixed && (
        <div className="mt-2 flex gap-1 px-1">
          {SCOPES.map((option) => (
            <button
              key={option}
              onClick={() => setScope(option)}
              className={`rounded-md px-2 py-0.5 text-[11.5px] ${
                scope === option ? "bg-active text-ink" : "text-faint hover:text-soft"
              }`}
            >
              {t(LABEL[option])}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
