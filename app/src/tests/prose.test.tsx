import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import Prose from "../ui/Prose";

const KNOWN = ["infra/notificaciones", "infra/gateway"];

function field(over: Partial<React.ComponentProps<typeof Prose>> = {}) {
  const onWrite = vi.fn();
  render(
    <Prose
      value="sale de **contabilidad**"
      hint="qué hay que hacer"
      label="Descripción"
      known={KNOWN}
      onWrite={onWrite}
      {...over}
    />,
  );
  return onWrite;
}

const region = () => screen.getByLabelText("Descripción");
const box = () => screen.getByRole("textbox", { name: "Descripción" }) as HTMLTextAreaElement;

describe("source where the cursor is, composed where it is not", () => {
  it("starts composed, with no textbox to be seen", () => {
    field();
    expect(region().querySelector("strong")?.textContent).toBe("contabilidad");
    expect(screen.queryByRole("textbox")).toBeNull();
  });

  it("shows its source on entering and composes again on leaving", async () => {
    const user = userEvent.setup();
    field();

    await user.click(region());
    expect(box().value).toBe("sale de **contabilidad**");

    await user.tab();
    expect(screen.queryByRole("textbox")).toBeNull();
    expect(region().querySelector("strong")).toBeTruthy();
  });

  it("reaches the same field with the keyboard alone", async () => {
    const user = userEvent.setup();
    field();

    await user.tab();
    expect(box()).toBe(document.activeElement);
  });

  it("says what it is for when there is nothing written yet", () => {
    field({ value: "" });
    expect(region().textContent).toContain("qué hay que hacer");
  });
});

describe("the second column", () => {
  it("stays away where there is no room for it", async () => {
    const user = userEvent.setup();
    field();

    await user.click(region());
    expect(screen.queryByLabelText("Composed")).toBeNull();
  });

  it("appears beside the source only once asked for", async () => {
    const user = userEvent.setup();
    field({ beside: true });

    await user.click(region());
    expect(screen.getByLabelText("Composed").querySelector("strong")).toBeTruthy();
  });
});

describe("the / menu", () => {
  const slash = async (user: ReturnType<typeof userEvent.setup>) => {
    await user.click(region());
    await user.type(box(), "{End} /");
  };

  it("offers two things, because a ticket is a link", async () => {
    const user = userEvent.setup();
    field();
    await slash(user);

    expect(screen.getByRole("button", { name: /Document/ })).toBeTruthy();
    expect(screen.getByRole("button", { name: /Link/ })).toBeTruthy();
    expect(screen.queryByRole("button", { name: /Ticket/ })).toBeNull();
  });

  it("writes a link from its text and its address, replacing the slash", async () => {
    const user = userEvent.setup();
    const onWrite = field({ value: "" });
    await slash(user);

    await user.click(screen.getByRole("button", { name: /Link/ }));
    await user.type(screen.getByLabelText("Text"), "OPS-3465");
    await user.type(screen.getByLabelText("URL"), "https://jira.example/OPS-3465{Enter}");

    expect(box().value).toBe(" [OPS-3465](https://jira.example/OPS-3465)");
    await user.tab();
    expect(onWrite).toHaveBeenCalledWith(" [OPS-3465](https://jira.example/OPS-3465)");
  });

  it("falls back to the address when no text is given", async () => {
    const user = userEvent.setup();
    field({ value: "" });
    await slash(user);

    await user.click(screen.getByRole("button", { name: /Link/ }));
    await user.type(screen.getByLabelText("URL"), "https://x.example{Enter}");

    expect(box().value).toBe(" [https://x.example](https://x.example)");
  });

  it("offers the references already in use and writes one", async () => {
    const user = userEvent.setup();
    field({ value: "" });
    await slash(user);

    await user.click(screen.getByRole("button", { name: /Document/ }));
    await user.click(screen.getByRole("button", { name: "infra/gateway" }));

    expect(box().value).toBe(" [[infra/gateway]]");
  });

  it("closes on Escape without discarding what was written", async () => {
    const user = userEvent.setup();
    const onWrite = field();
    await slash(user);

    await user.keyboard("{Escape}");
    expect(screen.queryByRole("button", { name: /Link/ })).toBeNull();
    expect(box().value).toBe("sale de **contabilidad** /");
    expect(onWrite).not.toHaveBeenCalled();
  });
});
