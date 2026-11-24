import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  agentsMoving,
  type Moving,
  movedOnceSure,
  movingAllowed,
  movingSaid,
} from "../agentsMoving";
import { adopt } from "../locales";
import { asksSure } from "../sure";

const calls: { cmd: string; args: Record<string, unknown> }[] = [];
const core = vi.hoisted(() => ({ answer: null as unknown }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args: Record<string, unknown>) => {
    calls.push({ cmd, args });
    return Promise.resolve(core.answer);
  },
}));

type Doc = Moving["docs"][number];
const doc = (title: string, here: Doc["here"], file = `${title.toLowerCase()}.md`): Doc => ({
  title,
  file,
  here,
});
const moving = (over: Partial<Moving>): Moving => ({
  from: { is: "everyone" },
  to: { is: "mine" },
  docs: [doc("Title", "keeps", "AGENTS.md")],
  older: [],
  ...over,
});

const answering = (yes: boolean) => {
  const asked: { said: string; buttons?: { yes: string; no: string } }[] = [];
  asksSure((said, buttons) => {
    asked.push({ said, buttons });
    return Promise.resolve(yes);
  });
  return asked;
};

beforeEach(() => {
  calls.length = 0;
  core.answer = null;
  asksSure(null);
  adopt("en");
});

describe("what moving an agent document says", () => {
  it("tells one document going from everyone to this computer", () => {
    expect(movingSaid(moving({}))).toBe(
      [
        "Move «Title» to this computer's folder?",
        "",
        "On this computer, it keeps being written to AGENTS.md.",
        "Your other computers keep their file as it is, as a document of their own.",
      ].join("\n"),
    );
  });

  it("counts a folder, prefixes each told line and skips the untouched", () => {
    const said = movingSaid(
      moving({
        docs: [doc("A", "writes"), doc("B", "untouched"), doc("C", "asks"), doc("D", "stops")],
      }),
    ).split("\n");
    expect(said[0]).toBe("Move 4 agent documents to this computer's folder?");
    expect(said[1]).toBe("");
    expect(said[2]).toBe("«A»: On this computer, Tisty writes it to a.md, with a backup first.");
    expect(said[3]).toBe(
      "«C»: On this computer, another document also writes c.md: Tisty asks which one rules here.",
    );
    expect(said[4]).toBe(
      "«D»: On this computer, Tisty stops writing d.md and asks what to do with the file, with a backup.",
    );
    expect(said.some((line) => line.includes("«B»"))).toBe(false);
  });

  it("tells the unapproved one by its file", () => {
    expect(movingSaid(moving({ docs: [doc("U", "unapproved", "u.md")] }))).toContain(
      "Its words are not approved yet: no computer writes u.md until you approve them.",
    );
  });

  it("tells the one no agent reads yet, and says Tisty writes nothing", () => {
    const said = movingSaid(moving({ docs: [doc("R", "unread", "r.md")] }));
    expect(said).toContain(
      "On this computer, no agent reads it yet, so Tisty writes nothing here.",
    );
  });

  it("tells the unread one in Spanish", () => {
    adopt("es");
    expect(movingSaid(moving({ docs: [doc("R", "unread", "r.md")] }))).toContain(
      "En este equipo, todavía ningún agente lo lee, así que Tisty no escribe nada aquí.",
    );
  });

  it("ends with how many more when over six are told", () => {
    const docs = Array.from({ length: 9 }, (_, i) => doc(`D${i}`, "writes"));
    const said = movingSaid(moving({ docs, to: { is: "everyone" } })).split("\n");
    const told = said.filter((line) => line.startsWith("«D"));
    expect(told).toHaveLength(6);
    expect(said).toContain("And 3 more.");
    expect(said.indexOf("And 3 more.")).toBe(said.indexOf(told[5] as string) + 1);
  });

  it("does not add more when exactly six are told", () => {
    const docs = Array.from({ length: 6 }, (_, i) => doc(`D${i}`, "writes"));
    expect(movingSaid(moving({ docs }))).not.toContain("more.");
  });

  it("says a document going to another computer waits there", () => {
    const said = movingSaid(moving({ to: { is: "other", name: "Work" } }));
    expect(said.split("\n")[0]).toBe("Move «Title» to Work's folder?");
    expect(said).toContain("On Work, it waits for Apply here.");
  });

  it("says another computer stops writing what comes to this one", () => {
    const said = movingSaid(moving({ from: { is: "other", name: "Work" } }));
    expect(said).toContain("Work keeps its file as it is, as a document of its own.");
  });

  it("says the other computers wait when it goes to everyone", () => {
    const said = movingSaid(moving({ from: { is: "mine" }, to: { is: "everyone" } }));
    expect(said).toContain("Move «Title» to the root, for all your computers?");
    expect(said).toContain("On your other computers, it waits for Apply here.");
  });

  it("adds no line about other computers when it stays on this one", () => {
    expect(movingSaid(moving({ from: { is: "mine" } })).split("\n")).toHaveLength(3);
  });

  it("adds the older computers joined, and nothing when there are none", () => {
    const older = movingSaid(moving({ older: ["Work", "Laptop"] })).split("\n");
    expect(older[older.length - 1]).toBe(
      "On Work, Laptop, an earlier Tisty reads it as a document for all your computers until it updates.",
    );
    expect(movingSaid(moving({ older: [] }))).not.toContain("earlier Tisty");
  });

  it("speaks Spanish when the language is Spanish", () => {
    adopt("es");
    expect(movingSaid(moving({ to: { is: "other", name: "Work" }, older: ["Work"] }))).toBe(
      [
        "¿Mover «Title» a la carpeta de Work?",
        "",
        "En este equipo, se sigue escribiendo en AGENTS.md.",
        "En Work, espera «Aplicar aquí».",
        "En Work, un Tisty anterior lo lee como un documento para todos tus equipos hasta que se actualice.",
      ].join("\n"),
    );
  });
});

describe("asking the core and the person", () => {
  it("asks the core with the kind, the id and the place", async () => {
    await agentsMoving("folder", "f1", "f2");
    expect(calls).toEqual([{ cmd: "agents_moving", args: { kind: "folder", id: "f1", to: "f2" } }]);
  });

  it("allows without asking when the core has nothing to say", async () => {
    const asked = answering(false);
    expect(await movingAllowed("doc", "d1", undefined)).toBe(true);
    expect(asked).toEqual([]);
  });

  it("asks with the preview and gives back the answer", async () => {
    core.answer = moving({});
    const asked = answering(true);
    expect(await movingAllowed("doc", "d1", "f1")).toBe(true);
    expect(asked).toEqual([
      { said: movingSaid(moving({})), buttons: { yes: "Move", no: "Cancel" } },
    ]);
    answering(false);
    expect(await movingAllowed("doc", "d1", "f1")).toBe(false);
  });

  it("moves after a yes and when nothing needs asking, never after a no", async () => {
    const move = vi.fn(() => Promise.resolve());
    core.answer = moving({});
    answering(true);
    await movedOnceSure("doc", "d1", "f1", move);
    expect(move).toHaveBeenCalledTimes(1);

    answering(false);
    await movedOnceSure("doc", "d1", "f1", move);
    expect(move).toHaveBeenCalledTimes(1);

    core.answer = null;
    await movedOnceSure("doc", "d1", "f1", move);
    expect(move).toHaveBeenCalledTimes(2);
  });
});
