import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Shape } from "../core";
import Cover from "../ui/Cover";

const ipc = vi.hoisted(() => ({ shape: null as Shape | null }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve(ipc.shape),
}));

const shape = (some: Partial<Shape>): Shape => ({
  closed: 1,
  dropped: 0,
  told: 0,
  months: [],
  ...some,
});

const shown = async (told: Shape) => {
  ipc.shape = told;
  render(<Cover />);
  return screen.findByRole("region");
};

beforeEach(() => {
  ipc.shape = null;
});

describe("what the archive holds", () => {
  it("shows nothing at all when nothing has been closed", () => {
    ipc.shape = shape({ closed: 0 });
    const { container } = render(<Cover />);

    expect(container.textContent).toBe("");
  });

  it("uses the singular for a lone one that left something written", async () => {
    await shown(shape({ closed: 3, told: 1 }));

    expect(screen.getByText(/one left something written/i)).toBeTruthy();
  });

  it("keeps quiet about drops when there are none", async () => {
    await shown(shape({ closed: 3, told: 2, dropped: 0 }));

    expect(screen.queryByText(/dropped/i)).toBeNull();
  });

  it("says it in the singular for a lone drop", async () => {
    await shown(shape({ closed: 3, told: 2, dropped: 1 }));

    expect(screen.getByText(/one dropped/i)).toBeTruthy();
  });

  it("draws no strip for a single month, because one bar is not a timeline", async () => {
    await shown(shape({ closed: 2, told: 1, months: [{ key: "2026-08", closed: 2 }] }));

    expect(screen.queryByRole("img")).toBeNull();
  });

  it("draws the strip once there is a stretch, quiet months included", async () => {
    await shown(
      shape({
        closed: 3,
        told: 2,
        months: [
          { key: "2026-06", closed: 2 },
          { key: "2026-07", closed: 0 },
          { key: "2026-08", closed: 1 },
        ],
      }),
    );

    const strip = screen.getByRole("img");
    expect(strip.children).toHaveLength(3);
    expect(screen.getByText(/most: 2/i)).toBeTruthy();
  });

  it("labels the months in words rather than in the key it stores them under", async () => {
    await shown(
      shape({
        closed: 2,
        told: 1,
        months: [
          { key: "2026-07", closed: 1 },
          { key: "2026-08", closed: 1 },
        ],
      }),
    );

    expect(screen.queryByText("2026-07")).toBeNull();
    expect(screen.getAllByText(/Jul/).length).toBeGreaterThan(0);
  });
});
