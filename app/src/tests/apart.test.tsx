import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import Apart from "../ui/Apart";

const doors = () =>
  screen
    .getAllByRole("button")
    .map((one) => one.textContent ?? "")
    .filter((text) => /merge the two|keep this machine|take what the folder has/i.test(text));

describe("what the window asks when a folder holds another history", () => {
  it("offers all three ways out to a stranger", () => {
    render(<Apart kin="strangers" onPick={vi.fn()} onElse={vi.fn()} onClose={vi.fn()} />);

    expect(screen.getByText(/already holds another tisty/i)).toBeTruthy();
    expect(doors()).toHaveLength(3);
    expect(screen.getByRole("button", { name: /pick another folder/i })).toBeTruthy();
  });

  it("never offers to merge two histories that acquired the same name", () => {
    render(<Apart kin="clash" onPick={vi.fn()} onElse={vi.fn()} onClose={vi.fn()} />);

    expect(doors()).toHaveLength(2);
    expect(screen.queryByText(/merge the two/i)).toBeNull();
  });

  it("asks nothing of its own lineage beyond taking the name", () => {
    render(<Apart kin="sameLineage" onPick={vi.fn()} onElse={vi.fn()} onClose={vi.fn()} />);

    expect(screen.getByText(/holds this machine's history/i)).toBeTruthy();
    expect(doors()).toHaveLength(0);
    expect(screen.getByRole("button", { name: /take the folder's name/i })).toBeTruthy();
  });

  it("offers nothing at all when it could not read enough to tell", () => {
    render(<Apart kin="unsure" onPick={vi.fn()} onElse={vi.fn()} onClose={vi.fn()} />);

    expect(screen.getByText(/could not be read well enough/i)).toBeTruthy();
    expect(doors()).toHaveLength(0);
    expect(screen.queryByRole("button", { name: /pick another folder/i })).toBeNull();
    expect(screen.getAllByRole("button", { name: /close/i })).toHaveLength(1);
  });
});
