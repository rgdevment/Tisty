import type { Keeper } from "./core";
import { fill, t } from "./locales";

export interface Warning {
  said: string;
  why: string;
  mild: boolean;
}

export const warningOf = (keeper: Keeper, named?: string): Warning => {
  if (keeper === "cloud" && named) {
    return { said: fill("keepersCloud", named), why: t("keepersCloudWhy"), mild: true };
  }
  if (keeper === "away") {
    return { said: t("keepersAway"), why: t("keepersAwayWhy"), mild: true };
  }
  return { said: t("keepersPlain"), why: t("keepersPlainWhy"), mild: false };
};
