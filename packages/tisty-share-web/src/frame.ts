import type { Facts } from "./fetched";
import { ago, day } from "./when";
import { filled } from "./words";

export type Signed =
  | { readonly is: "holds" | "revoked"; readonly fingerprint: string; readonly suffix: string }
  | { readonly is: "refused" | "unreadable" };

export type About = {
  readonly title?: string | null;
  readonly tags?: readonly string[];
  readonly version?: number;
  readonly wrote?: Signed | null;
  readonly sent?: Signed | null;
};

export type Seal = {
  readonly verdict: "holds" | "revoked" | "refused" | "unreadable" | "unsigned";
  readonly who?: string;
  readonly fingerprint?: string;
  readonly sender?: string;
};

type Say = (word: string) => string;

// A first version was written when it was shared, so only a later one says it was edited.
export const whenOf = (
  facts: Facts,
  version: number,
  now: number,
  language: string,
  say: Say,
): string =>
  [
    facts.shared === undefined ? "" : filled(say("shared"), day(facts.shared, language)),
    facts.written === undefined || version < 2
      ? ""
      : filled(say("written"), ago(facts.written, now, language)),
  ]
    .filter(Boolean)
    .join(" · ");

// Who wrote it is what the reader asks; who uploaded it is said too, but only when it was someone else.
export const sealOf = (about: About): Seal => {
  const signed = about.wrote ?? about.sent;
  if (!signed) return { verdict: "unsigned" };
  const writer = "suffix" in signed ? signed : undefined;
  const sent = about.sent && "suffix" in about.sent ? about.sent : undefined;
  return {
    verdict: signed.is,
    who: writer?.suffix,
    fingerprint: writer?.fingerprint,
    sender: sent && sent.fingerprint !== writer?.fingerprint ? sent.suffix : undefined,
  };
};

export const sealed = (box: HTMLElement, seal: Seal, say: Say) => {
  const part = (name: string) => box.querySelector<HTMLElement>(`.${name}`) as HTMLElement;
  box.dataset.verdict = seal.verdict;
  part("seal-who").textContent = seal.who ? `${say("signedby")} ${seal.who}` : "";
  part("seal-print").textContent = seal.fingerprint ?? "";
  part("seal-print").hidden = !seal.fingerprint;
  part("seal-verdict").textContent = say(seal.verdict);
  part("seal-sender").textContent = seal.sender ? `${say("sentby")} ${seal.sender}` : "";
  box.hidden = false;
};

// The body leaves its front matter undrawn, so its tags are shown above it, as the mock-up does.
export const tagged = (row: HTMLElement, tags: readonly string[]) => {
  row.replaceChildren(
    ...tags.map((tag) => {
      const one = document.createElement("span");
      one.className = "tag";
      one.textContent = `#${tag}`;
      return one;
    }),
  );
  row.hidden = tags.length === 0;
};

export const accessOf = (facts: Facts): string => (facts.gated ? "gated" : "unlisted");

// Every reading pushes the end back (decision 50), so the date says it holds only if nobody comes first.
export const endsOf = (facts: Facts, language: string, say: Say): string =>
  facts.ends === undefined ? "" : filled(say("endsunless"), day(facts.ends, language));

// A name every system takes: the title without the characters some forbid, or the page's own word for it.
export const fileOf = (title: string, fallback: string): string =>
  `${title.replaceAll(/[\\/:*?"<>|]/g, "").trim() || fallback}.md`;

// Tisty takes the whole link, key and all, from its own scheme; the browser hands it over only on a click.
export const tistyOf = (link: string): string => `tisty://share?link=${encodeURIComponent(link)}`;
