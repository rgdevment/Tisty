import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ended, unstuck } from "../coming";
import type { Carrying, CloudStatus, SealShown } from "../core";
import { adopt, speakOf, t } from "../locales";
import { plainly } from "../refusal";
import { refresh } from "../syncSeen";
import Clouded from "../ui/Clouded";
import Coming from "../ui/Coming";
import Security from "../ui/Security";
import type { Cards } from "../ui/useCards";
import { held } from "../ui/working";
import { LiveRows } from "./rowsShell";

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

vi.mock("@tauri-apps/api/event", () => ({ listen: () => Promise.resolve(() => {}) }));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: () => Promise.resolve() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: () => Promise.resolve(null),
  ask: () => Promise.resolve(true),
  save: () => Promise.resolve(null),
}));

const MARIO = { provider: "onedrive", account: "0123", email: "mario@example.com" };
const SERVER = { provider: "webdav", account: "https://dav/mario", email: "mario@dav" };

const round = (code: string) => ({ at: 1_800_000_000, code, name: null, left: 0, names: [] });

const now = {
  status: {} as CloudStatus,
  seal: {} as SealShown,
  identity: {} as Record<string, unknown>,
};

const status = (over: Partial<CloudStatus> = {}) => {
  now.status = {
    ...MARIO,
    joined: true,
    unitsSpent: 0,
    unitsADay: 0,
    bytesSpent: 0,
    bytesADay: 0,
    pausedHere: false,
    owed: 0,
    last: round("cloudSealed"),
    ...over,
  };
};

const seal = (over: Partial<SealShown> = {}) => {
  now.seal = {
    cloud: "onedrive",
    state: "locked",
    joined: true,
    held: false,
    confirmed: false,
    phraseElsewhere: true,
    behind: [],
    turning: null,
    orphan: false,
    ...over,
  };
};

const state = {
  asked: true,
  open: 0,
  archived: 0,
  lists: 0,
  attachments: 0,
  weight: 0,
  carries: 0,
  mode: "cloud",
  cloud: MARIO.email,
  cloudProvider: "onedrive",
  cloudOffered: true,
  clouds: ["google", "onedrive", "webdav"],
  freesSpace: true,
} as Carrying;

const sent = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

beforeEach(async () => {
  ipc.calls = [];
  status({ lost: true });
  seal();
  now.identity = { held: false, elsewhere: true, fingerprint: null, machines: [] };
  ipc.answers = {
    cloud_status: () => Promise.resolve(now.status),
    sealing_shown: () => Promise.resolve(now.seal),
    sealing_heard: () => Promise.resolve([]),
    identity_shown: () => Promise.resolve(now.identity),
    sync_state: () => Promise.resolve(state),
    cloud_connect: () => Promise.resolve(MARIO),
  };
  adopt("en");
  speakOf("onedrive");
  await refresh();
});

afterEach(() => {
  act(() => unstuck());
  speakOf(null);
});

const clouded = (carrying: (cards: Cards) => void = () => {}) => {
  const carry = vi.fn();
  let cards: Cards | undefined;
  const { unmount } = render(
    <LiveRows>
      {(live) => {
        cards = live;
        return (
          <Clouded
            state={state}
            busy={live.busy}
            run={live.run}
            fail={live.fail}
            carry={() => {
              // A round asked for while other work holds the cloud is turned away in silence, as useCarry does.
              if (held(live.busy, "sync")) return;
              carry();
              carrying(live);
            }}
            onMove={() => {}}
          />
        );
      }}
    </LiveRows>,
  );
  return { carry, unmount, cards: () => cards as Cards };
};

const security = () => {
  const carry = vi.fn();
  render(
    <LiveRows>
      {(live) => (
        <Security
          state={state}
          audit={null}
          setAudit={() => {}}
          busy={live.busy}
          run={live.run}
          tell={live.tell}
          fail={live.fail}
          clear={live.clear}
          onChanged={() => {}}
          carry={() => {
            if (!held(live.busy, "sync")) carry();
          }}
        />
      )}
    </LiveRows>,
  );
  return { carry };
};

const steps = async () => within(await screen.findByRole("list")).getAllByRole("listitem");

describe("the sync card of an encrypted cloud whose access is gone", () => {
  it("asks to reconnect, in order, and never calls the cloud only encrypted", async () => {
    clouded();

    const [first, second] = await steps();
    expect(screen.getByText(t("stateReconnect"))).toBeTruthy();
    expect(screen.queryByText(t("stateSealed"))).toBeNull();
    expect(first.textContent).toContain(t("backReconnect"));
    expect(within(first).getByRole("button", { name: t("stateReconnectDo") })).toBeTruthy();
    expect(second.textContent).toContain(t("backAfter"));
    expect(within(second).queryByRole("button")).toBeNull();
    expect(screen.queryByText(/on their own/)).toBeNull();
    expect(screen.getAllByRole("button", { name: t("stateReconnectDo") })).toHaveLength(1);
  });

  it("moves on to the phrase by itself once reconnected, with no round to tell it yet", async () => {
    const { carry } = clouded(() => status({ lost: false, last: null }));

    const [first] = await steps();
    await userEvent.click(within(first).getByRole("button"));

    await waitFor(() => expect(carry).toHaveBeenCalled());
    expect(sent("cloud_choose")[0].args.account).toBe(MARIO.account);
    expect(await screen.findByText(t("stateSealed"))).toBeTruthy();
    const [done, next] = await steps();
    expect(done.textContent).toContain(t("backConnected"));
    expect(done.textContent).toContain(t("backDone"));
    expect(next.textContent).toContain(t("sealLockedJoined"));
    expect(within(next).getByRole("button", { name: t("enterButton") })).toBeTruthy();
    expect(screen.queryByText(t("cloudConnectedTag"))).toBeNull();
  });

  it("opens the cloud from the card and runs a round by itself", async () => {
    status({ lost: false });
    const { carry } = clouded();
    const [, second] = await steps();
    await userEvent.click(within(second).getByRole("button", { name: t("enterButton") }));
    await userEvent.type(await screen.findByRole("textbox"), "abandon");

    await userEvent.click(screen.getByRole("button", { name: t("enterGo") }));

    await waitFor(() => expect(carry).toHaveBeenCalledTimes(1));
    expect(sent("sealing_phrase")[0].args.said).toBe("abandon");
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("goes back to the first step when the phrase finds the door shut", async () => {
    status({ lost: false });
    ipc.answers.sealing_phrase = () => Promise.reject({ code: "cloudLost", name: "OneDrive" });
    ipc.answers.cloud_connect = () => Promise.reject({ code: "cloudDenied", name: "OneDrive" });
    clouded();
    const [, second] = await steps();
    await userEvent.click(within(second).getByRole("button", { name: t("enterButton") }));
    await userEvent.type(await screen.findByRole("textbox"), "abandon");
    await userEvent.click(screen.getByRole("button", { name: t("enterGo") }));

    const dialog = await screen.findByRole("dialog");
    await userEvent.click(
      await within(dialog).findByRole("button", { name: t("stateReconnectDo") }),
    );

    expect(await screen.findByText(t("stateReconnect"))).toBeTruthy();
    expect(sent("cloud_connect")).toHaveLength(1);
    const [first] = await steps();
    expect(first.textContent).toContain(t("backReconnect"));
  });

  it("keeps the latest look when an older one answers after it", async () => {
    const late: (() => void)[] = [];
    const { cards } = clouded();
    await steps();
    const stale = { ...now.status, lost: true };
    ipc.answers.cloud_status = () => new Promise((done) => late.push(() => done(stale)));
    act(() => cards().start("sync"));
    status({ lost: false });
    ipc.answers.cloud_status = () => Promise.resolve(now.status);
    act(() => {
      cards().fail({ card: "sync", text: plainly({ code: "cloudSealed" }) });
      cards().stop("sync");
    });
    expect(await screen.findByText(t("stateSealed"))).toBeTruthy();

    await act(async () => {
      for (const answer of late) answer();
    });

    expect(screen.getByText(t("stateSealed"))).toBeTruthy();
    expect(await screen.findByText(t("syncStill"))).toBeTruthy();
  });

  it("lets go of a door found shut once a round goes well", async () => {
    status({ lost: false });
    ipc.answers.sealing_phrase = () => Promise.reject({ code: "cloudLost", name: "OneDrive" });
    ipc.answers.cloud_connect = () => Promise.reject({ code: "cloudDropped" });
    const { cards } = clouded();
    const [, second] = await steps();
    await userEvent.click(within(second).getByRole("button", { name: t("enterButton") }));
    await userEvent.type(await screen.findByRole("textbox"), "abandon");
    await userEvent.click(screen.getByRole("button", { name: t("enterGo") }));
    const dialog = await screen.findByRole("dialog");
    await userEvent.click(
      await within(dialog).findByRole("button", { name: t("stateReconnectDo") }),
    );
    expect(await screen.findByText(t("stateReconnect"))).toBeTruthy();

    status({ lost: false, last: round("ok") });
    seal({ state: "sealed" });
    act(() => cards().start("sync"));
    act(() => cards().stop("sync"));

    expect(await screen.findByText(t("cloudConnectedTag"))).toBeTruthy();
    expect(screen.queryByText(t("stateReconnect"))).toBeNull();
  });

  it("marks the key as had when only the access is missing", async () => {
    seal({ state: "sealed" });
    clouded();

    const [first, second] = await steps();
    expect(within(first).getByRole("button")).toBeTruthy();
    expect(second.textContent).toContain(t("backKeyed"));
  });

  it("lets go of the last round's word once the key came between rounds", async () => {
    status({ lost: false });
    seal({ state: "sealed" });
    clouded();

    expect(await screen.findByText(t("cloudConnectedTag"))).toBeTruthy();
    expect(screen.queryByText(t("stateSealed"))).toBeNull();
  });

  it("keeps the single button of a cloud that is not encrypted", async () => {
    seal({ state: "plain" });
    clouded();

    expect(await screen.findByRole("button", { name: t("stateReconnectDo") })).toBeTruthy();
    await waitFor(() => expect(sent("sealing_shown").length).toBeGreaterThan(0));
    expect(screen.queryByText(t("backTitle"))).toBeNull();
  });
});

describe("a round turned away by another just after reconnecting", () => {
  const busy = (cards: Cards) => {
    status({ lost: false, last: null });
    cards.tell({ card: "sync", text: t("syncBusy"), busy: true });
  };

  it("is no error, and runs again by itself once the other ends", async () => {
    const { carry } = clouded(busy);
    const [first] = await steps();
    await userEvent.click(within(first).getByRole("button"));
    await waitFor(() => expect(carry).toHaveBeenCalledTimes(1));
    await screen.findByText(t("syncBusy"));
    expect(document.querySelector("[data-note=err]")).toBeNull();

    act(() => ended(null));

    await waitFor(() => expect(carry).toHaveBeenCalledTimes(2));
  });

  it("runs again after a while when nothing says the other ended, and stops once it goes well", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    try {
      let turn = 0;
      const { carry } = clouded((cards) => {
        turn += 1;
        if (turn < 3) busy(cards);
        else cards.tell({ card: "sync", text: t("syncSame") });
      });
      const [first] = await steps();
      await userEvent.click(within(first).getByRole("button"));
      await waitFor(() => expect(carry).toHaveBeenCalledTimes(1));

      await act(() => vi.advanceTimersByTimeAsync(10_100));
      expect(carry).toHaveBeenCalledTimes(2);
      await act(() => vi.advanceTimersByTimeAsync(10_100));
      expect(carry).toHaveBeenCalledTimes(3);
      await act(() => vi.advanceTimersByTimeAsync(30_000));
      expect(carry).toHaveBeenCalledTimes(3);
    } finally {
      vi.useRealTimers();
    }
  });

  it("waits for other work to end before running again", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    try {
      const { carry, cards } = clouded(busy);
      const [first] = await steps();
      await userEvent.click(within(first).getByRole("button"));
      await waitFor(() => expect(carry).toHaveBeenCalledTimes(1));
      act(() => cards().start("backup"));

      await act(() => vi.advanceTimersByTimeAsync(10_100));
      expect(carry).toHaveBeenCalledTimes(1);
      act(() => cards().stop("backup"));
      await act(() => vi.advanceTimersByTimeAsync(10_100));
      expect(carry).toHaveBeenCalledTimes(2);
    } finally {
      vi.useRealTimers();
    }
  });

  it("still runs the round when the page was left while reconnecting", async () => {
    let done = () => {};
    ipc.answers.cloud_choose = () =>
      new Promise((ok) => {
        done = () => ok(null);
      });
    const { carry, unmount } = clouded();
    const [first] = await steps();
    await userEvent.click(within(first).getByRole("button"));
    await screen.findByText(t("cloudSettling"));
    unmount();

    await act(async () => done());

    await waitFor(() => expect(carry).toHaveBeenCalledTimes(1));
  });

  it("leaves a round asked for by hand as it was", async () => {
    status({ lost: false });
    const { carry } = clouded(busy);
    await steps();
    await userEvent.click(screen.getByRole("button", { name: t("syncNow") }));

    act(() => ended(null));

    expect(carry).toHaveBeenCalledTimes(1);
  });
});

describe("a round refused at once", () => {
  const refused = (cards: Cards) => {
    cards.start("sync");
    cards.clear("sync");
    void Promise.resolve().then(() => {
      cards.fail({ card: "sync", text: plainly({ code: "cloudSealed" }) });
      cards.stop("sync");
    });
  };

  it("is seen to run, then says once that it could not", async () => {
    status({ lost: false });
    clouded(refused);
    await steps();

    await userEvent.click(screen.getByRole("button", { name: t("syncNow") }));

    expect(screen.getByRole("button", { name: t("syncing_") })).toBeTruthy();
    expect(screen.queryByText(t("syncStill"))).toBeNull();
    expect(screen.queryByText(plainly({ code: "cloudSealed" }))).toBeNull();
    expect(await screen.findByText(t("syncStill"))).toBeTruthy();
    expect(screen.getByRole("button", { name: t("syncNow") })).toBeTruthy();
    expect(screen.queryByText(plainly({ code: "cloudSealed" }))).toBeNull();
  });

  it("keeps the red still while another card works", async () => {
    status({ lost: false });
    const { cards } = clouded();
    await steps();
    act(() => cards().fail({ card: "sync", text: t("cloudDenied") }));
    expect(screen.getByText(t("cloudDenied"))).toBeTruthy();
    ipc.answers.cloud_status = () => new Promise(() => {});

    act(() => cards().start("backup"));

    expect(screen.getByText(t("cloudDenied"))).toBeTruthy();
  });

  it("still says in red what the card does not", async () => {
    status({ lost: false });
    clouded((cards) => cards.fail({ card: "sync", text: t("cloudDenied") }));
    await steps();

    await userEvent.click(screen.getByRole("button", { name: t("syncNow") }));

    expect(await screen.findByText(t("cloudDenied"))).toBeTruthy();
    expect(screen.queryByText(t("syncStill"))).toBeNull();
  });
});

describe("the encryption card of a cloud whose access is gone", () => {
  it("asks to reconnect, with no promise that another computer hands the key over", async () => {
    security();

    const [first] = await steps();
    expect(screen.getByText(t("stateReconnect"))).toBeTruthy();
    expect(within(first).getByRole("button", { name: t("stateReconnectDo") })).toBeTruthy();
    expect(screen.queryByText(t("sealAskingKey"))).toBeNull();
    expect(screen.queryByText(/on their own/)).toBeNull();
    expect(screen.queryByText(t("sealLocked"))).toBeNull();
    expect(screen.queryByRole("button", { name: t("admitAskButton") })).toBeNull();
  });

  it("is not said again in the sidebar while the card has its way out", async () => {
    act(() => ended({ code: "cloudSealed" }));
    security();
    render(<Coming />);

    await steps();
    await waitFor(() => expect(screen.queryByRole("alert")).toBeNull());
  });

  it("reconnects there, runs a round and asks for the phrase next", async () => {
    const { carry } = security();
    const [first] = await steps();
    ipc.answers.cloud_choose = () => {
      status({ lost: false, last: null });
      return Promise.resolve(null);
    };

    await userEvent.click(within(first).getByRole("button"));

    await waitFor(() => expect(carry).toHaveBeenCalled());
    expect(await screen.findByText(t("sealLocked"))).toBeTruthy();
    expect(screen.getByText(t("sealAskingKey"))).toBeTruthy();
  });

  it("says there why the sign-in did not go through", async () => {
    ipc.answers.cloud_connect = () => Promise.reject({ code: "cloudDenied", name: "OneDrive" });
    security();
    const [first] = await steps();

    await userEvent.click(within(first).getByRole("button"));

    expect(await screen.findByText(/OneDrive was not given access/)).toBeTruthy();
  });

  it("waits for the browser beside the steps and lets the person stop it", async () => {
    let drop = () => {};
    ipc.answers.cloud_connect = () =>
      new Promise((_, no) => {
        drop = () => no({ code: "cloudDropped" });
      });
    const { carry } = security();
    const [first] = await steps();
    await userEvent.click(within(first).getByRole("button"));

    await userEvent.click(await screen.findByRole("button", { name: t("cancel") }));
    act(() => drop());

    await waitFor(() => expect(screen.queryByRole("button", { name: t("cancel") })).toBeNull());
    expect(sent("cloud_connect_cancel")).toHaveLength(1);
    expect(carry).not.toHaveBeenCalled();
  });

  it("stops offering to stop once the sign-in went through, and says the cloud is getting ready", async () => {
    let done = () => {};
    ipc.answers.cloud_choose = () =>
      new Promise((ok) => {
        done = () => {
          status({ lost: false, last: null });
          ok(null);
        };
      });
    const { carry } = security();
    const [first] = await steps();

    await userEvent.click(within(first).getByRole("button"));

    expect(await screen.findByText(t("cloudSettling"))).toBeTruthy();
    expect(screen.queryByText(new RegExp(t("cloudWaiting")))).toBeNull();
    expect(screen.queryByRole("button", { name: t("cancel") })).toBeNull();
    act(() => done());
    await waitFor(() => expect(carry).toHaveBeenCalled());
    expect(screen.queryByText(t("cloudSettling"))).toBeNull();
  });

  it("says nothing is getting ready when the account could not be taken", async () => {
    ipc.answers.cloud_choose = () => Promise.reject({ code: "cloudNeedsMove" });
    security();
    const [first] = await steps();

    await userEvent.click(within(first).getByRole("button"));

    expect(await screen.findByText(t("cloudNeedsMove"))).toBeTruthy();
    expect(screen.queryByText(t("cloudSettling"))).toBeNull();
  });

  it("asks only for the password of a server signed in to with one", async () => {
    status({ ...SERVER, lost: true });
    await refresh();
    ipc.answers.cloud_connect_dav = () => Promise.resolve(SERVER);
    let done = () => {};
    ipc.answers.cloud_choose = () =>
      new Promise((ok) => {
        done = () => ok(null);
      });
    const { carry } = security();
    const [first] = await steps();
    await userEvent.click(within(first).getByRole("button"));
    await userEvent.type(screen.getByLabelText(new RegExp(`^${t("davKey")}`)), "r0ck4me!");
    await userEvent.click(screen.getByRole("button", { name: t("davConnect") }));

    expect(await screen.findByText(t("cloudSettling"))).toBeTruthy();
    expect(screen.queryByRole("button", { name: t("cancel") })).toBeNull();
    act(() => done());
    await waitFor(() => expect(carry).toHaveBeenCalled());
    expect(sent("cloud_connect_dav")[0].args.account).toBe(SERVER.account);
  });

  it("never mixes in another account than the one this computer carried", async () => {
    ipc.answers.cloud_connect = () => Promise.resolve({ ...MARIO, account: "9999" });
    const { carry } = security();
    const [first] = await steps();

    await userEvent.click(within(first).getByRole("button"));

    await waitFor(() => expect(sent("cloud_forget")[0]?.args.account).toBe("9999"));
    expect(sent("cloud_choose")).toHaveLength(0);
    expect(carry).not.toHaveBeenCalled();
  });

  it("offers no way in to a cloud this build was made without", async () => {
    status({ ...MARIO, provider: "dropbox", lost: true });
    await refresh();
    security();

    expect(await screen.findByText(t("cloudNotOffered"))).toBeTruthy();
    expect(screen.queryByText(t("backTitle"))).toBeNull();
    expect(screen.queryByText(t("sealAskingKey"))).toBeNull();
  });

  it("goes back to reconnecting when the phrase finds the door shut", async () => {
    status({ lost: false });
    await refresh();
    ipc.answers.sealing_phrase = () => Promise.reject({ code: "cloudLost", name: "OneDrive" });
    ipc.answers.cloud_connect = () => Promise.reject({ code: "cloudDenied", name: "OneDrive" });
    security();
    await userEvent.click(await screen.findByRole("button", { name: t("enterButton") }));
    await userEvent.type(await screen.findByRole("textbox"), "abandon");
    await userEvent.click(screen.getByRole("button", { name: t("enterGo") }));
    await userEvent.click(await screen.findByRole("button", { name: t("stateReconnectDo") }));

    expect(await screen.findByText(t("stateReconnect"))).toBeTruthy();
    expect(screen.queryByText(t("sealAskingKey"))).toBeNull();
  });
});

describe("the two places a phrase is typed", () => {
  beforeEach(async () => {
    status({ lost: false });
    await refresh();
  });

  it("say apart what each keeps, and each points to the other while both are there", async () => {
    security();

    expect(await screen.findByRole("button", { name: t("phraseTypeOpen") })).toBeTruthy();
    expect(await screen.findByRole("button", { name: t("enterButton") })).toBeTruthy();
    expect(screen.getByText(t("phraseTypeWhat"))).toBeTruthy();
    expect(screen.getByText(t("phraseTypeOther"))).toBeTruthy();
  });

  it("name no other place once the cloud opens here", async () => {
    seal({ state: "sealed" });
    security();

    expect(await screen.findByText(t("phraseTypeWhat"))).toBeTruthy();
    await screen.findByText(t("sealSealed"));
    expect(screen.queryByText(t("phraseTypeOther"))).toBeNull();
  });

  it("run a round by themselves once the phrase is kept", async () => {
    const { carry } = security();
    await userEvent.click(await screen.findByRole("button", { name: t("phraseTypeOpen") }));
    await userEvent.type(screen.getByRole("textbox"), "abandon");

    await userEvent.click(screen.getByRole("button", { name: t("phraseTypeGo") }));

    await waitFor(() => expect(carry).toHaveBeenCalledTimes(1));
    expect(sent("identity_recover")[0].args.said).toBe("abandon");
  });

  it("run a round by themselves once the cloud is opened", async () => {
    const { carry } = security();
    await userEvent.click(await screen.findByRole("button", { name: t("enterButton") }));
    await userEvent.type(await screen.findByRole("textbox"), "abandon");

    await userEvent.click(screen.getByRole("button", { name: t("enterGo") }));

    await waitFor(() => expect(carry).toHaveBeenCalledTimes(1));
  });
});
