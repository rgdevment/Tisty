import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Carrying } from "../core";
import { adopt, fill, t } from "../locales";
import { endMove } from "../relocation";
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
};

const opened = (over: Partial<Parameters<typeof Relocating>[0]> = {}) => {
  const closed = vi.fn();
  const moved = vi.fn();
  render(
    <Relocating
      state={cloud}
      resume={{ kind: "alone" }}
      onClose={closed}
      onMoved={moved}
      {...over}
    />,
  );
  return { closed, moved };
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

afterEach(() => act(() => endMove()));

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
    opened();
    await agreedAndMoved();

    expect(
      await screen.findByText((text) => text.startsWith(fill("movePaused", "").trim())),
    ).toBeTruthy();
    expect(screen.queryByText(t("moveWaiting"))).toBeNull();
    await waitFor(() => expect(sent("move_to")).toHaveLength(2), { timeout: 5_000 });
    expect(await screen.findByText(t("moveDoneAlone"))).toBeTruthy();
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
    await userEvent.click(screen.getByRole("button", { name: t("close") }));
    expect(moved).toHaveBeenCalled();
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

describe("a move whose word could not reach the others", () => {
  it("keeps the old cloud and its access, and says the others are told later", async () => {
    ipc.answers.move_to = () => Promise.resolve({ state: "done", told: false, leftCloud: ANA });
    const { moved } = opened();
    await agreedAndMoved();

    expect(await screen.findByText(t("moveUntold"))).toBeTruthy();
    expect(screen.queryByRole("button", { name: t("moveWipe") })).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: t("close") }));
    expect(moved).toHaveBeenCalled();
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
        onMoved={() => undefined}
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
        onMoved={() => undefined}
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
