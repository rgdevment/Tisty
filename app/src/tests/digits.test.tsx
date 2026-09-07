import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";
import { fill } from "../locales";
import Digits, { HOW_MANY } from "../ui/Digits";

const box = (n: number) => screen.getByLabelText(fill("digitOf", String(n), String(HOW_MANY)));

function Asking({ onDone }: { onDone?: () => void }) {
  const [said, setSaid] = useState("");
  return (
    <>
      <Digits label="Seis dígitos" value={said} onChange={setSaid} onDone={onDone} />
      <button type="button">Fuera</button>
      <output>{said}</output>
    </>
  );
}

describe("six boxes for a number", () => {
  it("takes one digit per box and moves along on its own", async () => {
    render(<Asking />);

    await userEvent.type(box(1), "123456");

    expect(screen.getByRole("status").textContent).toBe("123456");
    expect((box(3) as HTMLInputElement).value).toBe("3");
  });

  it("takes nothing but digits", async () => {
    render(<Asking />);

    await userEvent.type(box(1), "1a2b3c");

    expect(screen.getByRole("status").textContent).toBe("123");
  });

  it("gives the digits back one at a time", async () => {
    render(<Asking />);
    await userEvent.type(box(1), "1234");

    await userEvent.keyboard("{Backspace}{Backspace}");

    expect(screen.getByRole("status").textContent).toBe("12");
    expect((box(3) as HTMLInputElement).value).toBe("");
  });

  it("keeps what is written when what was pasted holds no digits", async () => {
    render(<Asking />);
    await userEvent.type(box(1), "1234");

    await userEvent.click(box(1));
    await userEvent.paste("hola");

    expect(screen.getByRole("status").textContent).toBe("1234");
  });

  it("spreads a pasted number and drops what will not fit", async () => {
    render(<Asking />);

    await userEvent.click(box(1));
    await userEvent.paste("12-34-56-78");

    expect(screen.getByRole("status").textContent).toBe("123456");
  });

  it("lets the keyboard leave, written or not", async () => {
    render(<Asking />);
    await userEvent.type(box(1), "12");

    await userEvent.tab();

    expect(document.activeElement).not.toBe(box(3));
  });

  it("finishes on enter once the six are there, and not before", async () => {
    const done = vi.fn();
    render(<Asking onDone={done} />);

    await userEvent.type(box(1), "12345");
    await userEvent.keyboard("{Enter}");
    expect(done).not.toHaveBeenCalled();

    await userEvent.type(box(6), "6");
    await userEvent.keyboard("{Enter}");
    expect(done).toHaveBeenCalledTimes(1);
  });
});
