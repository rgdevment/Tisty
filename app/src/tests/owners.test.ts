import { renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Task, Whole } from "../core";

const task = (more: Partial<Task> = {}): Task =>
  ({
    id: "1",
    title: "ship it",
    status: "open",
    priority: "unset",
    order: "a0",
    tags: [],
    reminders: [],
    log: [],
    created_by: "dev_agent",
    ...more,
  }) as unknown as Task;

const said = (more: Record<string, unknown> = {}) => ({
  at: "2026-08-06T18:40:00Z",
  by: "dev_agent",
  entry: "e1",
  ...more,
});

const load = async () => {
  const who = await import("../who");
  const owners = await import("../owners");
  return { ...owners, knowAgents: who.knowAgents };
};

beforeEach(() => {
  vi.resetModules();
  localStorage.clear();
});

afterEach(() => localStorage.clear());

describe("who a task belongs to", () => {
  it("is the agent's when a known agent filed it", async () => {
    const { ownerOf, knowAgents } = await load();
    knowAgents({ dev_agent: "canelo 39" });

    expect(ownerOf(task())).toBe("agent");
  });

  it("is the person's when the one who filed it is not a known agent", async () => {
    const { ownerOf, knowAgents } = await load();
    knowAgents({ other: "x" });

    expect(ownerOf(task())).toBe("person");
  });

  it("is the person's when nobody is known as an agent", async () => {
    const { ownerOf } = await load();

    expect(ownerOf(task())).toBe("person");
  });

  it("is the person's when the agent filed it for the person", async () => {
    const { ownerOf, knowAgents } = await load();
    knowAgents({ dev_agent: "canelo 39" });

    expect(ownerOf(task({ for_person: true }))).toBe("person");
  });

  it("is the agent's when for_person is explicitly false", async () => {
    const { ownerOf, knowAgents } = await load();
    knowAgents({ dev_agent: "canelo 39" });

    expect(ownerOf(task({ for_person: false }))).toBe("agent");
  });

  it("is the person's when the task has no writer", async () => {
    const { ownerOf, knowAgents } = await load();
    knowAgents({ dev_agent: "canelo 39" });

    expect(ownerOf(task({ created_by: undefined }))).toBe("person");
  });
});

describe("what the person can confirm", () => {
  it("is an open task an agent said done", async () => {
    const { confirmable } = await load();

    expect(confirmable(task({ resolved: said() }))).toBe(true);
  });

  it("is not a task nobody said anything about", async () => {
    const { confirmable } = await load();

    expect(confirmable(task())).toBe(false);
  });

  it("is not a task said not to be done", async () => {
    const { confirmable } = await load();

    expect(confirmable(task({ resolved: said({ drop: true }) }))).toBe(false);
  });

  it("is not a repeating task", async () => {
    const { confirmable } = await load();
    const repeat = { every: 1, unit: "week" } as unknown as Task["repeat"];

    expect(confirmable(task({ resolved: said(), repeat }))).toBe(false);
  });

  it.each(["done", "dropped"] as const)("is not a task that is already %s", async (status) => {
    const { confirmable } = await load();

    expect(confirmable(task({ status, resolved: said() }))).toBe(false);
  });
});

describe("the proof an agent leaves", () => {
  const withLog = (body: string, entry = "e1") =>
    task({ resolved: said(), log: [{ id: entry, body }] as unknown as Task["log"] });

  it("is empty when nothing was said", async () => {
    const { proofOf } = await load();

    expect(proofOf(task())).toEqual([]);
  });

  it("is empty when the entry is not in the log", async () => {
    const { proofOf } = await load();

    expect(proofOf(withLog("all green", "other"))).toEqual([]);
  });

  it("is empty when the entry has no body", async () => {
    const { proofOf } = await load();
    const bare = task({ resolved: said(), log: [{ id: "e1" }] as unknown as Task["log"] });

    expect(proofOf(bare)).toEqual([]);
  });

  it("keeps the first two lines", async () => {
    const { proofOf } = await load();

    expect(proofOf(withLog("one\ntwo\nthree"))).toEqual(["one", "two"]);
  });

  it("skips blank lines", async () => {
    const { proofOf } = await load();

    expect(proofOf(withLog("\n\none\n   \n\ntwo\nthree"))).toEqual(["one", "two"]);
  });

  it("strips headings, bullets, numbers and quotes", async () => {
    const { proofOf } = await load();

    expect(proofOf(withLog("## Result\n- 12 tests passed"))).toEqual(["Result", "12 tests passed"]);
    expect(proofOf(withLog("* star\n+ plus"))).toEqual(["star", "plus"]);
    expect(proofOf(withLog("1. first\n2) second"))).toEqual(["first", "second"]);
    expect(proofOf(withLog("> quoted\nplain"))).toEqual(["quoted", "plain"]);
  });

  it("strips emphasis and code marks", async () => {
    const { proofOf } = await load();

    expect(proofOf(withLog("**all** `green` and _fast_"))).toEqual(["all green and fast"]);
  });

  it("picks the entry the resolution points at, among several", async () => {
    const { proofOf } = await load();
    const one = task({
      resolved: said({ entry: "e2" }),
      log: [
        { id: "e1", body: "old" },
        { id: "e2", body: "right one" },
      ] as unknown as Task["log"],
    });

    expect(proofOf(one)).toEqual(["right one"]);
  });
});

describe("the companion of a part for the person", () => {
  const wholes = (working: boolean): Record<string, Whole> => ({
    w1: { title: "the release", open: 2, closed: 0, working },
  });
  const part = (more: Partial<Task> = {}) => task({ for_person: true, part_of: "w1", ...more });

  it("waits on the whole while the agent works on it", async () => {
    const { companionOf } = await load();

    expect(companionOf(part(), wholes(true))).toEqual({ key: "waitsOn", title: "the release" });
  });

  it("is the person's own part once the agent is not working on the whole", async () => {
    const { companionOf } = await load();

    expect(companionOf(part(), wholes(false))).toEqual({
      key: "yourPartOf",
      title: "the release",
    });
  });

  it("is the person's part when the whole does not say", async () => {
    const { companionOf } = await load();
    const loose = { w1: { title: "the release", open: 1, closed: 0 } };

    expect(companionOf(part(), loose)?.key).toBe("yourPartOf");
  });

  it("is nothing for a part not meant for the person", async () => {
    const { companionOf } = await load();

    expect(companionOf(part({ for_person: false }), wholes(true))).toBeUndefined();
  });

  it("is nothing for a task that is part of nothing", async () => {
    const { companionOf } = await load();

    expect(companionOf(part({ part_of: undefined }), wholes(true))).toBeUndefined();
  });

  it("is nothing when the whole is unknown or no wholes are given", async () => {
    const { companionOf } = await load();

    expect(companionOf(part({ part_of: "zz" }), wholes(true))).toBeUndefined();
    expect(companionOf(part(), undefined)).toBeUndefined();
  });
});

describe("the first meeting with a section", () => {
  const meet = async (owner: "person" | "agent") => {
    const { useFirstMeeting } = await load();
    return renderHook(() => useFirstMeeting(owner));
  };

  it("is true the first time and stays true for the life of that component", async () => {
    const first = await meet("agent");

    expect(first.result.current).toBe(true);
    first.rerender();
    expect(first.result.current).toBe(true);
  });

  it("is false for the next component once the first has been seen", async () => {
    const { useFirstMeeting } = await load();
    renderHook(() => useFirstMeeting("agent")).unmount();

    expect(renderHook(() => useFirstMeeting("agent")).result.current).toBe(false);
  });

  it("keeps each section's meeting apart", async () => {
    const { useFirstMeeting } = await load();
    renderHook(() => useFirstMeeting("agent")).unmount();

    expect(renderHook(() => useFirstMeeting("person")).result.current).toBe(true);
  });

  it("remembers in the browser's storage that the section was seen", async () => {
    await meet("person");

    expect(JSON.parse(localStorage.getItem("tisty.owners") ?? "{}")).toEqual({
      person: true,
      agent: false,
    });
  });

  it("is false when the storage already says the section was seen", async () => {
    localStorage.setItem("tisty.owners", JSON.stringify({ agent: true }));

    expect((await meet("agent")).result.current).toBe(false);
  });

  it("copes with storage that holds nonsense", async () => {
    localStorage.setItem("tisty.owners", "not json");

    expect((await meet("agent")).result.current).toBe(true);
  });
});
