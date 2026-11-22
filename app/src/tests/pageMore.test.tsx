import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { StrictMode } from "react";
import { describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import { type Reaching, wired } from "../ui/dressing";
import { framed as dressedPage } from "../ui/framed";
import { PageMore, TitleRow } from "../ui/liveSheet";
import { unnavigated } from "../unnavigated";

const opened = async (markdown: boolean, onTurn = vi.fn()) => {
  render(<PageMore markdown={markdown} onTurn={onTurn} />);
  const button = screen.getByRole("button", { name: t("pageMore") });
  await userEvent.click(button);
  return { button, onTurn };
};

describe("the page's own choices", () => {
  it("stay hidden until the quiet button is pressed", async () => {
    render(<PageMore markdown={false} onTurn={vi.fn()} />);

    expect(screen.queryByRole("menu")).toBeNull();
    expect(screen.getByRole("button", { name: t("pageMore") }).getAttribute("aria-expanded")).toBe(
      "false",
    );
  });

  it("show both ways of seeing the words, with the one in use marked", async () => {
    const { button } = await opened(false);

    const rich = screen.getByRole("menuitem", { name: new RegExp(t("viewRich")) });
    const written = screen.getByRole("menuitem", { name: new RegExp(t("viewMarkdown")) });
    expect(button.getAttribute("aria-expanded")).toBe("true");
    expect(rich.className).toContain("font-semibold");
    expect(written.className).not.toContain("font-semibold");
  });

  it("turn the page to Markdown and close", async () => {
    const { onTurn } = await opened(false);

    await userEvent.click(screen.getByRole("menuitem", { name: new RegExp(t("viewMarkdown")) }));

    expect(onTurn).toHaveBeenCalledWith(true);
    expect(screen.queryByRole("menu")).toBeNull();
  });

  it("turn Markdown back to the rich page", async () => {
    const { onTurn } = await opened(true);

    await userEvent.click(screen.getByRole("menuitem", { name: new RegExp(t("viewRich")) }));

    expect(onTurn).toHaveBeenCalledWith(false);
  });

  it("leave the page as it is when the way in use is picked again", async () => {
    const { onTurn } = await opened(true);

    await userEvent.click(screen.getByRole("menuitem", { name: new RegExp(t("viewMarkdown")) }));

    expect(onTurn).not.toHaveBeenCalled();
  });
});

describe("the page's own choices on a document's card", () => {
  const framed = () => {
    const frame = document.createElement("iframe");
    document.body.append(frame);
    frame.getBoundingClientRect = () => new DOMRect(300, 120, 700, 800);
    return frame;
  };

  const wiredRow = async () => {
    const frame = framed();
    const page = frame.contentDocument as Document;
    page.open();
    page.write(dressedPage('<p data-at="0-5">leche</p>'));
    page.close();
    const heard = { away: 0 };
    const reach: Reaching = { body: "leche", onDoc: () => {}, onAway: () => (heard.away += 1) };
    wired(page, () => reach);
    page.addEventListener("keydown", unnavigated);
    render(
      <TitleRow frame={() => frame.contentWindow}>
        <PageMore markdown={false} onTurn={vi.fn()} />
      </TitleRow>,
    );
    const button = await waitFor(() => {
      const found = page.querySelector<HTMLButtonElement>(".page-more-row > button");
      expect(found).not.toBeNull();
      return found as HTMLButtonElement;
    });
    return { frame, page, button, heard };
  };

  it("stand in the frame, on the row the title opens, and open in the window", async () => {
    const frame = framed();
    render(
      <TitleRow frame={() => frame.contentWindow}>
        <PageMore markdown={false} onTurn={vi.fn()} />
      </TitleRow>,
    );
    const page = frame.contentDocument as Document;
    const button = await waitFor(() => {
      const found = page.querySelector<HTMLButtonElement>("body > .page-more-row > button");
      expect(found).not.toBeNull();
      return found as HTMLButtonElement;
    });
    expect(page.body.firstElementChild?.className).toBe("page-more-row");
    expect(document.querySelector(".page-more-row")).toBeNull();
    button.getBoundingClientRect = () => new DOMRect(600, 40, 32, 32);

    fireEvent.mouseDown(button);

    const menu = screen.getByRole("menu");
    expect(menu.ownerDocument).toBe(document);
    expect(button.getAttribute("aria-expanded")).toBe("true");
    frame.remove();
  });

  it("shut when the page under them scrolls, as the button leaves with the title", async () => {
    const frame = framed();
    render(
      <TitleRow frame={() => frame.contentWindow}>
        <PageMore markdown={false} onTurn={vi.fn()} />
      </TitleRow>,
    );
    const page = frame.contentDocument as Document;
    const button = await waitFor(() => {
      const found = page.querySelector<HTMLButtonElement>(".page-more-row > button");
      expect(found).not.toBeNull();
      return found as HTMLButtonElement;
    });
    fireEvent.mouseDown(button);
    expect(screen.getByRole("menu")).not.toBeNull();

    act(() => {
      frame.contentWindow?.dispatchEvent(new Event("scroll"));
    });

    expect(screen.queryByRole("menu")).toBeNull();
    frame.remove();
  });

  it("are no press on the page: the block being written, its keys and the caret stay", async () => {
    const { frame, page, button, heard } = await wiredRow();
    const reached: string[] = [];
    for (const kind of ["keydown", "keyup", "mousedown", "pointerdown", "mouseup", "contextmenu"])
      page.addEventListener(kind, () => reached.push(kind));

    fireEvent.pointerDown(button);
    fireEvent.mouseDown(button);
    fireEvent.mouseUp(button);
    fireEvent.click(button, { detail: 1 });
    fireEvent.keyDown(button, { key: "Enter" });
    fireEvent.keyDown(button, { key: " " });
    fireEvent.keyUp(button, { key: "Enter" });
    const pressed = fireEvent.contextMenu(button);

    expect(heard.away).toBe(0);
    expect(reached).toEqual([]);
    expect(pressed).toBe(false);
    expect(screen.getByRole("menu")).not.toBeNull();
    frame.remove();
  });

  it("let the window's shortcuts and the guard on Backspace hear the keys pressed on them", async () => {
    const { frame, button } = await wiredRow();
    const shortcuts: string[] = [];
    const heard = (event: KeyboardEvent) => shortcuts.push(event.key);
    window.addEventListener("keydown", heard);

    const backed = fireEvent.keyDown(button, { key: "Backspace" });
    fireEvent.keyDown(button, { key: "e", ctrlKey: true });
    fireEvent.keyDown(button, { key: "F1" });

    window.removeEventListener("keydown", heard);
    expect(backed).toBe(false);
    expect(shortcuts).toEqual(["Backspace", "e", "F1"]);
    frame.remove();
  });

  it("leave the focus where it went when the window lost it with them open", async () => {
    const { frame, page, button } = await wiredRow();
    fireEvent.click(button, { detail: 0 });
    expect(screen.getByRole("menu")).not.toBeNull();
    page.body.focus();
    const away = vi.spyOn(document, "hasFocus").mockReturnValue(false);

    await act(async () => {
      await new Promise((done) => setTimeout(done, 0));
      window.dispatchEvent(new Event("blur"));
    });

    away.mockRestore();
    expect(screen.queryByRole("menu")).toBeNull();
    expect(page.activeElement).not.toBe(button);
    frame.remove();
  });

  it("hand the keyboard back to the button once they shut", async () => {
    const frame = framed();
    render(
      <TitleRow frame={() => frame.contentWindow}>
        <PageMore markdown={false} onTurn={vi.fn()} />
      </TitleRow>,
    );
    const page = frame.contentDocument as Document;
    const button = await waitFor(() => {
      const found = page.querySelector<HTMLButtonElement>(".page-more-row > button");
      expect(found).not.toBeNull();
      return found as HTMLButtonElement;
    });
    fireEvent.click(button, { detail: 0 });
    expect(screen.getByRole("menu")).not.toBeNull();
    page.body.focus();

    fireEvent.click(button, { detail: 0 });

    expect(screen.queryByRole("menu")).toBeNull();
    expect(page.activeElement).toBe(button);
    frame.remove();
  });

  it("stand once on the row, however often the window mounts them", async () => {
    const frame = framed();
    render(
      <StrictMode>
        <TitleRow frame={() => frame.contentWindow}>
          <PageMore markdown={false} onTurn={vi.fn()} />
        </TitleRow>
      </StrictMode>,
    );
    const page = frame.contentDocument as Document;
    await waitFor(() => expect(page.querySelector(".page-more-row > button")).not.toBeNull());

    expect(page.querySelectorAll(".page-more-row")).toHaveLength(1);
    frame.remove();
  });

  it("go with the card when it goes", async () => {
    const frame = framed();
    const view = render(
      <TitleRow frame={() => frame.contentWindow}>
        <PageMore markdown={false} onTurn={vi.fn()} />
      </TitleRow>,
    );
    const page = frame.contentDocument as Document;
    await waitFor(() => expect(page.querySelector(".page-more-row")).not.toBeNull());

    view.unmount();

    expect(page.querySelector(".page-more-row")).toBeNull();
    frame.remove();
  });
});
