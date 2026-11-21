import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Filed, Settings, Task } from "../core";
import { awayOf, saidOf, type Earlier as Version, whenOf, writerOf } from "../earlier";
import { fill, t } from "../locales";
import { asksSure } from "../sure";
import Beside from "../ui/Beside";
import type { Run } from "../ui/Card";
import Docs from "../ui/Docs";
import Earlier from "../ui/Earlier";
import { toFirstChange } from "../ui/EarlierCompared";
import EarlierLimit from "../ui/EarlierLimit";
import TaskWords from "../ui/TaskWords";
import { knowAgents } from "../who";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  rows: [] as unknown[],
  bodies: {} as Record<string, string>,
  body: "# Acta\n\nlo de ahora",
  before: null as string | null,
  page: null as unknown,
  kept: null as number | null,
  moved: false,
  full: false,
  compared: null as unknown,
}));

vi.mock("@tauri-apps/api/core", async () => ({
  invoke: (await import("./opened")).opening((cmd: string, args: Record<string, unknown> = {}) => {
    ipc.calls.push({ cmd, args });
    switch (cmd) {
      case "doc_versions": {
        const here = (ipc.rows as Version[]).filter((one) => one.here).length;
        return Promise.resolve({ versions: ipc.rows, kept: ipc.kept ?? here });
      }
      case "doc_write":
        if (ipc.full) return Promise.reject({ code: "documentTooLong", name: "500 kB" });
        return ipc.moved
          ? Promise.reject({ code: "documentMoved" })
          : Promise.resolve({ id: args.id, title: "Acta", print: "p" });
      case "task_doc_merge":
        return Promise.resolve(false);
      case "doc_version": {
        const body = ipc.bodies[String(args.print)];
        return body === undefined ? Promise.reject({ code: "versionGone" }) : Promise.resolve(body);
      }
      case "doc_back_to":
        ipc.before = ipc.body;
        ipc.body = ipc.bodies[String(args.print)];
        return Promise.resolve({ id: args.id, title: "Acta" });
      case "doc_back":
        ipc.body = ipc.before ?? ipc.body;
        return Promise.resolve({ id: args.id, title: "Acta" });
      case "doc_history_empty":
        ipc.rows = (ipc.rows as Version[]).filter((one) => !one.here);
        return Promise.resolve(2);
      case "keep_history_days":
        return Promise.resolve(args.days);
      case "doc_compared":
        if (ipc.compared instanceof Error) return Promise.reject({ code: ipc.compared.message });
        return Promise.resolve(
          ipc.compared ?? {
            kind: "drawn",
            html: `<p>${String(args.was)[0]}|${String(args.now ?? "now")[0]}</p>`,
            gone: 1,
            came: 2,
          },
        );
      case "doc_read":
        return Promise.resolve(ipc.body);
      case "doc_backable":
        return Promise.resolve(ipc.before !== null);
      case "doc_facts":
        return Promise.resolve({ bytes: 7, pages: 0 });
      case "task_doc_open":
        return Promise.resolve(ipc.page && { ...(ipc.page as object), body: ipc.body });
      default:
        return Promise.resolve(null);
    }
  }),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: () => Promise.resolve(() => {}),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  ask: () => Promise.resolve(true),
  save: () => Promise.resolve(null),
  open: () => Promise.resolve(null),
}));

vi.mock("../ui/Sheet", () => ({
  default: ({ value, onWrite }: { value: string; onWrite?: (text: string) => void }) => (
    <textarea aria-label="editor" value={value} onChange={(e) => onWrite?.(e.target.value)} />
  ),
}));

vi.mock("../ui/Page", () => ({
  default: ({ body }: { body: string }) => <article aria-label="drawn by the core">{body}</article>,
}));

const A = "a".repeat(64);
const B = "b".repeat(64);
const C = "c".repeat(64);

const yours: Version = {
  print: A,
  wrote: "2026-11-10T10:40:00Z",
  replaced: "2026-11-11T09:00:00Z",
  by: "mac-1",
  bytes: 3200,
  here: true,
};
const claudes: Version = {
  print: B,
  wrote: "2026-11-12T08:15:00Z",
  replaced: "2026-11-12T18:00:00Z",
  via: "claude-code",
  by: "agent-1",
  bytes: 4100,
  here: true,
};
const elsewhere: Version = {
  print: C,
  wrote: "2026-11-11T20:00:00Z",
  replaced: "2026-11-12T08:15:00Z",
  by: "mac-1",
  keptBy: "mac-2",
  bytes: 3900,
  here: false,
};

const called = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

const view = (more: Partial<Parameters<typeof Earlier>[0]> = {}) => {
  const hands = {
    onBack: vi.fn(() => Promise.resolve()),
    onEmptied: vi.fn(),
    onClose: vi.fn(),
    onError: vi.fn(),
    ...more,
  };
  render(<Earlier doc="d-1" title="Acta" {...hands} />);
  return hands;
};

const list = () => screen.getByRole("list", { name: t("earlierList") });

beforeEach(() => {
  ipc.calls = [];
  ipc.rows = [claudes, elsewhere, yours];
  ipc.bodies = { [A]: "# Acta\n\nlo primero", [B]: "# Acta\n\nlo del asistente" };
  ipc.body = "# Acta\n\nlo de ahora";
  ipc.before = null;
  ipc.page = null;
  ipc.kept = null;
  ipc.moved = false;
  ipc.full = false;
  ipc.compared = null;
  asksSure(null);
  knowAgents(
    { "agent-1": "agent" },
    {
      clients: { "claude-code": "Claude Code" },
      hosts: { "agent-1": "mac-1" },
      machines: { "mac-1": "Escritorio", "mac-2": "Portátil" },
      here: "mac-1",
    },
  );
});

describe("the history of a document", () => {
  it("lists each version with when it was written, who wrote it and what it weighs", async () => {
    view();

    const rows = await within(list()).findAllByRole("listitem");

    expect(rows.map((row) => row.textContent)).toEqual([
      expect.stringContaining(whenOf(claudes)),
      expect.stringContaining(whenOf(elsewhere)),
      expect.stringContaining(whenOf(yours)),
    ]);
    expect(rows[0].textContent).toContain("Claude Code");
    expect(rows[0].textContent).toContain("4.1");
    expect(rows[2].textContent).toContain(t("earlierByYou"));
  });

  it("says where the history lives and that it is not synced", async () => {
    view();

    expect((await screen.findByRole("note")).textContent).toBe(t("earlierLocal"));
  });

  it("marks a version kept on another computer and offers nothing on it", async () => {
    view();

    const rows = await within(list()).findAllByRole("listitem");

    expect(rows[1].textContent).toContain(fill("earlierAwayOn", "Portátil"));
    expect(within(rows[1]).queryByRole("button")).toBeNull();
    expect(within(rows[0]).getByRole("button")).toBeTruthy();
    expect(awayOf({ ...elsewhere, keptBy: "mac-9" })).toBe(t("earlierAway"));
    expect(called("doc_version").some((one) => one.args.print === C)).toBe(false);
  });

  it("opens the newest version kept here, drawn from its own words", async () => {
    view();

    const page = await screen.findByRole("article");

    expect(page.textContent).toBe("# Acta\n\nlo del asistente");
    expect(screen.getByText(saidOf(claudes))).toBeTruthy();
    expect(called("doc_version")[0].args).toEqual({ id: "d-1", print: B });
  });

  it("reads another version when it is picked", async () => {
    view();
    const rows = await within(list()).findAllByRole("listitem");

    await userEvent.click(within(rows[2]).getAllByRole("button")[0]);

    await waitFor(() =>
      expect(screen.getByRole("article").textContent).toBe("# Acta\n\nlo primero"),
    );
    expect(within(rows[2]).getByRole("button").getAttribute("aria-current")).toBe("true");
  });

  it("goes back to the version read only once the person is sure", async () => {
    const asked = vi.fn(() => Promise.resolve(false));
    asksSure(asked);
    const { onBack } = view();
    const back = await screen.findByRole("button", { name: t("earlierBack") });

    await userEvent.click(back);
    expect(asked).toHaveBeenCalledWith(fill("earlierBackSure", whenOf(claudes)));
    expect(onBack).not.toHaveBeenCalled();

    asked.mockResolvedValue(true);
    await userEvent.click(back);
    expect(onBack).toHaveBeenCalledWith(claudes);
  });

  it("tells a task's page that its steps stay as the task has them", async () => {
    const asked = vi.fn(() => Promise.resolve(false));
    asksSure(asked);
    view({ page: true });

    await userEvent.click(await screen.findByRole("button", { name: t("earlierBack") }));

    expect(asked).toHaveBeenCalledWith(
      `${fill("earlierBackSure", whenOf(claudes))} ${t("earlierPageSure")}`,
    );
  });

  it("only reads where nothing can be written", async () => {
    view({ onBack: undefined });

    await screen.findByRole("article");

    expect(screen.queryByRole("button", { name: t("earlierBack") })).toBeNull();
  });

  it("empties the history of this computer once the person is sure, and reads it again", async () => {
    const asked = vi.fn(() => Promise.resolve(false));
    asksSure(asked);
    const { onEmptied } = view();
    const empty = await screen.findByRole("button", { name: t("earlierEmpty") });
    await screen.findByRole("article");

    await userEvent.click(empty);
    expect(asked).toHaveBeenCalledWith(t("earlierEmptySure"));
    expect(called("doc_history_empty")).toEqual([]);

    asked.mockResolvedValue(true);
    await userEvent.click(empty);

    await waitFor(() => expect(within(list()).getAllByRole("listitem")).toHaveLength(1));
    expect(called("doc_history_empty")[0].args).toEqual({ id: "d-1" });
    expect(onEmptied).toHaveBeenCalledTimes(1);
    expect(screen.queryByRole("article")).toBeNull();
    expect((empty as HTMLButtonElement).disabled).toBe(true);
  });

  it("still closes on Escape once emptied, though the button pressed went disabled", async () => {
    asksSure(() => Promise.resolve(true));
    ipc.kept = 3;
    const { onClose } = view();
    await screen.findByRole("article");

    await userEvent.click(screen.getByRole("button", { name: t("earlierEmpty") }));
    await waitFor(() => expect(called("doc_history_empty")).toHaveLength(1));

    await waitFor(() =>
      expect(document.activeElement).toBe(screen.getByRole("button", { name: t("close") })),
    );
    await userEvent.keyboard("{Escape}");
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("still empties a shelf that holds only what the list leaves out", async () => {
    ipc.rows = [];
    ipc.kept = 1;
    asksSure(() => Promise.resolve(true));
    view();

    const empty = await screen.findByRole("button", { name: t("earlierEmpty") });
    await waitFor(() => expect((empty as HTMLButtonElement).disabled).toBe(false));
    await userEvent.click(empty);

    await waitFor(() => expect(called("doc_history_empty")).toHaveLength(1));
  });

  it("says an assistant wrote what its device wrote under no name", () => {
    expect(writerOf({ ...claudes, via: undefined })).toBe(t("anAssistant"));
    expect(writerOf(yours)).toBe(t("earlierByYou"));
    expect(writerOf({ ...yours, by: undefined })).toBeUndefined();
  });

  it("names the assistant behind a task page's version only as the person chose", () => {
    expect(writerOf(claudes, "named")).toBe(writerOf(claudes));
    expect(writerOf(claudes, "agent")).toBe(t("anAgent"));
    expect(writerOf(claudes, "mark")).toBe(t("anAgent"));
    expect(writerOf(yours, "agent")).toBe(t("earlierByYou"));
  });

  it("says when there is nothing kept yet", async () => {
    ipc.rows = [];
    view();

    expect(await screen.findByText(t("earlierNone"))).toBeTruthy();
    expect(
      (screen.getByRole("button", { name: t("earlierEmpty") }) as HTMLButtonElement).disabled,
    ).toBe(true);
  });

  it("closes on Escape and from its button", async () => {
    const { onClose } = view();
    await screen.findByRole("article");

    await userEvent.keyboard("{Escape}");
    await userEvent.click(screen.getByRole("button", { name: t("close") }));

    expect(onClose).toHaveBeenCalledTimes(2);
  });

  it("says what went wrong when a version cannot be read", async () => {
    ipc.bodies = {};
    const { onError } = view();

    await waitFor(() => expect(onError).toHaveBeenCalledWith({ code: "versionGone" }));
    expect(screen.queryByRole("article")).toBeNull();
  });
});

describe("what changed between two versions", () => {
  const status = () => screen.getByRole("status");
  const comparing = () => screen.getByRole("button", { name: t("earlierCompare") });

  it("compares the version read with what the document says now", async () => {
    view();
    await screen.findByRole("article");

    await userEvent.click(comparing());

    await waitFor(() => expect(screen.getByRole("article").textContent).toBe("<p>b|n</p>"));
    expect(called("doc_compared")[0].args).toEqual({ id: "d-1", was: B, now: null });
    expect(status().textContent).toContain(fill("earlierToNow", whenOf(claudes)));
    expect(status().textContent).toContain(fill("earlierWent", "1"));
    expect(status().textContent).toContain(fill("earlierCame", "2"));
    expect(comparing().getAttribute("aria-pressed")).toBe("true");
  });

  it("compares two versions kept here, the older before the newer", async () => {
    view();
    const rows = await within(list()).findAllByRole("listitem");
    const older = fill("earlierCompareWith", whenOf(yours));

    await userEvent.click(within(rows[2]).getByRole("button", { name: older }));

    await waitFor(() => expect(called("doc_compared")).toHaveLength(1));
    expect(called("doc_compared")[0].args).toEqual({ id: "d-1", was: A, now: B });
    expect(status().textContent).toContain(fill("earlierBetween", whenOf(yours), whenOf(claudes)));
    expect(within(rows[2]).getByRole("button", { name: older }).getAttribute("aria-pressed")).toBe(
      "true",
    );
  });

  it("offers no comparison with the version read itself, nor with one kept on another computer", async () => {
    view();
    const rows = await within(list()).findAllByRole("listitem");

    expect(within(rows[0]).getAllByRole("button")).toHaveLength(1);
    expect(within(rows[1]).queryByRole("button")).toBeNull();
    expect(within(rows[2]).getAllByRole("button")).toHaveLength(2);
  });

  it("compares with what it says now once the version compared against is the one picked", async () => {
    view();
    const rows = await within(list()).findAllByRole("listitem");
    await userEvent.click(
      within(rows[2]).getByRole("button", { name: fill("earlierCompareWith", whenOf(yours)) }),
    );
    await waitFor(() => expect(called("doc_compared")).toHaveLength(1));

    await userEvent.click(within(rows[2]).getAllByRole("button")[0]);

    await waitFor(() => expect(called("doc_compared")).toHaveLength(2));
    expect(called("doc_compared")[1].args).toEqual({ id: "d-1", was: A, now: null });
  });

  it("lays the first change in sight, however far down the page it is", () => {
    const page = document.implementation.createHTMLDocument("compared");
    page.body.innerHTML =
      '<p>uno</p><p>dos</p><p data-change="gone">tres</p><p data-change="came">cuatro</p>';
    const first = page.querySelector("[data-change]") as HTMLElement;
    const last = page.querySelector('[data-change="came"]') as HTMLElement;
    first.scrollIntoView = vi.fn();
    last.scrollIntoView = vi.fn();

    toFirstChange({ document: page } as unknown as Window);

    expect(first.scrollIntoView).toHaveBeenCalledWith({ block: "center" });
    expect(last.scrollIntoView).not.toHaveBeenCalled();
    expect(() => toFirstChange(null)).not.toThrow();
  });

  it("drops a version compared with once it becomes the one read", async () => {
    const D = "d".repeat(64);
    const third: Version = { ...yours, print: D, wrote: "2026-11-09T10:00:00Z", bytes: 3000 };
    ipc.rows = [claudes, yours, third];
    ipc.bodies[D] = "# Acta\n\nlo de antes";
    const hands = { onBack: vi.fn(), onClose: vi.fn(), onError: vi.fn() };
    const { rerender } = render(<Earlier doc="d-1" title="Acta" {...hands} />);
    const rows = await within(list()).findAllByRole("listitem");
    await userEvent.click(
      within(rows[1]).getByRole("button", { name: fill("earlierCompareWith", whenOf(yours)) }),
    );
    await waitFor(() => expect(called("doc_compared")).toHaveLength(1));

    ipc.rows = [yours, third];
    rerender(<Earlier doc="d-2" title="Acta" {...hands} />);
    await waitFor(() =>
      expect(called("doc_compared").slice(-1)[0]?.args).toEqual({ id: "d-2", was: A, now: null }),
    );
    const left = await within(list()).findAllByRole("listitem");
    await userEvent.click(within(left[1]).getAllByRole("button")[0]);

    await waitFor(() =>
      expect(called("doc_compared").slice(-1)[0]?.args).toEqual({ id: "d-2", was: D, now: null }),
    );
  });

  it("says when the two say the same", async () => {
    ipc.compared = { kind: "drawn", html: "<p>igual</p>", gone: 0, came: 0 };
    view();
    await screen.findByRole("article");

    await userEvent.click(comparing());

    await waitFor(() => expect(status().textContent).toContain(t("earlierSame")));
    expect(status().textContent).not.toContain(fill("earlierWent", "0"));
  });

  it("offers the whole version when the document is too long to mark", async () => {
    ipc.compared = { kind: "tooLong" };
    view();
    await screen.findByRole("article");

    await userEvent.click(comparing());
    expect(await screen.findByText(t("earlierTooLong"))).toBeTruthy();
    expect(screen.queryByRole("article")).toBeNull();

    await userEvent.click(screen.getByRole("button", { name: t("earlierWhole") }));

    await waitFor(() =>
      expect(screen.getByRole("article").textContent).toBe("# Acta\n\nlo del asistente"),
    );
    expect(comparing().getAttribute("aria-pressed")).toBe("false");
    expect(document.activeElement).toBe(comparing());
  });

  it("says what went wrong and goes back to the version when it cannot compare", async () => {
    ipc.compared = new Error("versionGone");
    const { onError } = view();
    await screen.findByRole("article");

    await userEvent.click(comparing());

    await waitFor(() => expect(onError).toHaveBeenCalledWith({ code: "versionGone" }));
    expect(comparing().getAttribute("aria-pressed")).toBe("false");
    expect(screen.getByRole("article").textContent).toBe("# Acta\n\nlo del asistente");
  });

  it("compares with what it says now only once the words typed are saved", async () => {
    const onError = vi.fn();
    render(
      <Docs
        open="d-1"
        known={[
          { id: "D1", file: "d-1", title: "Acta", folder: null, archived: false, away: false },
        ]}
        onError={onError}
        onKept={vi.fn()}
      />,
    );
    const editor = await screen.findByDisplayValue(/lo de ahora/);
    const shut = screen.queryByRole("button", { name: t("beside") });
    if (shut) await userEvent.click(shut);
    await act(async () => {});
    const row = document.querySelector("details[data-row=earlier]") as HTMLDetailsElement;
    await userEvent.click(row.querySelector("summary") as HTMLElement);
    await userEvent.click(within(row).getByRole("button", { name: t("earlierSee") }));
    ipc.moved = true;
    fireEvent.change(editor, { target: { value: "# Acta\n\nlo que tecleo" } });

    await userEvent.click(await screen.findByRole("button", { name: t("earlierCompare") }));

    await waitFor(() => expect(onError).toHaveBeenCalledWith(t("unsavedUncompared")));
    expect(called("doc_compared")).toEqual([]);
  });
});

describe("the history in the document's column", () => {
  const beside = (onEarlier?: () => void) =>
    render(
      <Beside
        title="Acta"
        paper="d-1"
        body="# Acta"
        tags={[]}
        kept={0}
        heads={[]}
        pages={[]}
        leaf="a4"
        onLeaf={() => {}}
        making={false}
        onCopy={() => {}}
        onTakeOut={() => {}}
        onShut={() => {}}
        onApart={() => {}}
        onEarlier={onEarlier}
      />,
    );
  const row = () => document.querySelector("details[data-row=earlier]") as HTMLDetailsElement;

  it("has a row that says the history is this computer's and opens it", async () => {
    const onEarlier = vi.fn();
    beside(onEarlier);
    await act(async () => {});

    expect(row().textContent).toContain(t("earlierTitle"));
    expect(row().textContent).toContain(t("earlierHere"));
    expect(row().textContent).toContain(t("earlierLocal"));
    await userEvent.click(row().querySelector("summary") as HTMLElement);
    await userEvent.click(within(row()).getByRole("button", { name: t("earlierSee") }));

    expect(onEarlier).toHaveBeenCalledTimes(1);
    expect(called("doc_versions")).toEqual([]);
  });

  it("has no row where nobody can open a history", async () => {
    beside();
    await act(async () => {});

    expect(row()).toBeNull();
  });
});

describe("going back from the document's window", () => {
  const doc: Filed = {
    id: "D1",
    file: "d-1",
    title: "Acta",
    folder: null,
    archived: false,
    away: false,
  };

  const opened = async (more: Partial<Filed> = {}) => {
    const onKept = vi.fn();
    render(<Docs open="d-1" known={[{ ...doc, ...more }]} onError={vi.fn()} onKept={onKept} />);
    await screen.findByDisplayValue(/lo de ahora/);
    const shut = screen.queryByRole("button", { name: t("beside") });
    if (shut) await userEvent.click(shut);
    await act(async () => {});
    const row = document.querySelector("details[data-row=earlier]") as HTMLDetailsElement;
    await userEvent.click(row.querySelector("summary") as HTMLElement);
    await userEvent.click(within(row).getByRole("button", { name: t("earlierSee") }));
    await screen.findByRole("dialog");
    return { onKept };
  };

  it("goes back to the version, says which one, and undoes it from the same notice", async () => {
    asksSure(() => Promise.resolve(true));
    const { onKept } = await opened();

    await userEvent.click(await screen.findByRole("button", { name: t("earlierBack") }));

    await screen.findByDisplayValue(/lo del asistente/);
    expect(called("doc_back_to")[0].args).toEqual({ id: "d-1", print: B });
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(screen.getByText(fill("earlierWentBack", whenOf(claudes)))).toBeTruthy();
    expect(onKept).toHaveBeenCalledWith({ id: "d-1", title: "Acta" });

    await userEvent.click(await screen.findByRole("button", { name: t("docStirredBack") }));

    await screen.findByDisplayValue(/lo de ahora/);
    expect(called("doc_back")).toHaveLength(1);
    expect(screen.queryByText(fill("earlierWentBack", whenOf(claudes)))).toBeNull();
  });

  it("goes back to nothing while words typed could not be saved", async () => {
    asksSure(() => Promise.resolve(true));
    const onError = vi.fn();
    render(<Docs open="d-1" known={[doc]} onError={onError} onKept={vi.fn()} />);
    const editor = await screen.findByDisplayValue(/lo de ahora/);
    const shut = screen.queryByRole("button", { name: t("beside") });
    if (shut) await userEvent.click(shut);
    await act(async () => {});
    const row = document.querySelector("details[data-row=earlier]") as HTMLDetailsElement;
    await userEvent.click(row.querySelector("summary") as HTMLElement);
    await userEvent.click(within(row).getByRole("button", { name: t("earlierSee") }));
    ipc.moved = true;
    fireEvent.change(editor, { target: { value: "# Acta\n\nlo que tecleo" } });

    await userEvent.click(await screen.findByRole("button", { name: t("earlierBack") }));

    await waitFor(() => expect(onError).toHaveBeenCalledWith(t("unsavedStays")));
    expect(called("doc_back_to")).toEqual([]);
    expect(screen.getByRole("dialog")).toBeTruthy();
  });

  it("reads the versions of a locked document without offering to go back", async () => {
    await opened({ locked: true });

    await screen.findByRole("article");

    expect(screen.queryByRole("button", { name: t("earlierBack") })).toBeNull();
  });
});

describe("the versions of a task's page", () => {
  const task = (status: Task["status"] = "open") =>
    ({ id: "one", title: "Mudanza", status }) as unknown as Task;
  const versions = () => screen.findByRole("button", { name: t("earlierOfPage") });

  it("goes back to one, reads the page and the task again, and undoes it from its notice", async () => {
    asksSure(() => Promise.resolve(true));
    ipc.page = { file: "t_one-0001", held: false };
    const onWentBack = vi.fn();
    render(<TaskWords task={task()} onWentBack={onWentBack} />);
    await screen.findByDisplayValue(/lo de ahora/);

    await userEvent.click(await versions());
    await userEvent.click(await screen.findByRole("button", { name: t("earlierBack") }));

    await screen.findByDisplayValue(/lo del asistente/);
    expect(called("doc_back_to")[0].args).toEqual({ id: "t_one-0001", print: B });
    expect(onWentBack).toHaveBeenCalledTimes(1);
    expect(screen.getByText(fill("earlierWentBack", whenOf(claudes)))).toBeTruthy();

    await userEvent.click(screen.getByRole("button", { name: t("docStirredBack") }));

    await screen.findByDisplayValue(/lo de ahora/);
    expect(called("doc_back")[0].args).toEqual({ id: "t_one-0001" });
    expect(onWentBack).toHaveBeenCalledTimes(2);
    expect(screen.queryByText(fill("earlierWentBack", whenOf(claudes)))).toBeNull();
  });

  it("goes back to nothing while words typed on the page could not be woven in", async () => {
    asksSure(() => Promise.resolve(true));
    ipc.page = { file: "t_one-0001", held: false };
    const onError = vi.fn();
    render(<TaskWords task={task()} onError={onError} />);
    const editor = await screen.findByDisplayValue(/lo de ahora/);
    await userEvent.click(await versions());
    ipc.moved = true;
    fireEvent.change(editor, { target: { value: "# Acta\n\nlo que tecleo" } });

    await userEvent.click(await screen.findByRole("button", { name: t("earlierBack") }));

    await waitFor(() => expect(onError).toHaveBeenCalledWith({ code: "unsavedStays" }));
    expect(called("doc_back_to")).toEqual([]);
  });

  it("goes back to nothing while words typed on the page could not be saved", async () => {
    asksSure(() => Promise.resolve(true));
    ipc.page = { file: "t_one-0001", held: false };
    const onError = vi.fn();
    render(<TaskWords task={task()} onError={onError} />);
    const editor = await screen.findByDisplayValue(/lo de ahora/);
    await userEvent.click(await versions());
    ipc.full = true;
    fireEvent.change(editor, { target: { value: "# Acta\n\nlo que tecleo" } });

    await userEvent.click(await screen.findByRole("button", { name: t("earlierBack") }));

    await waitFor(() => expect(onError).toHaveBeenCalledWith({ code: "unsavedStays" }));
    expect(called("doc_back_to")).toEqual([]);
  });

  it("stops offering to undo once the page's history is emptied", async () => {
    asksSure(() => Promise.resolve(true));
    ipc.page = { file: "t_one-0001", held: false };
    render(<TaskWords task={task()} />);
    await screen.findByDisplayValue(/lo de ahora/);
    await userEvent.click(await versions());
    await userEvent.click(await screen.findByRole("button", { name: t("earlierBack") }));
    await screen.findByText(fill("earlierWentBack", whenOf(claudes)));

    await userEvent.click(await versions());
    await userEvent.click(await screen.findByRole("button", { name: t("earlierEmpty") }));

    await waitFor(() =>
      expect(screen.queryByText(fill("earlierWentBack", whenOf(claudes)))).toBeNull(),
    );
    expect(screen.queryByRole("button", { name: t("docStirredBack") })).toBeNull();
  });

  it("has no versions to show before the page is written", async () => {
    ipc.page = { file: "", held: false };
    render(<TaskWords task={task()} />);

    await screen.findByDisplayValue(/lo de ahora/);

    expect(screen.queryByRole("button", { name: t("earlierOfPage") })).toBeNull();
  });

  it("reads the versions of a closed task's page and goes back to none", async () => {
    ipc.page = { file: "t_one-0001", held: true };
    render(<TaskWords task={task("done")} />);

    await userEvent.click(await versions());
    await screen.findByRole("article");

    expect(screen.queryByRole("button", { name: t("earlierBack") })).toBeNull();
  });
});

describe("the limit of the history in the settings", () => {
  const kept = { quiet: [], holds: "everywhere", historyDays: 30 } as unknown as Settings;
  const ran: string[] = [];
  const run: Run = (card, work, then) => {
    ran.push(card);
    void work.then(then);
  };

  it("offers 1, 5, 15 and 30 days and shows the one kept", () => {
    render(<EarlierLimit kept={kept} setKept={vi.fn()} held={false} run={run} />);

    const limit = screen.getByRole("combobox", { name: t("earlierLimit") }) as HTMLSelectElement;

    expect(limit.value).toBe("30");
    expect(Array.from(limit.options, (one) => one.value)).toEqual(["1", "5", "15", "30"]);
    expect(limit.options[0].textContent).toBe(t("earlierOneDay"));
    expect(limit.options[1].textContent).toBe(fill("earlierDays", "5"));
    expect(screen.getByText(t("earlierLimitWhy"))).toBeTruthy();
  });

  it("keeps the limit chosen and shows what the settings answered", async () => {
    const setKept = vi.fn();
    render(<EarlierLimit kept={kept} setKept={setKept} held={false} run={run} />);

    await userEvent.selectOptions(screen.getByRole("combobox", { name: t("earlierLimit") }), "5");

    await waitFor(() => expect(setKept).toHaveBeenCalledWith({ ...kept, historyDays: 5 }));
    expect(called("keep_history_days")[0].args).toEqual({ days: 5 });
    expect(ran).toEqual(["earlier"]);
  });

  it("waits while another setting is being kept", () => {
    render(<EarlierLimit kept={kept} setKept={vi.fn()} held run={run} />);

    expect(
      (screen.getByRole("combobox", { name: t("earlierLimit") }) as HTMLSelectElement).disabled,
    ).toBe(true);
  });
});
