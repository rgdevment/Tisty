import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Ask } from "../ui/Rows";

const SAID = "Una línea pequeña y gris al final del PDF.";

const markAt = (left: number, top: number) => {
  const mark = screen.getByRole("button", { name: SAID });
  mark.getBoundingClientRect = () =>
    ({ left, right: left + 15, top, bottom: top + 15, width: 15, height: 15 }) as DOMRect;
  return mark;
};

const bubble = () => document.body.querySelector<HTMLElement>(":scope > span[aria-hidden]");

describe("the ? beside a choice", () => {
  it("opens its words outside the dialog that scrolls, so the dialog never grows a scrollbar for them", () => {
    render(
      <div role="dialog" className="scroller">
        <Ask said={SAID} />
      </div>,
    );
    fireEvent.mouseEnter(markAt(40, 100));
    expect(bubble()?.textContent).toBe(SAID);
    expect(screen.getByRole("dialog").contains(bubble())).toBe(false);
    expect(bubble()?.style.left).toBe("40px");
    expect(bubble()?.style.top).toBe("123px");

    fireEvent.mouseLeave(screen.getByRole("button", { name: SAID }));
    expect(bubble()).toBeNull();
  });

  it("stays inside the window near its right edge, leftward when asked", () => {
    const { rerender } = render(<Ask said={SAID} />);
    fireEvent.mouseEnter(markAt(window.innerWidth - 20, 100));
    expect(bubble()?.style.left).toBe(`${window.innerWidth - 300 - 8}px`);
    fireEvent.mouseLeave(screen.getByRole("button", { name: SAID }));

    rerender(<Ask said={SAID} end />);
    fireEvent.mouseEnter(markAt(500, 100));
    expect(bubble()?.style.left).toBe("215px");
  });

  it("opens above a mark too low for it to fit below", () => {
    render(<Ask said={SAID} />);
    fireEvent.mouseEnter(markAt(40, window.innerHeight - 20));
    expect(bubble()?.style.top).toBe(`${window.innerHeight - 20 - 8}px`);
  });
});
