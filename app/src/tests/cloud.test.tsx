import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { cloudStanding, STORAGE } from "../cloudState";
import { ended, unstuck } from "../coming";
import type { Carrying, CloudStatus } from "../core";
import { adopt, fill, t } from "../locales";
import Choosing from "../ui/Choosing";
import Clouded from "../ui/Clouded";
import Coming from "../ui/Coming";
import FreeSpace from "../ui/FreeSpace";

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

const sent = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

const ANA = { provider: "google", account: "0123", email: "ana@example.com" };

const status = (over: Partial<CloudStatus> = {}): CloudStatus => ({
  provider: "google",
  email: ANA.email,
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
  at: 1,
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

describe("what the cloud's state says", () => {
  it("puts the switch and a pause for every machine before the last round", () => {
    const lost = { last: round("cloudLost") };
    expect(cloudStanding(status({ ...lost, pausedHere: true }))?.said).toBe("stateOff");
    expect(cloudStanding(status({ ...lost, haltedUntil: 200 }), 100)?.said).toBe("stateHalted");
    expect(cloudStanding(status({ ...lost, haltedUntil: 50 }), 100)?.said).toBe("stateReconnect");
  });

  it("gives every round that went wrong its words and its way out", () => {
    const of = (code: string) => cloudStanding(status({ last: round(code) }));
    expect(of("cloudLost")?.exit).toBe("reconnect");
    expect(of("cloudElsewhere")?.exitSaid).toBe("stateElsewhereDo");
    expect(of("cloudFull")?.exit).toBe("storage");
    expect(of("deferred-offline")?.said).toBe("stateOffline");
    expect(of("deferred-budget")?.said).toBe("stateBudget");
    expect(of("deferred-limit")?.said).toBe("stateWait");
    expect(of("deferred-busy")?.said).toBe("stateWait");
    expect(of("cloudNotNow")?.said).toBe("stateWait");
    expect(of("ok")).toBeNull();
    expect(of("deferred-paused")).toBeNull();
  });

  it("names the other account a copy belongs to", () => {
    const said = cloudStanding(
      status({ last: round("cloudElsewhere", { name: "otra@gmail.com" }) }),
    );
    expect(said?.name).toBe("otra@gmail.com");
  });
});

const choosing = (offered = true) => {
  const done = vi.fn();
  render(<Choosing busy={false} offered={offered} onTrouble={() => {}} onDone={done} />);
  return done;
};

describe("choosing where the copies go", () => {
  it("offers this machine, your cloud service and a folder, one at a time", () => {
    choosing();
    expect(screen.getByRole("button", { name: new RegExp(t("welcomeAlone"), "i") })).toBeTruthy();
    expect(screen.getByRole("button", { name: new RegExp(t("chooseCloud"), "i") })).toBeTruthy();
    expect(screen.getByRole("button", { name: new RegExp(t("chooseFolder"), "i") })).toBeTruthy();
    expect(screen.getByText(t("chooseOneAtATime"))).toBeTruthy();
  });

  it("leaves the cloud out of a build made without it", () => {
    choosing(false);
    expect(screen.queryByRole("button", { name: new RegExp(t("chooseCloud"), "i") })).toBeNull();
  });

  it("takes «only here» at once", async () => {
    const done = choosing();
    await userEvent.click(screen.getByRole("button", { name: new RegExp(t("welcomeAlone"), "i") }));
    await waitFor(() => expect(done).toHaveBeenCalledWith(undefined));
    expect(sent("choose_sync")[0].args.dest).toBeUndefined();
  });

  it("connects Google, says how much goes up, and starts on the cloud", async () => {
    ipc.answers.cloud_connect = () => Promise.resolve(ANA);
    ipc.answers.sync_state = () => Promise.resolve(carrying());
    const done = choosing();

    await userEvent.click(screen.getByRole("button", { name: new RegExp(t("chooseCloud"), "i") }));
    await userEvent.click(screen.getByRole("button", { name: t("cloudOpenGoogle") }));

    expect(await screen.findByText(t("cloudConnectedTitle"))).toBeTruthy();
    expect(screen.getByText(ANA.email)).toBeTruthy();
    expect(await screen.findByText(fill("cloudToUpload", "3.0 MB"))).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: t("cloudStart") }));

    await waitFor(() => expect(done).toHaveBeenCalledWith("cloud"));
    expect(sent("cloud_choose")[0].args).toEqual({ account: ANA.account, email: ANA.email });
  });

  it("shows the providers still to come without letting them be picked", async () => {
    choosing();
    await userEvent.click(screen.getByRole("button", { name: new RegExp(t("chooseCloud"), "i") }));
    expect(screen.queryByRole("button", { name: /dropbox/i })).toBeNull();
    expect(screen.getByText(t("cloudDropbox"))).toBeTruthy();
  });

  it("lets go of an account connected and then turned back from", async () => {
    ipc.answers.cloud_connect = () => Promise.resolve(ANA);
    choosing();
    await userEvent.click(screen.getByRole("button", { name: new RegExp(t("chooseCloud"), "i") }));
    await userEvent.click(screen.getByRole("button", { name: t("cloudOpenGoogle") }));
    await screen.findByText(t("cloudConnectedTitle"));

    await userEvent.click(screen.getByRole("button", { name: t("welcomeBack") }));

    expect(sent("cloud_forget")[0].args.account).toBe(ANA.account);
    expect(sent("cloud_choose")).toHaveLength(0);
  });

  it("says why the cloud could not be chosen and stays on the step", async () => {
    ipc.answers.cloud_connect = () => Promise.resolve(ANA);
    ipc.answers.cloud_choose = () => Promise.reject({ code: "cloudNeedsMove" });
    const trouble = vi.fn();
    render(<Choosing busy={false} offered onTrouble={trouble} onDone={() => {}} />);
    await userEvent.click(screen.getByRole("button", { name: new RegExp(t("chooseCloud"), "i") }));
    await userEvent.click(screen.getByRole("button", { name: t("cloudOpenGoogle") }));
    await userEvent.click(await screen.findByRole("button", { name: t("cloudStart") }));

    await waitFor(() => expect(trouble).toHaveBeenCalledWith(t("cloudNeedsMove")));
    expect(screen.getByRole("button", { name: t("cloudStart") })).toBeTruthy();
  });
});

const card = (over: Partial<CloudStatus> = {}) => {
  ipc.answers.cloud_status = () => Promise.resolve(status(over));
  const carry = vi.fn();
  const run = vi.fn((_card: string, work: Promise<unknown>, then: (answer: unknown) => void) => {
    work.then(then);
  });
  render(
    <Clouded
      state={carrying({ last: "2026-10-12T10:00:00Z" })}
      busy={null}
      run={run as never}
      fail={() => {}}
      carry={carry}
    />,
  );
  return { carry, run };
};

describe("the cloud in Settings", () => {
  it("shows the account it syncs with and syncs on demand", async () => {
    const { carry } = card();
    expect(await screen.findByText(ANA.email)).toBeTruthy();
    expect(screen.getByText(t("cloudConnectedTag"))).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: t("syncNow") }));
    expect(carry).toHaveBeenCalled();
  });

  it("turns the cloud off on this machine", async () => {
    card();
    await userEvent.click(await screen.findByRole("button", { name: t("cloudTurnOff") }));
    expect(sent("cloud_pause")[0].args.paused).toBe(true);
  });

  it("says it is off, holds the round back and offers to turn it on", async () => {
    card({ pausedHere: true });
    expect(await screen.findByText(t("stateOff"))).toBeTruthy();
    expect(screen.getByRole("button", { name: t("syncNow") }).hasAttribute("disabled")).toBe(true);
    const on = screen.getAllByRole("button", { name: t("cloudTurnOn") });
    await userEvent.click(on[0]);
    expect(sent("cloud_pause")[0].args.paused).toBe(false);
  });

  it("reconnects when Google no longer lets Tisty in, then syncs", async () => {
    ipc.answers.cloud_connect = () => Promise.resolve(ANA);
    const { carry } = card({ last: round("cloudLost") });
    await userEvent.click(await screen.findByRole("button", { name: t("stateReconnectDo") }));
    await waitFor(() => expect(carry).toHaveBeenCalled());
    expect(sent("cloud_choose")[0].args.account).toBe(ANA.account);
  });

  it("opens Google's storage page when the Drive is full", async () => {
    card({ last: round("cloudFull") });
    await userEvent.click(await screen.findByRole("button", { name: t("stateFullDo") }));
    expect(opened.urls).toEqual([STORAGE]);
  });

  it("names the other account a copy belongs to", async () => {
    card({ last: round("cloudElsewhere", { name: "otra@gmail.com" }) });
    expect(await screen.findByText(fill("stateElsewhere", "otra@gmail.com"))).toBeTruthy();
  });

  it("says what was left for later and which ones on asking", async () => {
    card({ last: round("deferred-budget", { left: 2, names: ["attachments/ab/plano.pdf"] }) });
    expect(await screen.findByText(t("stateBudget"))).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: t("cloudSeeWhich") }));
    expect(screen.getByText("plano.pdf")).toBeTruthy();
  });

  it("disconnects this machine only once the person is sure", async () => {
    card();
    opened.sure = false;
    await userEvent.click(await screen.findByRole("button", { name: t("cloudForgetHere") }));
    expect(sent("cloud_forget")).toHaveLength(0);
    opened.sure = true;
    await userEvent.click(screen.getByRole("button", { name: t("cloudForgetHere") }));
    await waitFor(() => expect(sent("cloud_forget")).toHaveLength(1));
  });

  it("removes the access on every machine only once the person is sure", async () => {
    card();
    await userEvent.click(await screen.findByRole("button", { name: t("cloudRevokeAll") }));
    await waitFor(() => expect(sent("cloud_revoke")).toHaveLength(1));
  });
});

describe("around the cloud", () => {
  it("says today's spend next to freeing space", async () => {
    ipc.answers.cloud_status = () => Promise.resolve(status({ unitsSpent: 120, unitsADay: 900 }));
    ipc.answers.unused_attachments = () => Promise.resolve({ files: 0, bytes: 0 });
    render(
      <FreeSpace
        busy={null}
        freeing={null}
        setFreeing={() => {}}
        days={30}
        setDays={() => {}}
        fail={() => {}}
      />,
    );
    expect(await screen.findByText(new RegExp(fill("cloudTodayUnits", "120", "900")))).toBeTruthy();
  });

  it("offers to choose again when the others moved away", async () => {
    const open = vi.fn();
    render(<Coming onOpen={open} />);
    act(() => ended({ code: "movedAway", name: "a folder" }));
    await userEvent.click(await screen.findByRole("button", { name: t("movedChoose") }));
    expect(open).toHaveBeenCalled();
    act(() => unstuck());
  });

  it("offers no way out for a stop that is not a move", async () => {
    render(<Coming onOpen={() => {}} />);
    act(() => ended({ code: "syncBroke", name: "x" }));
    expect(screen.queryByRole("button", { name: t("movedChoose") })).toBeNull();
    act(() => unstuck());
  });
});
