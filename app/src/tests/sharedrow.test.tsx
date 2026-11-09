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

const card = {
  read: "https://tisty.cl/a#k",
  edit: null,
  version: 1,
  taken: false,
  held: false,
  told: [],
  owner: null,
  signer: null,
  editors: [],
  signsAs: null,
};
const status = {
  diesOn: 20_000,
  paused: false,
  blocked: false,
  reads: 3,
  appealed: false,
  mayCorrect: false,
  fixBy: null,
};
const today = () => Math.floor(Date.now() / 86_400_000);

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

  it("starts folded and marked as beta, shared or not", async () => {
    const { panel } = await held(card);
    expect(panel().open).toBeUndefined();
    expect(panel().mark).toBe(t("sharedBeta"));

    const none = await held(null);
    expect(none.panel().open).toBeUndefined();
    expect(none.panel().mark).toBe(t("sharedBeta"));
  });

  it("names what can be edited and its second link", async () => {
    const { panel } = await held({ ...card, edit: "https://tisty.cl/a#k.t" });

    expect(panel().gist).toBe(t("sharedCanEdit"));
    expect(panel().links?.map((one) => one.key)).toEqual(["read", "edit"]);
  });

  it("names the owner and whoever signed this version apart, and lets whoever edits sign as their other name", async () => {
    const editing = {
      ...card,
      edit: "https://tisty.cl/a#k.t",
      taken: true,
      owner: { name: "Mario Hidalgo", checked: true },
      signer: { name: "Ana", checked: false },
      editors: [
        { name: "Ana", checked: false },
        { name: "Juan", checked: true },
      ],
      signsAs: { now: "ana.dev", other: "Ana Pérez" },
    };
    const after = { ...editing, signsAs: { now: "Ana Pérez", other: "ana.dev" } };
    const { panel, pressed } = await held(editing, { doc_share_sign_as: after });

    expect(panel().said).toContain("Created by Mario Hidalgo ✓ · Last edited by Ana");
    expect(panel().said).toContain(fill("sharedEditedBy", `Ana, ${fill("sharedChecked", "Juan")}`));
    expect(panel().said).toContain(fill("shareSignsAs", "ana.dev"));
    await pressed("signs");

    expect(invoked).toHaveBeenCalledWith("doc_share_sign_as", { id: "d1", name: "Ana Pérez" });
    await waitFor(() => expect(panel().said).toContain(fill("shareSignsAs", "Ana Pérez")));
  });

  it("names only the owner when the owner signed the version a copy holds", async () => {
    const { panel } = await held({
      ...card,
      taken: true,
      owner: { name: "Mario", checked: false },
      signer: null,
    });

    expect(panel().said).toContain(fill("sharedOwner", "Mario"));
    expect(panel().acts?.map((one) => one.key)).not.toContain("signs");
  });

  it("lets the owner change the edit link once asked, and shows the new one", async () => {
    const editable = { ...card, edit: "https://tisty.cl/a#k.old" };
    const renewed = { ...editable, edit: "https://tisty.cl/a#k.new" };
    const { panel, pressed } = await held(editable, {
      doc_share_edit_link_renewed: { state: "sent", version: 3, told: [] },
    });
    invoked.mockImplementation((command: string) => {
      if (command === "doc_share_edit_link_renewed")
        return Promise.resolve({ state: "sent", version: 3, told: [] });
      if (command === "doc_share_card") return Promise.resolve(renewed);
      return Promise.resolve(null);
    });

    await pressed("renewAsk");
    expect(panel().said).toContain(t("shareRenewAsk"));
    expect(panel().acts?.map((one) => one.key)).toEqual(["keep", "renew"]);
    await pressed("renew");

    expect(invoked).toHaveBeenCalledWith("doc_share_edit_link_renewed", { id: "d1" });
    await waitFor(() => expect(panel().said).toContain(t("shareRenewed")));
    await waitFor(() =>
      expect(panel().links?.find((one) => one.key === "edit")?.url).toBe(renewed.edit),
    );
  });

  it("says the change still waits when the Sync inside it brought another version", async () => {
    const editable = { ...card, edit: "https://tisty.cl/a#k.old" };
    const { panel, pressed } = await held(editable);
    invoked.mockImplementation((command: string) => {
      if (command === "doc_share_edit_link_renewed")
        return Promise.resolve({
          state: "otherVersion",
          version: 5,
          signer: "Ana",
          doubtful: false,
        });
      if (command === "doc_share_card") return Promise.resolve(editable);
      return Promise.resolve(null);
    });

    await pressed("renewAsk");
    await pressed("renew");

    await waitFor(() =>
      expect(panel().said).toContain(
        `${fill("shareSyncOther", "5", "Ana")} ${t("shareRenewWaits")}`,
      ),
    );
    expect(panel().said).not.toContain(t("shareRenewed"));
  });

  it("offers no edit link to change where nobody may edit", async () => {
    const { panel } = await held(card);

    expect(panel().acts?.map((one) => one.key)).not.toContain("renewAsk");
  });

  it("tells the owner who signed the version it now holds", async () => {
    const { panel } = await held({ ...card, signer: { name: "Ana", checked: false } });

    expect(panel().said).toContain(fill("sharedVersionBy", "Ana"));
  });

  it.each([
    [{ state: "unchanged", told: [] }, () => t("shareSyncUnchanged")],
    [
      { state: "unchanged", told: [{ what: "takenBackNotKept" }] },
      () => `${t("shareSyncUnchanged")} ${t("toldTakenBackNotKept")}`,
    ],
    [{ state: "sent", version: 2, told: [] }, () => fill("shareSyncSent", "2")],
    [{ state: "sent", version: null, told: [] }, () => t("shareSyncSentAlong")],
    [
      { state: "unchanged", told: [], termsUnchecked: true },
      () => `${t("shareSyncUnchanged")} ${t("shareTermsUnchecked")}`,
    ],
    [
      { state: "sent", version: 2, told: [], termsUnchecked: true },
      () => `${fill("shareSyncSent", "2")} ${t("shareTermsUnchecked")}`,
    ],
    [
      { state: "otherVersion", version: 3, signer: "Ana", doubtful: false },
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
    await waitFor(() => expect(result.current.panel).toMatchObject({ key: "shared" }));
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

  it("says when Sync is free again and that changes are waiting, and sends nothing by itself", async () => {
    const { panel } = await held(card, {
      doc_share_waiting: { until: Date.now() + 3 * 60_000, pending: true },
    });

    await waitFor(() => {
      const sync = panel().acts?.find((one) => one.key === "sync");
      expect(sync?.off).toBe(true);
      expect(sync?.label).toBe(fill("shareSyncWait", "3"));
      expect(panel().said).toContain(t("shareSyncPending"));
    });
    expect(invoked).not.toHaveBeenCalledWith("doc_share_sync", expect.anything());
  });

  it("keeps the wait Folio asked for when this computer knows of none", async () => {
    const until = Date.now() + 4 * 60_000;
    const { panel, pressed } = await held(card, {
      doc_share_waiting: { until: null, pending: false },
    });
    invoked.mockImplementation((command: string) => {
      if (command === "doc_share_sync")
        return Promise.reject({ code: "folioWait", name: String(until) });
      if (command === "doc_share_waiting") return Promise.resolve({ until: null, pending: false });
      if (command === "doc_share_card") return Promise.resolve(card);
      return Promise.reject({ code: "internal" });
    });

    await pressed("sync");
    await act(async () => {});

    const sync = panel().acts?.find((one) => one.key === "sync");
    expect(sync?.off).toBe(true);
    expect(sync?.label).toBe(fill("shareSyncWait", "4"));
  });

  it("leaves Sync free when nothing waits, and says nothing of changes", async () => {
    const { panel } = await held(card, { doc_share_waiting: { until: null, pending: false } });

    await waitFor(() =>
      expect(invoked).toHaveBeenCalledWith("doc_share_waiting", { id: "d1", look: true }),
    );
    const sync = panel().acts?.find((one) => one.key === "sync");
    expect(sync?.off).toBe(false);
    expect(panel().said).not.toContain(t("shareSyncPending"));
  });

  it("syncs again by itself once the terms Folio asked for are accepted", async () => {
    let tries = 0;
    invoked.mockImplementation((command: string) => {
      if (command === "doc_share_card") return Promise.resolve(card);
      if (command === "doc_share_sync") {
        tries += 1;
        return tries === 1
          ? Promise.reject({ code: "folioTermsToAccept" })
          : Promise.resolve({ state: "unchanged", told: [] });
      }
      return Promise.reject({ code: "internal" });
    });
    const { result } = renderHook(() => useSharedRow("d1", vi.fn()));
    await waitFor(() => {
      const now = result.current.panel;
      expect(now?.kind === "held" && now.links?.length).toBe(1);
    });
    await act(async () => {
      const now = result.current.panel;
      if (now?.kind === "held")
        now.acts?.find((one) => one.key === "sync")?.onPick?.(document.body);
    });
    await waitFor(() => expect(result.current.termsOnly).toBe(true));

    await act(async () => result.current.onTermsAccepted());

    await waitFor(() => expect(tries).toBe(2));
    expect(result.current.asking).toBe(false);
    expect(result.current.termsOnly).toBe(false);
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

  it("says Folio's day as it is, wherever this computer stands", async () => {
    const { panel, pressed } = await held(card, { doc_share_status: status });

    await pressed("status");

    await waitFor(() => expect(panel().said?.[0]).toBe(fill("sharedStatus", "10/4/2024", "3")));
  });

  it("keeps a late status off the next document the row shows", async () => {
    let answer: (now: unknown) => void = () => {};
    answering({
      doc_share_card: card,
      doc_share_sync: { state: "unchanged", told: [] },
      doc_share_status: new Promise((resolve) => {
        answer = resolve;
      }),
    });
    const hook = renderHook(({ doc }) => useSharedRow(doc), { initialProps: { doc: "d1" } });
    await act(async () => {});
    const shown = () => {
      const now = hook.result.current.panel;
      if (now?.kind !== "held") throw new Error("no row");
      return now;
    };
    await act(async () => {
      shown()
        .acts?.find((one) => one.key === "sync")
        ?.onPick?.(document.body);
    });
    await waitFor(() => expect(invoked).toHaveBeenCalledWith("doc_share_status", { id: "d1" }));

    hook.rerender({ doc: "d2" });
    await act(async () => {});
    await act(async () => answer({ ...status, blocked: true }));

    expect(shown().gist).not.toBe(t("sharedBlockedGist"));
    expect(shown().links).toHaveLength(1);
  });

  it("drops a status that arrives after the share was stopped", async () => {
    let answer: (now: unknown) => void = () => {};
    const { hook, panel, pressed } = await held(card, {
      doc_share_sync: { state: "unchanged", told: [] },
      doc_share_status: new Promise((resolve) => {
        answer = resolve;
      }),
      doc_share_stop: null,
    });
    await pressed("sync");
    await waitFor(() => expect(invoked).toHaveBeenCalledWith("doc_share_status", { id: "d1" }));
    await pressed("ask");
    await pressed("stop");
    await waitFor(() => expect(hook.result.current.panel?.kind).toBe("held"));

    await act(async () => answer({ ...status, blocked: true }));
    await act(async () => hook.result.current.onShared(card));

    expect(panel().gist).not.toBe(t("sharedBlockedGist"));
    expect(panel().links).toHaveLength(1);
  });

  it("warns of reports and the day to correct them by", async () => {
    const { panel, pressed } = await held(card, {
      doc_share_status: { ...status, fixBy: today() + 3 },
    });

    await pressed("status");

    await waitFor(() => expect(panel().gist).toBe(t("sharedReportedGist")));
    expect(panel().tone).toBe("warn");
    expect(panel().said?.[0]).toBe(
      fill(
        "sharedFixBy",
        new Date((today() + 3) * 86_400_000).toLocaleDateString("en", { timeZone: "UTC" }),
      ),
    );
    expect(panel().links).toHaveLength(1);
  });

  it("still counts the last day to correct, and says so once it passed", async () => {
    const onTheDay = await held(card, { doc_share_status: { ...status, fixBy: today() } });
    await onTheDay.pressed("status");
    await waitFor(() =>
      expect(onTheDay.panel().said?.[0]).toBe(
        fill(
          "sharedFixBy",
          new Date(today() * 86_400_000).toLocaleDateString("en", { timeZone: "UTC" }),
        ),
      ),
    );

    const late = await held(card, { doc_share_status: { ...status, fixBy: today() - 1 } });
    await late.pressed("status");
    await waitFor(() => expect(late.panel().said?.[0]).toBe(t("sharedFixLate")));
    expect(late.panel().acts?.find((one) => one.key === "sync")?.off).toBe(false);
  });

  it("lets a block whose appeal was accepted be corrected with Sync", async () => {
    const { panel, pressed } = await held(card, {
      doc_share_status: { ...status, blocked: true, mayCorrect: true },
    });

    await pressed("status");

    await waitFor(() => expect(panel().said).toEqual([t("sharedMayCorrect")]));
    expect(panel().acts?.find((one) => one.key === "sync")?.off).toBe(false);
    expect(panel().acts?.some((one) => one.key === "appeal")).toBe(false);
  });

  it("says an appeal is under review and does not offer it again", async () => {
    const { panel, pressed } = await held(card, {
      doc_share_status: { ...status, blocked: true, appealed: true },
    });

    await pressed("status");

    await waitFor(() => expect(panel().said).toEqual([t("sharedAppealWaiting")]));
    expect(panel().acts?.some((one) => one.key === "appeal")).toBe(false);
    expect(panel().acts?.find((one) => one.key === "sync")?.off).toBe(true);
  });

  it("looks at the status after Sync when it holds the link to manage", async () => {
    const { panel, pressed } = await held(card, {
      doc_share_sync: { state: "unchanged", told: [] },
      doc_share_status: { ...status, fixBy: today() + 3 },
    });

    await pressed("sync");

    await waitFor(() => expect(panel().gist).toBe(t("sharedReportedGist")));
  });

  it("never asks the status of a share held through an edit link", async () => {
    const { pressed } = await held(
      { ...card, taken: true },
      { doc_share_sync: { state: "unchanged", told: [] }, doc_share_status: status },
    );

    await pressed("sync");

    await waitFor(() => expect(invoked).toHaveBeenCalledWith("doc_share_sync", expect.anything()));
    expect(invoked).not.toHaveBeenCalledWith("doc_share_status", expect.anything());
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
    const { panel, pressed } = await held(
      { ...card, taken: true, edit: "https://tisty.cl/a#k.t" },
      { doc_share_stop: null },
    );

    expect(panel().acts?.map((one) => one.key)).toEqual(["sync", "leave"]);
    expect(panel().links).toEqual([]);

    await pressed("leave");
    await pressed("stop");

    await waitFor(() => expect(panel().gist).toBe(t("sharedNot")));
  });
});

describe("what Sync warns of", () => {
  it("names a key it found and sends only what the answer was about", async () => {
    invoked.mockImplementation((command: string, asked: { accepted?: string }) => {
      if (command === "doc_share_card") return Promise.resolve(card);
      if (asked.accepted === "p1") return Promise.resolve({ state: "sent", version: 2, told: [] });
      return Promise.resolve({ state: "secrets", place: "Reservas", print: "p1" });
    });
    const { result } = renderHook(() => useSharedRow("d1"));
    const row = () => {
      const now = result.current.panel;
      if (now?.kind !== "held") throw new Error("no row");
      return now;
    };
    await waitFor(() => expect(row().links?.length).toBe(1));
    const press = () =>
      act(async () => {
        row()
          .acts?.find((one) => one.key === "sync")
          ?.onPick?.(document.body);
      });

    await press();
    await waitFor(() => expect(row().said).toContain(fill("shareSecretsSync", "Reservas")));
    expect(row().acts?.find((one) => one.key === "sync")?.label).toBe(t("shareGoAnyway"));
    expect(invoked).not.toHaveBeenCalledWith("doc_share_sync", { id: "d1", accepted: "other" });

    await press();
    await waitFor(() => expect(row().said).toContain(fill("shareSyncSent", "2")));
    expect(invoked).toHaveBeenCalledWith("doc_share_sync", { id: "d1", accepted: "p1" });
  });

  it("asks before a Sync sends what others edited, and sends once answered", async () => {
    invoked.mockImplementation((command: string, asked: { accepted?: string }) => {
      if (command === "doc_share_card") return Promise.resolve(card);
      if (asked.accepted === "p2") return Promise.resolve({ state: "sent", version: 4, told: [] });
      return Promise.resolve({ state: "othersEdited", editors: ["Ana"], print: "p2" });
    });
    const { result } = renderHook(() => useSharedRow("d1"));
    const row = () => {
      const now = result.current.panel;
      if (now?.kind !== "held") throw new Error("no row");
      return now;
    };
    await waitFor(() => expect(row().links?.length).toBe(1));
    const press = () =>
      act(async () => {
        row()
          .acts?.find((one) => one.key === "sync")
          ?.onPick?.(document.body);
      });

    await press();
    await waitFor(() => expect(row().said).toContain(fill("toldOthersEdited", "Ana")));
    expect(row().acts?.find((one) => one.key === "sync")?.label).toBe(t("shareGoAnyway"));

    await press();
    await waitFor(() => expect(row().said).toContain(fill("shareSyncSent", "4")));
    expect(invoked).toHaveBeenCalledWith("doc_share_sync", { id: "d1", accepted: "p2" });
  });

  it("forgets a link that ran out and says so", async () => {
    const { panel, pressed } = await held(card, { doc_share_sync: { state: "gone" } });

    await pressed("sync");

    await waitFor(() => expect(panel().gist).toBe(t("sharedNot")));
    expect(panel().said).toContain(t("folioGone"));
  });

  it("says Folio is updating its terms rather than calling it a failure", async () => {
    const { panel, pressed, onError } = await held(card);
    invoked.mockImplementation((command: string) =>
      command === "doc_share_sync"
        ? Promise.reject({ code: "folioTermsUpdating" })
        : Promise.reject({ code: "internal" }),
    );

    await pressed("sync");

    await waitFor(() => expect(panel().said).toContain(t("folioTermsUpdating")));
    expect(onError).not.toHaveBeenCalled();
  });

  it("says what stayed out and when a signature is in doubt", async () => {
    const { panel, pressed } = await held(card, {
      doc_share_sync: { state: "otherVersion", version: 3, signer: "Ana", doubtful: true },
    });

    await pressed("sync");

    await waitFor(() => expect(panel().said?.join(" ")).toContain(t("shareSyncOtherDoubt")));
  });

  it("leaves nothing to manage in what another share holds", async () => {
    const { panel } = await held({ ...card, held: true });

    expect(panel().gist).toBe(t("sharedHeldGist"));
    expect(panel().acts).toBeUndefined();
    expect(panel().links?.length).toBe(1);
  });

  it("asks only for the terms when Sync finds them changed", async () => {
    invoked.mockImplementation((command: string) =>
      command === "doc_share_card"
        ? Promise.resolve(card)
        : Promise.reject({ code: "folioTermsToAccept" }),
    );
    const { result } = renderHook(() => useSharedRow("d1"));
    await waitFor(() => {
      const now = result.current.panel;
      expect(now?.kind === "held" && now.links?.length).toBe(1);
    });

    await act(async () => {
      const now = result.current.panel;
      if (now?.kind === "held")
        now.acts?.find((one) => one.key === "sync")?.onPick?.(document.body);
    });

    await waitFor(() => expect(result.current.termsOnly).toBe(true));
    act(() => result.current.onClosed());
    expect(result.current.asking).toBe(false);
  });
});
