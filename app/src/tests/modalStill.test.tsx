import { fireEvent, render, screen } from "@testing-library/react";
import { expect, it, vi } from "vitest";
import Modal from "../ui/Modal";

it("takes the focus back without rolling what is being pressed from under the pointer", () => {
  render(
    <Modal title="Largo" closable onClose={() => {}}>
      <button type="button">Encender</button>
    </Modal>,
  );
  const close = screen.getByRole("button", { name: "Close" });
  const focused = vi.spyOn(close, "focus");
  close.blur();
  fireEvent.blur(close, { relatedTarget: null });
  expect(focused).toHaveBeenCalledWith({ preventScroll: true });
});
