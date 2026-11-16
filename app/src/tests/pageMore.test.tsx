import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import { PageMore } from "../ui/liveSheet";

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
