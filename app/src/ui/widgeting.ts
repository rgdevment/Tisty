import {
  type Lender,
  type Lending,
  type Span,
  lend as sheetLend,
  notWidget as sheetNotWidget,
} from "tisty-sheet";
import { fill, t } from "../locales";
import { darkNow, followTheme } from "./theming";

export {
  CLIMBS_AT_MOST,
  CLIMBS_WITHIN,
  type Lending,
  SHORTEST,
  type Span,
  TALLEST,
  tall,
} from "tisty-sheet";

export const LENDER: Lender = {
  say: (word, named) => (named === undefined ? t(word) : fill(word, named)),
  dark: () => darkNow(),
  onTheme: (run) => followTheme(run),
};

export const notWidget = (source: string): string | null => sheetNotWidget(source, LENDER.say);

export const lend = (drawn: HTMLElement, source: string, onSpan?: (span: Span) => void): Lending =>
  sheetLend(drawn, source, LENDER, onSpan);
