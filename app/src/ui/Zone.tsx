import type { ReactNode } from "react";
import { locale, t } from "../locales";
import { type Article, articleOf, pageOf } from "../settingsPages";
import type { RowId } from "../settingsRows";
import type { Tab } from "../views";
import type { Which } from "./Card";
import { Line, risky } from "./Rows";

export function PageHead({ tab, onHelp }: { tab: Tab; onHelp?: (article?: Article) => void }) {
  const page = pageOf(tab);
  const article = articleOf(tab, locale().toLowerCase().startsWith("es"));
  return (
    <div className="mb-1.5 flex items-start gap-3">
      <div className="min-w-0 flex-1">
        <h2 className="text-[21px] font-semibold">{t(page.label)}</h2>
        {page.lead && (
          <p className="mt-1 max-w-[62ch] text-[13px] leading-snug text-soft">{t(page.lead)}</p>
        )}
      </div>
      {onHelp && (
        <button
          type="button"
          onClick={() => onHelp(article)}
          className="mt-1.5 shrink-0 text-[12.5px] whitespace-nowrap text-accent hover:underline"
        >
          {t("howItWorks")} <span aria-hidden="true">↗</span>
        </button>
      )}
    </div>
  );
}

export function Zone({ children }: { children: ReactNode }) {
  return (
    <section
      aria-label={t("delicateZone")}
      className="mt-6 rounded-[10px] border border-urgent/40 px-3.5 pb-1"
    >
      <div className="mt-3 mb-1 text-[11.5px] font-semibold tracking-[0.06em] text-urgent uppercase">
        {t("delicateZone")}
      </div>
      <div className="[&>div:last-child]:border-b-0">{children}</div>
    </section>
  );
}

export interface Exit {
  row: RowId;
  which: Which;
  why: string;
  label: string;
  onPress: () => void;
  disabled?: boolean;
}

export function ZoneButtons({ exits }: { exits: (Exit | false)[] }) {
  const shown = exits.filter((one): one is Exit => one !== false);
  if (shown.length === 0) return null;
  return (
    <Zone>
      {shown.map((one) => (
        <Line key={one.row} row={one.row} why={one.why} which={one.which}>
          <button type="button" disabled={one.disabled} onClick={one.onPress} className={risky}>
            {one.label}
          </button>
        </Line>
      ))}
    </Zone>
  );
}

export function More({ children }: { children: ReactNode }) {
  return (
    <details className="mt-1.5">
      <summary className="inline-block cursor-pointer list-none text-[12.5px] text-accent hover:underline [&::-webkit-details-marker]:hidden">
        {t("knowMore")}
      </summary>
      <div className="mt-1.5 max-w-[64ch] text-[12.5px] leading-relaxed whitespace-pre-line text-soft">
        {children}
      </div>
    </details>
  );
}

export function Scope({ here }: { here?: boolean }) {
  return (
    <span className="rounded-md bg-active px-1.5 text-[10.5px] leading-[17px] font-medium whitespace-nowrap text-soft">
      {t(here ? "scopeHere" : "scopeTravels")}
    </span>
  );
}
