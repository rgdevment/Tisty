import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Pick, Rift } from "../core";
import { fill, t } from "../locales";

let { decideAll, decidesByBlock, decidesWhole } = await import("../deciding");

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
}));

const torn = vi.hoisted(() => ({
  said: { rifts: [] as Rift[], print: "" },
  refuses: false,
  moved: false,
  locked: false,
  blind: false,
  shut: false,
  still: null as string[] | null,
  refuseStill: false,
}));

const asked = vi.hoisted(() => ({
  held: [] as ((sure: boolean) => void)[],
  said: [] as string[],
  waiting: false,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    if (cmd === "docs") {
      if (torn.blind) return Promise.reject(new Error("no"));
      return Promise.resolve({
        folders: [],
        docs: [
          {
            id: "1",
            file: "dev_a-0001",
            title: "Kit de transmisión",
            folder: null,
            archived: false,
            locked: torn.locked,
          },
        ],
      });
    }
    if (cmd === "task_of") {
      return args?.id === "01M563T6T541A3Q2PP5YQW5XK1"
        ? Promise.resolve({ id: args.id, title: "Rina's booster" })
        : Promise.reject({ code: "notATaskId" });
    }
    if (cmd === "still_at_odds") {
      if (torn.refuseStill) return Promise.reject({ code: "internal" });
      return torn.still ? Promise.resolve(torn.still) : Promise.resolve(args?.ids);
    }
    if (cmd === "paper_rifts") {
      return torn.shut ? Promise.reject({ code: "documentLocked" }) : Promise.resolve(torn.said);
    }
    if (cmd === "settle_paper" && torn.moved) return Promise.reject({ code: "movedUnderfoot" });
    if (cmd === "weave_paper" && torn.refuses) return Promise.reject(new Error("cannotWeave"));
    return Promise.resolve(null);
  },
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  ask: (said: string) => {
    asked.said.push(said);
    return asked.waiting
      ? new Promise<boolean>((resolve) => asked.held.push(resolve))
      : Promise.resolve(true);
  },
}));

beforeEach(async () => {
  vi.resetModules();
  ({ decideAll, decidesByBlock, decidesWhole } = await import("../deciding"));
  torn.said = { rifts: [], print: "" };
  torn.refuses = false;
  torn.moved = false;
  torn.still = null;
  torn.refuseStill = false;
  torn.locked = false;
  torn.blind = false;
  torn.shut = false;
  decidesByBlock(null);
  decidesWhole(null);
  ipc.calls = [];
  asked.held = [];
  asked.said = [];
  asked.waiting = false;
});

const settled = () => ipc.calls.filter((one) => one.cmd === "settle_paper");

const woven = () => ipc.calls.filter((one) => one.cmd === "weave_paper");

describe("deciding a document block by block", () => {
  const rift: Rift = { was: ["antes"], mine: ["lo mio"], theirs: ["lo suyo"] };

  it("carries the print of what was shown back with the answers", async () => {
    torn.said = { rifts: [rift], print: "huella-1" };
    decidesByBlock(async () => ["mine"] as Pick[]);

    await decideAll(["dev_a-0001"]);

    expect(woven()).toHaveLength(1);
    expect(woven()[0].args.print).toBe("huella-1");
    expect(woven()[0].args.picks).toEqual(["mine"]);
  });

  it("writes nothing when the person closes without answering", async () => {
    torn.said = { rifts: [rift], print: "huella-1" };
    decidesByBlock(async () => null);

    await decideAll(["dev_a-0001"]);

    expect(woven()).toHaveLength(0);
    expect(settled()).toHaveLength(0);
  });

  it("asks outright rather than leaving the person stuck when the weave is refused", async () => {
    torn.said = { rifts: [rift], print: "huella-1" };
    torn.refuses = true;
    decidesByBlock(async () => ["mine"] as Pick[]);

    await decideAll(["dev_a-0001"]);

    expect(settled()).toHaveLength(1);
  });

  it("falls back to asking outright when the blocks cannot be worked out", async () => {
    torn.said = { rifts: [], print: "" };
    decidesByBlock(async () => ["mine"] as Pick[]);

    await decideAll(["dev_a-0001"]);

    expect(woven()).toHaveLength(0);
    expect(settled()).toHaveLength(1);
  });
});

describe("deciding what to do with a document written on both sides", () => {
  it("keeps both when that is the answer", async () => {
    await decideAll(["dev_a-0001"]);

    expect(settled()).toHaveLength(1);
    expect(settled()[0].args.keep).toBe("both");
  });

  it("calls the document by its title, never by the name of its file", async () => {
    await decideAll(["dev_a-0001"]);

    expect(asked.said[0]).toMatch(/Kit de transmisión/);
    expect(asked.said[0]).not.toMatch(/dev_a-0001/);
  });

  it("says untitled rather than a file name when the document has no title", async () => {
    await decideAll(["dev_a-0002"]);

    expect(asked.said[0]).not.toMatch(/dev_a-0002/);
    expect(asked.said[0]).toMatch(/Untitled/i);
  });

  it("calls a task's page by its task's title, since the documents leave it out", async () => {
    await decideAll(["t_01m563t6t541a3q2pp5yqw5xk1-0001"]);

    expect(asked.said[0]).toMatch(/Rina's booster/);
    expect(asked.said[0]).not.toMatch(/Untitled/i);
  });

  it("asks once for a document two syncs both call undecided", async () => {
    asked.waiting = true;
    const carrier = decideAll(["dev_a-0001"]);
    const panel = decideAll(["dev_a-0001"]);
    await vi.waitFor(() => expect(asked.held.length).toBeGreaterThan(0));
    for (const answer of asked.held) answer(true);
    await carrier;
    await panel;

    expect(settled()).toHaveLength(1);
  });
});

describe("a document the person locked, at odds with another machine", () => {
  it("asks nothing and writes nothing, and hands it back to be told about", async () => {
    torn.locked = true;
    torn.said = { rifts: [{ was: ["antes"], mine: ["lo mio"], theirs: ["lo suyo"] }], print: "h" };

    const { shut } = await decideAll(["dev_a-0001"]);

    expect(shut).toEqual(["dev_a-0001"]);
    expect(asked.said).toEqual([]);
    expect(settled()).toEqual([]);
    expect(woven()).toEqual([]);
  });

  it("decides nothing at all when it cannot find out what is locked, and never calls it locked", async () => {
    torn.blind = true;

    const { shut, said } = await decideAll(["dev_a-0001", "dev_a-0002"]);

    expect(shut).toEqual([]);
    expect(said).toBeTruthy();
    expect(asked.said).toEqual([]);
    expect(settled()).toEqual([]);
  });

  it("opens no block view when the window turns the reading down", async () => {
    torn.shut = true;
    torn.said = { rifts: [{ was: ["antes"], mine: ["lo mio"], theirs: ["lo suyo"] }], print: "h" };
    const byBlock = vi.fn();
    decidesByBlock(byBlock);

    await decideAll(["dev_a-0002"]);

    expect(byBlock).not.toHaveBeenCalled();
    expect(woven()).toEqual([]);
  });

  it("still settles the ones that are not locked", async () => {
    const { shut } = await decideAll(["dev_a-0002"]);

    expect(shut).toEqual([]);
    expect(settled().length).toBe(1);
  });
});

describe("a document that moved again while the person decided", () => {
  it("says it changed underfoot, never that it is locked", async () => {
    torn.moved = true;

    const { shut, said } = await decideAll(["dev_a-0001"]);

    expect(shut).toEqual([]);
    expect(said).toBe(fill("changedWhileDeciding", "Kit de transmisión"));
  });
});

describe("a locked document and another that failed, in one round", () => {
  it("says both, never only the lock", async () => {
    torn.locked = true;
    torn.moved = true;

    const { shut, said } = await decideAll(["dev_a-0001", "dev_a-0002"]);

    expect(shut).toEqual(["dev_a-0001"]);
    expect(said).toContain(fill("someLockedAtOdds", "«Kit de transmisión»"));
    expect(said).toContain(fill("changedWhileDeciding", t("untitledDoc")));
  });
});

describe("a title that carries a dollar sign", () => {
  it("is said as it is written", () => {
    expect(fill("changedWhileDeciding", "Gastos $$ 2026 $& $'")).toContain(
      "«Gastos $$ 2026 $& $'»",
    );
  });
});

describe("asking inside the window", () => {
  it("asks there instead of with the system's dialog, and keeps what was picked", async () => {
    const whole = vi.fn(async () => "theirs" as Pick);
    decidesWhole(whole);

    await decideAll(["dev_a-0001"]);

    expect(whole).toHaveBeenCalledWith("Kit de transmisión");
    expect(asked.said).toEqual([]);
    expect(settled()[0].args.keep).toBe("theirs");
  });

  it("puts off a document the person left for later until they sync by hand", async () => {
    const whole = vi.fn(async (): Promise<Pick | null> => null);
    decidesWhole(whole);

    expect((await decideAll(["dev_a-0003"])).putOff).toEqual(["dev_a-0003"]);
    expect((await decideAll(["dev_a-0003"])).putOff).toEqual([]);
    expect(whole).toHaveBeenCalledTimes(1);
    expect(settled()).toEqual([]);

    whole.mockImplementation(async () => "mine");
    await decideAll(["dev_a-0003"], true);
    expect(whole).toHaveBeenCalledTimes(2);
    expect(settled()[0].args.keep).toBe("mine");
  });

  it("asks one thing at a time, and never again for what a round waiting behind it named", async () => {
    const held: ((pick: Pick) => void)[] = [];
    decidesWhole(() => new Promise((answer) => held.push(answer)));

    const first = decideAll(["dev_a-0004", "dev_a-0005"]);
    const second = decideAll(["dev_a-0004", "dev_a-0005"]);
    await vi.waitFor(() => expect(held).toHaveLength(1));
    held[0]("mine");
    await vi.waitFor(() => expect(held).toHaveLength(2));
    held[1]("mine");
    await first;
    await second;

    expect(held).toHaveLength(2);
    expect(settled()).toHaveLength(2);
  });
});

describe("a document that comes back at odds after it was decided", () => {
  it("is asked about again by the next round that looks after the decision", async () => {
    const whole = vi.fn(async (): Promise<Pick | null> => "mine");
    decidesWhole(whole);

    await decideAll(["dev_a-0006"]);
    await decideAll(["dev_a-0006"]);

    expect(whole).toHaveBeenCalledTimes(2);
  });

  it("lets go of a put-off document that can no longer be settled, so it does not wait forever", async () => {
    const { leftForLater } = await import("../deciding");
    decidesWhole(async () => null);
    await decideAll(["dev_a-0008"]);
    expect(leftForLater()).toEqual(["dev_a-0008"]);

    decidesWhole(async () => "mine");
    torn.moved = true;
    await decideAll(["dev_a-0008"], true);

    expect(leftForLater()).toEqual([]);
  });

  it("lets go of a put-off document that is no longer at odds, and keeps the rest", async () => {
    const { leftForLater, settledElsewhere } = await import("../deciding");
    decidesWhole(async () => null);
    await decideAll(["dev_a-0010", "dev_a-0011"]);

    torn.still = ["dev_a-0011"];
    await settledElsewhere();

    expect(leftForLater()).toEqual(["dev_a-0011"]);
  });

  it("keeps every put-off document when it cannot tell which are still at odds", async () => {
    const { leftForLater, settledElsewhere } = await import("../deciding");
    decidesWhole(async () => null);
    await decideAll(["dev_a-0012"]);

    torn.refuseStill = true;
    await settledElsewhere();

    expect(leftForLater()).toEqual(["dev_a-0012"]);
  });

  it("tells a sync by hand what was left for later", async () => {
    decidesWhole(async () => null);

    const { putOff } = await decideAll(["dev_a-0007"], true);

    expect(putOff).toEqual(["dev_a-0007"]);
  });
});
