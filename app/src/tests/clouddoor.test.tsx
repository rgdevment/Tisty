import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ended, unstuck } from "../coming";
import type { Carrying, CloudStatus } from "../core";
import { adopt, t } from "../locales";
import Clouded from "../ui/Clouded";
import Coming from "../ui/Coming";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  answers: {} as Record<string, () => Promise<unknown>>,
}));

const opened = vi.hoisted(() => ({ urls: [] as string[], sure: true }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    return ipc.answers[cmd]?.() ?? Promise.resolve(null);
  },
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: () => Promise.resolve(() => {}),
}));

vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: (url: string) => {
    opened.urls.push(url);
    return Promise.resolve();
  },
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: () => Promise.resolve(null),
  ask: () => Promise.resolve(opened.sure),
}));

const ANA = { provider: "google", account: "0123", email: "ana@example.com" };

const status = (over: Partial<CloudStatus> = {}): CloudStatus => ({
  provider: "google",
  account: ANA.account,
  email: ANA.email,
  joined: true,
  unitsSpent: 1200,
  unitsADay: 20000,
  bytesSpent: 0,
  bytesADay: 0,
  pausedHere: false,
  haltedUntil: null,
  owed: 0,
  last: null,
  ...over,
});

const round = (code: string, over: Partial<NonNullable<CloudStatus["last"]>> = {}) => ({
  at: Math.floor(Date.now() / 1000),
  code,
  name: null,
  left: 0,
  names: [],
  ...over,
});

const carrying = (over: Partial<Carrying> = {}): Carrying =>
  ({
    asked: true,
    loose: 0,
    open: 0,
    archived: 0,
    lists: 0,
    attachments: 0,
    weight: 3_000_000,
    carries: 8 * 1024 * 1024 * 1024,
    mode: "cloud",
    cloud: ANA.email,
    cloudOffered: true,
    freesSpace: true,
    ...over,
  }) as Carrying;

beforeEach(() => {
  ipc.calls = [];
  ipc.answers = {};
  opened.urls = [];
  opened.sure = true;
  adopt("en");
});

afterEach(() => {
  act(() => unstuck());
});

const LOST = t("cloudLost");

const shown = (trouble?: string, code = "cloudLost") => {
  ipc.answers.cloud_status = () => Promise.resolve(status({ last: round(code) }));
  const carry = vi.fn();
  const run = vi.fn((_card: string, work: Promise<unknown>, then: (answer: unknown) => void) => {
    work.then(then).catch(() => undefined);
  });
  act(() => ended({ code }));
  const settings = render(
    <Clouded
      state={carrying({ last: "2026-10-12T10:00:00Z" })}
      busy={null}
      trouble={trouble ? { card: "sync", text: trouble } : undefined}
      run={run as never}
      fail={() => {}}
      carry={carry}
      onMove={() => {}}
    />,
  );
  render(<Coming />);
  return { carry, settings };
};

describe("a cloud Google shut its door on", () => {
  it("says it once, in the card that offers to reconnect", async () => {
    shown(LOST);

    expect(await screen.findByText(t("stateReconnect"))).toBeTruthy();
    await waitFor(() => expect(screen.queryByRole("alert")).toBeNull());
    expect(screen.queryByText(LOST)).toBeNull();
  });

  it("still says another trouble the card does not show", async () => {
    shown(t("cloudDenied"));

    expect(await screen.findByText(t("stateReconnect"))).toBeTruthy();
    expect(screen.getByText(t("cloudDenied"))).toBeTruthy();
  });

  it("tells the sidebar again once the card is no longer on screen", async () => {
    const { settings } = shown();
    await screen.findByText(t("stateReconnect"));

    settings.unmount();

    expect((await screen.findByRole("alert")).textContent).toContain(LOST);
  });

  it("lets go of the shut door as soon as the sign-in goes through, before the round ends", async () => {
    ipc.answers.cloud_connect = () => Promise.resolve(ANA);
    const { carry } = shown();

    await userEvent.click(await screen.findByRole("button", { name: t("stateReconnectDo") }));

    await waitFor(() => expect(carry).toHaveBeenCalled());
    expect(screen.queryByText(t("stateReconnect"))).toBeNull();
    expect(screen.queryByText(LOST)).toBeNull();
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("lets go of a door shut on another account once the sign-in goes through", async () => {
    ipc.answers.cloud_connect = () => Promise.resolve(ANA);
    const { carry } = shown(undefined, "cloudElsewhere");

    await userEvent.click(await screen.findByRole("button", { name: t("stateElsewhereDo") }));

    await waitFor(() => expect(carry).toHaveBeenCalled());
    expect(screen.queryByRole("button", { name: t("stateElsewhereDo") })).toBeNull();
    expect(screen.queryByRole("alert")).toBeNull();
  });
});
