import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { adopt, fill, t } from "../locales";
import { useSharedRow } from "../ui/sharedRow";

const invoked = vi.hoisted(() => vi.fn());
const copying = vi.hoisted(() => vi.fn(() => Promise.resolve()));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invoked }));
vi.mock("../core", async (original) => ({
  ...(await original<typeof import("../core")>()),
  copied: copying,
}));

const card = { read: "https://tisty.cl/a#k", edit: null, version: 1, taken: false };
const status = { diesOn: 20_000, paused: false, blocked: false, reads: 3 };

const answering = (answers: Record<string, unknown>) =>
  invoked.mockImplementation((command: string) =>
    command in answers
      ? Promise.resolve(answers[command])
      : Promise.reject({ code: "internal", command }),
  );

const held = async (cardNow: unknown, answers: Record<string, unknown> = {}, onError = vi.fn()) => {
  answering({ doc_share_card: cardNow, ...answers });
  const hook = renderHook(() => useSharedRow("d1", onError));
  await waitFor(() => expect(invoked).toHaveBeenCalledWith("doc_share_card", { id: "d1" }));
  await act(async () => {});
  const panel = () => {
    const now = hook.result.current.panel;
    if (now?.kind !== "held") throw new Error("no row");
    return now;
  };
  const pressed = async (key: string) => {
    await act(async () => {
      panel()
        .acts?.find((one) => one.key === key)
        ?.onPick?.(document.body);
    });
  };
  return { hook, panel, pressed, onError };
};

beforeEach(() => {
  invoked.mockReset();
  copying.mockClear();
  adopt("en");
});

describe("the Shared row", () => {
  it("offers nothing without a document", () => {
    answering({});
    const { result } = renderHook(() => useSharedRow(undefined));

    expect(result.current.panel).toBeNull();
  });

  it("offers to share what is not shared yet", async () => {
    const { hook, panel, pressed } = await held(null);

    expect(panel().gist).toBe(t("sharedNot"));
    await pressed("share");

    expect(hook.result.current.asking).toBe(true);
  });

  it("shows the link to read and copies it, and no link to edit unless it was made", async () => {
    const { panel } = await held(card);

    expect(panel().gist).toBe(t("sharedReadOnly"));
    expect(panel().links?.map((one) => one.key)).toEqual(["read"]);
    panel().links?.[0].onCopy();
    expect(copying).toHaveBeenCalledWith(card.read);
  });

  it("names what can be edited and its second link", async () => {
    const { panel } = await held({ ...card, edit: "https://tisty.cl/a#k.t" });

    expect(panel().gist).toBe(t("sharedCanEdit"));
    expect(panel().links?.map((one) => one.key)).toEqual(["read", "edit"]);
  });

  it.each([
    [{ state: "unchanged" }, () => t("shareSyncUnchanged")],
    [{ state: "sent", version: 2 }, () => fill("shareSyncSent", "2")],
    [
      { state: "otherVersion", version: 3, signer: "Ana" },
      () => fill("shareSyncOther", "3", "Ana"),
    ],
  ])("says what Sync did: %j", async (synced, words) => {
    const { panel, pressed } = await held(card, { doc_share_sync: synced });

    await pressed("sync");

    await waitFor(() => expect(panel().said).toContain(words()));
  });

  it("waits as long as Folio asks, with the button off", async () => {
    const until = Date.now() + 2 * 60_000;
    invoked.mockImplementation((command: string) =>
      command === "doc_share_card"
        ? Promise.resolve(card)
        : Promise.reject({ code: "folioWait", name: String(until) }),
    );
    const { result } = renderHook(() => useSharedRow("d1"));
    await waitFor(() => expect(result.current.panel).toMatchObject({ open: true }));
    await waitFor(() => {
      const now = result.current.panel;
      expect(now?.kind === "held" && now.links?.length).toBe(1);
    });

    await act(async () => {
      const now = result.current.panel;
      if (now?.kind === "held")
        now.acts?.find((one) => one.key === "sync")?.onPick?.(document.body);
    });

    await waitFor(() => {
      const now = result.current.panel;
      const sync = now?.kind === "held" ? now.acts?.find((one) => one.key === "sync") : undefined;
      expect(sync?.off).toBe(true);
      expect(sync?.label).toBe(fill("shareSyncWait", "2"));
    });
  });

  it("asks for the terms again when Folio changed them", async () => {
    invoked.mockImplementation((command: string) =>
      command === "doc_share_card"
        ? Promise.resolve(card)
        : Promise.reject({ code: "folioTermsToAccept" }),
    );
    const onError = vi.fn();
    const { result } = renderHook(() => useSharedRow("d1", onError));
    await waitFor(() => {
      const now = result.current.panel;
      expect(now?.kind === "held" && now.links?.length).toBe(1);
    });

    await act(async () => {
      const now = result.current.panel;
      if (now?.kind === "held")
        now.acts?.find((one) => one.key === "sync")?.onPick?.(document.body);
    });

    await waitFor(() => expect(result.current.asking).toBe(true));
    expect(onError).not.toHaveBeenCalled();
  });

  it("tells any other refusal to whoever shows it", async () => {
    const { onError, pressed } = await held(card);

    await pressed("sync");

    await waitFor(() => expect(onError).toHaveBeenCalled());
  });

  it("shows expiry and reads only once asked", async () => {
    const { panel, pressed } = await held(card, { doc_share_status: status });

    expect(panel().said).toEqual([]);
    await pressed("status");

    await waitFor(() => expect(panel().said?.[0]).toContain("3"));
  });

  it("pauses and resumes, saying what a pause means", async () => {
    const { panel, pressed } = await held(card, {
      doc_share_pause: null,
      doc_share_status: { ...status, paused: true },
    });

    await pressed("pause");

    await waitFor(() => expect(panel().gist).toBe(t("sharedPausedGist")));
    expect(panel().tone).toBe("warn");
    expect(panel().said).toContain(t("sharedPausedWhy"));
    expect(panel().acts?.find((one) => one.key === "pause")?.label).toBe(t("shareResume"));
    expect(panel().acts?.find((one) => one.key === "sync")?.off).toBe(true);
  });

  it("asks before stopping, and then the row offers to share again", async () => {
    const { panel, pressed } = await held(card, { doc_share_stop: null });

    await pressed("ask");
    expect(panel().said).toContain(t("shareStopAsk"));
    expect(invoked).not.toHaveBeenCalledWith("doc_share_stop", { id: "d1" });

    await pressed("keep");
    expect(panel().said).not.toContain(t("shareStopAsk"));

    await pressed("ask");
    await pressed("stop");

    await waitFor(() => expect(panel().gist).toBe(t("sharedNot")));
  });

  it("drops the links of what is blocked and offers to appeal", async () => {
    const { panel, pressed } = await held(card, {
      doc_share_status: { ...status, blocked: true },
      doc_share_appeal: true,
    });

    await pressed("status");
    await waitFor(() => expect(panel().gist).toBe(t("sharedBlockedGist")));
    expect(panel().tone).toBe("bad");
    expect(panel().links).toEqual([]);

    await pressed("appeal");

    await waitFor(() => expect(panel().said).toContain(t("shareAppealAccepted")));
  });

  it("says when an appeal goes to review", async () => {
    const { panel, pressed } = await held(card, {
      doc_share_status: { ...status, blocked: true },
      doc_share_appeal: false,
    });

    await pressed("status");
    await waitFor(() => expect(panel().gist).toBe(t("sharedBlockedGist")));
    await pressed("appeal");

    await waitFor(() => expect(panel().said).toContain(t("shareAppealReview")));
  });

  it("lets whoever edits through a link leave it, with nothing to manage", async () => {
    const { panel, pressed } = await held({ ...card, taken: true }, { doc_share_stop: null });

    expect(panel().acts?.map((one) => one.key)).toEqual(["sync", "leave"]);

    await pressed("leave");
    await pressed("stop");

    await waitFor(() => expect(panel().gist).toBe(t("sharedNot")));
  });
});
