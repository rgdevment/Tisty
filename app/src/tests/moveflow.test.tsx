import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { heard } from "../coming";
import type { Carrying } from "../core";
import { weigh } from "../format";
import { adopt, fill, t } from "../locales";
import { endMove, RETRY_IN } from "../relocation";
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

const ANA = { provider: "google", account: "0123", email: "ana@example.com" };
const TOWARD = { kind: "cloud" as const, account: ANA.account, email: ANA.email };
const FOLDER = { kind: "folder" as const, at: "D:/Tisty" };

const carrying = (over: Partial<Carrying>) =>
  ({
    asked: true,
    loose: 0,
    open: 0,
    archived: 0,
    lists: 0,
    attachments: 0,
    weight: 0,
    carries: 0,
    cloudOffered: true,
    freesSpace: false,
    ...over,
  }) as Carrying;

const plan = {
  down: { files: 2, bytes: 2_000 },
  up: { files: 4, bytes: 4_000 },
  fromCloud: false,
  toCloud: true,
  others: 0,
  free: null,
  short: 0,
};

const step = (n: number) => `${fill("moveStep", String(n), "2")} · `;

const along = (
  where: "moveUploading" | "moveCopying",
  sent: number,
  weighs: number,
  done: number,
  whole: number,
) =>
  `${step(2)}${t(where)} · ${fill("comingOf", weigh(sent), weigh(weighs))} · ${fill(
    "moveFiles",
    String(done),
    String(whole),
  )}`;

const up = (part: number, done: number, whole: number, sent: number, weighs: number) =>
  heard({ stage: "uploading", done, whole, up: { part, sent, weighs } });

const answered = () => {
  const answers: ((value: unknown) => void)[] = [];
  ipc.answers.move_to = () => new Promise((resolve) => answers.push(resolve));
  return answers;
};

const agreedAndMoved = async () => {
  await userEvent.click(await screen.findByRole("checkbox"));
  await userEvent.click(screen.getByRole("button", { name: t("moveGo") }));
};

beforeEach(() => {
  ipc.calls = [];
  ipc.answers = { move_plan: () => Promise.resolve(plan) };
  adopt("en");
  Object.defineProperty(navigator, "onLine", { configurable: true, get: () => true });
});

afterEach(() => {
  vi.useRealTimers();
  Reflect.deleteProperty(navigator, "onLine");
  act(() => endMove());
  heard(null);
});

describe("a move from a folder up to Google Drive, seen through the window", () => {
  it("brings home, uploads, waits out a stumble by itself and ends in the cloud", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    const answers = answered();
    render(
      <Relocating
        state={carrying({ mode: "folder", chosen: "D:/Old" })}
        resume={TOWARD}
        onClose={() => undefined}
      />,
    );
    await agreedAndMoved();
    expect(await screen.findByText(t("moveMoving"))).toBeTruthy();

    act(() => heard({ stage: "attachments", done: 1, whole: 2 }));
    expect(
      await screen.findByText(
        `${step(1)}${t("moveBringingFolder")} · ${fill("comingOf", "1", "2")}`,
      ),
    ).toBeTruthy();

    act(() => up(1, 0, 4, 0, 4_000));
    expect(
      await screen.findByText(`${step(2)}${t("moveUploading")} · ${t("movePreparing")}`),
    ).toBeTruthy();

    act(() => up(1, 2, 4, 2_000, 4_000));
    expect(await screen.findByText(along("moveUploading", 2_000, 4_000, 2, 4))).toBeTruthy();

    await act(async () => {
      answers[0]({
        state: "waiting",
        stage: "filling",
        left: { files: 2, bytes: 2_000 },
        reason: "troubled",
        wait: null,
      });
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(
      await screen.findByText((text) => text.startsWith(`${t("moveTroubled")} · `)),
    ).toBeTruthy();
    expect(screen.getByText(along("moveUploading", 2_000, 4_000, 2, 4))).toBeTruthy();
    expect(screen.queryByText(t("moveWaiting"))).toBeNull();

    await act(async () => {
      await vi.advanceTimersByTimeAsync(RETRY_IN * 1000 + 100);
    });
    expect(sent("move_to")).toHaveLength(2);
    expect(screen.queryByText((text) => text.startsWith(`${t("moveTroubled")} · `))).toBeNull();

    act(() => up(2, 2, 2, 2_000, 2_000));
    expect(await screen.findByText(along("moveUploading", 4_000, 4_000, 4, 4))).toBeTruthy();

    await act(async () => {
      answers[1]({ state: "done", told: true, leftCloud: null });
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(await screen.findByText(fill("moveDone", t("cloudGoogle")))).toBeTruthy();
    expect(screen.queryByRole("button", { name: t("moveWipe") })).toBeNull();
  });
});

describe("a move from Google Drive down to a folder, seen through the window", () => {
  it("brings from the cloud, copies, then deletes the copies left there even after a stop", async () => {
    const answers = answered();
    ipc.answers.move_plan = () => Promise.resolve({ ...plan, fromCloud: true, toCloud: false });
    render(
      <Relocating
        state={carrying({ mode: "cloud", cloud: ANA.email })}
        resume={FOLDER}
        onClose={() => undefined}
      />,
    );
    await agreedAndMoved();

    act(() => heard({ stage: "attachments", done: 3, whole: 4 }));
    expect(
      await screen.findByText(
        `${step(1)}${t("moveBringingCloud")} · ${fill("comingOf", "3", "4")}`,
      ),
    ).toBeTruthy();

    act(() => up(0, 1, 2, 100, 200));
    expect(await screen.findByText(along("moveCopying", 100, 200, 1, 2))).toBeTruthy();

    await act(async () => {
      answers[0]({ state: "done", told: true, leftCloud: ANA });
    });
    expect(await screen.findByText(fill("moveDone", FOLDER.at))).toBeTruthy();

    ipc.answers.cloud_wipe = () => Promise.reject({ code: "wipeStopped" });
    await userEvent.click(screen.getByRole("button", { name: t("moveWipe") }));
    expect((await screen.findByRole("alert")).textContent).toBe(t("wipeStopped"));
    expect(screen.queryByText(t("moveWiping"))).toBeNull();

    let finish: (gone: number) => void = () => undefined;
    ipc.answers.cloud_wipe = () => new Promise((done) => (finish = done));
    await userEvent.click(screen.getByRole("button", { name: t("moveWipe") }));
    expect(await screen.findByText(t("moveWiping"))).toBeTruthy();
    expect(screen.queryByRole("alert")).toBeNull();

    await act(async () => {
      finish(1);
    });
    expect(await screen.findByText(t("moveWipedOne"))).toBeTruthy();
    expect(screen.queryByText(t("moveWiping"))).toBeNull();
    expect(screen.queryByRole("button", { name: t("moveWipe") })).toBeNull();
    expect(sent("cloud_wipe")).toHaveLength(2);
  });
});
