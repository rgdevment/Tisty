import { act, fireEvent, render, renderHook, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Item } from "../attention";
import type { Settings } from "../core";
import { fill, t } from "../locales";
import type { Which } from "../ui/Card";
import Card from "../ui/Card";
import { Careful, useCareful } from "../ui/Careful";
import { DANGERS } from "../ui/dangers";
import { doing, Feedback, HeedContext, Heeded, Note, RowsContext } from "../ui/RowNotes";
import { Line } from "../ui/Rows";
import { useRemember } from "../ui/remembering";
import { nextTab, tabKeys } from "../ui/tabbing";
import { fadesAfter, useCards } from "../ui/useCards";
import { GROUPS, held, holder, IDLE, YOUR_DATA } from "../ui/working";
import { Rows } from "./rowsShell";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  writes: [] as ((value: unknown) => void)[],
  refusals: [] as ((why: unknown) => void)[],
  settings: { quiet: [] } as unknown,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args: Record<string, unknown> = {}) => {
    ipc.calls.push({ cmd, args });
    if (cmd === "keep_settings")
      return new Promise((done, refuse) => {
        ipc.writes.push(done);
        ipc.refusals.push(refuse);
      });
    if (cmd === "settings") return Promise.resolve(ipc.settings);
    return Promise.resolve(null);
  },
}));

beforeEach(() => {
  ipc.calls.length = 0;
  ipc.writes.length = 0;
  ipc.refusals.length = 0;
});

afterEach(() => {
  vi.useRealTimers();
});

function Strip({ all, picked }: { all: string[]; picked: (one: string) => void }) {
  const [on, setOn] = useState("a");
  const moved = tabKeys(
    all,
    false,
    (one) => `t-${one}`,
    (one) => {
      picked(one);
      setOn(one);
    },
  );
  return (
    <div role="tablist">
      {all.map((one) => (
        <button
          key={one}
          id={`t-${one}`}
          type="button"
          role="tab"
          aria-selected={on === one}
          onKeyDown={(event) => moved(event, one)}
        >
          {one}
        </button>
      ))}
    </div>
  );
}

const working = (...cards: Which[]) => new Map(cards.map((one) => [one, Date.now()]));

describe("which work keeps which row waiting", () => {
  it("lets a simple switch go on while a backup runs", () => {
    const busy = working("backup");
    expect(held(busy, "tongue")).toBe(false);
    expect(held(busy, "chime")).toBe(false);
    expect(held(busy, "door")).toBe(false);
  });

  it("keeps everything that touches your data waiting for the one that runs", () => {
    const busy = working("backup");
    for (const one of [
      "restore",
      "sync",
      "syncOff",
      "sealing",
      "machines",
      "store",
      "free",
      "holds",
    ] as Which[])
      expect(holder(busy, one)).toBe("backup");
  });

  it("names the row itself when it is the one at work", () => {
    expect(holder(working("tongue"), "tongue")).toBe("tongue");
    expect(holder(IDLE, "tongue")).toBeNull();
  });

  it("has one group with every operation on the history, folder or cloud", () => {
    expect(GROUPS).toEqual([YOUR_DATA]);
    expect(YOUR_DATA).toEqual(expect.arrayContaining(["backup", "restore", "sync", "sealing"]));
  });
});

describe("the state each row keeps", () => {
  it("lets two rows work at once and says each one under its own row", async () => {
    const { result } = renderHook(() => useCards(() => {}));
    let finish: (value: string) => void = () => {};
    act(() =>
      result.current.run(
        "backup",
        new Promise<string>((done) => {
          finish = done;
        }),
        (made) => result.current.tell({ card: "backup", text: made }),
      ),
    );
    await act(() =>
      result.current.quietly("chime", Promise.resolve(1), () =>
        result.current.tell({ card: "chime", text: t("settingsKept") }),
      ),
    );

    expect(result.current.busy.has("backup")).toBe(true);
    expect(result.current.busy.has("chime")).toBe(false);
    expect(result.current.said.chime?.text).toBe(t("settingsKept"));

    await act(async () => finish("412 MB"));
    expect(result.current.busy.size).toBe(0);
    expect(result.current.said.backup?.text).toBe("412 MB");
    expect(result.current.said.chime?.text).toBe(t("settingsKept"));
  });

  it("lets a confirmation go away on its own", () => {
    vi.useFakeTimers();
    const { result } = renderHook(() => useCards(() => {}));
    act(() => result.current.tell({ card: "chime", text: t("settingsKept") }));
    expect(result.current.said.chime).toBeTruthy();
    act(() => vi.advanceTimersByTime(fadesAfter(t("settingsKept")) + 10));
    expect(result.current.said.chime).toBeUndefined();
  });

  it("gives long confirmations more time to be read, up to a limit", () => {
    expect(fadesAfter(t("settingsKept"))).toBeLessThan(fadesAfter("x"));
    expect(fadesAfter("x".repeat(150))).toBeGreaterThan(fadesAfter("x"));
    expect(fadesAfter("x".repeat(5000))).toBe(12000);
  });

  it("keeps an error with a retry when the work can be asked again", async () => {
    const { result } = renderHook(() => useCards(() => {}));
    let tries = 0;
    const work = () => {
      tries++;
      return tries === 1 ? Promise.reject({ code: "x" }) : Promise.resolve("done");
    };
    const done = vi.fn();
    await act(async () => result.current.run("report", work, done));
    const wrong = result.current.trouble.report;
    expect(wrong?.again).toBeTypeOf("function");

    await act(async () => wrong?.again?.());
    expect(tries).toBe(2);
    expect(done).toHaveBeenCalledWith("done");
    expect(result.current.trouble.report).toBeUndefined();
  });

  it("starting one row leaves another row's error where it is", async () => {
    const { result } = renderHook(() => useCards(() => {}));
    act(() => result.current.fail({ card: "backup", text: "full" }));
    await act(async () => result.current.quietly("chime", Promise.resolve(1), () => {}));
    expect(result.current.trouble.backup?.text).toBe("full");
  });

  it("clears one row, or every error at once", () => {
    const { result } = renderHook(() => useCards(() => {}));
    act(() => {
      result.current.fail({ card: "backup", text: "a" });
      result.current.fail({ card: "sync", text: "b" });
      result.current.tell({ card: "chime", text: "c" });
    });
    act(() => result.current.clear("backup"));
    expect(result.current.trouble.backup).toBeUndefined();
    expect(result.current.trouble.sync?.text).toBe("b");
    act(() => result.current.fail());
    expect(result.current.trouble).toEqual({});
    act(() => result.current.tell());
    expect(result.current.said.chime?.text).toBe("c");
  });

  it("starts and stops a card by hand, as a round of sync does", () => {
    const { result } = renderHook(() => useCards(() => {}));
    act(() => result.current.start("sync"));
    expect(result.current.busy.has("sync")).toBe(true);
    act(() => result.current.stop("sync"));
    act(() => result.current.stop("sync"));
    expect(result.current.busy.size).toBe(0);
  });
});

describe("what hangs under a row", () => {
  it("always has a place that announces what happens", () => {
    render(
      <Rows>
        <Line row="chime" which="chime" />
      </Rows>,
    );
    const said = document.querySelector("[data-feedback=chime]");
    expect(said?.getAttribute("aria-live")).toBe("polite");
  });

  it("says Saved under the row that changed and nowhere else", () => {
    render(
      <Rows said={{ card: "chime", text: t("settingsKept") }}>
        <Line row="screen" which="screen" />
        <Line row="chime" which="chime" />
      </Rows>,
    );
    const chime = document.querySelector("[data-setting=chime]") as HTMLElement;
    const screenRow = document.querySelector("[data-setting=screen]") as HTMLElement;
    expect(within(chime).getByText(t("settingsKept"))).toBeTruthy();
    expect(within(screenRow).queryByText(t("settingsKept"))).toBeNull();
  });

  it("shows working with its seconds only in the row that works", () => {
    vi.useFakeTimers();
    render(
      <Rows busy={working("backup")}>
        <Line row="backup" which="backup" />
        <Line row="chime" which="chime" />
      </Rows>,
    );
    expect(document.querySelector("[data-working]")).toBeNull();
    act(() => vi.advanceTimersByTime(2100));
    const row = document.querySelector("[data-setting=backup]") as HTMLElement;
    expect(within(row).getByText(fill("rowWorkingFor", doing("backup"), "2"))).toBeTruthy();
    expect(
      (document.querySelector("[data-setting=chime]") as HTMLElement).querySelector(
        "[data-working]",
      ),
    ).toBeNull();
  });

  it("says which work a waiting row waits for, unless the row keeps quiet", () => {
    render(
      <Rows busy={working("backup")}>
        <Line row="restore" which="restore" />
        <Line row="syncMachines" which="machines" quiet />
      </Rows>,
    );
    expect(screen.getAllByText(fill("waitFor", t("backup")))).toHaveLength(1);
  });

  it("says what went wrong, with the retry beside it", async () => {
    const again = vi.fn();
    render(
      <Rows trouble={{ card: "report", text: "The folder is gone. Pick another.", again }}>
        <Line row="report" which="report" />
      </Rows>,
    );
    const note = document.querySelector("[data-note=err]") as HTMLElement;
    expect(within(note).getByText(t("noteErr"))).toBeTruthy();
    await userEvent.click(within(note).getByRole("button", { name: t("tryAgain") }));
    expect(again).toHaveBeenCalled();
  });

  it("can leave out what the row already shows its own way", () => {
    render(
      <Rows busy={working("sync")} said={{ card: "sync", text: "ok" }} trouble={undefined}>
        <Feedback which="sync" hideWorking hideSaid hideTrouble />
      </Rows>,
    );
    expect(screen.queryByText("ok")).toBeNull();
  });

  it("stays hushed while a dialog of its own speaks for it", () => {
    render(
      <Rows trouble={{ card: "report", text: "denied" }}>
        <Line row="report" which="report" hushed />
      </Rows>,
    );
    expect(screen.queryByText("denied")).toBeNull();
    expect(document.querySelector("[data-feedback=report]")).toBeTruthy();
  });

  it("names every kind of note", () => {
    for (const [kind, label] of [
      ["info", "noteInfo"],
      ["warn", "noteWarn"],
      ["dep", "noteDep"],
      ["sys", "noteSys"],
      ["na", "noteNa"],
    ] as const) {
      const { unmount } = render(<Note kind={kind}>x</Note>);
      expect(screen.getByText(t(label))).toBeTruthy();
      unmount();
    }
  });

  it("shows an item that asks for attention in the row it belongs to", () => {
    const items: Item[] = [
      {
        key: "phrase",
        spot: { page: "security", row: "identity" },
        said: t("attPhrase"),
        act: t("attPhraseAct"),
        why: t("attPhraseWhy"),
      },
      { key: "update", spot: { page: "about", row: "update" }, said: "Tisty 2", act: "" },
    ];
    render(
      <HeedContext.Provider value={items}>
        <Card row="identity" which="identity">
          <p>here</p>
        </Card>
        <Heeded row="backup" />
      </HeedContext.Provider>,
    );
    const note = document.querySelector("[data-note=warn]") as HTMLElement;
    expect(within(note).getByText(t("attPhrase"))).toBeTruthy();
    expect(within(note).getByText(t("attPhraseWhy"))).toBeTruthy();
    expect(document.querySelectorAll("[data-note=warn]")).toHaveLength(1);
  });

  it("shows nothing while the list is not known yet", () => {
    render(<Heeded row="identity" />);
    expect(document.querySelector("[data-note]")).toBeNull();
  });
});

describe("asking before what cannot be undone", () => {
  it("names the consequences and waits for the box before it goes", async () => {
    const answer = vi.fn();
    render(<Careful danger={DANGERS.restore()} onAnswer={answer} />);
    const dialog = screen.getByRole("dialog");
    expect(within(dialog).getByText(t("carefulRestoreLead"))).toBeTruthy();
    expect(within(dialog).getByText(t("carefulRestoreOne"))).toBeTruthy();
    const go = within(dialog).getByRole("button", { name: t("carefulRestoreGo") });
    expect((go as HTMLButtonElement).disabled).toBe(true);
    await userEvent.click(within(dialog).getByLabelText(t("carefulUndone")));
    await userEvent.click(go);
    expect(answer).toHaveBeenCalledWith(true);
  });

  it("goes without the box when it can be undone", async () => {
    const answer = vi.fn();
    render(<Careful danger={DANGERS.syncOff()} onAnswer={answer} />);
    expect(screen.queryByLabelText(t("carefulUndone"))).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: t("syncOffNow") }));
    expect(answer).toHaveBeenCalledWith(true);
  });

  it("answers no on cancel, and a second question cancels the first", async () => {
    const { result } = renderHook(() => useCareful());
    let first: Promise<boolean> = Promise.resolve(true);
    act(() => {
      first = result.current.ask(DANGERS.door());
    });
    let second: Promise<boolean> = Promise.resolve(true);
    act(() => {
      second = result.current.ask(DANGERS.syncOff());
    });
    expect(await first).toBe(false);
    render(<div>{result.current.shown}</div>);
    await userEvent.click(screen.getByRole("button", { name: t("cancel") }));
    expect(await second).toBe(false);
  });

  it("carries a confirmation for every irreversible action", () => {
    const all = [
      DANGERS.restore(),
      DANGERS.syncOff(),
      DANGERS.forget(),
      DANGERS.revoke(),
      DANGERS.abandon("gone"),
      DANGERS.wipe("Google Drive"),
      DANGERS.unseal("Google Drive"),
      DANGERS.drop("Portátil", "yesterday"),
      DANGERS.door(),
      DANGERS.stranded("Receta"),
      DANGERS.missing("Receta"),
      DANGERS.loose(5),
      DANGERS.loose(1),
    ];
    for (const one of all) {
      expect(one.title).toBeTruthy();
      expect(one.lead).toBeTruthy();
      expect(one.go).toBeTruthy();
    }
    expect(DANGERS.drop("Portátil", "yesterday").lead).toContain("Portátil");
    expect(DANGERS.wipe("Google Drive").lead).toContain("Google Drive");
    expect(DANGERS.missing("Receta").title).toContain("Receta");
    expect(DANGERS.loose(5).title).toContain("5");
    expect(DANGERS.loose(1).title).toBe(t("carefulLooseTitleOne"));
  });
});

describe("moving through tabs with the keyboard", () => {
  const all = ["a", "b", "c"];

  it("goes on, back, round the ends and to either end", () => {
    expect(nextTab("ArrowRight", all, "a", false)).toBe("b");
    expect(nextTab("ArrowLeft", all, "a", false)).toBe("c");
    expect(nextTab("ArrowRight", all, "c", false)).toBe("a");
    expect(nextTab("ArrowDown", all, "a", true)).toBe("b");
    expect(nextTab("ArrowUp", all, "b", true)).toBe("a");
    expect(nextTab("Home", all, "b", true)).toBe("a");
    expect(nextTab("End", all, "a", true)).toBe("c");
  });

  it("ignores the arrows across the list and keys that are not arrows", () => {
    expect(nextTab("ArrowDown", all, "a", false)).toBeNull();
    expect(nextTab("ArrowRight", all, "a", true)).toBeNull();
    expect(nextTab("Enter", all, "a", true)).toBeNull();
    expect(nextTab("ArrowRight", all, "z", false)).toBeNull();
  });

  it("moves the focus to the next tab and tells who listens", () => {
    const picked = vi.fn();
    render(<Strip all={all} picked={picked} />);
    const first = screen.getByRole("tab", { name: "a" });
    first.focus();
    fireEvent.keyDown(first, { key: "ArrowRight" });
    expect(picked).toHaveBeenCalledWith("b");
    expect(document.activeElement?.id).toBe("t-b");
    fireEvent.keyDown(document.activeElement as Element, { key: "Tab" });
    expect(picked).toHaveBeenCalledTimes(1);
  });
});

describe("settings written one after another", () => {
  const before = { quiet: [] } as unknown as Settings;

  it("shows each change at once and keeps the last one when answers come back", async () => {
    const setKept = vi.fn();
    const reread = vi.fn();
    const { result } = renderHook(() => {
      const cards = useCards(() => {});
      return {
        cards,
        remember: useRemember({
          setKept,
          quietly: cards.quietly,
          tell: cards.tell,
          reread,
          onChanged: () => {},
        }),
      };
    });
    const one = { ...before, quiet: ["chime"] } as unknown as Settings;
    const two = { ...before, quiet: ["chime", "screen"] } as unknown as Settings;
    act(() => result.current.remember(one, "chime"));
    act(() => result.current.remember(two, "screen"));
    expect(setKept).toHaveBeenLastCalledWith(two);

    await act(async () => {});
    expect(ipc.writes).toHaveLength(1);
    await act(async () => ipc.writes[0](one));
    expect(ipc.writes).toHaveLength(2);
    expect(setKept).toHaveBeenLastCalledWith(two);
    await act(async () => ipc.writes[1](two));
    expect(setKept).toHaveBeenLastCalledWith(two);
    expect(result.current.cards.said.screen?.text).toBe(t("settingsKept"));
    expect(reread).not.toHaveBeenCalled();
  });

  it("reads the settings again when the last write fails", async () => {
    const reread = vi.fn();
    const { result } = renderHook(() => {
      const cards = useCards(() => {});
      return useRemember({
        setKept: () => {},
        quietly: cards.quietly,
        tell: cards.tell,
        reread,
        onChanged: () => {},
      });
    });
    act(() => result.current(before, "chime"));
    await act(async () => {});
    await act(async () => ipc.refusals[0]({ code: "x" }));
    expect(reread).toHaveBeenCalledTimes(1);
  });
});

describe("the rows context", () => {
  it("starts empty outside the settings", () => {
    render(
      <RowsContext.Consumer>
        {(rows) => <span>{String(rows.busy.size)}</span>}
      </RowsContext.Consumer>,
    );
    expect(screen.getByText("0")).toBeTruthy();
  });
});
