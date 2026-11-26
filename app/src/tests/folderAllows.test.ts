import type { Asked } from "tisty-editor/edit";
import { describe, expect, it } from "vitest";
import { allowedKinds, folderAllows, type Lodged } from "../folderAllows";
import { insertChoices } from "../ui/InsertMenu";

const ALL: Asked[] = [
  "newPage",
  "page",
  "newDoc",
  "doc",
  "newTask",
  "task",
  "icon",
  "file",
  "diagram",
  "diagramFrom",
];

const plain: Lodged = { agents: false, diary: false, agentFile: false };
const kinds = (lodged: Partial<Lodged>) => allowedKinds(ALL, folderAllows({ ...plain, ...lodged }));

describe("what a folder allows", () => {
  it("lets a plain folder keep everything", () => {
    expect(folderAllows(plain)).toEqual({ newDocs: true, cards: true });
    expect(kinds({})).toEqual(ALL);
  });

  it("drops new pages and documents in Agents and in the diary, keeping the rest", () => {
    for (const where of [{ agents: true }, { diary: true }]) {
      const kept = kinds(where);

      expect(kept).not.toContain("newPage");
      expect(kept).not.toContain("newDoc");
      expect(kept).toEqual(ALL.filter((one) => one !== "newPage" && one !== "newDoc"));
    }
  });

  it("drops every card in an agent file, keeping what is no card", () => {
    expect(kinds({ agentFile: true })).toEqual(["icon", "file"]);
  });

  it("combines both rules", () => {
    expect(kinds({ agents: true, agentFile: true })).toEqual(["icon", "file"]);
  });
});

describe("the Insert menu in a folder", () => {
  const keysIn = (lodged: Partial<Lodged>) => {
    const { choices } = insertChoices({
      kinds: kinds(lodged),
      steps: [],
      sections: false,
      pick: () => {},
    });
    return {
      top: choices.map((one) => one.key),
      pageOrDoc: choices
        .find((one) => one.key === "pageOrDoc")
        ?.into?.choices.map((one) => one.key),
    };
  };

  it("offers no new page or document inside Agents", () => {
    const menu = keysIn({ agents: true });

    expect(menu.top).not.toContain("newpage");
    expect(menu.pageOrDoc).toEqual(["page", "doc"]);
    expect(menu.top).toContain("task");
  });

  it("offers no card at all in an agent file", () => {
    const menu = keysIn({ agents: true, agentFile: true });

    for (const key of ["newpage", "pageOrDoc", "task"]) expect(menu.top).not.toContain(key);
    expect(menu.top).toEqual(expect.arrayContaining(["file", "icon", "table", "code"]));
  });

  it("offers everything in a plain folder", () => {
    expect(keysIn({}).top).toEqual(
      expect.arrayContaining(["newpage", "pageOrDoc", "task", "diagram"]),
    );
  });
});
