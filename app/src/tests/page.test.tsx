import { render, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Page from "../ui/Page";
import type { Head } from "../ui/writing";

const ipc = vi.hoisted(() => ({
  drawn: "" as string | null,
  dropped: 0,
  released: 0,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
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
  lend: () => ({
    retheme: () => {},
    shade: () => {},
    drop: () => {
      ipc.dropped += 1;
    },
  }),
  locked: () => document.createElement("div"),
}));

Element.prototype.scrollIntoView ??= () => {};

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

const drawn = (more: Partial<Parameters<typeof Page>[0]> = {}) => {
  const heard = {
    docs: [] as string[],
    blocks: [] as [number, number][],
    heads: [] as Head[],
    shown: [] as (Window | null)[],
  };
  const view = render(
    <Page
      body={BODY}
      label="Notas"
      stranger={false}
      wide={900}
      zoom={1}
      onDoc={(file) => heard.docs.push(file)}
      onBlock={(at) => heard.blocks.push(at)}
      onOutline={(found) => {
        heard.heads = found;
      }}
      onShown={(at) => heard.shown.push(at)}
      {...more}
    />,
  );
  const frame = view.container.querySelector("iframe") as HTMLIFrameElement;
  return { heard, view, frame };
};

beforeEach(() => {
  ipc.drawn = DRAWN;
  ipc.dropped = 0;
  ipc.released = 0;
});

describe("a drawn page in the viewer", () => {
  it("shows the page, its outline, and answers a click on a card or a block", async () => {
    const { heard, frame } = drawn();
    const page = await loaded(frame);
    expect(frame.title).toBe("Notas");
    expect(page.querySelector(".page")?.hasAttribute("inert")).toBe(false);
    expect(heard.heads.map((one) => one.text)).toEqual(["Título"]);
    expect(heard.shown[heard.shown.length - 1]).toBe(frame.contentWindow);
    page.querySelector("a.card")?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    expect(heard.docs).toEqual(["ab12-0001"]);
    page
      .querySelector('p[data-at="11-19"]')
      ?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    expect(heard.blocks).toEqual([[10, 17]]);
    expect(page.documentElement.style.getPropertyValue("--wide")).toBe("900px");
  });

  it("follows a new sheet size or zoom without drawing again", async () => {
    const { frame, view } = drawn();
    const page = await loaded(frame);
    view.rerender(
      <Page body={BODY} label="Notas" stranger={false} wide={1200} zoom={1.5} onDoc={() => {}} />,
    );
    expect(page.documentElement.style.getPropertyValue("--wide")).toBe("1200px");
    expect(page.documentElement.style.getPropertyValue("zoom")).toBe("1.5");
  });

  it("follows the window when its theme changes", async () => {
    const { frame } = drawn();
    const page = await loaded(frame);
    document.documentElement.dataset.theme = "dark";
    await waitFor(() => expect(page.documentElement.classList.contains("dark")).toBe(true));
    document.documentElement.dataset.theme = "light";
    await waitFor(() => expect(page.documentElement.classList.contains("dark")).toBe(false));
  });

  it("hands its widgets and pictures back when it goes", async () => {
    const { frame, view, heard } = drawn();
    await loaded(frame);
    view.unmount();
    expect(ipc.dropped).toBe(1);
    expect(ipc.released).toBe(1);
    expect(heard.shown[heard.shown.length - 1]).toBeNull();
  });

  it("shows the words as written when the page cannot be drawn", async () => {
    ipc.drawn = null;
    const { frame } = drawn({ body: "<b>raw</b>" });
    await waitFor(() => expect(frame.srcdoc).toContain("<pre>&lt;b&gt;raw&lt;/b&gt;</pre>"));
  });

  it("draws a new frame when the words change", async () => {
    const { frame, view } = drawn();
    await loaded(frame);
    ipc.drawn = '<p data-at="0-5">Nuevo</p>';
    view.rerender(<Page body="Nuevo" label="Notas" stranger={false} onDoc={() => {}} />);
    await waitFor(() => {
      const now = view.container.querySelector("iframe") as HTMLIFrameElement;
      expect(now).not.toBe(frame);
      expect(now.srcdoc).toContain("Nuevo");
    });
  });
});
