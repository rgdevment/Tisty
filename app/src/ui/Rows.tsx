import type { Keeper } from "../core";
import { stamped } from "../format";
import { warningOf } from "../keepers";
import type { Which, Word } from "./Card";

export const dated = (when: number): string => {
  const at = new Date(when * 1000);
  return Number.isNaN(at.getTime()) ? "—" : stamped(at.toISOString());
};

export const ON_MAC = typeof navigator !== "undefined" && navigator.userAgent.includes("Macintosh");

export const off = "disabled:border-hair disabled:bg-hair disabled:text-soft";
export const mild = `rounded-[10px] border border-line px-2.5 py-1 text-[12.5px] hover:bg-hover ${off}`;
export const strong = `rounded-[10px] bg-accent px-2.5 py-1 text-[12.5px] text-bg ${off}`;
export const risky = `rounded-[10px] border border-urgent/40 px-2.5 py-1 text-[12.5px] text-urgent hover:bg-urgent/10 ${off}`;

export function Band({ label }: { label: string }) {
  return (
    <div className="mt-5 mb-1.5 text-[11.5px] font-semibold tracking-[0.06em] text-faint uppercase">
      {label}
    </div>
  );
}

export function Ask({ said }: { said: string }) {
  return (
    <span className="relative inline-flex">
      <button
        type="button"
        aria-label={said}
        className="peer flex h-[15px] w-[15px] items-center justify-center rounded-md text-[11.5px] leading-none text-faint hover:bg-line hover:text-ink focus-visible:ring-2 focus-visible:ring-accent focus-visible:outline-none"
      >
        ?
      </button>
      <span
        aria-hidden
        className="pointer-events-none absolute top-6 left-0 z-20 w-[300px] rounded-[10px] border border-line bg-bg p-3 text-[12.5px] leading-relaxed font-normal whitespace-pre-line text-soft opacity-0 shadow-lift transition-opacity peer-hover:opacity-100 peer-focus-visible:opacity-100 motion-reduce:transition-none"
      >
        {said}
      </span>
    </span>
  );
}

export function Line({
  title,
  why,
  which,
  said,
  trouble,
  children,
  more,
}: {
  title: React.ReactNode;
  why?: React.ReactNode;
  which: Which;
  said?: Word;
  trouble?: Word;
  children?: React.ReactNode;
  more?: React.ReactNode;
}) {
  return (
    <div className="border-b border-hair py-2.5">
      <div className="flex items-center gap-4">
        <span className="min-w-0 flex-1">
          <span className="block text-[13px] font-medium">{title}</span>
          {why && <span className="mt-px block text-[12.5px] leading-snug text-faint">{why}</span>}
        </span>
        {children && (
          <span className="flex shrink-0 flex-wrap items-center justify-end gap-2">{children}</span>
        )}
      </div>
      {more}
      {trouble?.card === which && (
        <p className="mt-1.5 text-[11.5px] text-urgent">{trouble.text}</p>
      )}
      {said?.card === which && <p className="mt-1.5 text-[11.5px] text-faint">{said.text}</p>}
    </div>
  );
}

export function Knob({
  on,
  label,
  disabled,
  onPress,
}: {
  on: boolean;
  label: string;
  disabled?: boolean;
  onPress: () => void;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={on}
      aria-label={label}
      disabled={disabled}
      onClick={onPress}
      className={`relative h-5 w-[34px] shrink-0 rounded-full transition-colors motion-reduce:transition-none disabled:opacity-50 ${
        on ? "bg-accent" : "bg-hair"
      }`}
    >
      <span
        className={`absolute top-0.5 block size-4 rounded-full bg-bg shadow-sm transition-[left] motion-reduce:transition-none ${
          on ? "left-[16px]" : "left-0.5"
        }`}
      />
    </button>
  );
}

export function Group({ label }: { label: string }) {
  return (
    <div className="mt-5 mb-2 flex items-center gap-2.5 text-[11.5px] font-semibold tracking-[0.05em] text-faint uppercase">
      <span>{label}</span>
      <span className="h-px flex-1 bg-hair" />
    </div>
  );
}

export function Warned({ keeper, named }: { keeper: Keeper; named?: string }) {
  const warning = warningOf(keeper, named);
  return (
    <div
      className={`mt-2 rounded-[10px] px-3 py-2 text-[12.5px] leading-relaxed text-soft ${
        warning.mild ? "bg-accent-soft" : "border border-hue-amber/40"
      }`}
    >
      <span className="block text-[12.5px] font-semibold text-ink">{warning.said}</span>
      {warning.why}
    </div>
  );
}
