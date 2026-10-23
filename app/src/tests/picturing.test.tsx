import { fireEvent, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { fill, t } from "../locales";
import PictureViewer from "../ui/PictureViewer";
import { pictureChoices, shotsOf, went } from "../ui/paging";

const page = () => {
  document.body.innerHTML = `<article class="page">
    <p data-at="0-30"><img src="asset://rina.jpg" alt="La Rina" data-kept="attachments/rina-ce960510.jpg"></p>
    <figure class="diagram" data-at="31-60"><img src="data:image/svg+xml,x" alt=""></figure>
    <p data-at="61-70">Palabras</p>
  </article>`;
  return document;
};

describe("a picture in a document", () => {
  it("is looked at with a click, not opened for writing", () => {
    const at = page();
    expect(went(at.querySelector("p img") as Element, false)).toEqual({ kind: "picture", nth: 0 });
    expect(went(at.querySelector("figure img") as Element, false)).toEqual({
      kind: "picture",
      nth: 1,
    });
    expect(went(at.querySelector("p:last-child") as Element, false)).toEqual({
      kind: "block",
      at: [61, 70],
    });
  });

  it("says which one is a kept file, so only that one is offered to save", () => {
    const shots = shotsOf(page());
    expect(shots.map((one) => one.kept)).toEqual(["attachments/rina-ce960510.jpg", null]);
  });
});

describe("the picture viewer", () => {
  const shots = [
    { src: "asset://rina.jpg", alt: "La Rina", kept: "attachments/rina-ce960510.jpg" },
    { src: "data:image/svg+xml,x", alt: "", kept: null },
  ];

  it("goes from one picture to the next and closes with Escape", async () => {
    const onClose = vi.fn();
    render(<PictureViewer shots={shots} start={0} onClose={onClose} onSave={() => {}} />);
    expect(screen.getByText(new RegExp(fill("pictureCount", "1", "2")))).toBeTruthy();

    await userEvent.keyboard("{ArrowRight}");
    expect(screen.getByText(new RegExp(fill("pictureCount", "2", "2")))).toBeTruthy();
    await userEvent.keyboard("{Escape}");
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("offers to save only what Tisty keeps", async () => {
    const onSave = vi.fn();
    render(<PictureViewer shots={shots} start={0} onClose={() => {}} onSave={onSave} />);
    await userEvent.click(screen.getByRole("button", { name: t("pictureSave") }));
    expect(onSave).toHaveBeenCalledWith("attachments/rina-ce960510.jpg");

    await userEvent.click(screen.getByRole("button", { name: t("pictureNext") }));
    expect(screen.queryByRole("button", { name: t("pictureSave") })).toBeNull();
  });

  it("zooms in and back", async () => {
    render(<PictureViewer shots={shots} start={0} onClose={() => {}} />);
    await userEvent.click(screen.getByRole("button", { name: t("pictureMore") }));
    expect(screen.getByRole("button", { name: "125%" })).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "125%" }));
    expect(screen.getByRole("button", { name: "100%" })).toBeTruthy();
  });
});

describe("the picture viewer over a task", () => {
  it("keeps its keys to itself, so Escape closes the picture and not what is behind it", async () => {
    const behind = vi.fn();
    const onClose = vi.fn();
    render(
      // biome-ignore lint/a11y/noStaticElementInteractions: stands in for a panel that closes on Escape
      <div onKeyDown={behind}>
        <PictureViewer
          shots={[{ src: "asset://rina.jpg", alt: "La Rina", kept: null }]}
          start={0}
          onClose={onClose}
        />
      </div>,
    );
    await userEvent.keyboard("{Escape}");
    expect(onClose).toHaveBeenCalledTimes(1);
    expect(behind).not.toHaveBeenCalled();
  });

  it("takes the focus in when it opens", () => {
    render(
      <PictureViewer
        shots={[{ src: "asset://rina.jpg", alt: "La Rina", kept: null }]}
        start={0}
        onClose={() => {}}
      />,
    );
    expect(screen.getByRole("dialog").contains(document.activeElement)).toBe(true);
  });
});

describe("a picture's own menu", () => {
  const shots = [
    { src: "asset://rina.jpg", alt: "La Rina", kept: "attachments/rina-ce960510.jpg" },
    { src: "data:image/svg+xml,x", alt: "", kept: null },
  ];
  const ways = { look: vi.fn(), save: vi.fn(), open: vi.fn() };

  it("offers to look, save and open a kept file, each with its own name", () => {
    const offered = pictureChoices(shots, 0, ways);
    expect(offered.map((one) => one.label)).toEqual([
      t("pictureLook"),
      t("pictureSave"),
      t("pictureOpen"),
    ]);
    for (const one of offered) one.onPick?.();
    expect(ways.look).toHaveBeenCalledWith(0);
    expect(ways.save).toHaveBeenCalledWith("attachments/rina-ce960510.jpg");
    expect(ways.open).toHaveBeenCalledWith("attachments/rina-ce960510.jpg");
  });

  it("offers only to look at a diagram, and nothing where there is no picture", () => {
    expect(pictureChoices(shots, 1, ways).map((one) => one.key)).toEqual(["look"]);
    expect(pictureChoices(shots, null, ways)).toEqual([]);
    expect(pictureChoices(shots, 7, ways)).toEqual([]);
  });
});

describe("handling a picture in the viewer", () => {
  const shots = [
    { src: "asset://rina.jpg", alt: "La Rina", kept: "attachments/rina-ce960510.jpg" },
    { src: "asset://otra.jpg", alt: "Otra", kept: null },
  ];
  const picture = () => within(screen.getByRole("dialog")).getByRole("img", { name: "La Rina" });

  it("zooms with the wheel and the keys, and a double click goes back and forth", () => {
    render(<PictureViewer shots={shots} start={0} onClose={() => {}} />);
    fireEvent.wheel(picture(), { deltaY: -100 });
    expect(screen.getByRole("button", { name: "115%" })).toBeTruthy();
    fireEvent.keyDown(picture(), { key: "0" });
    expect(screen.getByRole("button", { name: "100%" })).toBeTruthy();
    fireEvent.keyDown(picture(), { key: "+" });
    fireEvent.keyDown(picture(), { key: "-" });
    expect(screen.getByRole("button", { name: "100%" })).toBeTruthy();
    fireEvent.doubleClick(picture());
    expect(screen.getByRole("button", { name: "200%" })).toBeTruthy();
    fireEvent.doubleClick(picture());
    expect(screen.getByRole("button", { name: "100%" })).toBeTruthy();
  });

  it("moves a zoomed picture by dragging it, and not before", () => {
    render(<PictureViewer shots={shots} start={0} onClose={() => {}} />);
    fireEvent.mouseDown(picture(), { clientX: 10, clientY: 10 });
    fireEvent.mouseMove(picture(), { clientX: 60, clientY: 10 });
    expect(picture().style.transform).toContain("translate(0px, 0px)");

    fireEvent.doubleClick(picture());
    fireEvent.mouseDown(picture(), { clientX: 10, clientY: 10 });
    fireEvent.mouseMove(picture(), { clientX: 60, clientY: 30 });
    fireEvent.mouseUp(picture());
    fireEvent.mouseMove(picture(), { clientX: 200, clientY: 200 });
    expect(picture().style.transform).toContain("translate(50px, 20px)");
    fireEvent.mouseLeave(picture());
  });

  it("closes from a click beside the picture, and goes back with the left arrow", async () => {
    const onClose = vi.fn();
    render(<PictureViewer shots={shots} start={0} onClose={onClose} />);
    await userEvent.click(screen.getByRole("button", { name: t("picturePrevious") }));
    expect(within(screen.getByRole("dialog")).getByRole("img", { name: "Otra" })).toBeTruthy();
    fireEvent.keyDown(screen.getByRole("dialog"), { key: "ArrowLeft" });
    expect(picture()).toBeTruthy();
    fireEvent.mouseDown(picture().parentElement as HTMLElement);
    expect(onClose).toHaveBeenCalledTimes(1);
    fireEvent.mouseDown(screen.getByRole("dialog"));
    expect(onClose).toHaveBeenCalledTimes(2);
  });

  it("keeps Tab inside, lets no shortcut through, and opens a kept file with its app", async () => {
    const onOpen = vi.fn();
    const behind = vi.fn();
    render(
      // biome-ignore lint/a11y/noStaticElementInteractions: stands in for the window's own shortcuts
      <div onKeyDown={behind}>
        <PictureViewer shots={shots} start={0} onClose={() => {}} onOpen={onOpen} />
      </div>,
    );
    const close = screen.getByRole("button", { name: t("pictureClose") });
    close.focus();
    fireEvent.keyDown(close, { key: "Tab" });
    expect(screen.getByRole("dialog").contains(document.activeElement)).toBe(true);
    fireEvent.keyDown(close, { key: "Tab", shiftKey: true });
    fireEvent.keyDown(close, { key: "e", ctrlKey: true });
    fireEvent.keyDown(close, { key: "x" });
    expect(behind).not.toHaveBeenCalled();
    await userEvent.click(screen.getByRole("button", { name: t("pictureOpen") }));
    expect(onOpen).toHaveBeenCalledWith("attachments/rina-ce960510.jpg");
  });
});

describe("what counts as a picture to look at", () => {
  it("leaves a summary's picture to fold its block, and skips one folded away", () => {
    document.body.innerHTML = `<article class="page">
      <details data-at="0-20"><summary><img src="asset://a.png" alt="a"></summary>
        <img src="asset://b.png" alt="b"></details>
      <p data-at="21-30"><img src="asset://c.png" alt="c"></p>
    </article>`;
    expect(went(document.querySelector("summary img") as Element, false)).toBeNull();
    expect(shotsOf(document).map((one) => one.alt)).toEqual(["c"]);
  });

  it("names a picture with no words by its place alone", () => {
    render(
      <PictureViewer
        shots={[
          { src: "asset://a.png", alt: "", kept: null },
          { src: "asset://b.png", alt: "", kept: null },
        ]}
        start={0}
        onClose={() => {}}
      />,
    );
    expect(screen.getByText(fill("pictureCount", "1", "2"))).toBeTruthy();
  });
});
