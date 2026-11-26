import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Copy } from "../agentHistory";
import type { Earlier as Version } from "../earlier";
import { fill, t } from "../locales";
import { cut } from "../saving";
import { asksSure } from "../sure";
import { AGENTS_CHANGED } from "../ui/AgentsKeeper";
import Earlier from "../ui/Earlier";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  versions: [] as unknown[],
  copies: [] as unknown[],
  texts: {} as Record<number, string>,
  restoreFails: false,
  historyFails: false,
  heldFails: false,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args: Record<string, unknown> = {}) => {
    ipc.calls.push({ cmd, args });
    switch (cmd) {
      case "agent_history":
        if (ipc.historyFails) return Promise.reject({ code: "nope" });
        return Promise.resolve({
          versions: ipc.versions,
          kept: ipc.versions.length,
          copies: ipc.copies,
        });
      case "doc_versions":
        return Promise.resolve({ versions: ipc.versions, kept: ipc.versions.length });
      case "doc_version":
        return Promise.resolve(`# Rules\n\nversion ${String(args.print)[0]}`);
      case "doc_compared":
        return Promise.resolve({ kind: "drawn", html: "<p>diff</p>", gone: 1, came: 1 });
      case "doc_history_empty":
        return Promise.resolve(1);
      case "agent_doc": {
        const asked = args.asked as { do: string; backup: number };
        if (asked.do === "held" && ipc.heldFails) return Promise.reject({ code: "gone" });
        if (asked.do === "restore" && ipc.restoreFails) return Promise.reject({ code: "nope" });
        return Promise.resolve(asked.do === "held" ? (ipc.texts[asked.backup] ?? "") : null);
      }
      default:
        return Promise.resolve(null);
    }
  },
}));

vi.mock("@tauri-apps/api/event", () => ({ listen: () => Promise.resolve(() => {}) }));

vi.mock("../ui/Page", () => ({
  default: ({ body }: { body: string }) => <article aria-label="drawn by the core">{body}</article>,
}));

const A = "a".repeat(64);
const B = "b".repeat(64);
const AT = "/home/me/.claude/CLAUDE.md";

const older: Version = {
  print: A,
  wrote: "2026-11-10T10:40:00Z",
  replaced: "2026-11-11T09:00:00Z",
  by: "mac-1",
  bytes: 3200,
  here: true,
};
const newer: Version = {
  print: B,
  wrote: "2026-11-12T08:15:00Z",
  replaced: "2026-11-12T18:00:00Z",
  by: "mac-1",
  bytes: 4100,
  here: true,
};

const copy = (n: number, when: string | undefined, more: Partial<Copy> = {}): Copy => ({
  at: AT,
  named: "Claude",
  n,
  bytes: 2048,
  pinned: false,
  when,
  ...more,
});

const called = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);
const list = () => screen.getByRole("list", { name: t("earlierList") });
const restored = () =>
  called("agent_doc").filter((one) => (one.args.asked as { do: string }).do === "restore");

const view = (agentFile: { restores: boolean } | null = { restores: true }) => {
  const hands = { onClose: vi.fn(), onError: vi.fn(), onBack: vi.fn(() => Promise.resolve()) };
  render(<Earlier doc="d-1" title="Rules" agentFile={agentFile ?? undefined} {...hands} />);
  return hands;
};

const said = (one: Copy) => fill(one.pinned ? "agentCopyOriginal" : "agentCopyOf", one.named);

beforeEach(() => {
  cut("d-1");
  ipc.calls = [];
  ipc.versions = [];
  ipc.copies = [];
  ipc.restoreFails = false;
  ipc.historyFails = false;
  ipc.heldFails = false;
  ipc.texts = { 1: "The very first words.\n", 2: "Words before the last write.\n" };
  asksSure(null);
});

describe("the history of an agent document's file", () => {
  it("asks for the versions and the copies together, never for the versions alone", async () => {
    ipc.copies = [copy(2, "2026-11-13T00:00:00Z")];
    view();
    await within(list()).findAllByRole("listitem");
    expect(called("agent_history")[0].args).toEqual({ file: "d-1" });
    expect(called("doc_versions")).toHaveLength(0);
  });

  it("lists the copies among the versions with the agent's name, date and size", async () => {
    ipc.versions = [newer, older];
    ipc.copies = [copy(2, "2026-11-11T12:00:00Z"), copy(1, undefined, { pinned: true })];
    view();
    const rows = await within(list()).findAllByRole("listitem");
    expect(rows).toHaveLength(4);
    expect(rows[1].textContent).toContain(said(copy(2, undefined)));
    expect(rows[1].textContent).toContain("2");
    expect(rows[3].textContent).toContain(said(copy(1, undefined, { pinned: true })));
    expect(rows[3].textContent).toContain(t("agentCopyStays"));
    expect(rows[3].textContent).toContain(fill("agentCopyNumber", "1"));
    expect(screen.getByText(t("agentHistoryCopies"))).toBeTruthy();
  });

  it("opens the only copy of a document with no version and shows its text, not the empty message", async () => {
    ipc.copies = [copy(1, "2026-11-13T00:00:00Z", { pinned: true })];
    view();
    expect((await screen.findByLabelText("drawn by the core")).textContent).toContain(
      "The very first words.",
    );
    expect(called("agent_doc")[0].args).toEqual({
      file: "d-1",
      at: AT,
      asked: { do: "held", backup: 1 },
    });
    expect(screen.queryByText(t("agentHistoryNone"))).toBeNull();
    expect(screen.queryByText(t("earlierNone"))).toBeNull();
  });

  it("reads the copy chosen, not the first one, when the person picks another", async () => {
    ipc.copies = [copy(2, "2026-11-13T00:00:00Z"), copy(1, undefined, { pinned: true })];
    view();
    await screen.findByText(/Words before the last write/);
    const rows = within(list()).getAllByRole("listitem");
    await userEvent.click(within(rows[1]).getByRole("button"));
    expect((await screen.findByLabelText("drawn by the core")).textContent).toContain(
      "The very first words.",
    );
  });

  it("asks, then puts the copy back in the file and reads the history again", async () => {
    ipc.copies = [copy(2, "2026-11-13T00:00:00Z")];
    const asked: string[] = [];
    asksSure((words: string) => {
      asked.push(words);
      return Promise.resolve(true);
    });
    const changed = vi.fn();
    window.addEventListener(AGENTS_CHANGED, changed);
    view();
    await screen.findByText(/Words before the last write/);
    await userEvent.click(screen.getByRole("button", { name: t("agentCopyBack") }));
    await waitFor(() => expect(restored()).toHaveLength(1));
    expect(restored()[0].args).toEqual({
      file: "d-1",
      at: AT,
      asked: { do: "restore", backup: 2 },
    });
    expect(asked).toEqual([fill("agentRestoreSure", AT)]);
    await waitFor(() => expect(called("agent_history")).toHaveLength(2));
    expect(changed).toHaveBeenCalledTimes(1);
    window.removeEventListener(AGENTS_CHANGED, changed);
  });

  it("puts nothing back when the person says no", async () => {
    ipc.copies = [copy(2, "2026-11-13T00:00:00Z")];
    asksSure(() => Promise.resolve(false));
    view();
    await screen.findByText(/Words before the last write/);
    await userEvent.click(screen.getByRole("button", { name: t("agentCopyBack") }));
    await waitFor(() =>
      expect(screen.getByRole("button", { name: t("agentCopyBack") })).toBeTruthy(),
    );
    expect(restored()).toHaveLength(0);
    expect(called("agent_history")).toHaveLength(1);
  });

  it("tells the problem and does not read the history again when putting the copy back fails", async () => {
    ipc.copies = [copy(2, "2026-11-13T00:00:00Z")];
    ipc.restoreFails = true;
    asksSure(() => Promise.resolve(true));
    const hands = view();
    await screen.findByText(/Words before the last write/);
    await userEvent.click(screen.getByRole("button", { name: t("agentCopyBack") }));
    await waitFor(() => expect(hands.onError).toHaveBeenCalledWith({ code: "nope" }));
    expect(called("agent_history")).toHaveLength(1);
  });

  it("tells the problem and lists nothing when the history cannot be read", async () => {
    ipc.historyFails = true;
    const hands = view();
    await waitFor(() => expect(hands.onError).toHaveBeenCalledWith({ code: "nope" }));
    expect(await screen.findByText(t("agentHistoryNone"))).toBeTruthy();
  });

  it("tells the problem when a copy cannot be read", async () => {
    ipc.copies = [copy(2, "2026-11-13T00:00:00Z")];
    ipc.heldFails = true;
    const hands = view();
    await waitFor(() => expect(hands.onError).toHaveBeenCalledWith({ code: "gone" }));
    expect(screen.queryByLabelText("drawn by the core")).toBeNull();
  });

  it("offers no way to put a copy back where the document only reads its file", async () => {
    ipc.copies = [copy(2, "2026-11-13T00:00:00Z")];
    view({ restores: false });
    await screen.findByText(/Words before the last write/);
    expect(screen.queryByRole("button", { name: t("agentCopyBack") })).toBeNull();
  });

  it("offers neither to compare nor to go back to a copy", async () => {
    ipc.copies = [copy(2, "2026-11-13T00:00:00Z")];
    view();
    await screen.findByText(/Words before the last write/);
    expect(screen.queryByRole("button", { name: t("earlierCompare") })).toBeNull();
    expect(screen.queryByRole("button", { name: t("earlierBack") })).toBeNull();
    const rows = within(list()).getAllByRole("listitem");
    expect(within(rows[0]).getAllByRole("button")).toHaveLength(1);
  });

  it("still compares and goes back to a version listed next to the copies", async () => {
    ipc.versions = [newer, older];
    ipc.copies = [copy(2, "2026-11-13T00:00:00Z")];
    asksSure(() => Promise.resolve(true));
    const hands = view();
    await screen.findByText(/Words before the last write/);
    const rows = within(list()).getAllByRole("listitem");
    await userEvent.click(within(rows[1]).getAllByRole("button")[0]);
    await screen.findByText(/version b/);
    await userEvent.click(screen.getByRole("button", { name: t("earlierCompare") }));
    await waitFor(() => expect(called("doc_compared")).toHaveLength(1));
    await userEvent.click(screen.getByRole("button", { name: t("earlierBack") }));
    await waitFor(() => expect(hands.onBack).toHaveBeenCalledWith(newer));
  });

  it("says there is nothing yet when there is no version and no copy", async () => {
    view();
    expect(await screen.findByText(t("agentHistoryNone"))).toBeTruthy();
    expect(screen.queryByText(t("earlierNone"))).toBeNull();
  });

  it("tells that emptying the history leaves the copies of the file", async () => {
    ipc.versions = [newer];
    ipc.copies = [copy(2, "2026-11-13T00:00:00Z")];
    const asked: string[] = [];
    asksSure((words: string) => {
      asked.push(words);
      return Promise.resolve(true);
    });
    view();
    await screen.findByText(/Words before the last write/);
    await userEvent.click(screen.getByRole("button", { name: t("earlierEmpty") }));
    await waitFor(() => expect(called("doc_history_empty")).toHaveLength(1));
    expect(asked[0]).toContain(t("earlierEmptySure"));
    expect(asked[0]).toContain(t("agentHistoryEmptyKeeps"));
  });
});

describe("the history of any other document", () => {
  it("asks for the versions alone and says nothing of file copies", async () => {
    ipc.versions = [newer, older];
    ipc.copies = [copy(2, "2026-11-13T00:00:00Z")];
    view(null);
    expect(await within(list()).findAllByRole("listitem")).toHaveLength(2);
    expect(called("agent_history")).toHaveLength(0);
    expect(called("doc_versions")).toHaveLength(1);
    expect(screen.queryByText(t("agentHistoryCopies"))).toBeNull();
  });

  it("keeps its own empty message and its plain emptying question", async () => {
    view(null);
    expect(await screen.findByText(t("earlierNone"))).toBeTruthy();
    expect(screen.queryByText(t("agentHistoryNone"))).toBeNull();
  });

  it("asks the plain emptying question, without a word of file copies", async () => {
    ipc.versions = [newer];
    const asked: string[] = [];
    asksSure((words: string) => {
      asked.push(words);
      return Promise.resolve(true);
    });
    view(null);
    await screen.findByText(/version b/);
    await userEvent.click(screen.getByRole("button", { name: t("earlierEmpty") }));
    await waitFor(() => expect(called("doc_history_empty")).toHaveLength(1));
    expect(asked).toEqual([t("earlierEmptySure")]);
  });
});
