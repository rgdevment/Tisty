import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { heard } from "../coming";
import type { Carrying } from "../core";
import { adopt, fill, t } from "../locales";
import { afterMove, endMove, startMove, useMove } from "../relocation";
import MoveNotice from "../ui/MoveNotice";
import Relocating from "../ui/Relocating";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  answers: {} as Record<string, () => Promise<unknown>>,
}));

const asked = vi.hoisted(() => ({ sure: true }));

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
  ask: () => Promise.resolve(asked.sure),
}));

const sent = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

const ANA = { provider: "google", account: "0123", email: "ana@example.com" };

const cloud = {
  asked: true,
  loose: 0,
  open: 0,
  archived: 0,
  lists: 0,
  attachments: 0,
  weight: 0,
  carries: 0,
  mode: "cloud",
  cloud: ANA.email,
  cloudOffered: true,
  freesSpace: true,
} as Carrying;

const plan = {
  down: { files: 4, bytes: 2_000_000 },
  up: { files: 0, bytes: 0 },
  fromCloud: true,
  toCloud: false,
  others: 0,
  free: null,
  short: 0,
};

const unhooks: (() => void)[] = [];

const opened = (over: Partial<Parameters<typeof Relocating>[0]> = {}) => {
  const closed = vi.fn();
  const moved = vi.fn();
  unhooks.push(afterMove(moved));
  const shown = render(
    <Relocating state={cloud} resume={{ kind: "alone" }} onClose={closed} {...over} />,
  );
  return { closed, moved, shown };
};

const agreedAndMoved = async () => {
  await userEvent.click(await screen.findByRole("checkbox"));
  await userEvent.click(screen.getByRole("button", { name: t("moveGo") }));
};

beforeEach(() => {
  ipc.calls = [];
  ipc.answers = { move_plan: () => Promise.resolve(plan) };
  asked.sure = true;
  adopt("en");
});

afterEach(() => {
  for (const unhook of unhooks.splice(0)) unhook();
  vi.useRealTimers();
  act(() => endMove());
  heard(null);
});

describe("moving to another place", () => {
  it("says what comes down from the cloud and that nothing is deleted there", async () => {
    opened({ resume: { kind: "folder", at: "G:/Mi unidad/Tisty" } });

    expect(await screen.findByText(fill("moveDown", "2.0 MB"))).toBeTruthy();
    expect(screen.getByText(t("moveKeepsDrive"))).toBeTruthy();
    expect(screen.getByText(t("moveDesktopTip"))).toBeTruthy();
    expect(screen.getByText("G:/Mi unidad/Tisty")).toBeTruthy();
  });

  it("tells that computers which never confirmed this one will not follow it", async () => {
    ipc.answers.move_plan = () => Promise.resolve({ ...plan, others: 2 });
    opened({ resume: { kind: "folder", at: "G:/Mi unidad/Tisty" } });

    expect(await screen.findByText(t("moveOthersFollow"))).toBeTruthy();
  });

  it("says nothing of other computers when this one is alone in the place", async () => {
    opened({ resume: { kind: "folder", at: "G:/Mi unidad/Tisty" } });

    expect(await screen.findByText(t("moveKeepsDrive"))).toBeTruthy();
    expect(screen.queryByText(t("moveOthersFollow"))).toBeNull();
  });

  it("says a folder left behind brings home first only when it holds something to bring", async () => {
    const folder = { ...cloud, mode: "folder", chosen: "D:/Tisty", cloud: null } as Carrying;
    ipc.answers.move_plan = () => Promise.resolve({ ...plan, fromCloud: false });
    const { shown } = opened({ state: folder, resume: { kind: "alone" } });

    expect(await screen.findByText(t("moveHomeFirst"))).toBeTruthy();
    expect(screen.queryByText(fill("moveDown", "2.0 MB"))).toBeNull();
    shown.unmount();

    ipc.answers.move_plan = () =>
      Promise.resolve({ ...plan, fromCloud: false, down: { files: 0, bytes: 0 } });
    opened({ state: folder, resume: { kind: "alone" } });

    expect(await screen.findByText(t("moveResumes"))).toBeTruthy();
    expect(screen.queryByText(t("moveHomeFirst"))).toBeNull();
  });

  it("never says anything comes home when this computer was alone", async () => {
    const alone = { ...cloud, mode: "alone", cloud: null } as Carrying;
    ipc.answers.move_plan = () =>
      Promise.resolve({ ...plan, fromCloud: false, toCloud: true, down: { files: 0, bytes: 0 } });
    opened({ state: alone, resume: { kind: "cloud", account: ANA.account, email: ANA.email } });

    expect(await screen.findByText(t("moveResumes"))).toBeTruthy();
    expect(screen.queryByText(t("moveHomeFirst"))).toBeNull();
    expect(screen.queryByText(fill("moveDown", "2.0 MB"))).toBeNull();
  });

  it("will not move into a Drive that has no room and says how much is missing", async () => {
    ipc.answers.move_plan = () =>
      Promise.resolve({
        ...plan,
        toCloud: true,
        up: { files: 9, bytes: 4_000_000 },
        free: 1_000_000,
        short: 3_000_000,
      });
    opened();

    expect(await screen.findByText(fill("moveNoRoom", "3.0 MB", "1.0 MB"))).toBeTruthy();
    await userEvent.click(screen.getByRole("checkbox"));
    expect(screen.getByRole("button", { name: t("moveGo") }).hasAttribute("disabled")).toBe(true);
  });

  it("moves only once the person says they understand", async () => {
    ipc.answers.move_to = () => Promise.resolve({ state: "done", told: true, leftCloud: null });
    opened();
    const go = await screen.findByRole("button", { name: t("moveGo") });
    expect(go.hasAttribute("disabled")).toBe(true);

    await agreedAndMoved();

    expect(sent("move_to")[0].args.toward).toEqual({ kind: "alone" });
    expect(await screen.findByText(t("moveDoneAlone"))).toBeTruthy();
  });

  it("says why a move waits and how much is left, and carries on when asked", async () => {
    ipc.answers.move_to = () =>
      Promise.resolve({
        state: "waiting",
        stage: "homing",
        left: { files: 3, bytes: 1_000_000 },
        reason: "budget",
      });
    opened();
    await agreedAndMoved();

    expect(await screen.findByText(t("moveWaiting"))).toBeTruthy();
    expect(screen.getByText((text) => text.includes(fill("moveLeft", "3", "1.0 MB")))).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: t("moveAgain") }));
    await waitFor(() => expect(sent("move_to")).toHaveLength(2));
  });

  it("waits out a short pause Google asked for by itself and goes on without asking", async () => {
    const answers = [
      {
        state: "waiting",
        stage: "filling",
        left: { files: 2, bytes: 10 },
        reason: "limit",
        wait: 0,
      },
      { state: "done", told: true, leftCloud: null },
    ];
    ipc.answers.move_to = () => Promise.resolve(answers.shift());
    vi.useFakeTimers({ shouldAdvanceTime: true });
    opened();
    await agreedAndMoved();

    expect(
      await screen.findByText((text) => text.startsWith(fill("movePaused", "").trim())),
    ).toBeTruthy();
    expect(screen.queryByText(t("moveWaiting"))).toBeNull();
    await act(async () => {
      await vi.advanceTimersByTimeAsync(5_000);
    });
    vi.useRealTimers();
    expect(sent("move_to")).toHaveLength(2);
    expect(await screen.findByText(t("moveDoneAlone"))).toBeTruthy();
  });

  it("hands short pauses back to the person once several in a row moved nothing", async () => {
    ipc.answers.move_to = () =>
      Promise.resolve({
        state: "waiting",
        stage: "filling",
        left: { files: 5, bytes: 10 },
        reason: "limit",
        wait: 0,
      });
    vi.useFakeTimers({ shouldAdvanceTime: true });
    opened();
    await agreedAndMoved();
    for (let pause = 0; pause < 4; pause += 1) {
      await act(async () => {
        await vi.advanceTimersByTimeAsync(5_000);
      });
    }
    vi.useRealTimers();

    expect(await screen.findByText(t("moveWaiting"))).toBeTruthy();
    expect(sent("move_to")).toHaveLength(4);
    expect(screen.getByRole("button", { name: t("moveAgain") })).toBeTruthy();
  });

  it("leaves a long pause to the person instead of waiting it out", async () => {
    ipc.answers.move_to = () =>
      Promise.resolve({
        state: "waiting",
        stage: "filling",
        left: { files: 2, bytes: 10 },
        reason: "limit",
        wait: 3_600,
      });
    opened();
    await agreedAndMoved();

    expect(await screen.findByText(t("moveWaiting"))).toBeTruthy();
    expect(screen.getByRole("button", { name: t("moveAgain") })).toBeTruthy();
  });

  it("drops a move that waits only when asked", async () => {
    ipc.answers.move_to = () =>
      Promise.resolve({
        state: "waiting",
        stage: "filling",
        left: { files: 1, bytes: 1 },
        reason: null,
      });
    const { closed } = opened();
    await agreedAndMoved();
    await userEvent.click(await screen.findByRole("button", { name: t("moveDrop") }));

    expect(sent("move_dropped")).toHaveLength(1);
    expect(closed).toHaveBeenCalled();
  });

  it("deletes the copies left in the cloud for good once the person is sure, then closes", async () => {
    ipc.answers.move_to = () => Promise.resolve({ state: "done", told: true, leftCloud: ANA });
    ipc.answers.cloud_wipe = () => Promise.resolve(12);
    const { moved } = opened();
    await agreedAndMoved();
    await userEvent.click(await screen.findByRole("button", { name: t("moveWipe") }));

    expect(await screen.findByText(fill("moveWiped", "12"))).toBeTruthy();
    expect(sent("cloud_wipe")[0].args).toEqual({ account: ANA.account, email: ANA.email });
    expect(moved).toHaveBeenCalledTimes(1);
    await userEvent.click(screen.getByRole("button", { name: t("close") }));
    expect(moved).toHaveBeenCalledTimes(1);
    expect(sent("cloud_forget")).toHaveLength(0);
  });

  it("lets go of the old cloud's access when its copies are left there", async () => {
    ipc.answers.move_to = () => Promise.resolve({ state: "done", told: true, leftCloud: ANA });
    opened();
    await agreedAndMoved();
    await userEvent.click(await screen.findByRole("button", { name: t("moveWipeKeep") }));

    expect(sent("cloud_forget")[0].args.account).toBe(ANA.account);
    expect(sent("cloud_wipe")).toHaveLength(0);
  });

  it("offers the other ways to go, starting from the cloud", () => {
    opened({ resume: undefined });

    expect(screen.getByRole("button", { name: new RegExp(t("welcomeAlone"), "i") })).toBeTruthy();
    expect(screen.getByRole("button", { name: new RegExp(t("chooseFolder"), "i") })).toBeTruthy();
  });

  it("says why a move was refused and stays on the summary", async () => {
    ipc.answers.move_to = () => Promise.reject({ code: "moveFromUnreachable" });
    opened();
    await agreedAndMoved();

    expect(await screen.findByText(t("moveFromUnreachable"))).toBeTruthy();
    expect(screen.getByRole("button", { name: t("moveGo") })).toBeTruthy();
  });
});

describe("choosing the folder a move goes to", () => {
  it("offers one way back while a folder is weighed", async () => {
    ipc.answers.keepers = () =>
      Promise.resolve([
        { key: "gdrive", named: "Google Drive", at: "G:/Mi unidad", into: "G:/Mi unidad/Tisty" },
      ]);
    ipc.answers.strays_at = () => Promise.resolve({ adrift: 3, unreadable: false });
    opened({ heading: "folder", resume: undefined });

    expect(await screen.findAllByRole("button", { name: t("welcomeBack") })).toHaveLength(1);
    await userEvent.click(await screen.findByRole("button", { name: /google drive/i }));

    expect(await screen.findByRole("button", { name: t("keepersSave") })).toBeTruthy();
    expect(screen.getAllByRole("button", { name: t("welcomeBack") })).toHaveLength(1);
    expect(await screen.findByText(fill("keepersStrays", "3"))).toBeTruthy();
  });
});

describe("a move whose word could not reach the others", () => {
  it("keeps the old cloud and its access, and says the others are told later", async () => {
    ipc.answers.move_to = () => Promise.resolve({ state: "done", told: false, leftCloud: ANA });
    const { moved } = opened();
    await agreedAndMoved();

    expect(await screen.findByText(t("moveUntold"))).toBeTruthy();
    expect(screen.queryByRole("button", { name: t("moveWipe") })).toBeNull();
    expect(moved).toHaveBeenCalled();
    await userEvent.click(screen.getByRole("button", { name: t("close") }));
    expect(sent("cloud_forget")).toHaveLength(0);
  });

  it("asks to settle what waits on the person before moving on", async () => {
    ipc.answers.move_to = () =>
      Promise.resolve({
        state: "waiting",
        stage: "homing",
        left: { files: 0, bytes: 0 },
        reason: "unsettled",
      });
    opened();
    await agreedAndMoved();

    expect(await screen.findByText((text) => text.includes(t("moveUnsettled")))).toBeTruthy();
  });

  it("keeps a cloud connected for a move that has to wait", async () => {
    ipc.answers.move_to = () =>
      Promise.resolve({
        state: "waiting",
        stage: "filling",
        left: { files: 2, bytes: 10 },
        reason: "budget",
      });
    opened({ resume: { kind: "cloud", account: ANA.account, email: ANA.email } });
    await agreedAndMoved();
    await userEvent.click(await screen.findByRole("button", { name: t("moveLater") }));

    expect(sent("cloud_status")).toHaveLength(0);
    expect(sent("cloud_forget")).toHaveLength(0);
  });
});

describe("a move that runs out of sight", () => {
  it("hides while it moves and goes on telling it in the sidebar", async () => {
    ipc.answers.move_to = () => new Promise(() => undefined);
    const closed = vi.fn();
    const { unmount } = render(
      <Relocating
        state={cloud}
        resume={{ kind: "cloud", account: ANA.account, email: ANA.email }}
        onClose={closed}
      />,
    );
    await agreedAndMoved();
    await userEvent.click(await screen.findByRole("button", { name: t("moveHide") }));
    unmount();
    render(<MoveNotice onOpen={() => undefined} />);

    expect(closed).toHaveBeenCalled();
    expect(sent("cloud_forget")).toHaveLength(0);
    expect(screen.getByText(t("moveMoving"))).toBeTruthy();
  });

  it("tells a finished move nobody watched, and opens it when asked", async () => {
    ipc.answers.move_to = () => Promise.resolve({ state: "done", told: true, leftCloud: null });
    const open = vi.fn();
    const { unmount } = render(
      <Relocating
        state={cloud}
        resume={{ kind: "cloud", account: ANA.account, email: ANA.email }}
        onClose={() => undefined}
      />,
    );
    await agreedAndMoved();
    await screen.findByText(fill("moveDone", t("cloudGoogle")));
    unmount();
    render(<MoveNotice onOpen={open} />);

    expect(screen.getByText(fill("moveDone", t("cloudGoogle")))).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: t("moveShow") }));
    expect(open).toHaveBeenCalled();
  });

  it("stays quiet while the move window is open", async () => {
    ipc.answers.move_to = () => Promise.resolve({ state: "done", told: true, leftCloud: null });
    render(<MoveNotice onOpen={() => undefined} />);
    opened();
    await agreedAndMoved();

    expect(await screen.findAllByText(t("moveDoneAlone"))).toHaveLength(1);
  });
});

const PAUSED = {
  state: "waiting",
  stage: "filling",
  left: { files: 2, bytes: 10 },
  reason: "limit",
  wait: 0,
};
const TOWARD = { kind: "cloud" as const, account: ANA.account, email: ANA.email };

function Watching() {
  const run = useMove();
  return <span data-testid="run">{run ? "run" : "none"}</span>;
}

describe("a move let go of or seen again", () => {
  it("tells a move that failed out of sight as stopped, and shows why on reopening", async () => {
    ipc.answers.move_to = () => Promise.reject({ code: "moveFromUnreachable" });
    render(<MoveNotice onOpen={() => undefined} />);
    await act(async () => startMove(TOWARD));

    expect((await screen.findByRole("alert")).textContent).toContain(t("moveStopped"));
    opened({ resume: TOWARD });
    expect(await screen.findByText(t("moveFromUnreachable"))).toBeTruthy();
  });

  it("reopened while a pause is waited out, it never moves twice at once", async () => {
    vi.useFakeTimers();
    const answers: unknown[] = [PAUSED, { state: "done", told: true, leftCloud: null }];
    ipc.answers.move_to = () => Promise.resolve(answers.shift());
    await act(async () => {
      startMove(TOWARD);
      await vi.advanceTimersByTimeAsync(0);
    });
    render(<Relocating state={cloud} resume={TOWARD} onClose={() => undefined} />);
    expect(sent("move_to")).toHaveLength(1);

    await act(async () => {
      await vi.advanceTimersByTimeAsync(5_000);
    });

    expect(sent("move_to")).toHaveLength(2);
  });

  it("let go of, its pause timer and its answer still on the way never bring it back", async () => {
    vi.useFakeTimers();
    ipc.answers.move_to = () => Promise.resolve(PAUSED);
    render(<Watching />);
    await act(async () => {
      startMove(TOWARD);
      await vi.advanceTimersByTimeAsync(0);
    });
    act(() => endMove());
    await act(async () => {
      await vi.advanceTimersByTimeAsync(10_000);
    });
    expect(sent("move_to")).toHaveLength(1);

    let answer: (value: unknown) => void = () => undefined;
    ipc.answers.move_to = () =>
      new Promise((resolve) => {
        answer = resolve;
      });
    act(() => startMove(TOWARD));
    act(() => endMove());
    await act(async () => {
      answer(PAUSED);
      await vi.advanceTimersByTimeAsync(10_000);
    });

    expect(screen.getByTestId("run").textContent).toBe("none");
    expect(sent("move_to")).toHaveLength(2);
  });

  it("starts every try with no progress left over from the one before", async () => {
    ipc.answers.move_to = () => new Promise(() => undefined);
    heard({ stage: "uploading", done: 245, whole: 245 });
    opened({ resume: TOWARD });
    await agreedAndMoved();

    expect(await screen.findByText(t("moveMoving"))).toBeTruthy();
    expect(screen.queryByText((text) => text.includes(fill("comingOf", "245", "245")))).toBeNull();
  });
});

describe("a finished move told in the sidebar", () => {
  it("opens the last step when a cloud was left behind, so its copies are asked about first", async () => {
    ipc.answers.move_to = () => Promise.resolve({ state: "done", told: true, leftCloud: ANA });
    const open = vi.fn();
    render(<MoveNotice onOpen={open} />);
    await act(async () => startMove({ kind: "alone" }));

    await userEvent.click(await screen.findByRole("button", { name: t("viewGotIt") }));

    expect(open).toHaveBeenCalled();
    expect(sent("cloud_forget")).toHaveLength(0);
    expect(screen.getByRole("status")).toBeTruthy();
  });

  it("is put away when seen, with nothing left to choose", async () => {
    ipc.answers.move_to = () => Promise.resolve({ state: "done", told: true, leftCloud: null });
    const open = vi.fn();
    render(<MoveNotice onOpen={open} />);
    await act(async () => startMove({ kind: "alone" }));

    await userEvent.click(await screen.findByRole("button", { name: t("viewGotIt") }));

    expect(open).not.toHaveBeenCalled();
    expect(screen.queryByRole("status")).toBeNull();
  });
});
