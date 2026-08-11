import { useEffect, useState } from "react";
import { search, type Found, type Scope } from "../core";
import { t } from "../locales";
import { saidPlainly } from "../refusal";
import Field from "./Field";

interface Props {
  fixed?: Scope;
  onFound: (found: Found | null) => void;
  onError: (message: string) => void;
}

const SETTLES = 150;
const ENOUGH = 2;
const SCOPES: Scope[] = ["open", "archived", "either"];
const LABEL: Record<Scope, "scopeEither" | "scopeOpen" | "scopeArchived"> = {
  either: "scopeEither",
  open: "scopeOpen",
  archived: "scopeArchived",
};

export default function Search({ fixed, onFound, onError }: Props) {
  const [query, setQuery] = useState("");
  // The placeholder promises the archive, so the default has to reach it:
  // searching a ticket from six months ago found nothing and read as lost.
  const [scope, setScope] = useState<Scope>(fixed ?? "either");

  useEffect(() => {
    // One letter matches most of a long archive, and the cost of that lands on
    // every keystroke: the whole thing crosses the IPC boundary and gets drawn.
    if (query.trim().length < ENOUGH) {
      onFound(null);
      return;
    }
    const timer = setTimeout(() => {
      search(query, scope)
        .then(onFound)
        .catch((problem) => onError(saidPlainly(problem)));
    }, SETTLES);
    return () => clearTimeout(timer);
  }, [query, scope, onFound, onError]);

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
