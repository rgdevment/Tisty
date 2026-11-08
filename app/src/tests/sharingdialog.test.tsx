import { openUrl } from "@tauri-apps/plugin-opener";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { weigh } from "../format";
import { adopt, fill, t } from "../locales";
import { noteParts } from "../sharing";
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
    if (command === "folio_sharing_now") return Promise.resolve({ state: "open" });
    if (command === "folio_terms_drawn") return Promise.resolve("<p>Los términos</p>");
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

describe("what Folio signs to say while closed", () => {
  it("makes a link only of Tisty's own https addresses", () => {
    expect(noteParts("Mira [aquí](https://folio.tisty.app/hoy) hoy")).toEqual([
      { at: 0, text: "Mira " },
      { at: 5, text: "aquí", url: "https://folio.tisty.app/hoy" },
      { at: 40, text: " hoy" },
    ]);
    expect(noteParts("[a](https://tisty.cl)")[0].url).toBe("https://tisty.cl");
    expect(noteParts("[a](https://tisty.app)")[0].url).toBe("https://tisty.app");
  });

  it.each([
    "[a](https://otro.example)",
    "[a](http://tisty.app)",
    "[a](https://tisty.app@otro.example)",
    "[a](https://tisty.app.otro.example)",
    "[a](https://eviltisty.app)",
    "[a](https://folio.tisty.app:8443)",
    "[a](mailto:hola@tisty.app)",
    "[a](javascript:alert)",
  ])("leaves %s as the words it is", (said) => {
    expect(noteParts(said)).toEqual([{ at: 0, text: said }]);
  });

  it("leaves anything else of Markdown or HTML as words", () => {
    const said = "**fuerte** <b>x</b> # y ![i](https://tisty.app/a.png)";
    expect(noteParts(said)).toEqual([{ at: 0, text: said }]);
  });
});

describe("what travels with a document", () => {
  const opened = async (carrying: object, limits: object = look.limits) => {
    answering({
      folio_terms_standing: { state: "accepted" },
      doc_share_look: { ...look, carrying: { ...look.carrying, ...carrying }, limits },
    });
    shown();
    await screen.findByRole("button", { name: t("shareGo") });
  };

  it("says a document of words carries its text, and what room is left for the rest", async () => {
    await opened(
      { pages: 0 },
      {
        pictures_at_most: 10,
        pictures_bytes_at_most: 5_000_000,
        files_at_most: 2,
        files_bytes_at_most: 5_000_000,
      },
    );

    expect(screen.getByText(t("shareCarriesWhole"))).toBeTruthy();
    expect(screen.getAllByText(t("shareCarriesNoPictures"))).toHaveLength(2);
    expect(screen.getByText(fill("shareRoomUpTo", "10", weigh(5_000_000)))).toBeTruthy();
    expect(screen.getByText(fill("shareRoomUpTo", "2", weigh(5_000_000)))).toBeTruthy();
  });

  it("counts what it carries against the room there is", async () => {
    await opened(
      { pages: 2, pictures: 4, picturesBytes: 1_200_000, files: 1, filesBytes: 2_400_000 },
      {
        pictures_at_most: 10,
        pictures_bytes_at_most: 5_000_000,
        files_at_most: 2,
        files_bytes_at_most: 5_000_000,
      },
    );

    expect(screen.getByText(fill("shareCarriesWholeAndPages", "2"))).toBeTruthy();
    expect(screen.getByText(`4 · ${weigh(1_200_000)}`, { exact: false })).toBeTruthy();
    expect(screen.getByText(fill("shareRoomOf", "10", weigh(5_000_000)))).toBeTruthy();
    expect(screen.getByText(`1 · ${weigh(2_400_000)}`, { exact: false })).toBeTruthy();
    expect(screen.getByText(fill("shareRoomOf", "2", weigh(5_000_000)))).toBeTruthy();
  });

  it("says which limit a picture passed, since the row counts only the total", async () => {
    await opened(
      {
        pictures: 1,
        picturesBytes: 1_200_000,
        largestPicture: 1_200_000,
        largestPictureName: "rina.jpg",
      },
      { pictures_at_most: 10, pictures_bytes_at_most: 5_000_000, picture_bytes_at_most: 1_000_000 },
    );

    expect(
      screen.getByText(
        fill("shareOverPicture", `rina.jpg (${weigh(1_200_000)})`, weigh(1_000_000)),
      ),
    ).toBeTruthy();
  });

  it("says how many it carries and how many Folio takes when a count passed", async () => {
    await opened(
      { pictures: 11, picturesBytes: 1_200_000, largestPicture: 500_000 },
      { pictures_at_most: 10, pictures_bytes_at_most: 5_000_000, picture_bytes_at_most: 1_000_000 },
    );

    expect(screen.getByText(fill("shareOverPictures", "11", "10"))).toBeTruthy();
  });

  it("weighs the words, pages and pictures apart from the attachments", async () => {
    await opened(
      {
        pictures: 2,
        picturesBytes: 1_000_000,
        mainBytes: 26_000_000,
        files: 1,
        filesBytes: 20_000_000,
      },
      { files_bytes_at_most: 24_000_000, envelope_bytes_at_most: 24_000_000 },
    );

    expect(
      screen.getByText(fill("shareOverWhole", weigh(26_000_000), weigh(24_000_000))),
    ).toBeTruthy();
  });

  it("lets attachments under their own ceiling pass, whatever the words weigh", async () => {
    await opened(
      { mainBytes: 1_000_000, files: 1, filesBytes: 23_000_000 },
      { files_bytes_at_most: 24_000_000, envelope_bytes_at_most: 24_000_000 },
    );

    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("names one page apart, and says no room it was not told of", async () => {
    await opened({ pages: 1 }, {});

    expect(screen.getByText(t("shareCarriesWholeAndPage"))).toBeTruthy();
    expect(screen.queryByText(fill("shareRoomUpTo", "10", weigh(5_000_000)))).toBeNull();
  });
});

describe("sharing a document with Folio", () => {
  it("says it is not available yet and asks for no terms while Folio is closed", async () => {
    answering({ folio_sharing_now: { state: "closed", note: null } });
    const { onClose } = shown();

    await screen.findByText(t("shareClosedWhat"), { exact: false });
    expect(screen.getByRole("button", { name: "folio.tisty.app" })).toBeTruthy();
    expect(invoked).not.toHaveBeenCalledWith("folio_terms_standing");

    await userEvent.click(screen.getByRole("button", { name: t("shareClosedOk") }));
    expect(onClose).toHaveBeenCalled();
  });

  it("shows what Folio signed to say in this language, and the standard notice without it", async () => {
    answering({
      folio_sharing_now: { state: "closed", note: { en: "We open on Monday.", es: "El lunes." } },
    });
    shown();
    await screen.findByText("We open on Monday.", { exact: false });
    expect(screen.queryByText(t("shareClosedWhat"), { exact: false })).toBeNull();
    expect(screen.getByRole("button", { name: "folio.tisty.app" })).toBeTruthy();

    cleanup();
    answering({ folio_sharing_now: { state: "closed", note: { es: "El lunes." } } });
    shown();
    await screen.findByText(t("shareClosedWhat"), { exact: false });
  });

  it("asks again when the first look reached nothing", async () => {
    let looks = 0;
    invoked.mockImplementation((command: string) => {
      if (command === "folio_sharing_now") {
        looks += 1;
        return Promise.resolve(
          looks === 1 ? { state: "unreached" } : { state: "closed", note: null },
        );
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
      if (command === "folio_sharing_now") return Promise.resolve({ state: "open" });
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

  it("lets terms already accepted stand when the new ones cannot be verified, and says so quietly", async () => {
    answering({ folio_terms_standing: { state: "stillAccepted" }, doc_share_look: look });
    const { onError } = shown();

    await screen.findByRole("button", { name: t("shareGo") });

    expect(screen.getByText(t("shareTermsUnchecked"))).toBeTruthy();
    expect(onError).not.toHaveBeenCalled();
  });

  it("says nothing of the terms when the accepted ones are the latest", async () => {
    answering({ folio_terms_standing: { state: "accepted" }, doc_share_look: look });
    shown();

    await screen.findByRole("button", { name: t("shareGo") });

    expect(screen.queryByText(t("shareTermsUnchecked"))).toBeNull();
  });

  it("says Folio is updating its terms and reads them again only when asked", async () => {
    let readings = 0;
    invoked.mockImplementation((command: string) => {
      if (command === "folio_sharing_now") return Promise.resolve({ state: "open" });
      if (command === "doc_share_look") return Promise.resolve(look);
      if (command === "folio_terms_standing") {
        readings += 1;
        return readings === 1
          ? Promise.reject({ code: "folioTermsUpdating" })
          : Promise.resolve({ state: "accepted" });
      }
      return Promise.reject({ code: "internal" });
    });
    const { onError } = shown();

    expect(await screen.findByText(t("folioTermsUpdating"))).toBeTruthy();
    await vi.advanceTimersByTimeAsync(10 * 60_000);
    expect(readings).toBe(1);
    await userEvent.click(screen.getByRole("button", { name: t("shareUnreachedRetry") }));

    await screen.findByRole("button", { name: t("shareGo") });
    expect(readings).toBe(2);
    expect(onError).not.toHaveBeenCalled();
  });

  it("after a refused send sends again only once asked and the terms are accepted", async () => {
    let sends = 0;
    let readings = 0;
    invoked.mockImplementation((command: string) => {
      if (command === "folio_sharing_now") return Promise.resolve({ state: "open" });
      if (command === "folio_terms_accept") return Promise.resolve(null);
      if (command === "folio_terms_drawn") return Promise.resolve("<p>Los términos</p>");
      if (command === "folio_terms_standing") {
        readings += 1;
        return Promise.resolve(
          readings === 1
            ? { state: "accepted" }
            : { state: "toAccept", version: "13", text: "Los términos", acceptedBefore: "12" },
        );
      }
      if (command === "doc_share_look") return Promise.resolve(look);
      if (command === "doc_shared") {
        sends += 1;
        return sends === 1
          ? Promise.reject({ code: "folioTermsUpdating" })
          : Promise.resolve({
              read: "https://tisty.cl/Ab3xK9pQ#key",
              edit: null,
              version: 1,
              taken: false,
              held: false,
              told: [],
            });
      }
      return Promise.reject({ code: "internal" });
    });
    const { onError } = shown();

    await userEvent.click(await screen.findByRole("button", { name: t("shareGo") }));
    expect(await screen.findByText(t("folioTermsUpdating"))).toBeTruthy();
    await vi.advanceTimersByTimeAsync(10 * 60_000);
    expect(sends).toBe(1);
    expect(readings).toBe(1);
    await userEvent.click(screen.getByRole("button", { name: t("shareUnreachedRetry") }));
    await userEvent.click(await screen.findByRole("checkbox"));
    await userEvent.click(screen.getByRole("button", { name: t("termsGo") }));

    await screen.findByRole("heading", { name: t("sharedDone") });
    expect(sends).toBe(2);
    expect(readings).toBe(2);
    expect(onError).not.toHaveBeenCalled();
  });

  it("shows the terms Folio refused over and sends again once they are accepted", async () => {
    let sends = 0;
    let readings = 0;
    invoked.mockImplementation((command: string) => {
      if (command === "folio_sharing_now") return Promise.resolve({ state: "open" });
      if (command === "doc_share_look") return Promise.resolve(look);
      if (command === "folio_terms_accept") return Promise.resolve(null);
      if (command === "folio_terms_drawn") return Promise.resolve("<p>Los términos</p>");
      if (command === "folio_terms_standing") {
        readings += 1;
        return Promise.resolve(
          readings === 1
            ? { state: "accepted" }
            : { state: "toAccept", version: "2", text: "Los términos", acceptedBefore: "1" },
        );
      }
      if (command === "doc_shared") {
        sends += 1;
        return sends === 1
          ? Promise.reject({ code: "folioTermsToAccept" })
          : Promise.resolve({
              read: "https://tisty.cl/Ab3xK9pQ#key",
              edit: null,
              version: 1,
              taken: false,
              held: false,
              told: [],
            });
      }
      return Promise.reject({ code: "internal" });
    });
    shown();

    await userEvent.click(await screen.findByRole("button", { name: t("shareGo") }));
    await userEvent.click(await screen.findByRole("checkbox"));
    await userEvent.click(screen.getByRole("button", { name: t("termsGo") }));

    await screen.findByRole("heading", { name: t("sharedDone") });
    expect(sends).toBe(2);
    expect(invoked.mock.calls.filter(([one]) => one === "doc_share_look")).toHaveLength(1);
  });

  it("opens a web or mail link in the terms and nothing else", async () => {
    answering({
      folio_terms_standing: {
        state: "toAccept",
        version: "2",
        text: "Los términos",
        acceptedBefore: null,
      },
      folio_terms_drawn:
        '<p><a href="mailto:abuso@tisty.app">correo</a> <a href="https://tisty.cl/x">web</a> <a href="tisty://open?path=x">otro</a></p>',
    });
    shown();
    vi.mocked(openUrl).mockClear();

    await userEvent.click(await screen.findByRole("link", { name: "correo" }));
    await userEvent.click(screen.getByRole("link", { name: "web" }));
    await userEvent.click(screen.getByRole("link", { name: "otro" }));

    expect(vi.mocked(openUrl).mock.calls.map(([url]) => url)).toEqual([
      "mailto:abuso@tisty.app",
      "https://tisty.cl/x",
    ]);
  });

  it("hands over to whoever asked when only the terms were needed", async () => {
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
    const onTermsAccepted = vi.fn();
    render(
      <Sharing
        doc="d1"
        title="Viaje"
        termsOnly
        onTermsAccepted={onTermsAccepted}
        onShared={vi.fn()}
        onClose={onClose}
      />,
    );

    await userEvent.click(await screen.findByRole("checkbox"));
    await userEvent.click(screen.getByRole("button", { name: t("termsGo") }));

    await waitFor(() => expect(onTermsAccepted).toHaveBeenCalled());
    expect(onClose).not.toHaveBeenCalled();
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
