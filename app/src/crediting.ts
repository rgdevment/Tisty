import { stored } from "./stored";

export type Credit = { rows: boolean; inside: boolean; mark: boolean };
export type Signing = "named" | "agent" | "mark";

const AS_SHIPPED: Credit = { rows: false, inside: true, mark: false };

const chosen = stored<Credit>("tisty.credit", (said) => {
  const or = (key: keyof Credit) =>
    typeof said[key] === "boolean" ? (said[key] as boolean) : AS_SHIPPED[key];
  return { rows: or("rows"), inside: or("inside"), mark: or("mark") };
});

export const credit = (where: keyof Credit, on: boolean) =>
  chosen.put({ ...chosen.current(), [where]: on });

export const useCredit = chosen.use;

export const signing = (credited: Credit, where: "rows" | "inside"): Signing => {
  if (credited.mark) return "mark";
  return credited[where] ? "named" : "agent";
};

export const useSigning = (where: "rows" | "inside"): Signing => signing(useCredit(), where);
