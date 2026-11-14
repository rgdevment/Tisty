import { act, fireEvent, render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { FrameThumb } from "../ui/frameThumb";

const framed = (scrollHeight: number, clientHeight: number) => {
  const sheet = document.createElement("div");
  Object.defineProperty(sheet, "scrollHeight", { value: scrollHeight });
  Object.defineProperty(sheet, "clientHeight", { value: clientHeight });
  sheet.scrollTop = 0;
  const heard = new EventTarget();
  const inner = Object.assign(heard, { document: { scrollingElement: sheet } });
  return { sheet, frame: () => inner as unknown as Window, heard };
};

const thumbOf = (container: HTMLElement) =>
  container.querySelector<HTMLElement>("[aria-hidden] > div");

describe("the thumb laid over the edge of a sheet written in place", () => {
  it("is as tall as the part in sight, and shows while the page rolls", () => {
    const { sheet, frame, heard } = framed(2000, 500);
    const view = render(<FrameThumb frame={frame} ready turned={1} />);
    const thumb = thumbOf(view.container);
    expect(thumb?.style.height).toBe("125px");
    expect(thumb?.className).toContain("opacity-0");
    sheet.scrollTop = 1500;
    act(() => {
      heard.dispatchEvent(new Event("scroll"));
    });
    expect(thumbOf(view.container)?.style.top).toBe("375px");
    expect(thumbOf(view.container)?.className).toContain("opacity-100");
  });

  it("rolls the page as it is dragged, by as much as the page is taller than its frame", () => {
    const { sheet, frame } = framed(2000, 500);
    const view = render(<FrameThumb frame={frame} ready turned={1} />);
    const thumb = thumbOf(view.container) as HTMLElement;
    thumb.setPointerCapture = () => {};
    fireEvent.pointerDown(thumb, { clientY: 100, pointerId: 1 });
    fireEvent.pointerMove(thumb, { clientY: 175, pointerId: 1 });
    fireEvent.pointerUp(thumb, { pointerId: 1 });
    expect(sheet.scrollTop).toBe(300);
  });

  it("draws nothing for a page that fits its frame", () => {
    const { frame } = framed(400, 500);
    const view = render(<FrameThumb frame={frame} ready turned={1} />);
    expect(thumbOf(view.container)).toBeNull();
  });
});
