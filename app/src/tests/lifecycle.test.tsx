import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Papers } from "../core";
import { t } from "../locales";
import App from "../App";
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
  seq: 0,
}));

const ipc = vi.hoisted(() => ({
  answer: (_cmd: string, _args: Record<string, unknown>): Promise<unknown> =>
    Promise.resolve(null),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => ipc.answer(cmd, args ?? {}),
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
  open: () => Promise.resolve(null),
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
      return Promise.resolve({ tasks: [], lists: [], tags: [], refs: [], counts: {}, locale: "en" });
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
      const title = body.split("\n")[0].replace(/^#+\s*/, "").trim();
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
  store.seq = 0;
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
  await screen.findByRole("button", { name: new RegExp(t("unfiled")) });
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
      expect(within(screen.getByRole("list", { name: t("docs") })).queryByRole("button", { name: "Report" })).toBeNull(),
    );
    expect(
      within(screen.getByRole("list", { name: t("archived") })).getByRole("button", { name: "Report" }),
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

    expect(screen.getByRole("button", { name: new RegExp(t("unfiled")) })).toBeTruthy();
    expect(screen.getAllByRole("button")).toHaveLength(1);
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
      within(screen.getByRole("list", { name: t("archived") })).getByRole("button", { name: "Old" }),
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
