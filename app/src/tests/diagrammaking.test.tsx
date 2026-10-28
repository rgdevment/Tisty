import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import DiagramMaking from "../ui/DiagramMaking";
import { nameOf } from "../ui/drawing/starting";

const opened = () => {
  const heard = { make: vi.fn(), close: vi.fn() };
  render(<DiagramMaking onMake={heard.make} onClose={heard.close} />);
  return { ...heard, field: screen.getByLabelText(t("diagramName")) as HTMLInputElement };
};

describe("naming a new diagram", () => {
  it("names it after its kind until the person writes a name of their own", async () => {
    const { make, field } = opened();
    expect(field.value).toBe(nameOf("flowchart"));

    await userEvent.click(screen.getByLabelText(nameOf("erDiagram")));
    expect(field.value).toBe(nameOf("erDiagram"));

    await userEvent.clear(field);
    await userEvent.type(field, "Ventas");
    await userEvent.click(screen.getByLabelText(nameOf("mindmap")));
    expect(field.value).toBe("Ventas");

    fireEvent.keyDown(field, { key: "Enter" });
    expect(make).toHaveBeenCalledWith("Ventas", "mindmap");
  });

  it("makes nothing without a name, and closes when asked", async () => {
    const { make, close, field } = opened();
    await userEvent.clear(field);

    await userEvent.click(screen.getByRole("button", { name: t("create") }));
    await userEvent.click(screen.getByRole("button", { name: t("cancel") }));

    expect(make).not.toHaveBeenCalled();
    expect(close).toHaveBeenCalled();
  });
});
