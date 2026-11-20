import { useState } from "react";
import type { Panel } from "tisty-editor/read";
import { t } from "../locales";
import LanguageReview from "./LanguageReview";
import Sharing from "./Sharing";
import type { useSharedRow } from "./sharedRow";

type Shared = ReturnType<typeof useSharedRow>;
type Held = Extract<Panel, { kind: "held" }>;

const TONE = { warn: "text-hue-amber", bad: "text-hue-red" } as const;

function Detail({ panel }: { panel: Held }) {
  return (
    <dd className="m-0 basis-full pt-1.5 text-left text-[11.5px] text-soft">
      {panel.said?.map((one) => (
        <p key={one} className="mb-1">
          {one}
        </p>
      ))}
      {panel.prints && (
        <div className="mb-1">
          <p>{panel.prints.label}</p>
          {panel.prints.rows.map(([who, print]) => (
            <p key={`${who}:${print}`} className="font-mono text-faint">
              {who} · {print}
            </p>
          ))}
        </div>
      )}
      {panel.links?.map((link) => (
        <div key={link.key} className="mb-1 flex items-center gap-2">
          <span className="shrink-0">{link.label}</span>
          <code className="min-w-0 flex-1 truncate font-mono text-faint" title={link.url}>
            {link.url}
          </code>
          <button
            type="button"
            onClick={link.onCopy}
            className="shrink-0 text-accent hover:underline"
          >
            {link.copy}
          </button>
        </div>
      ))}
      {panel.acts && panel.acts.length > 0 && (
        <div className="mt-1.5 flex flex-wrap gap-2">
          {panel.acts.map((act) => (
            <button
              key={act.key}
              type="button"
              disabled={act.off}
              onClick={(e) => act.onPick?.(e.currentTarget)}
              className={`rounded-md border border-line px-2.5 py-1 text-[12.5px] hover:bg-hover disabled:opacity-60 ${act.danger ? "text-hue-red" : "text-ink"}`}
            >
              {act.label}
            </button>
          ))}
        </div>
      )}
    </dd>
  );
}

// What the column beside a document told of its share, now one row of the agent card's facts.
export function SharedFact({ shared }: { shared: Shared }) {
  const [open, setOpen] = useState(false);
  const panel = shared.panel;
  if (!shared.shared || panel?.kind !== "held") return null;
  return (
    <div className="flex flex-wrap items-baseline justify-between gap-x-4 border-b border-hair py-1.5 last:border-b-0">
      <dt className="shrink-0 text-soft">{panel.label}</dt>
      <dd className="m-0 min-w-0 text-right text-ink">
        <span className={panel.tone ? TONE[panel.tone] : undefined}>{panel.gist}</span>
        {" · "}
        <button
          type="button"
          aria-expanded={open}
          onClick={() => setOpen(!open)}
          className="text-accent hover:underline"
        >
          {t(open ? "agentSharedHide" : "agentSee")}
        </button>
      </dd>
      {open && <Detail panel={panel} />}
    </div>
  );
}

interface Windows {
  readonly shared: Shared;
  readonly id?: string;
  readonly title: string;
  readonly onError: (problem: unknown) => void;
}

export function SharedWindows({ shared, id, title, onError }: Windows) {
  if (!id) return null;
  return (
    <>
      {shared.asking && (
        <Sharing
          doc={id}
          title={title}
          termsOnly={shared.termsOnly}
          onTermsAccepted={shared.onTermsAccepted}
          onShared={shared.onShared}
          onClose={shared.onClosed}
          onError={onError}
        />
      )}
      {shared.reviewing && (
        <LanguageReview
          doc={id}
          word={shared.reviewing}
          onDone={shared.onReviewed}
          onClose={shared.onReviewClosed}
          onError={onError}
        />
      )}
    </>
  );
}
