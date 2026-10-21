import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  answered,
  datedSteps,
  fitted,
  iconed,
  pictures,
  type Reaching,
  widgets,
  wired,
  worn,
} from "../ui/dressing";
import { charsAt, colored, dressed, escaped, saidKinds, shell, went } from "../ui/paging";

const calls = vi.hoisted(() => ({
  links: [] as string[],
  served: [] as string[],
  shown: [] as string[],
  released: [] as string[],
  lent: [] as string[],
  dropped: 0,
  here: true,
  refused: false,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: { url?: string }) => {
    if (cmd === "open_link") calls.links.push(args?.url ?? "");
    if (cmd === "kept_here") return Promise.resolve(calls.here);
    return Promise.resolve(null);
  },
  convertFileSrc: (path: string) => `http://asset.localhost/${path}`,
}));

vi.mock("../inUse", () => ({
  serve: (at: string) => {
    calls.served.push(at);
    if (calls.refused) return Promise.reject({ code: "cloudLater", name: at });
    return Promise.resolve(`C:/kept/${at}`);
  },
  askAgain: () => {},
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
  lendKept: (_: HTMLElement, reference: string) => {
    calls.lent.push(reference);
    return { retheme: () => {}, shade: () => {}, drop: () => {} };
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
  calls.here = true;
  calls.refused = false;
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
    const { held } = pictures(drawn, () => true);
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

  it("shows a file alone in its paragraph as a card that opens it, and one in a sentence as a chip", () => {
    const drawn = page(
      '<p><img src="attachments/plan.pdf" alt=""></p><p>see <img src="attachments/acta.pdf" alt="Acta"> here</p>',
    );
    pictures(drawn, () => true);
    const card = drawn.querySelector("a.doc-card.is-file");
    expect(card?.getAttribute("href")).toBe("attachments/plan.pdf");
    expect(card?.querySelector(".card-name")?.textContent).toBe("plan.pdf");
    expect(card?.getAttribute("data-family")).toBe("pdf");
    const chip = drawn.querySelector("a.card");
    expect(chip?.getAttribute("href")).toBe("attachments/acta.pdf");
    expect(chip?.textContent).toBe("Acta");
    expect(calls.served).toEqual([]);
  });

  it("plays audio and video where they are written", async () => {
    const drawn = page(
      '<p><img src="attachments/walk-0006bf09.mp4" alt="A walk"></p><p><img src="attachments/bark-0006bf10.mp3" alt="Bark"></p>',
    );
    const { held } = pictures(drawn, () => true);
    const video = drawn.querySelector<HTMLVideoElement>(".media.is-video video");
    const audio = drawn.querySelector<HTMLAudioElement>(".media.is-audio audio");
    expect(video?.controls).toBe(true);
    expect(audio?.getAttribute("aria-label")).toBe("Bark");
    expect(drawn.querySelector(".media.is-audio .media-name")?.textContent).toBe("Bark");
    await vi.waitFor(() => expect(video?.getAttribute("src")).toContain("walk-0006bf09.mp4"));
    expect(held).toHaveLength(2);
  });

  it("brings a film still in the cloud only when asked, saying so while it comes", async () => {
    calls.here = false;
    const drawn = page('<p><img src="attachments/walk-0006bf09.mp4" alt="A walk"></p>');
    const { held } = pictures(drawn, () => true);
    const box = drawn.querySelector<HTMLElement>(".media.is-video");
    await vi.waitFor(() => expect(box?.dataset.state).toBe("away"));
    expect(calls.served).toEqual([]);
    drawn.querySelector<HTMLButtonElement>(".media-bring")?.click();
    expect(box?.dataset.state).toBe("bringing");
    await vi.waitFor(() =>
      expect(box?.querySelector("video")?.getAttribute("src")).toContain("walk-0006bf09.mp4"),
    );
    expect(box?.dataset.state).toBeUndefined();
    expect(held).toHaveLength(0);
  });

  it("says why a film or a picture did not come, and tries again when asked", async () => {
    calls.refused = true;
    const drawn = page(
      '<p><img src="attachments/walk-0006bf09.mp4" alt="A walk"></p><p><img src="attachments/a.png" alt="A"></p>',
    );
    pictures(drawn, () => true);
    const box = drawn.querySelector<HTMLElement>(".media.is-video");
    await vi.waitFor(() => expect(box?.dataset.state).toBe("failed"));
    await vi.waitFor(() => expect(drawn.querySelector(".picture-away")).not.toBeNull());
    expect(drawn.querySelector<HTMLImageElement>('img[alt="A"]')?.hidden).toBe(true);
    expect(box?.querySelector(".media-state")?.textContent).toContain("walk-0006bf09.mp4");
    calls.refused = false;
    drawn.querySelector<HTMLButtonElement>(".picture-away .media-again")?.click();
    await vi.waitFor(() =>
      expect(drawn.querySelector('img[alt="A"]')?.getAttribute("src")).toContain("a.png"),
    );
    expect(drawn.querySelector(".picture-away")).toBeNull();
    expect(drawn.querySelector<HTMLImageElement>('img[alt="A"]')?.hidden).toBe(false);
    box?.querySelector<HTMLButtonElement>(".media-again")?.click();
    await vi.waitFor(() => expect(box?.dataset.state).toBeUndefined());
  });

  it("draws an attached page live, unless it asks to be a file or comes from outside", () => {
    const drawn = page(
      '<p><img src="walks-0006bf09.html" alt="Walks"></p><p><img src="walks-0006bf09.html" alt="Walks" data-as="file"></p>',
    );
    const { lent } = pictures(drawn, () => true, { stranger: false, apart: true, more: true });
    expect(lent).toHaveLength(1);
    expect(drawn.querySelector("figure.widget .widget-name")?.textContent).toBe("Walks");
    expect(drawn.querySelector('figure.widget button[data-asked="apart"]')).not.toBeNull();
    expect(drawn.querySelector("a.doc-card.is-file")?.getAttribute("href")).toBe(
      "walks-0006bf09.html",
    );

    const outside = page('<p><img src="walks-0006bf09.html" alt="Walks"></p>');
    expect(pictures(outside, () => true).lent).toHaveLength(0);
    expect(outside.querySelector("figure.widget")).toBeNull();
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
      onApart: (lent: { source: string } | { reference: string }) =>
        apart.push("source" in lent ? lent.source : lent.reference),
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

  it("keeps the block being written when a selection is dragged out of it and let go", () => {
    const drawn = page('<p data-at="0-5">Leche</p><div class="editing"><span>x</span></div>');
    const { asked, reach, away } = writer("Leche");
    wired(drawn, () => reach);
    drawn
      .querySelector(".editing span")
      ?.dispatchEvent(new MouseEvent("mousedown", { bubbles: true, button: 0 }));
    press(drawn.querySelector("p"), 0);
    expect(away()).toBe(0);
    expect(asked).toEqual([]);
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

describe("a task's template hints", () => {
  const body =
    "## Steps\n<!-- One box per step. -->\n\n## Notes\n<!-- Anything. -->\n\nCall at nine.\n";
  const span = (said: string) => {
    const from = body.indexOf(said);
    return `${from}-${from + said.length}`;
  };
  const drawn = () => {
    const one = page(
      `<h2 data-at="${span("## Steps")}">Steps</h2>` +
        `<div data-at="${span("<!-- One box per step. -->")}"></div>` +
        `<h2 data-at="${span("## Notes")}">Notes</h2>` +
        `<div data-at="${span("<!-- Anything. -->")}"></div>` +
        `<p data-at="${span("Call at nine.")}">Call at nine.</p>`,
    );
    one.documentElement.classList.add("untitled");
    return one;
  };

  it("shows a hint while its section is empty, and hides one whose section has words", () => {
    const one = drawn();
    worn(one, false, () => true, writer(body).reach);

    const [first, second] = Array.from(one.querySelectorAll<HTMLElement>(".page > div[data-at]"));
    expect(first.classList.contains("hint")).toBe(true);
    expect(first.dataset.hint).toBe("One box per step.");
    expect(second.classList.contains("hint-gone")).toBe(true);
  });

  it("leaves a document's comments alone", () => {
    const one = drawn();
    one.documentElement.classList.remove("untitled");
    worn(one, false, () => true, writer(body).reach);

    expect(one.querySelector(".hint, .hint-gone")).toBeNull();
  });

  it("begins writing right after the hint, keeping the comment", () => {
    const one = drawn();
    const { asked, reach } = writer(body);
    worn(one, false, () => true, reach);
    wired(one, () => reach);
    press(one.querySelector(".hint"), 0);

    const end = body.indexOf("<!-- One box per step. -->") + "<!-- One box per step. -->".length;
    expect(asked).toEqual([{ at: [end, end], seed: "" }]);
  });
});

describe("a task's steps on its page", () => {
  it("shows a box hint as a ghost step that begins a box when pressed", () => {
    const body = "## Steps\n<!-- - [ ] The first step -->\n";
    const one = page(`<h2 data-at="0-8">Steps</h2><div data-at="9-${body.length - 1}"></div>`);
    one.documentElement.classList.add("untitled");
    const { asked, reach } = writer(body);
    worn(one, false, () => true, reach);
    wired(one, () => reach);

    const hint = one.querySelector<HTMLElement>(".hint");
    expect(hint?.classList.contains("step-hint")).toBe(true);
    expect(hint?.dataset.hint).toBe("The first step");
    press(hint, 0);
    expect(asked).toEqual([{ at: [body.length - 1, body.length - 1], seed: "- [ ] " }]);
  });

  it("fills a bar as far as the boxes are ticked, and offers to add one after the list", () => {
    const body = "- [x] one\n- [ ] two\n";
    const one = page(
      '<ul data-at="0-20"><li class="task" data-at="0-10"><span class="check" data-done="true"></span>one</li>' +
        '<li class="task" data-at="10-20"><span class="check" data-done="false"></span>two</li></ul>',
    );
    one.documentElement.classList.add("untitled");
    const { asked, reach } = writer(body);
    worn(one, false, () => true, reach);
    wired(one, () => reach);

    expect(one.querySelector<HTMLElement>("ul")?.style.getPropertyValue("--done")).toBe("50%");
    const add = one.querySelector<HTMLElement>(".page > .add-step");
    expect(add).not.toBeNull();
    press(add, 0);
    expect(asked).toEqual([{ at: [20, 20], seed: "- [ ] " }]);
  });

  it("adds a step with the list's own mark, so it joins that list", () => {
    const body = "* [ ] one\n";
    const one = page(
      '<ul data-at="0-10"><li class="task" data-at="0-10"><span class="check" data-done="false"></span>one</li></ul>',
    );
    one.documentElement.classList.add("untitled");
    const { asked, reach } = writer(body);
    worn(one, false, () => true, reach);
    wired(one, () => reach);

    press(one.querySelector<HTMLElement>(".page > .add-step"), 0);
    expect(asked).toEqual([{ at: [10, 10], seed: "* [ ] " }]);
  });

  it("adds nothing of the kind to a document", () => {
    const one = page(
      '<ul data-at="0-10"><li class="task" data-at="0-10"><span class="check" data-done="false"></span>one</li></ul>',
    );
    worn(one, false, () => true, writer("- [ ] one\n").reach);

    expect(one.querySelector(".add-step")).toBeNull();
  });
});

describe("a task page only read", () => {
  it("offers no hints and no way to add a step", () => {
    const body = "## Steps\n<!-- - [ ] The first step -->\n";
    const one = page(`<h2 data-at="0-8">Steps</h2><div data-at="9-${body.length - 1}"></div>`);
    one.documentElement.classList.add("untitled");
    worn(one, false, () => true, { body, onDoc: () => {} });

    expect(one.querySelector(".hint, .add-step")).toBeNull();
  });
});

describe("a task's section labels", () => {
  it("start writing past their section while the task is filled in, and open the block while its structure is edited", () => {
    const one = page(
      '<h2 data-at="0-8">Steps <a href="https://x.y">x</a></h2><hr data-at="10-13"><h2 data-at="15-22">Notes</h2>',
    );
    one.documentElement.classList.add("untitled");
    const label = one.querySelector("h2") as Element;

    expect(went(label, false)).toEqual({ kind: "section", at: [10, 13] });
    expect(went(one.querySelectorAll("h2")[1] as Element, false)).toEqual({
      kind: "section",
      at: [15, 22],
    });
    expect(went(one.querySelector("a") as Element, false)).toEqual({
      kind: "web",
      href: "https://x.y",
    });

    one.documentElement.classList.add("shaping");
    expect(went(label, false)).toEqual({ kind: "block", at: [0, 8] });
  });

  it("let an empty label be mended, so a stray mark never locks the section", () => {
    const one = page('<h1 data-at="0-2"></h1>');
    one.documentElement.classList.add("untitled");

    expect(went(one.querySelector("h1") as Element, false)).toEqual({ kind: "block", at: [0, 2] });
  });
});

describe("a step's dates", () => {
  it("show under the pointer on its box, by its words, and leave the others bare", () => {
    const one = page(
      '<ul data-at="0-20"><li class="task" data-at="0-10"><span class="check"></span>Call the vet</li>' +
        '<li class="task" data-at="10-20"><span class="check"></span>Other</li></ul>',
    );

    datedSteps(one, { "Call the vet": "Added 18 Oct" });

    const [first, second] = one.querySelectorAll<HTMLElement>("li.task");
    expect(first.title).toBe("Added 18 Oct");
    expect(second.hasAttribute("title")).toBe(false);
  });
});
