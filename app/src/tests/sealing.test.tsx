import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { SealShown, SealTold } from "../core";
import { asksSure } from "../sure";
import type { Word } from "../ui/Card";
import Sealing from "../ui/Sealing";

const WORDS = Array.from({ length: 24 }, (_, at) => `word${at + 1}`).join(" ");

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  shown: null as unknown,
  room: { free: 10_000_000_000, needed: 2_000_000 } as unknown,
  refuse: null as string | null,
  slow: false,
}));

const bus = vi.hoisted(() => ({
  heard: new Map<string, ((said: { payload: unknown }) => void)[]>(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    if (cmd === ipc.refuse) return Promise.reject({ code: "sealingBehind" });
    if (cmd === "sealing_begin" && ipc.slow) return new Promise(() => {});
    if (cmd === "sealing_shown") return Promise.resolve(ipc.shown);
    if (cmd === "sealing_room") return Promise.resolve(ipc.room);
    if (cmd === "sealing_wipe") return Promise.resolve(12);
    if (cmd === "sealing_heard") return Promise.resolve([]);
    if (cmd === "identity_begin" || cmd === "identity_phrase") return Promise.resolve(WORDS);
    if (cmd === "identity_shown")
      return Promise.resolve({ fingerprint: "11111 22222 33333 44444", machines: [] });
    return Promise.resolve(null);
  },
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: (named: string, fn: (said: { payload: unknown }) => void) => {
    bus.heard.set(named, [...(bus.heard.get(named) ?? []), fn]);
    return Promise.resolve(() => {});
  },
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  save: () => Promise.resolve(null),
  ask: () => Promise.resolve(true),
}));

const plain: SealShown = {
  cloud: "onedrive",
  state: "plain",
  joined: true,
  held: true,
  confirmed: true,
  phraseElsewhere: false,
  behind: [],
  turning: null,
  orphan: false,
};

const copying: SealTold = {
  on: true,
  step: "copying",
  done: 40,
  left: 60,
  refused: 0,
  stopped: false,
  why: null,
};

const sent = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

function Shell({ round, onPhrased }: { round?: string; onPhrased?: () => void }) {
  const [said, setSaid] = useState<Word>();
  const [trouble, setTrouble] = useState<Word>();
  return (
    <Sealing
      round={round}
      onPhrased={onPhrased}
      busy={null}
      said={said}
      trouble={trouble}
      tell={setSaid}
      fail={setTrouble}
      onChanged={() => {}}
    />
  );
}

const heard = (told: SealTold | null) =>
  act(() => {
    for (const fn of bus.heard.get("sealing") ?? []) fn({ payload: told });
  });

beforeEach(() => {
  ipc.calls.length = 0;
  ipc.shown = plain;
  ipc.room = { free: 10_000_000_000, needed: 2_000_000 };
  ipc.refuse = null;
  ipc.slow = false;
  bus.heard.clear();
  asksSure(() => Promise.resolve(true));
});

describe("encrypting the cloud", () => {
  it("says it is looking at the cloud until it knows, instead of showing nothing", async () => {
    ipc.shown = new Promise(() => {});
    render(<Shell />);

    expect((await screen.findByRole("status")).textContent).toBe("Checking…");
  });

  it("only says to connect a cloud while there is none", async () => {
    ipc.shown = { ...plain, cloud: null, state: "none" };
    render(<Shell />);

    expect(await screen.findByText(/Connect a cloud in Sync/)).toBeTruthy();
  });

  it("lists what has to stand before it begins, and begins once all of it does", async () => {
    render(<Shell />);
    await userEvent.click(await screen.findByRole("button", { name: "Encrypt my cloud…" }));

    expect(await screen.findByText("Your phrase is confirmed")).toBeTruthy();
    expect(await screen.findByText("There is room in OneDrive")).toBeTruthy();
    expect(screen.getByText("Your computers are up to date")).toBeTruthy();
    expect(screen.getByText(/keeps earlier unencrypted versions/)).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Encrypt my cloud" }));

    await waitFor(() => expect(sent("sealing_begin")).toHaveLength(1));
    expect(sent("sealing_begin")[0].args.on).toBe(true);
  });

  it("will not begin without a phrase, room or computers up to date", async () => {
    ipc.shown = { ...plain, held: false, confirmed: false, behind: ["PC de la oficina"] };
    ipc.room = { free: 1_000, needed: 2_000_000 };
    render(<Shell />);
    await userEvent.click(await screen.findByRole("button", { name: "Encrypt my cloud…" }));

    expect(await screen.findByText("You do not have your phrase yet")).toBeTruthy();
    expect(await screen.findByText("There is not enough room in OneDrive")).toBeTruthy();
    expect(screen.getByText("PC de la oficina runs an older Tisty")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Encrypt my cloud" }).hasAttribute("disabled")).toBe(
      true,
    );

    await userEvent.click(screen.getByRole("button", { name: "Create my phrase" }));
    expect(await screen.findByText("word24")).toBeTruthy();
    expect(sent("identity_begin")).toHaveLength(1);
  });

  it("asks a phrase not yet confirmed to be confirmed, and goes on when the room is not said", async () => {
    ipc.shown = { ...plain, confirmed: false };
    ipc.room = { free: null, needed: 2_000_000 };
    render(<Shell />);
    await userEvent.click(await screen.findByRole("button", { name: "Encrypt my cloud…" }));

    expect(await screen.findByText(/does not say how much room is left/)).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Confirm it" }));
    expect(await screen.findByText("word24")).toBeTruthy();
    expect(sent("identity_phrase")).toHaveLength(1);
    await userEvent.click(screen.getByRole("button", { name: "Close" }));
    await userEvent.click(screen.getByRole("button", { name: "Not now" }));
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("shows the phrase over the list it was asked from, and tells the identity card once it shuts", async () => {
    ipc.shown = { ...plain, held: false, confirmed: false };
    const phrased = vi.fn();
    render(<Shell onPhrased={phrased} />);
    await userEvent.click(await screen.findByRole("button", { name: "Encrypt my cloud…" }));
    await userEvent.click(await screen.findByRole("button", { name: "Create my phrase" }));

    expect(await screen.findByText("word24")).toBeTruthy();
    const dialogs = screen.getAllByRole("dialog");
    expect(dialogs[dialogs.length - 1].textContent).toContain("word24");
    await userEvent.click(screen.getByRole("button", { name: "Close" }));
    expect(phrased).toHaveBeenCalledTimes(1);
  });

  it("looks again after a round, so a computer that updated stops being waited for", async () => {
    ipc.shown = { ...plain, behind: ["MacBook Pro"] };
    const { rerender } = render(<Shell round="12:30" />);
    await userEvent.click(await screen.findByRole("button", { name: "Encrypt my cloud…" }));
    expect(await screen.findByText("MacBook Pro runs an older Tisty")).toBeTruthy();

    ipc.shown = plain;
    rerender(<Shell round="12:31" />);

    expect(await screen.findByText("Your computers are up to date")).toBeTruthy();
  });

  it("says why it could not begin", async () => {
    ipc.refuse = "sealing_begin";
    render(<Shell />);
    await userEvent.click(await screen.findByRole("button", { name: "Encrypt my cloud…" }));
    await screen.findByText("There is room in OneDrive");

    await userEvent.click(screen.getByRole("button", { name: "Encrypt my cloud" }));

    expect(await screen.findByText(/still run an older Tisty/)).toBeTruthy();
  });
});

describe("a turn under way", () => {
  it("says the cloud is being made ready from the moment it is asked to begin", async () => {
    ipc.slow = true;
    render(<Shell />);
    await userEvent.click(await screen.findByRole("button", { name: "Encrypt my cloud…" }));
    await screen.findByText("There is room in OneDrive");

    await userEvent.click(screen.getByRole("button", { name: "Encrypt my cloud" }));

    expect((await screen.findByRole("status")).textContent).toMatch(/Getting your cloud ready/);
    expect(screen.queryByRole("button", { name: "Encrypt my cloud…" })).toBeNull();
  });

  it("says why it waits, and that it goes on by itself, without offering to try again", async () => {
    ipc.shown = { ...plain, state: "turning", turning: copying };
    render(<Shell />);
    await screen.findByText("Copy");

    await heard({ ...copying, waiting: "offline" });
    expect(screen.getByRole("status").textContent).toMatch(/connection is back/);
    expect(screen.queryByRole("button", { name: "Try again" })).toBeNull();

    await heard({ ...copying, waiting: "local" });
    expect(screen.getByRole("status").textContent).toMatch(/antivirus/);

    await heard({ ...copying, waiting: "busy" });
    expect(screen.getByRole("status").textContent).toMatch(/already talking to your cloud/);

    await heard({ ...copying, waiting: null });
    expect(screen.queryByRole("status")).toBeNull();
  });

  it("says how many changes are left to bring over, and not that others are waiting", async () => {
    ipc.shown = {
      ...plain,
      state: "turning",
      turning: { ...copying, step: "settling", done: 0, left: 3 },
    };
    render(<Shell />);

    expect(await screen.findByText("3 changes left to bring over")).toBeTruthy();
    expect(screen.queryByText(/Waiting a few minutes/)).toBeNull();
  });

  it("says the copy is starting before the first numbers, and says a stop with no reason", async () => {
    ipc.shown = { ...plain, state: "turning", turning: { ...copying, done: 0, left: 0 } };
    render(<Shell />);

    expect(await screen.findByText("Starting to copy…")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Try again" })).toBeNull();

    await heard({ ...copying, done: 0, left: 0, stopped: true, why: null });
    expect(screen.getByRole("alert").textContent).toMatch(/It stopped/);
    expect(screen.getByRole("button", { name: "Try again" })).toBeTruthy();
  });

  it("shows how far the copy went, follows what the turn says and can be cancelled", async () => {
    ipc.shown = { ...plain, state: "turning", turning: copying };
    render(<Shell />);

    expect(await screen.findByText("Encrypting your cloud")).toBeTruthy();
    expect(screen.getByText("40 files copied, 60 left")).toBeTruthy();
    await heard({ ...copying, step: "settling", refused: 2 });
    expect(screen.getByText(/2 files are too big to encrypt/)).toBeTruthy();
    await heard({ ...copying, stopped: true, why: "cloudOtherPhrase" });
    expect(screen.getByRole("alert").textContent).toMatch(/different 24-word phrase/);
    expect(screen.getByText("40 files copied, 60 left")).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Try again" }));
    await waitFor(() => expect(sent("sealing_resume")).toHaveLength(1));

    await userEvent.click(screen.getByRole("button", { name: "Cancel" }));
    await waitFor(() => expect(sent("sealing_cancel")).toHaveLength(1));
  });

  it("looks again once the turn is done", async () => {
    ipc.shown = { ...plain, state: "turning", turning: { ...copying, on: false } };
    render(<Shell />);
    expect(await screen.findByText("Decrypting your cloud")).toBeTruthy();
    ipc.shown = { ...plain, state: "sealed" };

    await heard({ ...copying, step: "done" });

    expect(await screen.findByText("Your cloud is encrypted")).toBeTruthy();
  });
});

describe("an encrypted cloud", () => {
  it("deletes the old copy and can be decrypted", async () => {
    ipc.shown = { ...plain, state: "sealed" };
    render(<Shell />);
    expect(await screen.findByText(/old unencrypted copy stays in OneDrive/)).toBeTruthy();

    await userEvent.click(screen.getByRole("button", { name: "Delete the unencrypted copy" }));
    expect(await screen.findByText("12 old files deleted.")).toBeTruthy();

    await userEvent.click(screen.getByRole("button", { name: "Decrypt my cloud" }));
    await waitFor(() => expect(sent("sealing_begin")[0]?.args.on).toBe(false));
  });

  it("leaves another machine's turn alone until it has stood still for a week", async () => {
    ipc.shown = { ...plain, state: "elsewhere" };
    const { unmount } = render(<Shell />);
    expect(await screen.findByText(/Another of your computers is encrypting/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Cancel it from here" })).toBeNull();
    unmount();

    ipc.shown = { ...plain, state: "elsewhere", orphan: true };
    render(<Shell />);
    await userEvent.click(await screen.findByRole("button", { name: "Cancel it from here" }));
    await waitFor(() => expect(sent("sealing_cancel")).toHaveLength(1));
  });
});

describe("what stands in the way", () => {
  it("never offers to make a phrase that lives on another computer", async () => {
    ipc.shown = { ...plain, held: false, confirmed: false, phraseElsewhere: true };
    render(<Shell />);
    await userEvent.click(await screen.findByRole("button", { name: "Encrypt my cloud…" }));

    expect(await screen.findByText(/Your phrase is on another of your computers/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Create my phrase" })).toBeNull();
  });

  it("says the room could not be looked at rather than calling it enough", async () => {
    ipc.refuse = "sealing_room";
    render(<Shell />);
    await userEvent.click(await screen.findByRole("button", { name: "Encrypt my cloud…" }));

    expect(await screen.findByText(/could not look at the room in OneDrive/)).toBeTruthy();
    expect(screen.getByRole("button", { name: "Encrypt my cloud" }).hasAttribute("disabled")).toBe(
      true,
    );
  });
});

describe("a cloud this computer cannot open yet", () => {
  it("says a computer on the list is handed the key on its own, with nothing to ask", async () => {
    ipc.shown = { ...plain, state: "locked" };
    render(<Shell />);

    expect(await screen.findByText(/hand it over on their own/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Let in from another computer" })).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: "Type my phrase" }));
    await waitFor(() => expect(sent("sealing_heard")).toHaveLength(1));
  });

  it("lets a computer new to the place in from another one", async () => {
    ipc.shown = { ...plain, state: "locked", joined: false };
    render(<Shell />);

    expect(await screen.findByText(/let this computer in from another/)).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Let in from another computer" }));

    await waitFor(() => expect(sent("admitting_ask")).toHaveLength(1));
  });

  it("offers to let a new computer in once the cloud is encrypted", async () => {
    ipc.shown = { ...plain, state: "sealed" };
    render(<Shell />);

    await userEvent.click(await screen.findByRole("button", { name: "Let a new computer in" }));

    await waitFor(() => expect(sent("admitting_look")).toHaveLength(1));
  });
});
