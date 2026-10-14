import { act, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { heard } from "../coming";
import type { Carrying } from "../core";
import { goesOnAt, weigh } from "../format";
import { adopt, fill, t } from "../locales";
import { endMove, RETRY_IN, type Run, startMove } from "../relocation";
import Coming, { SHOWN_AFTER } from "../ui/Coming";
import MoveNotice from "../ui/MoveNotice";
import Relocating from "../ui/Relocating";
import { alongSaid, pauseSaid, Underway, Waiting, waitingSaid } from "../ui/RelocatingSteps";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  answers: {} as Record<string, () => Promise<unknown>>,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    return ipc.answers[cmd]?.() ?? Promise.resolve(null);
  },
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: () => Promise.resolve(() => {}),
}));

vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: () => Promise.resolve() }));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: () => Promise.resolve(null),
  ask: () => Promise.resolve(true),
}));

const sent = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

const TOWARD = { kind: "cloud" as const, account: "0123", email: "ana@example.com" };

const folder = {
  asked: true,
  loose: 0,
  open: 0,
  archived: 0,
  lists: 0,
  attachments: 0,
  weight: 0,
  carries: 0,
  mode: "folder",
  chosen: "/tmp/tisty",
  cloudOffered: true,
  freesSpace: false,
} as Carrying;

const waiting = (reason: string, files: number) => ({
  state: "waiting",
  stage: "filling",
  left: { files, bytes: files * 10 },
  reason,
  wait: null,
});

const step = (n: number) => `${fill("moveStep", String(n), "2")} · `;

const uploading = (sent: number, weighs: number, done: number, whole: number) =>
  `${step(2)}${t("moveUploading")} · ${fill("comingOf", weigh(sent), weigh(weighs))} · ${fill(
    "moveFiles",
    String(done),
    String(whole),
  )}`;

const up = (part: number, done: number, whole: number, sent: number, weighs: number) =>
  heard({ stage: "uploading", done, whole, up: { part, sent, weighs } });

let onLine = true;

beforeEach(() => {
  ipc.calls = [];
  ipc.answers = { move_plan: () => new Promise(() => undefined) };
  adopt("en");
  onLine = true;
  Object.defineProperty(navigator, "onLine", { configurable: true, get: () => onLine });
});

afterEach(() => {
  vi.useRealTimers();
  Reflect.deleteProperty(navigator, "onLine");
  act(() => endMove());
  heard(null);
});

const watched = () =>
  render(<Relocating state={folder} resume={TOWARD} onClose={() => undefined} />);

describe("the steps of a move", () => {
  it("says it checks the data, with no count, while nothing comes home", async () => {
    ipc.answers.move_to = () => new Promise(() => undefined);
    act(() => startMove(TOWARD, "folder"));
    watched();

    act(() => heard({ stage: "papers", done: 308, whole: 308 }));

    expect(screen.getByText(`${step(1)}${t("moveChecking")}`)).toBeTruthy();
    expect(screen.queryByText((text) => text.includes(t("comingPapers")))).toBeNull();
    expect(screen.queryByText((text) => text.includes(fill("comingOf", "308", "308")))).toBeNull();
  });

  it("counts only what actually comes home, from the place it comes from", async () => {
    ipc.answers.move_to = () => new Promise(() => undefined);
    act(() => startMove(TOWARD, "folder"));
    watched();

    act(() => heard({ stage: "attachments", done: 2, whole: 5 }));

    expect(
      screen.getByText(`${step(1)}${t("moveBringingFolder")} · ${fill("comingOf", "2", "5")}`),
    ).toBeTruthy();
  });

  it("tells one upload in bytes and files across every part, getting ready before the first byte", async () => {
    ipc.answers.move_to = () => new Promise(() => undefined);
    act(() => startMove(TOWARD, "folder"));
    watched();

    act(() => up(7, 0, 10, 0, 4_000_000));
    expect(
      screen.getByText(`${step(2)}${t("moveUploading")} · ${t("movePreparing")}`),
    ).toBeTruthy();

    act(() => {
      up(7, 4, 10, 1_000_000, 4_000_000);
      up(8, 1, 2, 500_000, 1_000_000);
    });

    expect(screen.getByText(uploading(1_500_000, 5_000_000, 5, 12))).toBeTruthy();
  });

  it("never goes back to the first step once it sends, not even on the try after a pause", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    let answer: (value: unknown) => void = () => undefined;
    ipc.answers.move_to = () =>
      new Promise((resolve) => {
        answer = resolve;
      });
    act(() => startMove(TOWARD, "folder"));
    watched();
    act(() => up(3, 4, 10, 400, 1_000));

    await act(async () => {
      answer(waiting("troubled", 6));
      await vi.advanceTimersByTimeAsync(RETRY_IN * 1000 + 100);
    });
    expect(sent("move_to")).toHaveLength(2);
    act(() => heard({ stage: "papers", done: 308, whole: 308 }));

    expect(screen.getByText(uploading(400, 1_000, 4, 10))).toBeTruthy();
    expect(screen.queryByText((text) => text.includes(t("moveChecking")))).toBeNull();

    act(() => up(4, 1, 6, 100, 600));
    expect(screen.getByText(uploading(500, 1_000, 5, 10))).toBeTruthy();
  });
});

describe("a move that waits", () => {
  it("says Google stumbled and counts down to the next try while this computer is online", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    const answers: unknown[] = [waiting("offline", 6), new Promise(() => undefined)];
    ipc.answers.move_to = () => Promise.resolve(answers.shift());
    await act(async () => {
      startMove(TOWARD, "folder");
      await vi.advanceTimersByTimeAsync(0);
    });
    watched();

    expect(
      screen.getByText(`${t("moveTroubled")} · ${fill("moveRetryIn", String(RETRY_IN))}`),
    ).toBeTruthy();
    await act(async () => {
      await vi.advanceTimersByTimeAsync(5_000);
    });
    expect(
      screen.getByText(`${t("moveTroubled")} · ${fill("moveRetryIn", String(RETRY_IN - 5))}`),
    ).toBeTruthy();
    expect(screen.queryByText(t("moveWaiting"))).toBeNull();

    await act(async () => {
      await vi.advanceTimersByTimeAsync(RETRY_IN * 1000);
    });
    expect(sent("move_to")).toHaveLength(2);
  });

  it("says there is no connection only when this computer is offline", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    onLine = false;
    ipc.answers.move_to = () => Promise.resolve(waiting("offline", 6));
    await act(async () => {
      startMove(TOWARD, "folder");
      await vi.advanceTimersByTimeAsync(0);
    });
    render(<MoveNotice onOpen={() => undefined} />);

    expect(
      screen.getByText(`${t("moveOffline")} · ${fill("moveRetryIn", String(RETRY_IN))}`),
    ).toBeTruthy();
  });

  it("hands it to the person in plain words once tries in a row moved nothing", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    ipc.answers.move_to = () => Promise.resolve(waiting("troubled", 6));
    await act(async () => {
      startMove(TOWARD, "folder");
      await vi.advanceTimersByTimeAsync(0);
    });
    for (let tried = 0; tried < 4; tried += 1) {
      await act(async () => {
        await vi.advanceTimersByTimeAsync(RETRY_IN * 1000 + 100);
      });
    }
    vi.useRealTimers();
    watched();

    expect(sent("move_to")).toHaveLength(4);
    expect(await screen.findByText(t("moveWaiting"))).toBeTruthy();
    expect(screen.getByText((text) => text.includes(t("moveTroubledWhy")))).toBeTruthy();
    expect(screen.getByText(t("moveAgainWhy"))).toBeTruthy();
  });

  it("keeps trying by itself while each try leaves fewer files", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    const answers: unknown[] = [6, 5, 4, 3, 2].map((files) => waiting("troubled", files));
    ipc.answers.move_to = () => Promise.resolve(answers.shift() ?? new Promise(() => undefined));
    await act(async () => {
      startMove(TOWARD, "folder");
      await vi.advanceTimersByTimeAsync(0);
    });
    for (let tried = 0; tried < 5; tried += 1) {
      await act(async () => {
        await vi.advanceTimersByTimeAsync(RETRY_IN * 1000 + 100);
      });
    }

    expect(sent("move_to")).toHaveLength(6);
  });

  it("tells the bar the same step as the window", async () => {
    ipc.answers.move_to = () => new Promise(() => undefined);
    act(() => startMove(TOWARD, "folder"));
    render(<MoveNotice onOpen={() => undefined} />);

    act(() => up(9, 1, 4, 100, 400));

    expect(screen.getByText(uploading(100, 400, 1, 4))).toBeTruthy();
  });
});

describe("deleting the copies left in Google Drive", () => {
  it("says it is working while the copies go, and nothing once done", async () => {
    const { Done } = await import("../ui/RelocatingSteps");
    const shown = (wiping: boolean) => (
      <Done
        toward={{ kind: "folder", at: "G:Tisty" } as never}
        untold={false}
        wiping={wiping}
        offersWipe
        busy={wiping}
        onKeep={() => {}}
        onWipe={() => {}}
        onClose={() => {}}
      />
    );
    const { rerender } = render(shown(true));
    expect(screen.getByRole("status").textContent).toBe(t("moveWiping"));

    rerender(shown(false));
    expect(screen.queryByRole("status")).toBeNull();
  });
});

const FOLDER = { kind: "folder" as const, at: "D:/Tisty" };

const base: Run = { toward: TOWARD, from: "folder", going: true, watched: false };

const sending = (done: number, whole: number, sent: number, weighs: number) => ({
  step: 2 as const,
  done,
  whole,
  sent,
  weighs,
});

describe("a step said in a line", () => {
  it("says nothing until the move has heard anything", () => {
    expect(alongSaid(base)).toBeUndefined();
  });

  it("numbers the steps only when one place is left for another", () => {
    const checking = { step: 1 as const };

    expect(alongSaid({ ...base, along: checking })).toBe(`${step(1)}${t("moveChecking")}`);
    expect(alongSaid({ ...base, from: "alone", along: checking })).toBe(t("moveChecking"));
    expect(alongSaid({ ...base, from: undefined, along: checking })).toBe(t("moveChecking"));
    expect(alongSaid({ ...base, from: "cloud", toward: { kind: "alone" }, along: checking })).toBe(
      t("moveChecking"),
    );
  });

  it("says what comes home from the cloud apart from what comes from a folder", () => {
    const bringing = { step: 1 as const, bringing: { done: 2, whole: 5 } };

    expect(alongSaid({ ...base, from: "cloud", toward: FOLDER, along: bringing })).toBe(
      `${step(1)}${t("moveBringingCloud")} · ${fill("comingOf", "2", "5")}`,
    );
    expect(alongSaid({ ...base, along: bringing })).toBe(
      `${step(1)}${t("moveBringingFolder")} · ${fill("comingOf", "2", "5")}`,
    );
  });

  it("copies into a folder and uploads into the cloud", () => {
    expect(alongSaid({ ...base, from: "cloud", toward: FOLDER, along: sending(0, 4, 0, 0) })).toBe(
      `${step(2)}${t("moveCopying")} · ${t("movePreparing")}`,
    );
    expect(alongSaid({ ...base, along: sending(0, 4, 0, 0) })).toBe(
      `${step(2)}${t("moveUploading")} · ${t("movePreparing")}`,
    );
  });

  it("stops getting ready as soon as a byte or a file is sent", () => {
    expect(alongSaid({ ...base, along: sending(0, 10, 500, 1_000) })).toBe(
      uploading(500, 1_000, 0, 10),
    );
    expect(alongSaid({ ...base, along: sending(2, 3, 0, 0) })).toBe(
      `${step(2)}${t("moveUploading")} · ${fill("moveFiles", "2", "3")}`,
    );
  });
});

describe("a pause said in a line", () => {
  const AT = 100_000;

  it("says nothing of a move that does not pause", () => {
    expect(pauseSaid(base, AT)).toBeUndefined();
  });

  it("counts whole seconds up to the next try and never below zero", () => {
    const troubled = { ...base, waited: "troubled" as const };

    expect(pauseSaid({ ...troubled, pausedUntil: AT + 19_001 }, AT)).toBe(
      `${t("moveTroubled")} · ${fill("moveRetryIn", "20")}`,
    );
    expect(pauseSaid({ ...troubled, pausedUntil: AT - 5_000 }, AT)).toBe(
      `${t("moveTroubled")} · ${fill("moveRetryIn", "0")}`,
    );
  });

  it("calls a lost line a lost connection only while this computer is offline", () => {
    const offline = { ...base, waited: "offline" as const, pausedUntil: AT + 3_000 };

    expect(pauseSaid(offline, AT)).toBe(`${t("moveTroubled")} · ${fill("moveRetryIn", "3")}`);
    onLine = false;
    expect(pauseSaid(offline, AT)).toBe(`${t("moveOffline")} · ${fill("moveRetryIn", "3")}`);
  });

  it("says when a pause Google asked for ends instead of counting it down", () => {
    const until = AT + 60_000;

    expect(pauseSaid({ ...base, waited: "limit", pausedUntil: until }, AT)).toBe(goesOnAt(until));
    expect(pauseSaid({ ...base, pausedUntil: until }, AT)).toBe(goesOnAt(until));
  });
});

describe("a waiting move said in a line", () => {
  const said = (reason: string | null) => waitingSaid(waiting(reason as string, 2) as never);

  it("names the reason Google gave", () => {
    expect(said("limit")).toBe(`${t("moveWaiting")} · ${t("moveLimit")}`);
    expect(said("troubled")).toBe(`${t("moveWaiting")} · ${t("moveTroubled")}`);
    expect(said("offline")).toBe(`${t("moveWaiting")} · ${t("moveTroubled")}`);
    onLine = false;
    expect(said("offline")).toBe(`${t("moveWaiting")} · ${t("moveOffline")}`);
  });

  it("names what else holds it back, or says it got stuck", () => {
    expect(said("unsettled")).toBe(`${t("moveWaiting")} · ${t("moveUnsettled")}`);
    expect(said(null)).toBe(`${t("moveWaiting")} · ${t("moveStuck")}`);
  });

  it("explains a lost connection in the card only while this computer is offline", () => {
    const card = (reason: string) => (
      <Waiting
        result={waiting(reason, 2) as never}
        busy={false}
        onDrop={() => {}}
        onLater={() => {}}
        onAgain={() => {}}
      />
    );
    const { rerender } = render(card("offline"));
    expect(screen.getByText((text) => text.includes(t("moveTroubledWhy")))).toBeTruthy();

    onLine = false;
    rerender(card("troubled"));
    expect(screen.getByText((text) => text.includes(t("moveTroubledWhy")))).toBeTruthy();

    rerender(card("offline"));
    expect(screen.getByText((text) => text.includes(t("moveOfflineWhy")))).toBeTruthy();
  });
});

describe("the bar of a move", () => {
  const width = (run: Run) => {
    const { container, unmount } = render(<Underway run={run} onHide={() => {}} />);
    const bar = container.querySelector("[aria-hidden] span") as HTMLElement;
    const said = bar.style.width;
    unmount();
    return said;
  };

  it("starts a little filled before it hears anything", () => {
    expect(width(base)).toBe("10%");
  });

  it("fills with what comes home", () => {
    expect(width({ ...base, along: { step: 1, bringing: { done: 1, whole: 4 } } })).toBe("25%");
    expect(width({ ...base, along: { step: 1, bringing: { done: 0, whole: 0 } } })).toBe("0%");
  });

  it("fills with bytes once it sends, and with files when there are no bytes to tell", () => {
    expect(width({ ...base, along: sending(9, 10, 250, 1_000) })).toBe("25%");
    expect(width({ ...base, along: sending(3, 4, 0, 0) })).toBe("75%");
    expect(width({ ...base, along: sending(0, 0, 0, 0) })).toBe("0%");
    expect(width({ ...base, along: sending(1, 1, 2_000, 1_000) })).toBe("100%");
  });

  it("follows whichever is further behind, so all the bytes up never look like the end", () => {
    expect(width({ ...base, along: sending(315, 553, 5_900, 5_900) })).toBe("57%");
    expect(width({ ...base, along: sending(553, 553, 5_900, 5_900) })).toBe("100%");
  });
});

describe("the last small files of a move", () => {
  it("adds how many are left once the bytes are all up, keeping both totals in sight", () => {
    const run = (done: number, whole: number, sent = 5_900_000_000) =>
      ({ ...base, along: sending(done, whole, sent, 5_900_000_000) }) as Run;
    const said = alongSaid(run(315, 553)) ?? "";

    expect(said).toContain(fill("moveFiles", "315", "553"));
    expect(said).toContain(fill("moveFilesLeft", "238"));
    expect(alongSaid(run(552, 553))).toContain(t("moveFileLeft"));
    expect(alongSaid(run(553, 553))).not.toContain(t("moveFileLeft"));
    expect(alongSaid(run(315, 553, 5_000_000_000))).not.toContain(fill("moveFilesLeft", "238"));
  });
});

describe("what a move hears", () => {
  it("counts files alone when a step sends no parts", () => {
    ipc.answers.move_to = () => new Promise(() => undefined);
    act(() => startMove(FOLDER, "cloud"));
    render(<MoveNotice onOpen={() => undefined} />);

    act(() => heard({ stage: "uploading", done: 3, whole: 7 }));

    expect(
      screen.getByText(`${step(2)}${t("moveCopying")} · ${fill("moveFiles", "3", "7")}`),
    ).toBeTruthy();
  });

  it("keeps checking while nothing is to be brought, and keeps bringing past later stages", () => {
    ipc.answers.move_to = () => new Promise(() => undefined);
    act(() => startMove(TOWARD, "folder"));
    render(<MoveNotice onOpen={() => undefined} />);

    act(() => heard({ stage: "attachments", done: 0, whole: 0 }));
    expect(screen.getByText(`${step(1)}${t("moveChecking")}`)).toBeTruthy();

    act(() => heard({ stage: "attachments", done: 2, whole: 5 }));
    act(() => heard({ stage: "papers", done: 1, whole: 9 }));
    expect(
      screen.getByText(`${step(1)}${t("moveBringingFolder")} · ${fill("comingOf", "2", "5")}`),
    ).toBeTruthy();
  });

  it("makes nothing of a round heard while no move goes on", () => {
    const { container } = render(<MoveNotice onOpen={() => undefined} />);

    act(() => up(1, 2, 4, 200, 400));

    expect(container.innerHTML).toBe("");
  });

  it("lets nothing heard during a pause change what it has sent", async () => {
    vi.useFakeTimers();
    const answers: unknown[] = [waiting("troubled", 6), new Promise(() => undefined)];
    ipc.answers.move_to = () => Promise.resolve(answers.shift());
    act(() => startMove(TOWARD, "folder"));
    act(() => up(1, 2, 10, 200, 1_000));
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    watched();
    expect(screen.getByText(uploading(200, 1_000, 2, 10))).toBeTruthy();

    act(() => up(1, 9, 10, 900, 1_000));
    expect(screen.getByText(uploading(200, 1_000, 2, 10))).toBeTruthy();

    await act(async () => {
      await vi.advanceTimersByTimeAsync(RETRY_IN * 1000 + 100);
    });
    act(() => up(2, 1, 8, 100, 800));
    expect(screen.getByText(uploading(300, 1_000, 3, 10))).toBeTruthy();
  });

  it("goes on from where it stood when the person carries on, from the same place", async () => {
    const answers: ((value: unknown) => void)[] = [];
    ipc.answers.move_to = () => new Promise((resolve) => answers.push(resolve));
    act(() => startMove(TOWARD, "folder"));
    render(<MoveNotice onOpen={() => undefined} />);
    act(() => up(1, 2, 5, 200, 500));
    await act(async () => {
      answers[0](waiting("budget", 3));
    });
    expect(
      await screen.findByText((text) => text.startsWith(`${t("moveWaiting")} · `)),
    ).toBeTruthy();

    act(() => startMove(TOWARD));

    expect(await screen.findByText(uploading(200, 500, 2, 5))).toBeTruthy();
    act(() => up(1, 1, 3, 100, 300));
    expect(screen.getByText(uploading(300, 500, 3, 5))).toBeTruthy();
  });

  it("starts a move afresh once the one before it is over", async () => {
    const answers: ((value: unknown) => void)[] = [];
    ipc.answers.move_to = () => new Promise((resolve) => answers.push(resolve));
    act(() => startMove(TOWARD, "folder"));
    render(<MoveNotice onOpen={() => undefined} />);
    act(() => up(1, 5, 5, 500, 500));
    await act(async () => {
      answers[0]({ state: "done", told: true, leftCloud: null });
    });
    expect(await screen.findByText(fill("moveDone", t("cloudGoogle")))).toBeTruthy();

    act(() => startMove(TOWARD, "folder"));
    expect(screen.getByText(t("moveMoving"))).toBeTruthy();

    act(() => up(1, 1, 3, 100, 300));
    expect(screen.getByText(uploading(100, 300, 1, 3))).toBeTruthy();
  });
});

describe("when a move tries again by itself", () => {
  it("waits out Google's pause with two seconds to spare", async () => {
    vi.useFakeTimers();
    ipc.answers.move_to = () => Promise.resolve({ ...waiting("limit", 2), wait: 0 });
    await act(async () => {
      startMove(TOWARD, "folder");
      await vi.advanceTimersByTimeAsync(0);
    });

    await act(async () => {
      await vi.advanceTimersByTimeAsync(1_500);
    });
    expect(sent("move_to")).toHaveLength(1);

    await act(async () => {
      await vi.advanceTimersByTimeAsync(600);
    });
    expect(sent("move_to")).toHaveLength(2);
  });

  it("waits out a pause of exactly three minutes, and leaves a longer one to the person", async () => {
    vi.useFakeTimers();
    ipc.answers.move_to = () => Promise.resolve({ ...waiting("limit", 2), wait: 180 });
    await act(async () => {
      startMove(TOWARD, "folder");
      await vi.advanceTimersByTimeAsync(0);
    });
    render(<MoveNotice onOpen={() => undefined} />);

    expect(screen.getByText((text) => text.startsWith(fill("movePaused", "").trim()))).toBeTruthy();
    expect(screen.queryByText(`${t("moveWaiting")} · ${t("moveLimit")}`)).toBeNull();

    act(() => endMove());
    ipc.answers.move_to = () => Promise.resolve({ ...waiting("limit", 2), wait: 181 });
    await act(async () => {
      startMove(TOWARD, "folder");
      await vi.advanceTimersByTimeAsync(0);
    });

    expect(screen.getByText(`${t("moveWaiting")} · ${t("moveLimit")}`)).toBeTruthy();
  });

  it("tries again only once its twenty seconds are up", async () => {
    vi.useFakeTimers();
    const answers: unknown[] = [waiting("troubled", 6), new Promise(() => undefined)];
    ipc.answers.move_to = () => Promise.resolve(answers.shift());
    await act(async () => {
      startMove(TOWARD, "folder");
      await vi.advanceTimersByTimeAsync(0);
    });

    await act(async () => {
      await vi.advanceTimersByTimeAsync(RETRY_IN * 1000 - 500);
    });
    expect(sent("move_to")).toHaveLength(1);

    await act(async () => {
      await vi.advanceTimersByTimeAsync(600);
    });
    expect(sent("move_to")).toHaveLength(2);
  });

  it("counts down in the bar too", async () => {
    vi.useFakeTimers();
    const answers: unknown[] = [waiting("troubled", 6), new Promise(() => undefined)];
    ipc.answers.move_to = () => Promise.resolve(answers.shift());
    await act(async () => {
      startMove(TOWARD, "folder");
      await vi.advanceTimersByTimeAsync(0);
    });
    render(<MoveNotice onOpen={() => undefined} />);
    expect(
      screen.getByText(`${t("moveTroubled")} · ${fill("moveRetryIn", String(RETRY_IN))}`),
    ).toBeTruthy();

    await act(async () => {
      await vi.advanceTimersByTimeAsync(3_000);
    });

    expect(
      screen.getByText(`${t("moveTroubled")} · ${fill("moveRetryIn", String(RETRY_IN - 3))}`),
    ).toBeTruthy();
  });

  it("starts counting stalled tries again once a try leaves fewer files", async () => {
    vi.useFakeTimers();
    const answers: unknown[] = [6, 6, 6, 5, 5, 5].map((files) => waiting("troubled", files));
    ipc.answers.move_to = () => Promise.resolve(answers.shift() ?? new Promise(() => undefined));
    await act(async () => {
      startMove(TOWARD, "folder");
      await vi.advanceTimersByTimeAsync(0);
    });
    for (let tried = 0; tried < 6; tried += 1) {
      await act(async () => {
        await vi.advanceTimersByTimeAsync(RETRY_IN * 1000 + 100);
      });
    }

    expect(sent("move_to")).toHaveLength(7);
  });

  it("tells a move that gave up in plain words in the bar, never a bare wait", async () => {
    vi.useFakeTimers();
    ipc.answers.move_to = () => Promise.resolve(waiting("troubled", 6));
    await act(async () => {
      startMove(TOWARD, "folder");
      await vi.advanceTimersByTimeAsync(0);
    });
    for (let tried = 0; tried < 3; tried += 1) {
      await act(async () => {
        await vi.advanceTimersByTimeAsync(RETRY_IN * 1000 + 100);
      });
    }
    render(<MoveNotice onOpen={() => undefined} />);

    expect(screen.getByText(`${t("moveWaiting")} · ${t("moveTroubled")}`)).toBeTruthy();
    expect(screen.queryByText(t("moveWaiting"))).toBeNull();
  });
});

describe("the sidebar during a move", () => {
  it("keeps a round's own stages out while a move tells its steps", () => {
    ipc.answers.move_to = () => new Promise(() => undefined);
    render(<Coming />);
    act(() => startMove(TOWARD, "folder"));

    act(() => heard({ stage: "papers", done: 3, whole: 10 }, Date.now() - SHOWN_AFTER));

    expect(screen.queryByText(t("comingPapers"))).toBeNull();
  });

  it("tells a round's stages again once the move is over", async () => {
    ipc.answers.move_to = () => Promise.resolve({ state: "done", told: true, leftCloud: null });
    render(<Coming />);
    render(<MoveNotice onOpen={() => undefined} />);
    act(() => startMove(TOWARD, "folder"));
    expect(await screen.findByText(fill("moveDone", t("cloudGoogle")))).toBeTruthy();

    act(() => heard({ stage: "papers", done: 3, whole: 10 }, Date.now() - SHOWN_AFTER));

    expect(screen.getByText(t("comingPapers"))).toBeTruthy();
  });
});
