import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { keyHeard } from "../src/keys";
import { blockAt, remember, remembered } from "../src/place";
import { wordsFor } from "../src/words";

const host = vi.hoisted(() => ({
  openLink: vi.fn(),
  follow: vi.fn(),
  pictureNear: vi.fn(),
  pictureFar: vi.fn(),
}));
vi.mock("../src/host", () => host);

const { arrivingAt, canGoBack, goBack, linkPressed } = await import("../src/links");
const { bringFar, bringNearPictures, farPictures } = await import("../src/pictures");

const keyed = () => ({
  find: vi.fn(),
  side: vi.fn(),
  theme: vi.fn(),
  zoom: vi.fn(),
  head: vi.fn(),
  back: vi.fn(),
  source: vi.fn(),
  print: vi.fn(),
  close: vi.fn(),
  escape: vi.fn(),
});

const press = (init: KeyboardEventInit) => {
  const acts = keyed();
  const event = new KeyboardEvent("keydown", { cancelable: true, ...init });
  keyHeard(acts)(event);
  return { acts, event };
};

describe("the keys", () => {
  it("takes what it answers and leaves the rest alone", () => {
    const { acts, event } = press({ key: "f", ctrlKey: true });
    expect(acts.find).toHaveBeenCalled();
    expect(event.defaultPrevented).toBe(true);
    const plain = press({ key: "f" });
    expect(plain.acts.find).not.toHaveBeenCalled();
    expect(plain.event.defaultPrevented).toBe(false);
  });

  it("zooms each way and back", () => {
    expect(press({ key: "=", ctrlKey: true }).acts.zoom).toHaveBeenCalledWith(1);
    expect(press({ key: "-", ctrlKey: true }).acts.zoom).toHaveBeenCalledWith(-1);
    expect(press({ key: "0", ctrlKey: true }).acts.zoom).toHaveBeenCalledWith(0);
  });

  it("goes back with Alt and the left arrow", () => {
    expect(press({ key: "ArrowLeft", altKey: true }).acts.back).toHaveBeenCalled();
    expect(press({ key: "ArrowLeft" }).acts.back).not.toHaveBeenCalled();
  });

  it("goes from heading to heading", () => {
    expect(press({ key: "ArrowDown", altKey: true }).acts.head).toHaveBeenCalledWith(1);
    expect(press({ key: "ArrowUp", altKey: true }).acts.head).toHaveBeenCalledWith(-1);
  });

  it("keeps the shifted keys apart from the plain ones", () => {
    const { acts } = press({ key: "M", ctrlKey: true, shiftKey: true });
    expect(acts.source).toHaveBeenCalled();
    expect(press({ key: "L", ctrlKey: true, shiftKey: true }).acts.theme).toHaveBeenCalled();
    expect(press({ key: "F", ctrlKey: true, shiftKey: true }).acts.find).not.toHaveBeenCalled();
  });

  it("answers Escape, whatever else is held", () => {
    expect(press({ key: "Escape" }).acts.escape).toHaveBeenCalled();
  });
});

describe("where the reading was left", () => {
  beforeEach(() => localStorage.clear());

  it("is kept per file", () => {
    remember("a.md", 3);
    remember("b.md", 8);
    expect(remembered("a.md")).toBe(3);
    expect(remembered("b.md")).toBe(8);
  });

  it("is forgotten at the top, where a file opens anyway", () => {
    remember("a.md", 3);
    remember("a.md", 0);
    expect(remembered("a.md")).toBeUndefined();
  });

  it("keeps the latest two hundred files", () => {
    for (let at = 0; at < 205; at++) remember(`${at}.md`, 1);
    expect(remembered("0.md")).toBeUndefined();
    expect(remembered("204.md")).toBe(1);
    expect(
      Object.keys(JSON.parse(localStorage.getItem("tisty.viewer.places") ?? "{}")),
    ).toHaveLength(200);
  });

  it("survives what was kept being broken", () => {
    localStorage.setItem("tisty.viewer.places", "{");
    expect(remembered("a.md")).toBeUndefined();
  });

  const laid = (bottoms: number[]) => {
    const page = document.createElement("article");
    for (const bottom of bottoms) {
      const block = document.createElement("p");
      block.getBoundingClientRect = () => ({ bottom }) as DOMRect;
      page.append(block);
    }
    return page;
  };

  it("names the block at the top of the view", () => {
    expect(blockAt(laid([50, 120, 300]), 90)).toBe(1);
    expect(blockAt(laid([50, 120, 300]), 10)).toBe(0);
  });

  it("names the next block when the top of the view falls between two", () => {
    expect(blockAt(laid([85, 95, 300]), 90)).toBe(1);
    expect(blockAt(laid([60, 80, 100, 300]), 90)).toBe(2);
  });

  it("names the last block past the end, and nothing on an empty page", () => {
    expect(blockAt(laid([10, 20]), 90)).toBe(1);
    expect(blockAt(laid([]), 90)).toBe(0);
  });
});

describe("a link in the document", () => {
  beforeEach(() => vi.clearAllMocks());

  const pressed = (href: string) => {
    const page = document.createElement("article");
    page.innerHTML = `<h2 id="a b">t</h2><a href="${href}"><span>go</span></a>`;
    document.body.append(page);
    const heading = page.querySelector("h2") as HTMLElement;
    heading.scrollIntoView = vi.fn();
    const event = new MouseEvent("click", { bubbles: true, cancelable: true });
    page.addEventListener("click", (e) => linkPressed(e, "C:/here.md", () => {}));
    page.querySelector("span")?.dispatchEvent(event);
    page.remove();
    return { event, heading };
  };

  it("opens the web outside the window", () => {
    host.openLink.mockResolvedValue(undefined);
    const { event } = pressed("https://example.com/x");
    expect(host.openLink).toHaveBeenCalledWith("https://example.com/x");
    expect(event.defaultPrevented).toBe(true);
  });

  it("jumps to an anchor, encoded or not", () => {
    const { heading } = pressed("#a%20b");
    expect(heading.scrollIntoView).toHaveBeenCalled();
    expect(host.openLink).not.toHaveBeenCalled();
  });

  it("takes a neighbouring file into the same window, and remembers the way back", async () => {
    host.follow.mockResolvedValue(undefined);
    const reload = vi.fn();
    vi.stubGlobal("location", { reload });
    sessionStorage.clear();
    pressed("./other.md#part%202");
    expect(host.follow).toHaveBeenCalledWith("./other.md");
    await vi.waitFor(() => expect(reload).toHaveBeenCalled());
    expect(canGoBack()).toBe(true);
    expect(arrivingAt()).toBe("part 2");
    expect(arrivingAt()).toBeNull();
    await goBack();
    expect(host.follow).toHaveBeenLastCalledWith("C:/here.md");
    expect(canGoBack()).toBe(false);
    vi.unstubAllGlobals();
  });

  it("never navigates the window, nor opens what it does not know", () => {
    const { event } = pressed("file:///C:/secret.md");
    expect(event.defaultPrevented).toBe(true);
    expect(host.openLink).not.toHaveBeenCalled();
    pressed("javascript:alert(1)");
    expect(host.openLink).not.toHaveBeenCalled();
  });
});

describe("the words", () => {
  it("fill in a value wherever it is named", () => {
    expect(wordsFor("es").say("editTooBig", "500 KB")).toContain("500 KB");
    expect(wordsFor("en").say("editTooBig", "500 KB")).toContain("500 KB");
  });

  it("speak to the sheet in the same language", () => {
    expect(wordsFor("es").sayToSheet("saidnote")).toBe("Nota");
    expect(wordsFor("en").sayToSheet("saidnote")).toBe("Note");
    expect(wordsFor(null).locale).toBe("en");
  });
});

describe("the pictures", () => {
  const seen: { watch?: (entries: { isIntersecting: boolean; target: Element }[]) => void } = {};

  beforeEach(() => {
    vi.clearAllMocks();
    vi.stubGlobal(
      "IntersectionObserver",
      class {
        constructor(watch: (entries: { isIntersecting: boolean; target: Element }[]) => void) {
          seen.watch = watch;
        }
        observe = vi.fn();
        unobserve = vi.fn();
      },
    );
  });

  afterEach(() => vi.unstubAllGlobals());

  const sheet = (html: string) => {
    const page = document.createElement("article");
    page.innerHTML = html;
    return page;
  };

  it("brings one beside the file only when it comes near, and leaves the others", async () => {
    host.pictureNear.mockResolvedValue("data:image/png;base64,AA==");
    const page = sheet(
      '<img src="./a.png"><img src="tisty:b"><img src="data:x"><img src="https://x.y/c.png"><img>',
    );
    bringNearPictures(page);
    const first = page.querySelector("img") as HTMLImageElement;
    seen.watch?.([{ isIntersecting: false, target: first }]);
    expect(host.pictureNear).not.toHaveBeenCalled();
    seen.watch?.([{ isIntersecting: true, target: first }]);
    expect(host.pictureNear).toHaveBeenCalledTimes(1);
    expect(host.pictureNear).toHaveBeenCalledWith("./a.png");
    await vi.waitFor(() => expect(first.src).toContain("data:image/png"));
  });

  it("leaves its words when one beside the file cannot be read", async () => {
    host.pictureNear.mockRejectedValue({ code: "notAllowed" });
    const page = sheet('<img src="../secret.png" alt="x">');
    bringNearPictures(page);
    const only = page.querySelector("img") as HTMLImageElement;
    seen.watch?.([{ isIntersecting: true, target: only }]);
    await vi.waitFor(() => expect(host.pictureNear).toHaveBeenCalled());
    expect(only.getAttribute("src")).toBe("../secret.png");
  });

  it("brings all at once without an observer", () => {
    vi.stubGlobal("IntersectionObserver", undefined);
    delete (window as unknown as Record<string, unknown>).IntersectionObserver;
    host.pictureNear.mockResolvedValue("data:x");
    bringNearPictures(sheet('<img src="a.png"><img src="b.png">'));
    expect(host.pictureNear).toHaveBeenCalledTimes(2);
  });

  it("brings those from the web only when asked, and keeps each failure to itself", async () => {
    host.pictureFar.mockResolvedValueOnce("data:one").mockRejectedValueOnce({ code: "x" });
    const page = sheet(
      '<img src="HTTPS://x.y/a.png"><img src="http://x.y/b.png"><img src="c.png">',
    );
    const far = farPictures(page);
    expect(far).toHaveLength(2);
    expect(host.pictureFar).not.toHaveBeenCalled();
    expect(await bringFar(far)).toBe(1);
    expect(far[0].getAttribute("src")).toBe("data:one");
    expect(far[1].getAttribute("src")).toBe("http://x.y/b.png");
  });
});
