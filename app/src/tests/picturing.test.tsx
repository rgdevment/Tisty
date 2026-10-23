import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { fill, t } from "../locales";
import PictureViewer from "../ui/PictureViewer";
import { shotsOf, went } from "../ui/paging";

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
