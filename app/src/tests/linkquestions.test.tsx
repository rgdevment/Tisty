import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { adopt, fill, t } from "../locales";
import LinkQuestions from "../ui/LinkQuestions";

const invoked = vi.hoisted(() => vi.fn());
const heard = vi.hoisted(() => ({ fn: (() => {}) as () => void }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invoked }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: (_: string, fn: () => void) => {
    heard.fn = fn;
    return Promise.resolve(() => {});
  },
}));

const standing = (question: unknown) =>
  invoked.mockImplementation((command: string) =>
    Promise.resolve(command === "link_question_standing" ? question : null),
  );

const answers = () =>
  invoked.mock.calls
    .filter(([command]) => command === "link_question_answered")
    .map(([, said]) => said.yes);

beforeEach(() => {
  invoked.mockReset();
  adopt("en");
});

describe("what a link asks in the window", () => {
  it("shows nothing while nothing is asked", async () => {
    standing(null);
    const { container } = render(<LinkQuestions />);

    await waitFor(() => expect(invoked).toHaveBeenCalledWith("link_question_standing"));
    expect(container.textContent).toBe("");
  });

  it("asks before Tisty goes to Folio, and says either answer", async () => {
    standing({ kind: "look" });
    render(<LinkQuestions />);
    expect(await screen.findByText(t("linkLookWhy"))).toBeTruthy();

    await userEvent.click(screen.getByRole("button", { name: t("linkLookGo") }));

    expect(answers()).toEqual([true]);
    expect(screen.queryByText(t("linkLookWhy"))).toBeNull();
  });

  it("takes leaving the question as not now", async () => {
    standing({ kind: "look" });
    render(<LinkQuestions />);
    await screen.findByText(t("linkLookWhy"));

    await userEvent.keyboard("{Escape}");

    expect(answers()).toEqual([false]);
  });

  it("comes up when the event says a question stands", async () => {
    standing(null);
    render(<LinkQuestions />);
    await waitFor(() => expect(invoked).toHaveBeenCalledTimes(1));

    standing({ kind: "look" });
    heard.fn();

    expect(await screen.findByText(t("linkLookWhy"))).toBeTruthy();
  });

  it("shows what it would bring, who signed it and its fingerprint", async () => {
    standing({ kind: "keep", title: "Viaje", name: "Ana", print: "AB-CD", doubtful: false });
    render(<LinkQuestions />);

    expect(await screen.findByText("Viaje")).toBeTruthy();
    expect(screen.getByText(fill("linkKeepSigned", "Ana"))).toBeTruthy();
    expect(screen.getByText("AB-CD")).toBeTruthy();
    expect(screen.queryByRole("alert")).toBeNull();

    await userEvent.click(screen.getByRole("button", { name: t("linkNotNow") }));
    expect(answers()).toEqual([false]);
  });

  it("warns when the key that signed it cannot be trusted", async () => {
    standing({ kind: "keep", title: "Viaje", name: "", print: "AB", doubtful: true });
    render(<LinkQuestions />);

    expect((await screen.findByRole("alert")).textContent).toBe(t("linkKeepDoubt"));
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
    standing({ kind: "failed", code });
    render(<LinkQuestions />);

    expect(await screen.findByText(t(word))).toBeTruthy();

    await userEvent.click(screen.getByRole("button", { name: t("linkFailedOk") }));
    expect(answers()).toEqual([true]);
  });
});
