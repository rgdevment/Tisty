import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../App";
import type { Papers } from "../core";
import { t } from "../locales";
import Docs from "../ui/Docs";
import Tree from "../ui/Tree";

interface FakeFolder {
  id: string;
  name: string;
  parent: string | null;
  icon: string | null;
}

interface FakeDoc {
  id: string;
  file: string;
  title: string;
  folder: string | null;
  archived: boolean;
}

const store = vi.hoisted(() => ({
  folders: [] as FakeFolder[],
  docs: [] as FakeDoc[],
  bodies: {} as Record<string, string>,
  writes: [] as { id: string; body: string }[],
  copied: [] as string[],
  seq: 0,
}));

const picked = vi.hoisted(() => ({ path: Promise.resolve(null as string | null) }));

const carrier = vi.hoisted(() => ({ made: 0, asked: 0 }));

const ipc = vi.hoisted(() => ({
  answer: (_cmd: string, _args: Record<string, unknown>): Promise<unknown> => Promise.resolve(null),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => ipc.answer(cmd, args ?? {}),
}));

vi.mock("../carrying", () => ({
  carrying: () => {
    carrier.made += 1;
    return {
      changed: () => {
        carrier.asked += 1;
      },
      recheck: () => {},
      stop: () => {},
    };
  },
}));

vi.mock("@tauri-apps/api/webview", () => ({
  getCurrentWebview: () => ({
    onDragDropEvent: () => Promise.resolve(() => {}),
  }),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    minimize: () => Promise.resolve(),
    toggleMaximize: () => Promise.resolve(),
    close: () => Promise.resolve(),
  }),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  ask: () => Promise.resolve(true),
  open: () => picked.path,
}));

vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
  writeText: (text: string) => {
    store.copied.push(text);
    return Promise.resolve();
  },
}));

vi.mock("../ui/Editor", () => ({
  default: ({
    value,
    label,
    onWrite,
  }: {
    value: string;
    label: string;
    onWrite: (text: string) => void;
  }) => (
    <textarea
      aria-label={label}
      data-testid="editor"
      value={value}
      onChange={(e) => onWrite(e.target.value)}
    />
  ),
}));

const mkId = (prefix: string) => `${prefix}${++store.seq}`;

function countIn(folder: string): number {
  return store.docs.filter((doc) => doc.folder === folder && !doc.archived).length;
}

function papersOut(): Papers {
  return {
    folders: store.folders.map((folder) => ({ ...folder, holds: countIn(folder.id) })),
    docs: store.docs.map((doc) => ({ ...doc })),
  };
}

function backend(cmd: string, args: Record<string, unknown>): Promise<unknown> {
  switch (cmd) {
    case "settle_in":
      return Promise.resolve({ ran: false, brought: false, agrees: true });
    case "sync_state":
      return Promise.resolve({ asked: true, backsUp: true, loose: 0 });
    case "snapshot":
      return Promise.resolve({
        tasks: [],
        lists: [],
        tags: [],
        refs: [],
        counts: {},
        locale: "en",
      });
    case "icons":
      return Promise.resolve([]);
    case "docs":
      return Promise.resolve(papersOut());
    case "doc_new": {
      const id = mkId("doc");
      const folder = (args.folder as string | undefined) ?? null;
      store.docs.push({ id, file: id, title: "", folder, archived: false });
      store.bodies[id] = "";
      return Promise.resolve({ id, title: "" });
    }
    case "doc_read":
      return Promise.resolve(store.bodies[String(args.id)] ?? "");
    case "doc_write": {
      const id = String(args.id);
      const body = String(args.body);
      store.writes.push({ id, body });
      store.bodies[id] = body;
      const title = body
        .split("\n")[0]
        .replace(/^#+\s*/, "")
        .trim();
      const doc = store.docs.find((one) => one.file === id);
      if (doc) doc.title = title;
      return Promise.resolve({ id, title });
    }
    case "doc_away": {
      const doc = store.docs.find((one) => one.id === args.id);
      if (doc) doc.archived = Boolean(args.away);
      return Promise.resolve(null);
    }
    case "doc_drop": {
      store.docs = store.docs.filter((one) => one.id !== args.id);
      return Promise.resolve(null);
    }
    case "doc_copy": {
      const from = store.docs.find((one) => one.id === args.id);
      if (!from) return Promise.resolve(null);
      const id = mkId("doc");
      const title = from.title ? `${from.title} copy` : "";
      store.docs.push({ id, file: id, title, folder: from.folder, archived: false });
      store.bodies[id] = store.bodies[from.file] ?? "";
      return Promise.resolve({ id, title });
    }
    case "doc_file": {
      const doc = store.docs.find((one) => one.id === args.id);
      if (doc) doc.folder = (args.folder as string | undefined) ?? null;
      return Promise.resolve(null);
    }
    case "doc_export":
      return Promise.resolve(0);
    case "folder_rename": {
      const folder = store.folders.find((one) => one.id === args.id);
      if (folder) folder.name = String(args.name);
      return Promise.resolve(null);
    }
    case "folder_file": {
      const folder = store.folders.find((one) => one.id === args.id);
      if (folder) folder.parent = (args.parent as string | undefined) ?? null;
      return Promise.resolve(null);
    }
    case "folder_drop": {
      store.folders = store.folders.filter((one) => one.id !== args.id);
      store.docs = store.docs.map((one) =>
        one.folder === args.id ? { ...one, folder: null } : one,
      );
      return Promise.resolve(null);
    }
    case "folder_add": {
      const id = mkId("folder");
      store.folders.push({
        id,
        name: String(args.name),
        parent: (args.parent as string | undefined) ?? null,
        icon: (args.icon as string | undefined) ?? null,
      });
      return Promise.resolve(null);
    }
    default:
      return Promise.resolve(null);
  }
}

beforeEach(() => {
  store.folders = [];
  store.docs = [];
  store.bodies = {};
  store.writes = [];
  store.copied = [];
  store.seq = 0;
  picked.path = Promise.resolve(null);
  carrier.made = 0;
  carrier.asked = 0;
  ipc.answer = backend;
});

function seedDoc(over: Partial<FakeDoc> = {}): FakeDoc {
  const id = mkId("doc");
  const doc: FakeDoc = { id, file: id, title: "", folder: null, archived: false, ...over };
  store.docs.push(doc);
  store.bodies[doc.file] = doc.title ? `# ${doc.title}` : "";
  return doc;
}

function seedFolder(over: Partial<FakeFolder> = {}): FakeFolder {
  const id = mkId("folder");
  const folder: FakeFolder = { id, name: "Folder", parent: null, icon: null, ...over };
  store.folders.push(folder);
  return folder;
}

async function boot() {
  render(<App />);
  await screen.findByRole("button", { name: t("unfiled") });
}

function menuFor(rowLabel: string): HTMLElement {
  return screen.getByRole("button", { name: rowLabel }).parentElement as HTMLElement;
}

async function chooseFor(rowLabel: string, itemLabel: string) {
  fireEvent.contextMenu(menuFor(rowLabel), { clientX: 5, clientY: 5 });
  await userEvent.click(await screen.findByRole("menuitem", { name: itemLabel }));
}

async function moveTo(rowLabel: string, destination: string) {
  fireEvent.contextMenu(menuFor(rowLabel), { clientX: 5, clientY: 5 });
  await userEvent.click(await screen.findByRole("menuitem", { name: t("moveTo") }));
  await userEvent.click(await screen.findByRole("menuitem", { name: destination }));
}

function countBadge(rowLabel: string): string {
  const button = screen.getByRole("button", { name: rowLabel });
  const spans = button.querySelectorAll("span");
  return spans[spans.length - 1]?.textContent ?? "";
}

describe("creating a document", () => {
  it("opens the document it just made, without being asked to pick it", async () => {
    await boot();

    await userEvent.click(screen.getByRole("button", { name: t("docsActions") }));
    await userEvent.click(await screen.findByRole("menuitem", { name: t("newDoc") }));

    expect(await screen.findByTestId("editor")).toBeTruthy();
  });

  it("shows a document with nothing written in it yet as its untitled self", async () => {
    await boot();

    await userEvent.click(screen.getByRole("button", { name: t("docsActions") }));
    await userEvent.click(await screen.findByRole("menuitem", { name: t("newDoc") }));

    const editor = await screen.findByTestId("editor");
    expect(editor.getAttribute("aria-label")).toBe(t("untitledDoc"));
    expect((editor as HTMLTextAreaElement).value).toBe("");
  });

  it("creates a document inside the folder it was asked from, and leaves it there", async () => {
    seedFolder({ name: "Personal" });
    await boot();

    await chooseFor("Personal", t("newDoc"));
    await screen.findByTestId("editor");

    const branch = screen.getByRole("button", { name: "Personal" }).closest("li") as HTMLElement;
    expect(within(branch).getByRole("button", { name: t("untitledDoc") })).toBeTruthy();
  });
});

describe("archiving and bringing back a document", () => {
  it("takes an archived document out of its folder, into its own place, and off the folder's count", async () => {
    const folder = seedFolder({ name: "Work" });
    seedDoc({ title: "Report", folder: folder.id });
    await boot();
    expect(countBadge("Work")).toBe("1");

    await chooseFor("Report", t("putAway"));

    await waitFor(() =>
      expect(
        within(screen.getByRole("list", { name: t("docs") })).queryByRole("button", {
          name: "Report",
        }),
      ).toBeNull(),
    );
    expect(
      within(screen.getByRole("list", { name: t("archived") })).getByRole("button", {
        name: "Report",
      }),
    ).toBeTruthy();
    expect(countBadge("Work")).toBe("");
  });

  it("returns an unarchived document to the folder it was filed in", async () => {
    const folder = seedFolder({ name: "Work" });
    seedDoc({ title: "Report", folder: folder.id, archived: true });
    await boot();
    expect(countBadge("Work")).toBe("");

    await chooseFor("Report", t("bringBack"));

    await waitFor(() => expect(countBadge("Work")).toBe("1"));
    expect(
      within(screen.getByRole("list", { name: t("docs") })).getByRole("button", { name: "Report" }),
    ).toBeTruthy();
    expect(screen.queryByRole("list", { name: t("archived") })).toBeNull();
  });
});

describe("deleting a document", () => {
  it("closes the editor when the document open in it gets deleted", async () => {
    seedDoc({ title: "Notes" });
    await boot();
    await userEvent.click(screen.getByRole("button", { name: "Notes" }));
    await screen.findByTestId("editor");

    await chooseFor("Notes", t("deleteIt"));

    await waitFor(() => expect(screen.queryByTestId("editor")).toBeNull());
    expect(screen.queryByRole("button", { name: "Notes" })).toBeNull();
  });

  it("leaves the editor alone when a document deleted elsewhere is not the one open", async () => {
    seedDoc({ title: "Alpha" });
    seedDoc({ title: "Beta" });
    await boot();
    await userEvent.click(screen.getByRole("button", { name: "Alpha" }));
    const editor = await screen.findByTestId("editor");
    expect(editor.getAttribute("aria-label")).toBe("Alpha");

    await chooseFor("Beta", t("deleteIt"));

    await waitFor(() => expect(screen.queryByRole("button", { name: "Beta" })).toBeNull());
    expect(screen.getByTestId("editor").getAttribute("aria-label")).toBe("Alpha");
  });

  it("discards a pending edit when its document is deleted, and it never gets written", async () => {
    const doc = seedDoc({ title: "Draft" });
    await boot();
    await userEvent.click(screen.getByRole("button", { name: "Draft" }));
    const editor = await screen.findByTestId("editor");
    fireEvent.change(editor, { target: { value: "half finished thought" } });

    await chooseFor("Draft", t("deleteIt"));

    await waitFor(() => expect(screen.queryByTestId("editor")).toBeNull());
    await new Promise((go) => setTimeout(go, 900));
    expect(store.writes.filter((one) => one.id === doc.file)).toHaveLength(0);
  });
});

describe("moving a document", () => {
  it("moves a document into another folder, and the tree shows it there", async () => {
    seedFolder({ name: "Personal" });
    seedDoc({ title: "Todo" });
    await boot();
    expect(countBadge("Personal")).toBe("");

    await moveTo("Todo", "Personal");

    await waitFor(() => expect(countBadge("Personal")).toBe("1"));
    const branch = screen.getByRole("button", { name: "Personal" }).closest("li") as HTMLElement;
    expect(within(branch).getByRole("button", { name: "Todo" })).toBeTruthy();
  });

  it("moves a document out to unclassified, and the tree no longer files it under its folder", async () => {
    const folder = seedFolder({ name: "Personal" });
    seedDoc({ title: "Todo", folder: folder.id });
    await boot();
    expect(countBadge("Personal")).toBe("1");

    await moveTo("Todo", t("unfiled"));

    await waitFor(() => expect(countBadge("Personal")).toBe(""));
    const branch = screen.getByRole("button", { name: "Personal" }).closest("li") as HTMLElement;
    expect(within(branch).queryByRole("button", { name: "Todo" })).toBeNull();
    expect(screen.getByRole("button", { name: "Todo" })).toBeTruthy();
  });
});

describe("duplicating a document", () => {
  it("adds a copy of a document and leaves the original in place", async () => {
    seedDoc({ title: "Recipe" });
    await boot();

    await chooseFor("Recipe", t("duplicate"));

    await screen.findByRole("button", { name: "Recipe copy" });
    expect(screen.getByRole("button", { name: "Recipe" })).toBeTruthy();
  });
});

describe("nothing filed yet", () => {
  it("has nowhere to file anything yet, and shows only the unfiled shelf", () => {
    render(<Tree papers={{ folders: [], docs: [] }} onOpen={vi.fn()} onFile={vi.fn()} />);

    expect(screen.getByRole("button", { name: t("unfiled") })).toBeTruthy();
    expect(screen.getAllByRole("button")).toHaveLength(2);
    expect(screen.getByText(t("noDocsYet"))).toBeTruthy();
  });

  it("says there is nothing yet, rather than showing an empty shelf and no words", () => {
    render(<Tree papers={{ folders: [], docs: [] }} onOpen={vi.fn()} onFile={vi.fn()} />);

    expect(screen.getByText(t("noDocsYet"))).toBeTruthy();
  });

  it("shows only what was archived, when that is all there ever was", () => {
    const papers: Papers = {
      folders: [],
      docs: [{ id: "01A", file: "f1", title: "Old", folder: null, archived: true }],
    };
    render(<Tree papers={papers} onOpen={vi.fn()} onFile={vi.fn()} />);

    expect(
      within(screen.getByRole("list", { name: t("archived") })).getByRole("button", {
        name: "Old",
      }),
    ).toBeTruthy();
    expect(
      within(screen.getByRole("list", { name: t("docs") })).queryByRole("button", { name: "Old" }),
    ).toBeNull();
  });

  it("has nothing to open when nothing has been filed, and says so instead of guessing", async () => {
    render(<Docs known={[]} onKept={vi.fn()} onError={vi.fn()} />);

    expect(await screen.findByText(t("pickADoc"))).toBeTruthy();
    expect(screen.queryByTestId("editor")).toBeNull();
  });
});

describe("what a folder's own menu can do", () => {
  it("makes a document inside the folder it was asked from", async () => {
    const work = seedFolder({ name: "Trabajo" });
    await boot();

    await chooseFor("Trabajo", t("newDoc"));

    await waitFor(() => expect(countIn(work.id)).toBe(1));
  });

  it("offers a folder inside a folder until the deepest level, then stops", async () => {
    seedFolder({ name: "Uno" });
    const two = seedFolder({ name: "Dos", parent: "folder1" });
    const three = seedFolder({ name: "Tres", parent: two.id });
    seedFolder({ name: "Cuatro", parent: three.id });
    await boot();

    fireEvent.contextMenu(menuFor("Tres"), { clientX: 5, clientY: 5 });
    expect(await screen.findByRole("menuitem", { name: t("newFolder") })).toBeTruthy();
    await userEvent.keyboard("{Escape}");

    fireEvent.contextMenu(menuFor("Cuatro"), { clientX: 5, clientY: 5 });
    await screen.findByRole("menuitem", { name: t("newDoc") });
    expect(screen.queryByRole("menuitem", { name: t("newFolder") })).toBeNull();
  });

  it("renames the folder from the sheet the menu opens", async () => {
    seedFolder({ name: "Trabajo" });
    await boot();

    await chooseFor("Trabajo", t("rename"));
    const box = await screen.findByLabelText(t("folderName"));
    fireEvent.change(box, { target: { value: "Oficina" } });
    await userEvent.click(screen.getByRole("button", { name: t("renameIt") }));

    await waitFor(() => expect(store.folders[0].name).toBe("Oficina"));
  });

  it("files a folder inside another one, spelling out the whole way there", async () => {
    seedFolder({ name: "Uno" });
    const two = seedFolder({ name: "Dos", parent: "folder1" });
    seedFolder({ name: "Suelta" });
    await boot();

    await moveTo("Suelta", `Uno / Dos`);

    await waitFor(() =>
      expect(store.folders.find((one) => one.name === "Suelta")?.parent).toBe(two.id),
    );
  });

  it("deletes the folder and leaves what was inside it unfiled", async () => {
    const work = seedFolder({ name: "Trabajo" });
    seedDoc({ title: "Dentro", folder: work.id });
    await boot();

    await chooseFor("Trabajo", t("deleteIt"));

    await waitFor(() => expect(store.folders).toHaveLength(0));
    expect(store.docs[0].folder).toBeNull();
  });
});

describe("standing inside a folder", () => {
  it("opens the folder in the pane instead of the document that was there", async () => {
    const work = seedFolder({ name: "Trabajo" });
    seedDoc({ title: "Acta", folder: work.id });
    await boot();

    await userEvent.click(screen.getByRole("button", { name: "Acta" }));
    await screen.findByTestId("editor");

    await userEvent.click(screen.getByRole("button", { name: "Trabajo" }));

    expect(screen.queryByTestId("editor")).toBeNull();
    expect(await screen.findByRole("heading", { name: /Trabajo/ })).toBeTruthy();
  });

  it("shows the loose papers when you stand in unfiled", async () => {
    seedDoc({ title: "Suelto" });
    await boot();

    await userEvent.click(screen.getByRole("button", { name: t("unfiled") }));

    expect(await screen.findByRole("heading", { name: new RegExp(t("unfiled")) })).toBeTruthy();
  });
});

describe("what the menus reach for outside the tree", () => {
  it("copies a document as plain Markdown and says it did", async () => {
    seedDoc({ title: "Acta" });
    await boot();

    await chooseFor("Acta", t("copyPlain"));

    await waitFor(() => expect(store.copied).toHaveLength(1));
    expect(await screen.findByText(t("copied"))).toBeTruthy();
  });

  it("asks for a file to bring in, from the sidebar and from a folder alike", async () => {
    seedFolder({ name: "Trabajo" });
    await boot();

    await userEvent.click(screen.getByRole("button", { name: t("docsActions") }));
    await userEvent.click(await screen.findByRole("menuitem", { name: t("importDoc") }));
    await waitFor(() => expect(store.docs).toHaveLength(0));

    await chooseFor("Trabajo", t("importDoc"));
    await waitFor(() => expect(store.docs).toHaveLength(0));
  });

  it("makes a folder from the sidebar menu without standing anywhere first", async () => {
    await boot();

    await userEvent.click(screen.getByRole("button", { name: t("docsActions") }));
    await userEvent.click(await screen.findByRole("menuitem", { name: t("newFolder") }));

    const box = await screen.findByLabelText(t("folderName"));
    fireEvent.change(box, { target: { value: "Casa" } });
    await userEvent.click(screen.getByRole("button", { name: t("create") }));

    await waitFor(() => expect(store.folders.map((one) => one.name)).toEqual(["Casa"]));
  });

  it("writes a document out to the folder it was pointed at, and counts what went", async () => {
    seedDoc({ title: "Acta" });
    picked.path = Promise.resolve("D:/salida");
    await boot();

    await chooseFor("Acta", t("takeOut"));

    await waitFor(() => expect(screen.getByText(t("takenOutAlone"))).toBeTruthy());
  });

  it("says nothing at all when the export was called off", async () => {
    seedDoc({ title: "Acta" });
    await boot();

    await chooseFor("Acta", t("takeOut"));

    await waitFor(() => expect(screen.queryByText(t("takenOutAlone"))).toBeNull());
  });

  it("makes a folder inside the one the menu was opened on", async () => {
    const work = seedFolder({ name: "Trabajo" });
    await boot();

    await chooseFor("Trabajo", t("newFolder"));
    const box = await screen.findByLabelText(t("folderName"));
    fireEvent.change(box, { target: { value: "Legal" } });
    await userEvent.click(screen.getByRole("button", { name: t("create") }));

    await waitFor(() =>
      expect(store.folders.find((one) => one.name === "Legal")?.parent).toBe(work.id),
    );
  });
});

describe("what asks the other machine to be told", () => {
  it("asks after a document is made, moved, archived or written", async () => {
    const folder = seedFolder({ name: "Trabajo" });
    seedDoc({ title: "Acta", folder: null });
    await boot();

    await userEvent.click(screen.getByRole("button", { name: t("docsActions") }));
    await userEvent.click(await screen.findByRole("menuitem", { name: t("newDoc") }));
    await waitFor(() => expect(carrier.asked).toBeGreaterThan(0));

    const afterMaking = carrier.asked;
    await moveTo("Acta", "Trabajo");
    await waitFor(() => expect(carrier.asked).toBeGreaterThan(afterMaking));
    expect(store.docs.find((one) => one.title === "Acta")?.folder).toBe(folder.id);

    const afterMoving = carrier.asked;
    await chooseFor("Acta", t("archive"));
    await waitFor(() => expect(carrier.asked).toBeGreaterThan(afterMoving));
  });

  it("does not ask merely for opening the window", async () => {
    seedDoc({ title: "Acta" });
    await boot();

    expect(carrier.made).toBeGreaterThan(0);
    expect(carrier.asked).toBe(0);
  });
});
