import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  answered,
  fitted,
  iconed,
  pictures,
  type Reaching,
  widgets,
  wired,
  worn,
} from "../ui/dressing";
import { charsAt, colored, dressed, escaped, saidKinds, shell } from "../ui/paging";

const calls = vi.hoisted(() => ({
  links: [] as string[],
  served: [] as string[],
  shown: [] as string[],
  released: [] as string[],
  lent: [] as string[],
  dropped: 0,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: { url?: string }) => {
    if (cmd === "open_link") calls.links.push(args?.url ?? "");
    return Promise.resolve(null);
  },
  convertFileSrc: (path: string) => `http://asset.localhost/${path}`,
}));

vi.mock("../inUse", () => ({
  serve: (at: string) => {
    calls.served.push(at);
    return Promise.resolve(`C:/kept/${at}`);
  },
  showing: (at: string) => {
    calls.shown.push(at);
    return () => calls.released.push(at);
  },
}));

vi.mock("../ui/widgeting", () => ({
  lend: (_: HTMLElement, source: string) => {
    calls.lent.push(source);
    return {
      retheme: () => {},
      shade: () => {},
      drop: () => {
        calls.dropped += 1;
      },
    };
  },
  locked: () => {
    const said = document.createElement("div");
    said.className = "lit-locked";
    return said;
  },
}));

vi.mock("../glyphs", () => ({
  markup: (name: string) => (name === "star" ? '<path d="M1 1"/>' : null),
}));

const page = (html: string): Document => new DOMParser().parseFromString(shell(html), "text/html");

const reaching = () => {
  const heard = {
    docs: [] as string[],
    opened: [] as string[],
    tags: [] as string[],
    blocks: [] as [number, number][],
  };
  const reach: Reaching = {
    body: "# Título\n\nÁrbol.\n",
    onDoc: (file) => heard.docs.push(file),
    onOpen: (reference) => heard.opened.push(reference),
    onTag: (name) => heard.tags.push(name),
    onBlock: (at) => heard.blocks.push(at),
  };
  return { heard, reach };
};

beforeEach(() => {
  calls.links = [];
  calls.served = [];
  calls.shown = [];
  calls.released = [];
  calls.lent = [];
  calls.dropped = 0;
  document.documentElement.removeAttribute("data-theme");
});

describe("the shell of a drawn page", () => {
  it("speaks the window's language, starts untouchable and carries the theme", () => {
    document.documentElement.lang = "es";
    document.documentElement.dataset.theme = "dark";
    const said = shell("<p>x</p>");
    expect(said).toContain('lang="es"');
    expect(said).toContain('class="dark"');
    expect(said).toContain('<article class="page" inert>');
    expect(said).toContain(":root{--tisty-sheet:");
  });

  it("escapes what it is asked to show as plain words", () => {
    expect(escaped("<b>&</b>")).toBe("&lt;b&gt;&amp;&lt;/b&gt;");
  });
});

describe("dressing a drawn page", () => {
  it("follows the window's theme", () => {
    const drawn = page("<p>x</p>");
    document.documentElement.dataset.theme = "dark";
    dressed(drawn);
    expect(drawn.documentElement.classList.contains("dark")).toBe(true);
    document.documentElement.dataset.theme = "light";
    dressed(drawn);
    expect(drawn.documentElement.classList.contains("dark")).toBe(false);
  });

  it("sizes the page to the sheet picked and its zoom", () => {
    const drawn = page("<p>x</p>");
    fitted(drawn, 900, 1.25);
    expect(drawn.documentElement.style.getPropertyValue("--wide")).toBe("900px");
    expect(drawn.documentElement.style.getPropertyValue("zoom")).toBe("1.25");
    fitted(drawn);
    expect(drawn.documentElement.style.getPropertyValue("zoom")).toBe("1");
  });

  it("names every callout in the window's words", () => {
    const drawn = page('<blockquote class="callout" data-callout="tip"><p>x</p></blockquote>');
    saidKinds(drawn, (kind) => `said ${kind}`);
    expect(drawn.querySelector("blockquote")?.dataset.said).toBe("said tip");
  });

  it("draws a known icon with its hue and leaves an odd hue out", () => {
    const drawn = page(
      '<p><span class="ico" data-ico="star" data-hue="blue"></span><span class="ico" data-ico="star" data-hue="a b"></span><span class="ico" data-ico="nope"></span></p>',
    );
    iconed(drawn);
    const [blue, odd, unknown] = Array.from(drawn.querySelectorAll<HTMLElement>("span.ico"));
    expect(blue.querySelector("svg")?.getAttribute("class")).toBe("glyph");
    expect(blue.classList.contains("ico-blue")).toBe(true);
    expect(odd.querySelector("svg")).not.toBeNull();
    expect(odd.className).toBe("ico");
    expect(unknown.querySelector("svg")).toBeNull();
  });

  it("brings a kept picture in, keeps it alive while shown and lets it go after", async () => {
    const drawn = page(
      '<p><img src="attachments/a.png" alt="A"><img src="https://x.y/b.png" alt="B"></p>',
    );
    const held = pictures(drawn, () => true);
    expect(calls.shown).toEqual(["attachments/a.png"]);
    await vi.waitFor(() =>
      expect(drawn.querySelector('img[alt="A"]')?.getAttribute("src")).toBe(
        "http://asset.localhost/C:/kept/attachments/a.png",
      ),
    );
    expect(drawn.querySelector('img[alt="B"]')?.getAttribute("src")).toBe("https://x.y/b.png");
    for (const one of held) one();
    expect(calls.released).toEqual(["attachments/a.png"]);
  });

  it("shows an attachment that is no picture as a card that opens it", () => {
    const drawn = page('<p><img src="attachments/plan.pdf" alt=""></p>');
    pictures(drawn, () => true);
    const card = drawn.querySelector("a.card");
    expect(card?.getAttribute("href")).toBe("attachments/plan.pdf");
    expect(card?.textContent).toBe("plan.pdf");
    expect(calls.served).toEqual([]);
  });

  it("does not dress a picture once the page is gone", async () => {
    const drawn = page('<p><img src="attachments/a.png" alt="A"></p>');
    pictures(drawn, () => false);
    await Promise.resolve();
    await Promise.resolve();
    expect(drawn.querySelector("img")?.hasAttribute("src")).toBe(false);
  });

  it("lends a widget of one's own and only says why for a file from outside", () => {
    const code =
      '<figure class="code" data-at="4-40"><pre><code class="language-widget">&lt;p&gt;hi&lt;/p&gt;</code></pre></figure>';
    const own = page(code);
    const lent = widgets(own, false);
    expect(lent).toHaveLength(1);
    expect(calls.lent).toEqual(["<p>hi</p>"]);
    expect(own.querySelector<HTMLElement>("figure.widget")?.dataset.at).toBe("4-40");
    const theirs = page(code);
    expect(widgets(theirs, true)).toHaveLength(0);
    expect(theirs.querySelector(".lit-locked + figure.code")).not.toBeNull();
  });

  it("gives a widget a bar with its name, and ways to its code and apart only where they lead", () => {
    const code =
      '<figure class="code" data-at="4-40"><figcaption>Rina</figcaption><pre><code class="language-widget">&lt;b&gt;42&lt;/b&gt;</code></pre></figure>';
    const apart: string[] = [];
    const reach = {
      body: "",
      onDoc: () => {},
      onBlock: () => {},
      onApart: (source: string) => apart.push(source),
    };
    const own = page(code);
    widgets(own, false, reach);
    const bar = own.querySelector(".widget-bar");
    expect(bar?.querySelector(".widget-name")?.textContent).toBe("Rina");
    expect(
      Array.from(bar?.querySelectorAll<HTMLElement>("button") ?? []).map(
        (one) => one.dataset.asked,
      ),
    ).toEqual(["code", "apart"]);
    answered(own, { kind: "apart", at: [4, 40] }, reach);
    expect(apart).toEqual(["<b>42</b>"]);
    const read = page(code);
    widgets(read, false, { body: "", onDoc: () => {} });
    expect(read.querySelectorAll(".widget-bar button")).toHaveLength(0);
  });

  it("colours code in a language it knows and leaves the rest as written", async () => {
    const drawn = page(
      '<figure class="code"><pre><code class="language-ts">const a = "b";</code></pre></figure><figure class="code"><pre><code class="language-nope">x</code></pre></figure>',
    );
    colored(drawn, () => true);
    const [known, odd] = Array.from(drawn.querySelectorAll("code"));
    expect(known.querySelector(".hljs-keyword")?.textContent).toBe("const");
    expect(known.textContent).toBe('const a = "b";');
    expect(odd.children).toHaveLength(0);
  });

  it("dresses the whole page in one go", () => {
    const drawn = page(
      '<blockquote class="callout" data-callout="note"><p>x</p></blockquote><figure class="code" data-at="1-2"><pre><code class="language-widget">w</code></pre></figure><p><img src="attachments/a.png" alt=""></p>',
    );
    const done = worn(drawn, false, () => true);
    expect(done.lent).toHaveLength(1);
    expect(done.kept).toHaveLength(1);
    expect(drawn.querySelector("blockquote")?.dataset.said).toBeTruthy();
  });
});

describe("what a click on a drawn page does", () => {
  it("opens documents, the web, attachments, tags and blocks where each belongs", () => {
    const drawn = page('<h2 id="here">Here</h2><p data-at="11-19">Árbol.</p>');
    const { heard, reach } = reaching();
    answered(drawn, { kind: "doc", id: "ab12-0001" }, reach);
    answered(drawn, { kind: "web", href: "https://x.y" }, reach);
    answered(drawn, { kind: "attachment", reference: "attachments/a.pdf" }, reach);
    answered(drawn, { kind: "tag", name: "work" }, reach);
    answered(drawn, { kind: "block", at: [11, 19] }, reach);
    answered(drawn, { kind: "here", id: "here" }, reach);
    answered(drawn, { kind: "block", at: [0, 3] }, reach);
    answered(drawn, null, reach);
    expect(heard.docs).toEqual(["ab12-0001"]);
    expect(calls.links).toEqual(["https://x.y"]);
    expect(heard.opened).toEqual(["attachments/a.pdf"]);
    expect(heard.tags).toEqual(["work"]);
    expect(heard.blocks).toEqual([[10, 17]]);
  });

  it("counts a block by characters, whatever the accents before it", () => {
    expect(charsAt("# Título\n\nÁrbol.\n", 11)).toBe(10);
    expect(charsAt("abc", 2)).toBe(2);
  });
});

describe("a drawn page wired to the window", () => {
  it("routes a click on a card and keeps the page from going anywhere", () => {
    const drawn = page('<p data-at="0-9"><a class="card" data-doc="ab12-0001">A</a></p>');
    const { heard, reach } = reaching();
    wired(drawn, () => reach);
    const click = new MouseEvent("click", { bubbles: true, cancelable: true, button: 0 });
    drawn.querySelector("a")?.dispatchEvent(click);
    expect(click.defaultPrevented).toBe(true);
    expect(heard.docs).toEqual(["ab12-0001"]);
  });

  it("ignores a click that is not the main button", () => {
    const drawn = page('<p data-at="0-9"><a class="card" data-doc="ab12-0001">A</a></p>');
    const { heard, reach } = reaching();
    wired(drawn, () => reach);
    drawn
      .querySelector("a")
      ?.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true, button: 1 }));
    expect(heard.docs).toEqual([]);
  });

  it("holds the browser's own menu, drags, forms and middle clicks", () => {
    const drawn = page("<p>x</p>");
    const { reach } = reaching();
    wired(drawn, () => reach);
    for (const kind of ["contextmenu", "auxclick", "dragstart", "submit"]) {
      const event = new Event(kind, { bubbles: true, cancelable: true });
      drawn.querySelector("p")?.dispatchEvent(event);
      expect(event.defaultPrevented).toBe(true);
    }
  });

  it("passes a key pressed in the page to the window, and keeps what the window took", () => {
    const drawn = page("<p>x</p>");
    const { reach } = reaching();
    wired(drawn, () => reach);
    const took = (e: KeyboardEvent) => {
      if (e.key === "f" && e.ctrlKey) e.preventDefault();
    };
    window.addEventListener("keydown", took);
    const find = new KeyboardEvent("keydown", {
      key: "f",
      ctrlKey: true,
      bubbles: true,
      cancelable: true,
    });
    drawn.body.dispatchEvent(find);
    const copy = new KeyboardEvent("keydown", {
      key: "c",
      ctrlKey: true,
      bubbles: true,
      cancelable: true,
    });
    drawn.body.dispatchEvent(copy);
    window.removeEventListener("keydown", took);
    expect(find.defaultPrevented).toBe(true);
    expect(copy.defaultPrevented).toBe(false);
  });

  it("keeps the keys typed into a block to the block, bar the window's own shortcuts", () => {
    const drawn = page('<div class="editing"><p>x</p></div>');
    const { reach } = reaching();
    wired(drawn, () => reach);
    const heard: string[] = [];
    const listen = (e: KeyboardEvent) => heard.push(e.key);
    window.addEventListener("keydown", listen);
    const inside = drawn.querySelector(".editing p") as HTMLElement;
    for (const key of ["a", "Escape"]) {
      inside.dispatchEvent(new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true }));
    }
    const taken = new KeyboardEvent("keydown", {
      key: "z",
      ctrlKey: true,
      bubbles: true,
      cancelable: true,
    });
    taken.preventDefault();
    inside.dispatchEvent(taken);
    inside.dispatchEvent(
      new KeyboardEvent("keydown", { key: "w", ctrlKey: true, bubbles: true, cancelable: true }),
    );
    window.removeEventListener("keydown", listen);
    expect(heard).toEqual(["w"]);
  });

  it("opens the document a widget asks for", () => {
    const drawn = page("<p>x</p>");
    const { heard, reach } = reaching();
    wired(drawn, () => reach);
    drawn.body.dispatchEvent(new CustomEvent("widgetdoc", { bubbles: true, detail: "ab12-0003" }));
    drawn.body.dispatchEvent(new CustomEvent("widgetdoc", { bubbles: true, detail: "" }));
    expect(heard.docs).toEqual(["ab12-0003"]);
  });
});

const writer = (body = "") => {
  const asked: { at: [number, number]; seed?: string; read?: string }[] = [];
  let away = 0;
  const reach: Reaching = {
    body,
    onDoc: () => {},
    onBlock: (at, _, seed, read) => asked.push({ at, seed, read }),
    onAway: () => {
      away += 1;
    },
  };
  return { asked, reach, away: () => away };
};

const press = (on: Element | null, clientY = 900) =>
  on?.dispatchEvent(
    new MouseEvent("click", { bubbles: true, cancelable: true, button: 0, clientY }),
  );

describe("a sheet with nothing written yet", () => {
  it("shows where the title goes on an empty sheet, and nowhere else", () => {
    const empty = page("");
    worn(empty, false, () => true, writer().reach);
    expect(empty.querySelector(".page > h1[data-title]")).not.toBeNull();
    expect(empty.querySelector<HTMLElement>(".page")?.dataset.empty).toBeTruthy();

    const said = page('<p data-at="0-5">Leche</p>');
    worn(said, false, () => true, writer().reach);
    expect(said.querySelector("h1[data-title]")).toBeNull();

    const reading = page("");
    worn(reading, false, () => true, { body: "", onDoc: () => {} });
    expect(reading.querySelector("h1[data-title]")).toBeNull();
  });

  it("writes the title from its own slot, starting with its mark", () => {
    const empty = page("");
    const { asked, reach } = writer();
    worn(empty, false, () => true, reach);
    wired(empty, () => reach);
    press(empty.querySelector("h1[data-title]"), 0);
    expect(asked).toEqual([{ at: [0, 0], seed: "# " }]);
  });

  it("begins a new block past the last one, and leaves nothing behind when none is opened", () => {
    const drawn = page('<p data-at="0-5">Leche</p>');
    const { asked, reach } = writer("Leche");
    wired(drawn, () => reach);
    press(drawn.querySelector(".page"));
    expect(asked.map((one) => one.at)).toEqual([[5, 5]]);
    expect(drawn.querySelectorAll(".page > p")).toHaveLength(1);
  });

  it("ends the block being written on a press in an empty spot, never on one inside it", () => {
    const drawn = page('<p data-at="0-5">Leche</p><div class="editing"><span>x</span></div>');
    const { reach, away } = writer("Leche");
    wired(drawn, () => reach);
    press(drawn.querySelector(".editing span"), 0);
    expect(away()).toBe(0);
    press(drawn.body, 0);
    expect(away()).toBe(1);
  });

  it("hands the block the words read before the click, so the caret lands there", () => {
    const drawn = page('<p data-at="0-11">Leche y pan</p>');
    const text = drawn.querySelector("p")?.firstChild as Text;
    Object.assign(drawn, {
      caretRangeFromPoint: () => {
        const range = drawn.createRange();
        range.setStart(text, 7);
        return range;
      },
    });
    const { asked, reach } = writer("Leche y pan");
    wired(drawn, () => reach);
    press(drawn.querySelector("p"), 0);
    expect(asked[0]).toEqual({ at: [0, 11], seed: "", read: "Leche y" });
  });
});
