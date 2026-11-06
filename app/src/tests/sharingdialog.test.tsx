import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { adopt, t } from "../locales";
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
  name: "Rina Soto",
  alias: "rina",
  carrying: { pages: 1, pictures: 0, picturesBytes: 0, files: 0, filesBytes: 0 },
  cited: [],
  shared: [],
  limits: { ttl_days: [1, 3, 7, 15], shares_at_most: 2 },
  told: [],
};

const answering = (answers: Record<string, unknown>) =>
  invoked.mockImplementation((command: string) => {
    if (command in answers) return Promise.resolve(answers[command]);
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

    await screen.findByRole("heading", { name: t("termsChangedTitle") });
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
      doc_shared: { read: "https://tisty.cl/Ab3xK9pQ#key", edit: null, version: 1, taken: false },
    });
    const { onShared } = shown();

    await userEvent.click(await screen.findByRole("button", { name: t("shareGo") }));

    await screen.findByRole("heading", { name: t("sharedDone") });
    expect(invoked).toHaveBeenCalledWith("doc_shared", {
      id: "d1",
      cited: [],
      signedAs: "rina",
      editable: false,
      ttlDays: 1,
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

  it("closes and tells why when the terms cannot be read", async () => {
    invoked.mockImplementation(() => Promise.reject({ code: "folioTermsNotPublished" }));
    const { onError, onClose } = shown();

    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(onError).toHaveBeenCalledWith({ code: "folioTermsNotPublished" });
  });
});
