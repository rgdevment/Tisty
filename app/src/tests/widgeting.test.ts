import { NodeSelection } from "@tiptap/pm/state";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { heard, lend, openable, SHORTEST, TALLEST, tall } from "../ui/widgeting";
import { opened } from "./mounted";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args?: Record<string, unknown> }[],
  refuse: false,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args });
    if (cmd === "widget_lend") {
      return ipc.refuse
        ? Promise.reject({ code: "widgetTooBig", name: "512 KB" })
        : Promise.resolve("w1");
    }
    return Promise.resolve(null);
  },
  convertFileSrc: (path: string, scheme: string) => `http://${scheme}.localhost/${path}`,
}));

const acting = { isActive: false };
Object.defineProperty(navigator, "userActivation", { configurable: true, get: () => acting });

beforeEach(() => {
  ipc.calls = [];
  ipc.refuse = false;
  acting.isActive = false;
  document.documentElement.removeAttribute("data-theme");
});

const clickedOn = (frame: HTMLIFrameElement) => {
  acting.isActive = true;
  frame.focus();
};

const asked = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

const mounted = async (source = "<p>hola</p>") => {
  const drawn = document.createElement("div");
  document.body.append(drawn);
  const lending = lend(drawn, source);
  await vi.waitFor(() => expect(drawn.querySelector("iframe")).toBeTruthy());
  const frame = drawn.querySelector("iframe") as HTMLIFrameElement;
  const from = (data: unknown, source: MessageEventSource | null = frame.contentWindow) =>
    window.dispatchEvent(new MessageEvent("message", { data, source, origin: "null" }));
  return { drawn, frame, lending, from };
};

describe("what a widget may say to the window", () => {
  it("is a size or a request to open, and nothing else", () => {
    expect(heard({ type: "resize", height: 120 })).toEqual({ type: "resize", height: 120 });
    expect(heard({ type: "open", href: "https://example.com" })).toEqual({
      type: "open",
      href: "https://example.com",
    });
    for (const one of [
      null,
      "resize",
      { type: "resize", height: "120" },
      { type: "resize", height: Number.NaN },
      { type: "open", href: 3 },
      { type: "write", body: "x" },
      { type: "eval", code: "1" },
    ]) {
      expect(heard(one), JSON.stringify(one)).toBeNull();
    }
  });

  it("opens only a document of this store or a page of the web", () => {
    expect(openable("tisty:doc/abc-0001")).toBe("doc");
    expect(openable("https://example.com/a?b=1")).toBe("web");
    expect(openable("http://example.com")).toBe("web");
    for (const one of [
      "javascript:alert(1)",
      "file:///C:/Windows/win.ini",
      "data:text/html,x",
      "tisty:doc/",
      "ms-settings:",
      "https://exa mple.com",
      "",
    ]) {
      expect(openable(one), one).toBeNull();
    }
  });

  it("keeps its height between a sliver and a screen", () => {
    expect(tall(0)).toBe(SHORTEST);
    expect(tall(321.2)).toBe(322);
    expect(tall(1e9)).toBe(TALLEST);
  });
});

describe("the frame a widget is drawn in", () => {
  it("runs scripts and nothing else, from its own scheme", async () => {
    const { frame, lending } = await mounted();

    expect(frame.getAttribute("sandbox")).toBe("allow-scripts");
    expect(frame.src).toBe("http://widget.localhost/w1");
    expect(asked("widget_lend")[0].args).toEqual({ body: "<p>hola</p>" });
    lending.drop();
  });

  it("asks for the dark page in a dark window", async () => {
    document.documentElement.setAttribute("data-theme", "dark");
    const { frame, lending } = await mounted();

    expect(frame.src).toBe("http://widget.localhost/w1?dark=1");
    lending.drop();
  });

  it("tells the frame which theme it sits in, so a widget's own dark styles follow Tisty", async () => {
    const { frame, lending } = await mounted();
    expect(frame.style.colorScheme).toBe("light");

    document.documentElement.setAttribute("data-theme", "dark");
    lending.retheme();
    expect(frame.style.colorScheme).toBe("dark");
    expect(frame.src, "the page was reloaded rather than told").toBe(
      "http://widget.localhost/w1#dark",
    );

    document.documentElement.removeAttribute("data-theme");
    lending.retheme();
    expect(frame.src).toBe("http://widget.localhost/w1#light");
    lending.drop();
  });

  it("hears its own frame only from the sealed origin it was given", async () => {
    const { frame, lending } = await mounted();

    window.dispatchEvent(
      new MessageEvent("message", {
        data: { type: "resize", height: 300 },
        source: frame.contentWindow,
        origin: "http://widget.localhost",
      }),
    );

    expect(frame.style.height).toBe(`${SHORTEST}px`);
    lending.drop();
  });

  it("grows to what the widget measures, and only for its own frame", async () => {
    const { frame, lending, from } = await mounted();

    from({ type: "resize", height: 300 }, window);
    expect(frame.style.height).toBe(`${SHORTEST}px`);

    from({ type: "resize", height: 300 });
    expect(frame.style.height).toBe("300px");
    lending.drop();
  });

  it("opens a page of the web through the window, and refuses anything else", async () => {
    const { frame, lending, from } = await mounted();

    clickedOn(frame);
    from({ type: "open", href: "javascript:alert(1)" });
    from({ type: "open", href: "file:///C:/x" });
    from({ type: "write", href: "https://example.com" });
    expect(asked("open_link")).toHaveLength(0);

    from({ type: "open", href: "https://example.com" });
    expect(asked("open_link")[0].args).toEqual({ url: "https://example.com" });
    lending.drop();
  });

  it("hands a document link to whoever holds the page", async () => {
    const { drawn, frame, lending, from } = await mounted();
    const papers: unknown[] = [];
    drawn.addEventListener("widgetdoc", (e) => papers.push((e as CustomEvent).detail));

    clickedOn(frame);
    from({ type: "open", href: "tisty:doc/abc-0001" });

    expect(papers).toEqual(["abc-0001"]);
    lending.drop();
  });

  it("opens nothing unless the person just clicked inside that very frame", async () => {
    const { frame, lending, from } = await mounted();
    const other = document.createElement("button");
    document.body.append(other);

    from({ type: "open", href: "https://example.com" });
    expect(asked("open_link"), "a script opened a page on its own").toHaveLength(0);

    acting.isActive = true;
    other.focus();
    from({ type: "open", href: "https://example.com" });
    expect(
      asked("open_link"),
      "a click elsewhere in the window was taken for one in the widget",
    ).toHaveLength(0);

    frame.focus();
    from({ type: "open", href: "https://example.com" });
    expect(asked("open_link")).toHaveLength(1);
    other.remove();
    lending.drop();
  });

  it("opens one thing per click, however often the widget asks", async () => {
    const { frame, lending, from } = await mounted();

    clickedOn(frame);
    for (let n = 0; n < 5; n += 1) from({ type: "open", href: "https://example.com" });

    expect(asked("open_link")).toHaveLength(1);
    lending.drop();
  });

  it("stops climbing when each answer is the frame plus the same again", async () => {
    const { frame, lending, from } = await mounted();
    const told = () => Number.parseFloat(frame.style.height);

    for (let n = 0; n < 6; n += 1) from({ type: "resize", height: told() + 50 });
    expect(told(), "a page sized by its own frame grew to the ceiling").toBeLessThan(
      SHORTEST + 50 * 6,
    );

    from({ type: "resize", height: 60 });
    expect(told()).toBe(60);
    lending.drop();
  });

  it("gives its page back and stops listening once dropped", async () => {
    const { frame, lending, from } = await mounted();

    lending.drop();
    expect(asked("widget_take_back")[0].args).toEqual({ id: "w1" });

    from({ type: "resize", height: 500 });
    expect(frame.style.height).toBe(`${SHORTEST}px`);
  });

  it("says why when the widget is refused", async () => {
    ipc.refuse = true;
    const drawn = document.createElement("div");
    lend(drawn, "x");

    await vi.waitFor(() => expect(drawn.querySelector(".lit-refused")).toBeTruthy());
    expect(drawn.querySelector("iframe")).toBeNull();
  });
});

describe("a widget block in a document", () => {
  it("folds its code away once it has some, and opens it on asking", () => {
    const one = opened("```widget\n<p>hola</p>\n```");
    const body = one.dom.querySelector<HTMLElement>(".lit-body");
    const fold = one.dom.querySelector<HTMLButtonElement>(".lit-fold");

    expect(fold?.hidden).toBe(false);
    expect(body?.hidden).toBe(true);
    expect(one.dom.querySelector(".lit")?.classList.contains("lit-folding")).toBe(true);
    expect(fold?.getAttribute("aria-expanded")).toBe("false");

    fold?.click();
    expect(body?.hidden).toBe(false);
    expect(fold?.getAttribute("aria-expanded")).toBe("true");
    expect(one.markdown()).toBe("```widget\n<p>hola</p>\n```");
    one.shut();
  });

  it("opens its code when the widget is refused, since that is what it says it shows", async () => {
    ipc.refuse = true;
    const one = opened("```widget\n<p>hola</p>\n```");

    await vi.waitFor(() => expect(one.dom.querySelector(".lit-refused")).toBeTruthy());
    expect(one.dom.querySelector<HTMLElement>(".lit-body")?.hidden).toBe(false);
    one.shut();
  });

  it("takes the whole block on Backspace after it, rather than its hidden code", () => {
    const one = opened("```widget\n<p>hola</p>\n```\n\ntexto");
    let after = -1;
    one.editor.state.doc.descendants((node, at) => {
      if (node.type.name === "paragraph") after = at + 1;
    });
    one.at(after);

    one.pressed("Backspace");

    expect(one.editor.state.selection).toBeInstanceOf(NodeSelection);
    expect(one.markdown()).toBe("```widget\n<p>hola</p>\n```\n\ntexto");
    one.shut();
  });

  it("takes the whole block on Delete before it, rather than pulling its code up", () => {
    const one = opened("antes\n\n```widget\n<p>hola</p>\n```");
    one.at(6);

    one.pressed("Delete");

    expect(one.editor.state.selection).toBeInstanceOf(NodeSelection);
    expect(one.markdown()).toBe("antes\n\n```widget\n<p>hola</p>\n```");
    one.shut();
  });

  it("opens its code when the caret finds its way inside", () => {
    const one = opened("```widget\n<p>hola</p>\n```");
    const body = one.dom.querySelector<HTMLElement>(".lit-body");
    expect(body?.hidden).toBe(true);

    one.at(3);

    expect(body?.hidden).toBe(false);
    one.shut();
  });

  it("keeps an empty one open, or there would be nowhere to write it", () => {
    const one = opened("```widget\n```");

    expect(one.dom.querySelector<HTMLElement>(".lit-body")?.hidden).toBe(false);
    one.shut();
  });

  it("offers no fold to a block that is not a widget", () => {
    const one = opened("```mermaid\ngraph TD\n```");

    expect(one.dom.querySelector<HTMLButtonElement>(".lit-fold")?.hidden).toBe(true);
    expect(one.dom.querySelector<HTMLElement>(".lit-body")?.hidden).toBe(false);
    one.shut();
  });
});
