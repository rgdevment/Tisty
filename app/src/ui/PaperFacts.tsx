import type { DocFacts } from "../core";
import { stamped, weigh } from "../format";
import { fill, t } from "../locales";
import { DOC } from "../markdown";

export const worded = (body: string): number =>
  body
    .replace(/```[\s\S]*?```/g, " ")
    .replace(/[#>*_`~[\]()!|-]/g, " ")
    .split(/\s+/)
    .filter(Boolean).length;

export const counted = (body: string, needle: string): number => body.split(needle).length - 1;

const dated = (when: number | null): string =>
  when ? stamped(new Date(when * 1000).toISOString()) : "—";

export default function PaperFacts({ facts, body }: { facts: DocFacts | null; body: string }) {
  const words = worded(body);
  const papers = counted(body, DOC);
  const files = counted(body, "attachments/");
  return (
    <dl className="flex flex-col gap-1.5 text-[12.5px]">
      {facts?.author && (
        <div className="flex items-baseline justify-between gap-2.5">
          <dt className="text-faint">{t("paperAuthor")}</dt>
          <dd className="truncate text-ink">{facts.author}</dd>
        </div>
      )}
      {facts?.born && (
        <div className="flex items-baseline justify-between gap-2.5">
          <dt className="text-faint">{t("paperBorn")}</dt>
          <dd className="truncate text-soft">{facts.born}</dd>
        </div>
      )}
      {facts?.editor && (
        <div className="flex items-baseline justify-between gap-2.5">
          <dt className="text-faint">{t("paperEditor")}</dt>
          <dd className="truncate text-ink">{facts.editor}</dd>
        </div>
      )}
      <div className="flex items-baseline justify-between gap-2.5">
        <dt className="text-faint">{t("paperMade")}</dt>
        <dd className="tabular-nums text-soft">{dated(facts?.made ?? null)}</dd>
      </div>
      <div className="flex items-baseline justify-between gap-2.5">
        <dt className="text-faint">{t("paperWrote")}</dt>
        <dd className="tabular-nums text-soft">{dated(facts?.wrote ?? null)}</dd>
      </div>
      <div className="flex items-baseline justify-between gap-2.5">
        <dt className="text-faint">{t("paperLong")}</dt>
        <dd className="tabular-nums text-soft">
          {words ? fill("paperWords", String(words)) : t("paperEmpty")}
        </dd>
      </div>
      <div className="flex items-baseline justify-between gap-2.5">
        <dt className="text-faint">{t("paperWeighs")}</dt>
        <dd className="tabular-nums text-soft">{facts ? weigh(facts.bytes) : "—"}</dd>
      </div>
      {(papers > 0 || files > 0 || (facts?.pages ?? 0) > 0) && (
        <div className="flex items-baseline justify-between gap-2.5">
          <dt className="text-faint">{t("paperHolds")}</dt>
          <dd className="text-right tabular-nums text-soft">
            {[
              facts?.pages
                ? facts.pages === 1
                  ? t("pageHeld")
                  : fill("pagesHeld", String(facts.pages))
                : "",
              papers > 0 ? fill("paperDocs", String(papers)) : "",
              files > 0 ? fill("paperFiles", String(files)) : "",
            ]
              .filter(Boolean)
              .join(" · ")}
          </dd>
        </div>
      )}
    </dl>
  );
}
