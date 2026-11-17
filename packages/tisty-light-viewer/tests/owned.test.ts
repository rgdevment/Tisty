import { afterEach, describe, expect, it, vi } from "vitest";
import { wordsFor } from "../src/words";

const host = vi.hoisted(() => ({
  served: vi.fn(),
  keptHere: vi.fn(),
  weighs: vi.fn(),
  fileUrl: (real: string) => `asset://${real}`,
  pictureNear: vi.fn(),
  pictureFar: vi.fn(),
}));
vi.mock("../src/host", () => host);

const { drawnOwn } = await import("../src/owned");

const THEME = { dark: () => false, onTheme: () => () => {} };

const sheet = (html: string) => {
  const page = document.implementation.createHTMLDocument("");
  page.body.innerHTML = `<article class="page">${html}</article>`;
  return page;
};

const WIDGET =
  '<figure class="code"><figcaption>Paseos</figcaption><pre><code class="language-widget">';

describe("a document of one's own, drawn apart", () => {
  afterEach(() => vi.clearAllMocks());

  it("draws a file as its card, with its kind and weight, in the window's language", async () => {
    host.weighs.mockResolvedValue(2048);
    const page = sheet('<p><img src="cartilla.pdf" alt="La cartilla de Rina"></p>');
    drawnOwn(page, wordsFor("es"), THEME);
    const card = page.querySelector("a.doc-card.is-file") as HTMLAnchorElement;
    expect(card.getAttribute("href")).toBe("cartilla.pdf");
    expect(card.textContent).toContain("La cartilla de Rina");
    await vi.waitFor(() => expect(card.textContent).toContain("PDF · 2.0 kB"));
    expect(host.weighs).toHaveBeenCalledWith("cartilla.pdf");
  });

  it("plays a sound kept here, and offers to bring one still in the cloud", async () => {
    host.keptHere.mockImplementation(async (src: string) => src === "bark.mp3");
    host.served.mockResolvedValue("C:/store/bark.mp3");
    host.weighs.mockResolvedValue(5_000_000);
    const page = sheet('<p><img src="bark.mp3" alt="Un ladrido"></p><p><img src="walk.mp4"></p>');
    drawnOwn(page, wordsFor("es"), THEME);
    const sound = page.querySelector("audio") as HTMLAudioElement;
    await vi.waitFor(() => expect(sound.getAttribute("src")).toBe("asset://C:/store/bark.mp3"));
    const away = page.querySelector(".media.is-video .media-bring") as HTMLButtonElement;
    await vi.waitFor(() => expect(away.textContent).toBe("▶ Traer de tu nube · 5.0 MB"));
    expect(host.served).toHaveBeenCalledTimes(1);
  });

  it("leaves pictures and what comes from elsewhere to the pictures, and the web to its card", () => {
    const page = sheet(
      '<img src="rina.jpg"><img src="https://x.y/a.pdf"><img src="data:image/png;base64,AA=="><img src="tisty:doc/ab">',
    );
    host.pictureNear.mockResolvedValue("data:image/jpeg;base64,AA==");
    drawnOwn(page, wordsFor("es"), THEME);
    expect(page.querySelectorAll("img")).toHaveLength(4);
    expect(host.weighs).not.toHaveBeenCalled();
    expect(host.pictureNear).toHaveBeenCalledTimes(1);
    expect(host.pictureNear).toHaveBeenCalledWith("rina.jpg");
  });

  it("draws a widget of HTML and CSS in a frame that runs nothing", () => {
    const page = sheet(`${WIDGET}&lt;div class="card"&gt;42&lt;/div&gt;</code></pre></figure>`);
    const lent = drawnOwn(page, wordsFor("es"), THEME);
    expect(lent).toHaveLength(1);
    expect(page.querySelector(".widget-name")?.textContent).toBe("Paseos");
    const frame = page.querySelector("figure.widget iframe") as HTMLIFrameElement;
    expect(frame.getAttribute("sandbox")).toBe("allow-same-origin");
    expect(frame.title).toBe("Widget dibujado a partir de este bloque");
    expect(frame.srcdoc).toContain("42");
  });

  it("keeps as code, and says why, a widget that would run something", () => {
    const page = sheet(`${WIDGET}&lt;script&gt;alert(1)&lt;/script&gt;</code></pre></figure>`);
    expect(drawnOwn(page, wordsFor("es"), THEME)).toHaveLength(0);
    expect(page.querySelector("iframe")).toBeNull();
    expect(page.querySelector("code.language-widget")).not.toBeNull();
    expect(page.querySelector(".lit-locked")?.textContent).toContain("no es un widget");
  });
});
