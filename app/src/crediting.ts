import { stored } from "./stored";

export type Signing = "named" | "agent" | "mark";
export type Place = "rows" | "inside";
export type Credit = Record<Place, Signing>;

export const SIGNINGS: readonly Signing[] = ["named", "agent", "mark"];

const AS_SHIPPED: Credit = { rows: "agent", inside: "named" };

const isSigning = (said: unknown): said is Signing => SIGNINGS.includes(said as Signing);

// Before each place had its own choice, a switch per place said named or not, and «mark» overrode both.
const fromSwitch = (said: unknown, mark: unknown): Signing | undefined => {
  if (mark === true) return "mark";
  if (typeof said === "boolean") return said ? "named" : "agent";
  return undefined;
};

export const creditRead = (said: Record<string, unknown>): Credit => {
  const at = (place: Place): Signing => {
    const kept = said[place];
    if (isSigning(kept)) return kept;
    return fromSwitch(kept, said.mark) ?? AS_SHIPPED[place];
  };
  return { rows: at("rows"), inside: at("inside") };
};

const chosen = stored<Credit>("tisty.credit", creditRead);

export const credit = (place: Place, how: Signing) =>
  chosen.put({ ...chosen.current(), [place]: how });

export const useCredit = chosen.use;

export const useSigning = (place: Place): Signing => useCredit()[place];
