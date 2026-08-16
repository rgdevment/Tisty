import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it } from "vitest";
import type { Rift } from "../core";
import Rifts from "../ui/Rifts";

const one: Rift = {
  was: ["la introduccion"],
  mine: ["la introduccion del mac"],
  theirs: ["la introduccion de windows"],
};

const two: Rift = { was: [], mine: ["algo mio"], theirs: ["algo suyo"] };

let picked: unknown;

beforeEach(() => {
  picked = undefined;
});

const shown = (rifts: Rift[]) =>
  render(
    <Rifts
      named="Kit de transmisión"
      rifts={rifts}
      onDone={(picks) => {
        picked = picks;
      }}
      onClose={() => {
        picked = null;
      }}
    />,
  );

describe("deciding a document block by block", () => {
  it("shows what each side wrote and what was there before", () => {
    shown([one]);

    expect(screen.getByText("la introduccion del mac")).toBeTruthy();
    expect(screen.getByText("la introduccion de windows")).toBeTruthy();
    expect(screen.getByText("la introduccion")).toBeTruthy();
  });

  it("never puts the name of a file in front of a person", () => {
    shown([one]);

    expect(screen.getByRole("dialog").textContent).toMatch(/Kit de transmisión/);
  });

  it("will not finish until every block has an answer", async () => {
    shown([one, two]);
    const done = screen.getByRole("button", { name: /^done$/i });

    expect((done as HTMLButtonElement).disabled).toBe(true);

    await userEvent.click(screen.getAllByRole("button", { name: /^this one$/i })[0]);
    expect((done as HTMLButtonElement).disabled).toBe(true);

    await userEvent.click(screen.getAllByRole("button", { name: /^the other$/i })[1]);
    expect((done as HTMLButtonElement).disabled).toBe(false);
  });

  it("hands back one answer per block, in the order they were shown", async () => {
    shown([one, two]);

    await userEvent.click(screen.getAllByRole("button", { name: /^this one$/i })[0]);
    await userEvent.click(screen.getAllByRole("button", { name: /^both$/i })[1]);
    await userEvent.click(screen.getByRole("button", { name: /^done$/i }));

    await waitFor(() => expect(picked).toEqual(["mine", "both"]));
  });

  it("says how many are left, so nobody hunts for the one they missed", async () => {
    shown([one, two]);

    expect(screen.getByText(/2 left/i)).toBeTruthy();

    await userEvent.click(screen.getAllByRole("button", { name: /^this one$/i })[0]);

    expect(screen.getByText(/1 left/i)).toBeTruthy();
  });

  it("answers nothing at all when it is closed, so no side is picked by accident", async () => {
    shown([one]);

    await userEvent.click(screen.getByRole("button", { name: /^close$/i }));

    expect(picked).toBeNull();
  });
});
