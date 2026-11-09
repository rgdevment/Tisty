import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { adopt, fill, t } from "../locales";
import LinkQuestions from "../ui/LinkQuestions";

const invoked = vi.hoisted(() => vi.fn());
const heard = vi.hoisted(() => ({ fn: (() => {}) as () => void }));
const listening = vi.hoisted(() => ({ now: Promise.resolve() as Promise<unknown> }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invoked }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: (_: string, fn: () => void) => {
    heard.fn = fn;
    return listening.now.then(() => () => {});
  },
}));

const standing = (question: unknown) =>
  invoked.mockImplementation((command: string) =>
    Promise.resolve(command === "link_question_standing" ? question : null),
  );

const answers = () =>
  invoked.mock.calls
    .filter(([command]) => command === "link_question_answered")
    .map(([, said]) => said);

const read = () => vi.advanceTimersByTime(500);

beforeEach(() => {
  invoked.mockReset();
  listening.now = Promise.resolve();
  adopt("en");
});

describe("what a link asks in the window", () => {
  it("shows nothing while nothing is asked", async () => {
    standing(null);
    const { container } = render(<LinkQuestions />);

    await waitFor(() => expect(invoked).toHaveBeenCalledWith("link_question_standing"));
    expect(container.textContent).toBe("");
  });

  it("reads what stands only once it listens, so nothing asked in between is lost", async () => {
    standing({ id: 1, kind: "look" });
    let ready = () => {};
    listening.now = new Promise<void>((resolve) => {
      ready = resolve;
    });
    render(<LinkQuestions />);
    expect(invoked).not.toHaveBeenCalled();

    ready();

    expect(await screen.findByText(t("linkLookWhy"))).toBeTruthy();
  });

  it("asks before Tisty goes to Folio, naming the question it answers", async () => {
    standing({ id: 7, kind: "look" });
    render(<LinkQuestions />);
    expect(await screen.findByText(t("linkLookWhy"))).toBeTruthy();
    read();

    await userEvent.click(screen.getByRole("button", { name: t("linkLookGo") }));

    expect(answers()).toEqual([{ id: 7, yes: true }]);
    expect(screen.queryByText(t("linkLookWhy"))).toBeNull();
  });

  it("takes no answer in the instant a question opens", async () => {
    standing({ id: 7, kind: "look" });
    render(<LinkQuestions />);
    await screen.findByText(t("linkLookWhy"));

    await userEvent.click(screen.getByRole("button", { name: t("linkLookGo") }));

    expect(answers()).toEqual([]);
    expect(screen.getByText(t("linkLookWhy"))).toBeTruthy();
  });

  it("says it is looking until the next question arrives", async () => {
    standing({ id: 7, kind: "look" });
    render(<LinkQuestions />);
    await screen.findByText(t("linkLookWhy"));
    read();
    await userEvent.click(screen.getByRole("button", { name: t("linkLookGo") }));
    expect(await screen.findByText(t("linkLookingWhy"))).toBeTruthy();

    standing(null);
    await act(async () => heard.fn());
    expect(screen.getByText(t("linkLookingWhy"))).toBeTruthy();

    standing({ id: 8, kind: "failed", code: "folioGone" });
    await act(async () => heard.fn());
    expect(screen.queryByText(t("linkLookingWhy"))).toBeNull();
    expect(screen.getByText(t("linkFailedGone"))).toBeTruthy();
  });

  it("does not say it is looking after not now", async () => {
    standing({ id: 7, kind: "look" });
    render(<LinkQuestions />);
    await screen.findByText(t("linkLookWhy"));
    read();

    await userEvent.keyboard("{Escape}");

    expect(answers()).toEqual([{ id: 7, yes: false }]);
    expect(screen.queryByText(t("linkLookingWhy"))).toBeNull();
  });

  it("comes up when the event says a question stands", async () => {
    standing(null);
    render(<LinkQuestions />);
    await waitFor(() => expect(invoked).toHaveBeenCalledTimes(1));

    standing({ id: 1, kind: "look" });
    await act(async () => heard.fn());

    expect(await screen.findByText(t("linkLookWhy"))).toBeTruthy();
  });

  const plain = (name: string) => ({ name, standing: "plain" });
  const keep = {
    id: 2,
    kind: "keep",
    title: "Viaje",
    owner: null,
    signer: plain(""),
    print: "AB-CD",
  };

  it("shows owner and signer apart when another root signed this version", async () => {
    standing({ ...keep, owner: plain("Mario"), signer: plain("Ana"), doubtful: false });
    render(<LinkQuestions />);

    expect(await screen.findByText("Viaje")).toBeTruthy();
    expect(screen.getByText(fill("linkKeepOwner", "Mario"))).toBeTruthy();
    expect(screen.getByText(fill("linkKeepSigned", "Ana"))).toBeTruthy();
    expect(screen.getByText("AB-CD")).toBeTruthy();
    expect(screen.queryByRole("alert")).toBeNull();
    read();

    await userEvent.click(screen.getByRole("button", { name: t("linkNotNow") }));
    expect(answers()).toEqual([{ id: 2, yes: false }]);
  });

  it("names one person once when the owner's root signed it", async () => {
    standing({ ...keep, signer: plain("Mario"), doubtful: false });
    render(<LinkQuestions />);

    expect(await screen.findByText(fill("linkKeepSigned", "Mario"))).toBeTruthy();
    expect(screen.queryByText(fill("linkKeepOwner", "Mario"))).toBeNull();
  });

  it("names the maker by Folio's word, verified, with the domain beneath and no link", async () => {
    const maker = {
      name: "otro alias",
      standing: "verified",
      maker: "rgdevment",
      domain: "rgdevment.dev",
    };
    standing({ ...keep, signer: maker, doubtful: false });
    render(<LinkQuestions />);

    const signed = await screen.findByText(fill("linkKeepSigned", "rgdevment"), { exact: false });
    expect(signed.textContent).toContain(t("linkKeepVerified"));
    expect(screen.getByText("rgdevment.dev").closest("a")).toBeNull();
    expect(screen.queryByText(fill("linkKeepSigned", "otro alias"), { exact: false })).toBeNull();
  });

  it("marks a name that imitates the maker's as another key's, where the domain would go", async () => {
    standing({ ...keep, signer: { name: "rgdevrnent", standing: "imitates" }, doubtful: false });
    render(<LinkQuestions />);

    expect(await screen.findByText(fill("linkKeepSigned", "rgdevrnent"))).toBeTruthy();
    expect(screen.getByText(t("linkKeepOtherKey"))).toBeTruthy();
    expect(screen.queryByText(t("linkKeepVerified"))).toBeNull();
  });

  it("tells a key the maker took back that it no longer speaks for the name", async () => {
    standing({ ...keep, signer: { name: "rgdevment", standing: "takenBack" }, doubtful: true });
    render(<LinkQuestions />);

    expect(await screen.findByText(fill("linkKeepSigned", "rgdevment"))).toBeTruthy();
    expect(screen.getByText(t("linkKeepTakenBack"))).toBeTruthy();
    expect(screen.queryByText(t("linkKeepVerified"))).toBeNull();
  });

  it("says a checked domain is checked, and never verified", async () => {
    standing({
      ...keep,
      signer: { name: "ana.dev", standing: "checked", domain: "ana.dev" },
      doubtful: false,
    });
    render(<LinkQuestions />);

    expect(await screen.findByText(fill("linkKeepChecked", "ana.dev"))).toBeTruthy();
    expect(screen.queryByText(t("linkKeepVerified"))).toBeNull();
  });

  it("names a document with no title, and warns of a key that cannot be trusted", async () => {
    standing({ ...keep, title: "", doubtful: true });
    render(<LinkQuestions />);

    expect((await screen.findByRole("alert")).textContent).toBe(t("linkKeepDoubt"));
    expect(screen.getByText(t("untitledDoc"))).toBeTruthy();
    expect(screen.queryByText(fill("linkKeepSigned", ""))).toBeNull();
  });

  it.each([
    ["folioGone", "linkFailedGone"],
    ["folioBlocked", "linkFailedBlocked"],
    ["folioPaused", "linkFailedPaused"],
    ["folioOutdated", "linkFailedOutdated"],
    ["folioUnopened", "linkFailedUnopened"],
    ["folioUnreached", "linkFailedUnreached"],
    ["anythingElse", "linkFailedUnreached"],
  ] as const)("says why a link failed with %s", async (code, word) => {
    standing({ id: 3, kind: "failed", code });
    render(<LinkQuestions />);

    expect(await screen.findByText(t(word))).toBeTruthy();
    read();

    await userEvent.click(screen.getByRole("button", { name: t("linkFailedOk") }));
    expect(answers()).toEqual([{ id: 3, yes: true }]);
  });
});
