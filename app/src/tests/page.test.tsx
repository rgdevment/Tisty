import { render, waitFor } from "@testing-library/react";
import type { Head } from "tisty-editor/read";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Page from "../ui/Page";

const ipc = vi.hoisted(() => ({
  drawn: "" as string | null,
  dropped: 0,
  released: 0,
  cards: [] as unknown[],
  version: 1,
  held: false,
  waiting: [] as (() => void)[],
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: { asked?: { id: string; words: string }[] }) => {
    if (cmd === "cards_drawn") {
      ipc.cards.push(args);
      const drawn = (one: { id: string; words: string }) =>
        `<span class="doc-card is-doc" role="link" tabindex="0" data-doc="${one.id}">${one.words} v${ipc.version}</span>`;
      const answer = args?.asked?.map(drawn);
      if (ipc.held) return new Promise((done) => ipc.waiting.push(() => done(answer)));
      return Promise.resolve(answer);
    }
    if (cmd !== "doc_drawn") return Promise.resolve(null);
    if (ipc.drawn === null) return Promise.reject(new Error("no"));
    return Promise.resolve(new TextEncoder().encode(ipc.drawn).buffer);
  },
  convertFileSrc: (path: string) => `http://asset.localhost/${path}`,
}));

vi.mock("../inUse", () => ({
  serve: () => new Promise(() => {}),
  showing: () => () => {
    ipc.released += 1;
  },
}));

vi.mock("../ui/widgeting", () => ({
  notWidget: () => null,
  lend: () => ({
    retheme: () => {},
    shade: () => {},
    drop: () => {
      ipc.dropped += 1;
    },
  }),
}));

const BODY = "# Título\n\nÁrbol.\n";
const DRAWN =
  '<h1 id="titulo" data-at="0-10">Título</h1><p data-at="11-19">Árbol. <a class="card" data-doc="ab12-0001">Otro</a></p><figure class="code" data-at="20-40"><pre><code class="language-widget">w</code></pre></figure><p data-at="41-60"><img src="attachments/a.png" alt=""></p>';

const loaded = async (frame: HTMLIFrameElement) => {
  await waitFor(() => expect(frame.srcdoc).toContain("<article"));
  const page = frame.contentDocument as Document;
  page.open();
  page.write(frame.srcdoc);
  page.close();
  frame.dispatchEvent(new Event("load"));
  return page;
};

const drawn = async (more: Partial<Parameters<typeof Page>[0]> = {}) => {
  const heard = {
    docs: [] as string[],
    heads: [] as Head[],
    shown: [] as (Window | null)[],
  };
  const view = render(
    <Page
      body={BODY}
      label="Notas"
      wide={900}
      zoom={1}
      onDoc={(file) => heard.docs.push(file)}
      onOutline={(found) => {
        heard.heads = found;
      }}
      onShown={(at) => heard.shown.push(at)}
      {...more}
    />,
  );
  const frame = await waitFor(() => {
    const found = view.container.querySelector("iframe");
    expect(found).toBeTruthy();
    return found as HTMLIFrameElement;
  });
  return { heard, view, frame };
};

const TICKED =
  '<ul data-at="0-11"><li class="task" data-at="0-11"><span class="check" data-done="false"></span>Nuevo</li></ul>';

beforeEach(() => {
  ipc.drawn = DRAWN;
  ipc.dropped = 0;
  ipc.released = 0;
  ipc.cards = [];
  ipc.version = 1;
  ipc.held = false;
  ipc.waiting = [];
});

describe("a drawn page in the viewer", () => {
  it("shows the page, its outline, and answers a click on a card", async () => {
    const { heard, frame } = await drawn();
    const page = await loaded(frame);
    expect(frame.title).toBe("Notas");
    expect(page.querySelector(".page")?.hasAttribute("inert")).toBe(false);
    expect(heard.heads.map((one) => one.text)).toEqual(["Título"]);
    expect(heard.shown[heard.shown.length - 1]).toBe(frame.contentWindow);
    page.querySelector("a.card")?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    expect(heard.docs).toEqual(["ab12-0001"]);
    expect(page.documentElement.style.getPropertyValue("--wide")).toBe("900px");
  });

  it("follows a new sheet size or zoom without drawing again", async () => {
    const { frame, view } = await drawn();
    const page = await loaded(frame);
    view.rerender(<Page body={BODY} label="Notas" wide={1200} zoom={1.5} onDoc={() => {}} />);
    expect(page.documentElement.style.getPropertyValue("--wide")).toBe("1200px");
    expect(page.documentElement.style.getPropertyValue("zoom")).toBe("1.5");
  });

  it("follows the window when its theme changes", async () => {
    const { frame } = await drawn();
    const page = await loaded(frame);
    document.documentElement.dataset.theme = "dark";
    await waitFor(() => expect(page.documentElement.classList.contains("dark")).toBe(true));
    document.documentElement.dataset.theme = "light";
    await waitFor(() => expect(page.documentElement.classList.contains("dark")).toBe(false));
  });

  it("hands its widgets and pictures back when it goes", async () => {
    const { frame, view, heard } = await drawn();
    await loaded(frame);
    view.unmount();
    expect(ipc.dropped).toBe(1);
    expect(ipc.released).toBe(1);
    expect(heard.shown[heard.shown.length - 1]).toBeNull();
  });

  it("shows the words as written when the page cannot be drawn", async () => {
    ipc.drawn = null;
    const { frame } = await drawn({ body: "<b>raw</b>" });
    await waitFor(() => expect(frame.srcdoc).toContain("<pre>&lt;b&gt;raw&lt;/b&gt;</pre>"));
  });

  it("changes the words in the same page once they are drawn, keeping the old ones till then", async () => {
    const { frame, view } = await drawn();
    const page = await loaded(frame);
    const before = frame.srcdoc;
    ipc.drawn = '<p data-at="0-5">Nuevo</p>';
    view.rerender(<Page body="Nuevo" label="Notas" onDoc={() => {}} />);
    expect(page.querySelector(".page")?.textContent).toContain("Árbol");
    await waitFor(() => expect(page.querySelector(".page")?.textContent).toContain("Nuevo"));
    expect(Array.from(view.container.querySelectorAll("iframe"))).toEqual([frame]);
    expect(frame.srcdoc).toBe(before);
    expect(frame.classList.contains("invisible")).toBe(false);
  });

  it("ticks nothing on words a newer drawing has replaced, and ticks the new ones by their own bytes", async () => {
    const ticks: [number, number][] = [];
    const { frame, view } = await drawn({ onTick: (at) => ticks.push(at) });
    const page = await loaded(frame);
    const old = page.querySelector('p[data-at="11-19"]');
    ipc.drawn = TICKED;
    view.rerender(
      <Page body="- [ ] Nuevo" label="Notas" onDoc={() => {}} onTick={(at) => ticks.push(at)} />,
    );
    await waitFor(() => expect(page.querySelector(".check")).not.toBeNull());
    old?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    expect(ticks).toEqual([]);
    page.querySelector(".check")?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    expect(ticks).toEqual([[0, 11]]);
  });

  it("lets go of the widgets and pictures of the words it replaces", async () => {
    const { frame, view } = await drawn();
    await loaded(frame);
    const [dropped, released] = [ipc.dropped, ipc.released];
    ipc.drawn = '<p data-at="0-5">Nuevo</p>';
    view.rerender(<Page body="Nuevo" label="Notas" onDoc={() => {}} />);
    await waitFor(() => expect(ipc.dropped).toBe(dropped + 1));
    expect(ipc.released).toBe(released + 1);
  });

  it("dresses a drawing again and says it is shown, even when its words come back the same", async () => {
    const { frame, view, heard } = await drawn({ again: "a" });
    await loaded(frame);
    const [shown, dropped] = [heard.shown.length, ipc.dropped];
    view.rerender(
      <Page
        body={BODY}
        label="Notas"
        onDoc={() => {}}
        onShown={(at) => heard.shown.push(at)}
        again="b"
      />,
    );
    await waitFor(() => expect(heard.shown.length).toBe(shown + 1));
    expect(ipc.dropped).toBe(dropped + 1);
  });

  it("leaves nothing the last words set on the sheet itself", async () => {
    const { frame, view } = await drawn();
    const page = await loaded(frame);
    const sheet = page.querySelector(".page") as HTMLElement;
    sheet.dataset.empty = "Vacía";
    sheet.classList.add("whole");
    ipc.drawn = '<p data-at="0-5">Nuevo</p>';
    view.rerender(<Page body="Nuevo" label="Notas" onDoc={() => {}} />);
    await waitFor(() => expect(sheet.textContent).toContain("Nuevo"));
    expect(sheet.hasAttribute("data-empty")).toBe(false);
    expect(sheet.className).toBe("page");
  });

  it("puts the latest words back when the frame loads its first ones again", async () => {
    const ticks: [number, number][] = [];
    const { frame, view } = await drawn({ onTick: (at) => ticks.push(at) });
    await loaded(frame);
    ipc.drawn = TICKED;
    view.rerender(
      <Page body="- [ ] Nuevo" label="Notas" onDoc={() => {}} onTick={(at) => ticks.push(at)} />,
    );
    await waitFor(() =>
      expect(frame.contentDocument?.querySelector(".page")?.textContent).toContain("Nuevo"),
    );
    const again = await loaded(frame);
    expect(again.querySelector(".page")?.textContent).toBe("Nuevo");
    again.querySelector(".check")?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    expect(ticks).toContainEqual([0, 11]);
  });

  it("takes the window's language when its words change, though the document was built before", async () => {
    const { frame, view } = await drawn();
    const page = await loaded(frame);
    const was = document.documentElement.lang;
    document.documentElement.lang = "en";
    ipc.drawn = '<p data-at="0-5">Nuevo</p>';
    view.rerender(<Page body="Nuevo" label="Notas" onDoc={() => {}} />);
    await waitFor(() => expect(page.documentElement.lang).toBe("en"));
    document.documentElement.lang = was;
  });
});

describe("the live cards on a page", () => {
  const CARDED = `${DRAWN}<p data-at="61-80"><a class="card" data-doc="cd34-0002">Huerta</a></p>`;
  const SHOWN = { docs: true, here: "ab12-0001", heard: 1 };

  it("lays the cards the core drew before the page is shown, and leaves a word in a sentence alone", async () => {
    ipc.drawn = CARDED;
    const { frame } = await drawn({ cardsShown: SHOWN });
    const page = await loaded(frame);
    expect(page.querySelector(".page > p > .doc-card")?.textContent).toBe("Huerta v1");
    expect(page.querySelector('p[data-at="11-19"] a.card')?.textContent).toBe("Otro");
    expect(ipc.cards).toEqual([
      {
        asked: [
          { kind: "doc", id: "ab12-0001", words: "Otro" },
          { kind: "doc", id: "cd34-0002", words: "Huerta" },
        ],
        seen: { here: "ab12-0001", acting: false, more: false, examples: false },
      },
    ]);
  });

  it("draws its words again without asking again for the cards it already holds", async () => {
    ipc.drawn = CARDED;
    const { frame, view, heard } = await drawn({ cardsShown: SHOWN });
    await loaded(frame);
    const shownBefore = heard.shown.length;
    view.rerender(
      <Page
        body={`${BODY}más
`}
        label="Notas"
        onDoc={() => {}}
        onShown={(at) => heard.shown.push(at)}
        cardsShown={SHOWN}
      />,
    );
    const page = await loaded(frame);
    await waitFor(() => expect(heard.shown.length).toBeGreaterThan(shownBefore));
    expect(page.querySelector(".page > p > .doc-card")?.textContent).toBe("Huerta v1");
    expect(ipc.cards).toHaveLength(1);
  });

  it("draws every card anew for another document, never laying one drawn for the last", async () => {
    ipc.drawn = CARDED;
    const { frame, view } = await drawn({ cardsShown: SHOWN });
    await loaded(frame);
    ipc.version = 2;
    view.rerender(
      <Page
        body={`${BODY}otro
`}
        label="Notas"
        onDoc={() => {}}
        cardsShown={{ ...SHOWN, here: "ef56-0003" }}
      />,
    );
    const page = await loaded(frame);
    await waitFor(() =>
      expect(page.querySelector(".page > p > .doc-card")?.textContent).toBe("Huerta v2"),
    );
    expect(ipc.cards[ipc.cards.length - 1]).toMatchObject({ seen: { here: "ef56-0003" } });
  });

  it("drops a late answer asked for the last document, whatever order the answers come in", async () => {
    ipc.drawn = CARDED;
    const { frame, view } = await drawn({ cardsShown: SHOWN });
    const page = await loaded(frame);
    ipc.held = true;
    ipc.version = 2;
    view.rerender(
      <Page body={BODY} label="Notas" onDoc={() => {}} cardsShown={{ ...SHOWN, heard: 2 }} />,
    );
    ipc.version = 3;
    const other = { ...SHOWN, heard: 2, here: "ef56-0003" };
    view.rerender(<Page body={BODY} label="Notas" onDoc={() => {}} cardsShown={other} />);
    await waitFor(() => expect(ipc.waiting).toHaveLength(2));
    const [late, fresh] = ipc.waiting;
    fresh();
    await waitFor(() =>
      expect(page.querySelector(".page > p > .doc-card")?.textContent).toBe("Huerta v3"),
    );
    late();
    await new Promise((done) => setTimeout(done, 20));
    expect(page.querySelector(".page > p > .doc-card")?.textContent).toBe("Huerta v3");
  });

  it("asks nothing of the core where the window shows no cards", async () => {
    ipc.drawn = CARDED;
    const { frame } = await drawn();
    const page = await loaded(frame);
    expect(page.querySelector(".doc-card")).toBeNull();
    expect(ipc.cards).toEqual([]);
  });

  it("asks again when the window hears of a change, and lays only the card whose drawing changed", async () => {
    ipc.drawn = CARDED;
    const { frame, view } = await drawn({ cardsShown: SHOWN });
    const page = await loaded(frame);
    const before = page.querySelector(".page > p > .doc-card");
    ipc.version = 2;
    view.rerender(
      <Page body={BODY} label="Notas" onDoc={() => {}} cardsShown={{ ...SHOWN, heard: 2 }} />,
    );
    await waitFor(() =>
      expect(page.querySelector(".page > p > .doc-card")?.textContent).toBe("Huerta v2"),
    );
    expect(page.querySelector(".page > p > .doc-card")).not.toBe(before);
    expect(ipc.cards).toHaveLength(2);
    view.rerender(
      <Page body={BODY} label="Notas" onDoc={() => {}} cardsShown={{ ...SHOWN, heard: 2 }} />,
    );
    expect(ipc.cards).toHaveLength(2);
  });
});
