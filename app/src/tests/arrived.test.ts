import { describe, expect, it } from "vitest";
import { t } from "../locales";
import { cameWords } from "../ui/arrived";
import { dated } from "../ui/Rows";

const WHEN = 1_760_000_000;

describe("how an attachment came, said beside it in Maintenance", () => {
  it("says how, from which computer and when", () => {
    expect(cameWords({ how: "assistant", machine: "Portátil", mine: false, when: WHEN })).toBe(
      `${t("cameAssistant")} · Portátil · ${dated(WHEN)}`,
    );
  });

  it("calls this computer by that name rather than its own", () => {
    expect(cameWords({ how: "window", machine: "Escritorio", mine: true, when: WHEN })).toBe(
      `${t("cameWindow")} · ${t("cameHere")} · ${dated(WHEN)}`,
    );
  });

  it("tells only where and when for one kept before anyone said how", () => {
    expect(cameWords({ how: null, machine: "Portátil", mine: false, when: WHEN })).toBe(
      `Portátil · ${dated(WHEN)}`,
    );
  });

  it("passes over a way it does not know rather than showing its code", () => {
    expect(cameWords({ how: "teleport", machine: "Portátil", mine: false, when: WHEN })).toBe(
      `Portátil · ${dated(WHEN)}`,
    );
  });

  it("says nothing when the log says nothing", () => {
    expect(cameWords(null)).toBeNull();
  });
});
