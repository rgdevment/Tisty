import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { adopt, fill, t } from "../locales";
import Sharing from "../ui/Sharing";

const invoked = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke: invoked }));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: vi.fn(() => Promise.resolve()) }));
vi.mock("../core", async (original) => ({
  ...(await original<typeof import("../core")>()),
  copied: vi.fn(() => Promise.resolve()),
}));

const look = {
  secrets: [],
  citedSecrets: [],
  name: "Rina Soto",
  alias: "rina",
  carrying: { pages: 1, pictures: 0, picturesBytes: 0, files: 0, filesBytes: 0 },
  cited: [],
  shared: [],
  limits: { ttl_days: [1, 3, 7, 15], shares_at_most: 2 },
  told: [],
  seen: "print",
};

const answering = (answers: Record<string, unknown>) =>
  invoked.mockImplementation((command: string) => {
    if (command in answers) return Promise.resolve(answers[command]);
    if (command === "folio_sharing_now") return Promise.resolve("open");
    if (command === "help_drawn") return Promise.resolve("<p>Los términos</p>");
    return Promise.reject({ code: "internal" });
  });

const shown = (onError = vi.fn(), onClose = vi.fn(), onShared = vi.fn()) => {
  render(
    <Sharing doc="d1" title="Viaje" onShared={onShared} onClose={onClose} onError={onError} />,
  );
  return { onError, onClose, onShared };
};

beforeEach(() => {
  invoked.mockReset();
  adopt("en");
});

describe("sharing a document with Folio", () => {
  it("says it is not available yet and asks for no terms while Folio is closed", async () => {
    answering({ folio_sharing_now: "closed" });
    const { onClose } = shown();

    await screen.findByText(t("shareClosedWhat"), { exact: false });
    expect(screen.getByRole("button", { name: "folio.tisty.app" })).toBeTruthy();
    expect(invoked).not.toHaveBeenCalledWith("folio_terms_standing");

    await userEvent.click(screen.getByRole("button", { name: t("shareClosedOk") }));
    expect(onClose).toHaveBeenCalled();
  });

  it("asks again when the first look reached nothing", async () => {
    let looks = 0;
    invoked.mockImplementation((command: string) => {
      if (command === "folio_sharing_now") {
        looks += 1;
        return Promise.resolve(looks === 1 ? "unreached" : "closed");
      }
      return Promise.reject({ code: "internal" });
    });
    shown();

    await userEvent.click(await screen.findByRole("button", { name: t("shareUnreachedRetry") }));

    await screen.findByRole("button", { name: t("shareClosedOk") });
    expect(looks).toBe(2);
  });

  it("turns to the notice when Folio closes between the look and the upload", async () => {
    answering({
      folio_terms_standing: { state: "accepted" },
      doc_share_look: look,
    });
    const answered = invoked.getMockImplementation();
    invoked.mockImplementation((command: string) =>
      command === "doc_shared" ? Promise.reject({ code: "folioClosed" }) : answered?.(command),
    );
    shown();

    await userEvent.click(await screen.findByRole("button", { name: t("shareGo") }));

    await screen.findByRole("button", { name: t("shareClosedOk") });
  });

  it("shows the current terms first and goes on only once they are accepted", async () => {
    answering({
      folio_terms_standing: {
        state: "toAccept",
        version: "2026-11-06",
        text: "Los términos",
        acceptedBefore: null,
      },
      folio_terms_accept: null,
      doc_share_look: look,
    });
    shown();

    await screen.findByText("Los términos");
    const go = screen.getByRole<HTMLButtonElement>("button", { name: t("termsGo") });
    expect(go.disabled).toBe(true);

    await userEvent.click(screen.getByRole("checkbox"));
    await userEvent.click(go);

    await screen.findByRole("button", { name: t("shareGo") });
    expect(invoked).toHaveBeenCalledWith("folio_terms_accept", { version: "2026-11-06" });
  });

  it("says which version was accepted when the terms changed", async () => {
    answering({
      folio_terms_standing: {
        state: "toAccept",
        version: "2",
        text: "Los términos",
        acceptedBefore: "1",
      },
    });
    shown();

    expect(await screen.findByRole("heading", { name: t("termsChangedTitle") })).toBeTruthy();
    expect(screen.getByText(fill("termsChangedWhy", "1", "2"))).toBeTruthy();
  });

  it("asks for nothing when the current terms are already accepted", async () => {
    answering({ folio_terms_standing: { state: "accepted" }, doc_share_look: look });
    shown();

    await screen.findByRole("button", { name: t("shareGo") });
    expect(screen.queryByRole("checkbox")).toBeNull();
  });

  it("signs with the alias unless the name is chosen, and asks for what was chosen", async () => {
    answering({
      folio_terms_standing: { state: "accepted" },
      doc_share_look: look,
      doc_shared: {
        read: "https://tisty.cl/Ab3xK9pQ#key",
        edit: null,
        version: 1,
        taken: false,
        held: false,
        told: [],
      },
    });
    const { onShared } = shown();

    await userEvent.click(await screen.findByRole("button", { name: t("shareGo") }));

    await screen.findByRole("heading", { name: t("sharedDone") });
    expect(invoked).toHaveBeenCalledWith("doc_shared", {
      id: "d1",
      chosen: { cited: [], signedAs: "rina", editable: false, ttlDays: 1, seen: "print" },
    });
    expect(onShared).toHaveBeenCalled();
    expect(screen.getByText("https://tisty.cl/Ab3xK9pQ#key")).toBeTruthy();
  });

  it("warns of a secret and renames the button", async () => {
    answering({
      folio_terms_standing: { state: "accepted" },
      doc_share_look: { ...look, secrets: [{ named: "api_key", under: "Reservas" }] },
    });
    shown();

    await screen.findByRole("button", { name: t("shareGoAnyway") });
    expect(screen.getByText("api_key")).toBeTruthy();
  });

  it("warns of a secret in what it cites only once that is going along", async () => {
    answering({
      folio_terms_standing: { state: "accepted" },
      doc_share_look: {
        ...look,
        cited: ["d2"],
        citedSecrets: [{ named: "api_key", under: "Reservas" }],
      },
    });
    shown();

    await screen.findByRole("button", { name: t("shareGo") });
    expect(screen.queryByText("api_key")).toBeNull();

    await userEvent.click(screen.getByRole("checkbox"));

    expect(screen.getByText("api_key")).toBeTruthy();
    expect(screen.getByRole("button", { name: t("shareGoAnyway") })).toBeTruthy();
  });

  it("will not share past what Folio takes", async () => {
    answering({
      folio_terms_standing: { state: "accepted" },
      doc_share_look: {
        ...look,
        shared: [
          { doc: "a", title: "Uno" },
          { doc: "b", title: "Dos" },
        ],
      },
    });
    shown();

    const go = await screen.findByRole<HTMLButtonElement>("button", { name: t("shareGo") });
    expect(go.disabled).toBe(true);
    expect(screen.getByText("Uno")).toBeTruthy();
  });

  it("looks again when the document changed while choosing", async () => {
    let tries = 0;
    invoked.mockImplementation((command: string) => {
      if (command === "folio_terms_standing") return Promise.resolve({ state: "accepted" });
      if (command === "doc_share_look") {
        tries += 1;
        return Promise.resolve(look);
      }
      return Promise.reject({ code: "folioChanged" });
    });
    shown();

    await userEvent.click(await screen.findByRole("button", { name: t("shareGo") }));

    expect(await screen.findByText(t("folioChanged"))).toBeTruthy();
    expect(tries).toBe(2);
  });

  it("says what could not go, before and after sharing", async () => {
    const told = { what: "pageLeftBehind", document: "Viaje" };
    answering({
      folio_terms_standing: { state: "accepted" },
      doc_share_look: { ...look, told: [told] },
      doc_shared: {
        read: "https://tisty.cl/Ab3xK9pQ#key",
        edit: null,
        version: 1,
        taken: false,
        held: false,
        told: [told],
      },
    });
    shown();

    expect(await screen.findByText(fill("toldPageLeftBehind", "Viaje"))).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: t("shareGo") }));
    await screen.findByRole("heading", { name: t("sharedDone") });
    expect(screen.getByText(fill("toldPageLeftBehind", "Viaje"))).toBeTruthy();
  });

  it("asks only for the terms when that is all that was needed", async () => {
    answering({
      folio_terms_standing: {
        state: "toAccept",
        version: "2",
        text: "Los términos",
        acceptedBefore: "1",
      },
      folio_terms_accept: null,
    });
    const onClose = vi.fn();
    render(<Sharing doc="d1" title="Viaje" termsOnly onShared={vi.fn()} onClose={onClose} />);

    await userEvent.click(await screen.findByRole("checkbox"));
    await userEvent.click(screen.getByRole("button", { name: t("termsGo") }));

    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(invoked).not.toHaveBeenCalledWith("doc_share_look", expect.anything());
  });

  it("closes and tells why when the terms cannot be read", async () => {
    invoked.mockImplementation(() => Promise.reject({ code: "folioTermsNotPublished" }));
    const { onError, onClose } = shown();

    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(onError).toHaveBeenCalledWith({ code: "folioTermsNotPublished" });
  });
});
