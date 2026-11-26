import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const core = vi.hoisted(() => ({
  write: vi.fn(),
  merge: vi.fn(),
  trouble: vi.fn(),
}));

vi.mock("../core", async (real) => ({
  ...(await real<typeof import("../core")>()),
  docWrite: core.write,
  docMerge: core.merge,
  noteTrouble: core.trouble,
}));

import { draftOf, keepDraft, letDraftGo } from "../drafts";
import { asideWords, letAsideGo } from "../keptAside";
import { cut, kept } from "../saving";
import { savedOnce } from "../weaving";

const ID = "borrador";
const BASE = "# Compras\n\nleche";
const MINE = "# Compras\n\nleche y pan";
const read = () => ({ text: BASE, print: "print:base" });

const attempt = (mineStands: boolean, seen = read) =>
  savedOnce(ID, MINE, mineStands, seen).catch((e: unknown) => e);

beforeEach(() => {
  core.write.mockReset();
  core.merge.mockReset();
  core.trouble.mockReset().mockResolvedValue(undefined);
  localStorage.clear();
  cut(ID);
  letDraftGo(ID);
});

afterEach(() => {
  cut(ID);
  letDraftGo(ID);
  letAsideGo(ID);
  vi.restoreAllMocks();
});

describe("the draft a save keeps", () => {
  it("stays when the weave is refused as held open", async () => {
    core.write.mockRejectedValue({ code: "documentMoved" });
    core.merge.mockRejectedValue({ code: "documentHeldOpen" });

    const said = await attempt(false);

    expect(said).toEqual({ code: "documentHeldOpen" });
    expect(draftOf(ID)).toEqual({ words: MINE, base: BASE });
  });

  it("stays when the weave is refused as moved", async () => {
    core.write.mockRejectedValue({ code: "documentMoved" });
    core.merge.mockRejectedValue({ code: "documentMoved" });

    await attempt(false);

    expect(draftOf(ID)).toEqual({ words: MINE, base: BASE });
  });

  it("stays when standing by mine is refused", async () => {
    core.merge.mockRejectedValue({ code: "documentHeldOpen" });

    await attempt(true);

    expect(core.merge).toHaveBeenCalledWith(ID, BASE, MINE, expect.anything(), "mine");
    expect(draftOf(ID)).toEqual({ words: MINE, base: BASE });
  });

  it("is stored with an empty base when nothing was read to weave from", async () => {
    core.write.mockRejectedValue({ code: "documentMoved" });

    const said = await attempt(false, () => undefined as never);

    expect(said).toEqual({ code: "documentMoved" });
    expect(core.merge).not.toHaveBeenCalled();
    expect(draftOf(ID)).toEqual({ words: MINE, base: "" });
  });

  it("stays when the window lost track of what an earlier save wrote", async () => {
    await kept(ID, "# Compras\n\nantes", () => Promise.reject({ code: "saveStuck" })).catch(
      () => {},
    );
    core.write.mockRejectedValue({ code: "documentMoved" });

    const said = await attempt(false);

    expect(said).toEqual({ code: "documentMoved" });
    expect(core.merge).not.toHaveBeenCalled();
    expect(draftOf(ID)).toEqual({ words: MINE, base: BASE });
  });

  it("goes when a refusal for good clears the words that were never answered", async () => {
    keepDraft(ID, { words: MINE, base: BASE });

    for (const code of ["documentTooLong", "documentLocked", "noSuchDoc"]) {
      keepDraft(ID, { words: MINE, base: BASE });
      core.write.mockRejectedValueOnce({ code });

      await attempt(false);

      expect(draftOf(ID), code).toBeUndefined();
    }
    letAsideGo(ID);
  });

  it("goes when the save lands", async () => {
    keepDraft(ID, { words: MINE, base: BASE });
    core.write.mockResolvedValue({ id: ID, title: "Compras", print: "print:mine" });

    await attempt(false);

    expect(draftOf(ID)).toBeUndefined();
    expect(asideWords(ID)).toBeUndefined();
  });
});

describe("a draft the browser has no room for", () => {
  it("is told to the log once, and still lives in this window", () => {
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
      throw new DOMException("full", "QuotaExceededError");
    });

    keepDraft(ID, { words: MINE, base: BASE });

    expect(core.trouble).toHaveBeenCalledTimes(1);
    expect(core.trouble).toHaveBeenCalledWith("draftUnkept", ID);
    expect(draftOf(ID)).toEqual({ words: MINE, base: BASE });
  });

  it("says nothing when it is kept", () => {
    keepDraft(ID, { words: MINE, base: BASE });

    expect(core.trouble).not.toHaveBeenCalled();
    expect(JSON.parse(localStorage.getItem("tisty.clashing") ?? "{}")[ID]).toEqual({
      words: MINE,
      base: BASE,
    });
  });

  it("makes the stored value say it was refused", async () => {
    const { stored } = await import("../stored");
    const box = stored<Record<string, never>>("tisty.probe", () => ({}));
    expect(box.put({})).toBe(true);
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
      throw new DOMException("full", "QuotaExceededError");
    });

    expect(box.put({})).toBe(false);
  });
});
