import { useEffect, useState } from "react";
import { type Found, type Scope, search } from "../core";
import { t } from "../locales";
import { saidPlainly } from "../refusal";
import Field from "./Field";

interface Props {
  fixed?: Scope;
  onFound: (found: Found | null) => void;
  onError: (message: string) => void;
}

const SETTLES = 150;
const SCOPES: Scope[] = ["open", "archived", "either"];
const LABEL: Record<Scope, "scopeEither" | "scopeOpen" | "scopeArchived"> = {
  either: "scopeEither",
  open: "scopeOpen",
  archived: "scopeArchived",
};

export default function Search({ fixed, onFound, onError }: Props) {
  const [query, setQuery] = useState("");
  const [scope, setScope] = useState<Scope>(fixed ?? "either");

  useEffect(() => {
    if (!query.trim()) {
      onFound(null);
      return;
    }
    let mine = true;
    const timer = setTimeout(() => {
      search(query, scope)
        .then((found) => {
          // A wider question takes longer, so its answer can land after a narrower one and leave
          // the screen saying what nobody asked any more.
          if (mine) onFound(found);
        })
        .catch((problem) => mine && onError(saidPlainly(problem)));
    }, SETTLES);
    return () => {
      mine = false;
      clearTimeout(timer);
    };
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
              type="button"
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
