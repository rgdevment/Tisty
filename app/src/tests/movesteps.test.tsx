import { act, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { heard } from "../coming";
import type { Carrying } from "../core";
import { weigh } from "../format";
import { adopt, fill, t } from "../locales";
import { endMove, RETRY_IN, startMove } from "../relocation";
import MoveNotice from "../ui/MoveNotice";
import Relocating from "../ui/Relocating";

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
