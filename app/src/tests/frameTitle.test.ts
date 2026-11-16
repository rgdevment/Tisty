import { renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useFrameScroll } from "../ui/liveSheet";

const framed = (page: string) => {
  const frame = document.createElement("iframe");
  document.body.append(frame);
  const inner = frame.contentWindow as Window;
  inner.document.body.innerHTML = `<div class="page">${page}</div>`;
  Object.defineProperty(inner.document, "scrollingElement", {
    value: inner.document.documentElement,
  });
  return inner;
};

const watched = (inner: Window) => {
  const onTitleGone = vi.fn();
  renderHook(() =>
    useFrameScroll({
      lively: true,
      frame: () => inner,
      putting: { current: 0 },
      onStuck: vi.fn(),
      onTitleGone,
      onSeen: vi.fn(),
      ready: true,
      turned: 0,
    }),
  );
  return onTitleGone;
};

const rolledTo = (inner: Window, top: number) => {
  (inner.document.scrollingElement as Element).scrollTop = top;
  inner.dispatchEvent(new Event("scroll"));
};

afterEach(() => {
  document.body.innerHTML = "";
});

describe("the ribbon naming the page", () => {
  it("waits while the page's own title is still in view", () => {
    const inner = framed('<dl class="front"></dl><h1>Recetas</h1><p>Pan</p>');
    const onTitleGone = watched(inner);
    const title = inner.document.querySelector("h1") as HTMLElement;
    vi.spyOn(title, "getBoundingClientRect").mockReturnValue({ bottom: 40 } as DOMRect);

    rolledTo(inner, 30);

    expect(onTitleGone).toHaveBeenLastCalledWith(false);
  });

  it("names it once the page's title has scrolled away", () => {
    const inner = framed('<dl class="front"></dl><h1>Recetas</h1><p>Pan</p>');
    const onTitleGone = watched(inner);
    const title = inner.document.querySelector("h1") as HTMLElement;
    vi.spyOn(title, "getBoundingClientRect").mockReturnValue({ bottom: -12 } as DOMRect);

    rolledTo(inner, 400);

    expect(onTitleGone).toHaveBeenLastCalledWith(true);
  });

  it("names it as soon as the page moves when it opens without a title", () => {
    const inner = framed("<p>Pan</p><h1>Más tarde</h1>");
    const onTitleGone = watched(inner);

    rolledTo(inner, 4);
    expect(onTitleGone).toHaveBeenLastCalledWith(false);

    rolledTo(inner, 20);
    expect(onTitleGone).toHaveBeenLastCalledWith(true);
  });
});
