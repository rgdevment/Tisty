import { cleanup, render, screen, waitFor } from "@solidjs/testing-library";
import pageCss from "tisty-sheet/page.css?raw";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import App from "../src/App";
import type { Opened } from "../src/host";

const host = vi.hoisted(() => ({
  opened: vi.fn(),
  theRest: vi.fn(),
  source: vi.fn(),
  facts: vi.fn(),
  editCopy: vi.fn(),
  editAsked: vi.fn(),
  shown: vi.fn(),
  openLink: vi.fn(),
  reveal: vi.fn(),
  pictureNear: vi.fn(),
  pictureFar: vi.fn(),
  follow: vi.fn(),
  themed: vi.fn(),
}));

vi.mock("../src/host", () => host);
vi.mock("../src/drawing", () => ({
  readAs: () => ({ kept: [], drawn: Promise.resolve() }),
  diagrams: () => Promise.resolve(),
}));

const drawn = (first: string, more = false): Opened => ({
  kind: "drawn",
  name: "notes.md",
  place: "Downloads",
  path: "C:/Downloads/notes.md",
  first,
  more,
  heads: [{ level: 2, said: "One", id: "one" }],
});

const open = (asked: string | null = "es") => {
  const close = vi.fn();
  render(() => <App asked={asked} close={close} />);
  return close;
};

describe("the light viewer", () => {
  afterEach(cleanup);

  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
    host.facts.mockResolvedValue({ wrote: 1_700_000_000, bytes: 2048 });
    host.editAsked.mockResolvedValue(() => {});
    host.themed.mockResolvedValue(undefined);
    host.shown.mockResolvedValue(undefined);
    host.theRest.mockResolvedValue([]);
    window.scrollTo = vi.fn();
    Element.prototype.scrollIntoView = vi.fn();
  });

  it("puts the first words on the page before it shows the window", async () => {
    host.opened.mockResolvedValue(drawn('<h2 id="one">One</h2>'));
    let wordsWhenShown = "";
    host.shown.mockImplementation(async () => {
      wordsWhenShown = document.querySelector(".page")?.textContent ?? "";
    });
    open();
    await waitFor(() => expect(host.shown).toHaveBeenCalled());
    expect(wordsWhenShown).toBe("One");
  });

  it("draws a file's first line as it was written, never as a document's title", async () => {
    const sheet = document.createElement("style");
    sheet.textContent = pageCss;
    document.head.append(sheet);
    const titles = Array.from(sheet.sheet?.cssRules ?? [])
      .filter((rule): rule is CSSStyleRule => rule instanceof CSSStyleRule)
      .filter((rule) => rule.style.getPropertyValue("letter-spacing") === "-0.01em")
      .map((rule) => rule.selectorText);
    const titled = (one: Element | null) => titles.some((title) => one?.matches(title));
    const theirs = document.createElement("article");
    theirs.className = "page";
    theirs.innerHTML = "<h2>Theirs</h2><p>after</p>";
    document.body.append(theirs);
    host.opened.mockResolvedValue(drawn('<h2 id="one">One</h2><p>after</p>'));
    open();
    await waitFor(() => expect(document.getElementById("one")).not.toBeNull());
    expect(titled(theirs.firstElementChild)).toBe(true);
    expect(titled(document.getElementById("one"))).toBe(false);
    theirs.remove();
    sheet.remove();
  });

  it("lays what is left after the first piece, in order", async () => {
    host.opened.mockResolvedValue(drawn("<p>first</p>", true));
    host.theRest.mockResolvedValue(["<p>second</p>", "<p>third</p>"]);
    open();
    await waitFor(() =>
      expect(document.querySelector(".page")?.textContent).toBe("firstsecondthird"),
    );
  });

  it("does not ask for the rest when the first piece is all there is", async () => {
    host.opened.mockResolvedValue(drawn("<p>all</p>"));
    open();
    await waitFor(() => expect(host.facts).toHaveBeenCalled());
    expect(host.theRest).not.toHaveBeenCalled();
  });

  it("says Editar, and nothing about Tisty, on its main button", async () => {
    host.opened.mockResolvedValue(drawn("<p>all</p>"));
    open();
    const button = await screen.findByRole("button", { name: "Editar" });
    expect(button.textContent).toBe("Editar");
  });

  it("closes once the copy is made and nothing was left behind", async () => {
    host.opened.mockResolvedValue(drawn("<p>all</p>"));
    host.editCopy.mockResolvedValue({ missed: 0, notes: 0 });
    const close = open();
    (await screen.findByRole("button", { name: "Editar" })).click();
    await waitFor(() => expect(close).toHaveBeenCalled());
  });

  it("waits for the welcome on a Tisty not set up yet, and makes the copy when it ends", async () => {
    host.opened.mockResolvedValue(drawn("<p>all</p>"));
    host.editCopy.mockResolvedValue(null);
    const close = open();
    (await screen.findByRole("button", { name: "Editar" })).click();
    expect((await screen.findByRole("alert")).textContent).toContain("configurar Tisty");
    expect(close).not.toHaveBeenCalled();

    host.editCopy.mockResolvedValue({ missed: 0, notes: 0 });
    host.editAsked.mock.calls[0][0]();
    await waitFor(() => expect(close).toHaveBeenCalled());
    expect(host.editCopy).toHaveBeenCalledTimes(2);
  });

  it("stops listening when it goes away", async () => {
    const stop = vi.fn();
    host.opened.mockResolvedValue(drawn("<p>all</p>"));
    host.editAsked.mockResolvedValue(stop);
    open();
    await screen.findByRole("button", { name: "Editar" });
    cleanup();
    await waitFor(() => expect(stop).toHaveBeenCalled());
  });

  it("stays open to say what a copy left behind", async () => {
    host.opened.mockResolvedValue(drawn("<p>all</p>"));
    host.editCopy.mockResolvedValue({ missed: 2, notes: 0 });
    const close = open();
    (await screen.findByRole("button", { name: "Editar" })).click();
    const said = await screen.findByRole("alert");
    expect(said.textContent).toContain("2");
    expect(close).not.toHaveBeenCalled();
  });

  it("says a file it reads is too long to edit, and that the file stays open", async () => {
    host.opened.mockResolvedValue(drawn("<p>all</p>"));
    host.editCopy.mockRejectedValue({ code: "documentTooBig", name: "500 KB" });
    const close = open();
    (await screen.findByRole("button", { name: "Editar" })).click();
    expect((await screen.findByRole("alert")).textContent).toContain("500 KB");
    expect(close).not.toHaveBeenCalled();
  });

  it("does not show a refusal's code to whoever reads", async () => {
    host.opened.mockResolvedValue(drawn("<p>all</p>"));
    host.editCopy.mockRejectedValue({ code: "internal" });
    open();
    (await screen.findByRole("button", { name: "Editar" })).click();
    expect((await screen.findByRole("alert")).textContent).not.toContain("internal");
  });

  it("shows what was written, and goes back to the document", async () => {
    host.opened.mockResolvedValue(drawn("<p>all</p>"));
    host.source.mockResolvedValue("# written");
    open();
    (await screen.findByRole("button", { name: "Ver Markdown" })).click();
    await waitFor(() => expect(document.querySelector(".written")?.textContent).toBe("# written"));
    expect((document.querySelector(".page") as HTMLElement).hidden).toBe(true);
    (await screen.findByRole("button", { name: "Ver el documento" })).click();
    await waitFor(() => expect(document.querySelector(".written")).toBeNull());
    expect((document.querySelector(".page") as HTMLElement).hidden).toBe(false);
  });

  it("offers to edit an empty file rather than an empty page", async () => {
    host.opened.mockResolvedValue(drawn(""));
    open();
    expect(await screen.findByText("Este archivo está vacío.")).toBeTruthy();
    expect(host.facts).not.toHaveBeenCalled();
  });

  it("says a file weighs too much for text, and what to do, and still shows the window", async () => {
    host.opened.mockResolvedValue({
      kind: "missed",
      name: "big.md",
      place: "Downloads",
      path: "C:/Downloads/big.md",
      refusal: { code: "documentTooBig", name: "5 MB" },
    });
    open();
    expect(
      await screen.findByText(
        /pesa demasiado para ser solo texto.*más de 1 MB.*Divídelo en dos o intenta con otro editor/,
      ),
    ).toBeTruthy();
    expect(host.shown).toHaveBeenCalled();
  });

  it("still shows the window when the file could not be asked for", async () => {
    host.opened.mockRejectedValue(new Error("broken"));
    open();
    await waitFor(() => expect(host.shown).toHaveBeenCalled());
    expect((await screen.findByRole("alert")).textContent).toContain("broken");
  });

  it("brings a picture beside the file, and leaves one from the web until it is asked for", async () => {
    host.opened.mockResolvedValue(
      drawn('<p><img src="./a.png" alt="near"><img src="https://x.y/b.png" alt="far"></p>'),
    );
    host.pictureNear.mockResolvedValue("data:image/png;base64,AA==");
    host.pictureFar.mockResolvedValue("data:image/png;base64,BB==");
    open();
    await waitFor(() => expect(host.pictureNear).toHaveBeenCalledWith("./a.png"));
    expect(host.pictureFar).not.toHaveBeenCalled();
    (await screen.findByRole("button", { name: /imágenes de internet/ })).click();
    await waitFor(() => expect(host.pictureFar).toHaveBeenCalledWith("https://x.y/b.png"));
    await waitFor(() =>
      expect(document.querySelector('img[alt="far"]')?.getAttribute("src")).toContain("data:"),
    );
    await waitFor(() =>
      expect(screen.queryByRole("button", { name: /imágenes de internet/ })).toBeNull(),
    );
  });

  it("prints a dark window on light paper, and goes back to dark after", async () => {
    host.opened.mockResolvedValue(drawn("<p>all</p>"));
    const root = document.documentElement;
    root.classList.add("dark");
    root.dataset.theme = "dark";
    let themeOnPaper = "";
    window.print = () => {
      themeOnPaper = root.dataset.theme ?? "";
    };
    open();
    await screen.findByRole("button", { name: "Editar" });
    window.dispatchEvent(new KeyboardEvent("keydown", { key: "p", ctrlKey: true }));
    await waitFor(() => expect(themeOnPaper).toBe("light"));
    window.dispatchEvent(new Event("afterprint"));
    expect(root.dataset.theme).toBe("dark");
    expect(root.classList.contains("dark")).toBe(true);
    root.classList.remove("dark");
    delete root.dataset.theme;
  });

  it("says how many pictures from the web could not be brought", async () => {
    host.opened.mockResolvedValue(drawn('<p><img src="https://x.y/b.png"></p>'));
    host.pictureFar.mockRejectedValue({ code: "notAllowed" });
    open();
    (await screen.findByRole("button", { name: /imágenes de internet/ })).click();
    expect((await screen.findByRole("alert")).textContent).toContain("1");
  });

  it("speaks English when not asked for Spanish", async () => {
    host.opened.mockResolvedValue(drawn("<p>all</p>"));
    open("en");
    expect(await screen.findByRole("button", { name: "Edit" })).toBeTruthy();
  });

  it("marks the next and the last words of a search across the page", async () => {
    host.opened.mockResolvedValue(drawn("<p>uno dos</p><p>dos</p>"));
    open();
    (await screen.findByRole("button", { name: "Editar" })).focus();
    window.dispatchEvent(new KeyboardEvent("keydown", { key: "f", ctrlKey: true }));
    const field = await screen.findByRole("searchbox");
    (field as HTMLInputElement).value = "DOS";
    field.dispatchEvent(new InputEvent("input", { bubbles: true }));
    expect((await screen.findByRole("status")).textContent).toBe("1 / 2");
  });
});
