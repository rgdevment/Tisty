import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { Folded } from "../core";
import { type FolderAnswer, type FolderAsked, folderDropAsked } from "../folderDropping";
import { fill, t } from "../locales";
import FolderDropping from "../ui/FolderDropping";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve([]),
}));

const folder = (id: string, name: string, more: Partial<Folded> = {}): Folded => ({
  id,
  name,
  parent: null,
  icon: null,
  holds: 0,
  archived: false,
  away: false,
  ...more,
});

const above = folder("up", "Oficina");
const going = folder("go", "Trabajo", { parent: "up" });
const inside = folder("in", "Dentro", { parent: "go" });
const aside = folder("side", "Casa");
const shelved = folder("old", "Vieja", { away: true, archived: true });
const agents = folder("ag", "AGENTS", { icon: "robot" });
const agentsKid = folder("agk", "Claude", { parent: "ag" });
const folders = [above, going, inside, aside, shelved, agents, agentsKid];

const asked = (more: Partial<FolderAsked> = {}): FolderAsked => ({
  folder: going,
  up: above,
  docs: 3,
  folders: 1,
  handsOnly: false,
  ...more,
});

const ask = async (more: Partial<FolderAsked> = {}) => {
  render(<FolderDropping folders={folders} />);
  let answer: Promise<FolderAnswer | null> = Promise.resolve(null);
  act(() => {
    answer = folderDropAsked(asked(more));
  });
  await screen.findByRole("dialog");
  return { answer };
};

const yes = () => screen.getByRole("button", { name: t("folderDropYes") });

describe("what happens to what a deleted folder holds", () => {
  it("says what the folder holds and offers three ways out, the folder above first", async () => {
    const { answer } = await ask();

    expect(screen.getByText(fill("folderHolds", "3", "1"))).toBeTruthy();
    const radios = screen.getAllByRole("radio");
    expect(radios).toHaveLength(3);
    expect((radios[0] as HTMLInputElement).checked).toBe(true);
    expect(screen.getByRole("radio", { name: new RegExp(fill("folderDropUp", "Oficina")) })).toBe(
      radios[0],
    );

    await userEvent.click(screen.getByRole("button", { name: t("cancel") }));
    await answer;
  });

  it("moves what it holds up when nothing else was chosen", async () => {
    const { answer } = await ask();

    await userEvent.click(yes());

    expect(await answer).toEqual({ contents: "up" });
  });

  it("names Unfiled as the place above when the folder is at the top", async () => {
    const { answer } = await ask({ up: null });

    expect(screen.getByRole("radio", { name: new RegExp(t("folderDropUnfiled")) })).toBeTruthy();
    await userEvent.click(yes());
    expect(await answer).toEqual({ contents: "up" });
  });

  it("deletes what it holds when that is chosen", async () => {
    const { answer } = await ask();

    await userEvent.click(screen.getByRole("radio", { name: new RegExp(t("folderDropAll")) }));
    await userEvent.click(yes());

    expect(await answer).toEqual({ contents: "drop" });
  });

  it("will not go on with «into» until a folder is picked, and answers with the one picked", async () => {
    const { answer } = await ask();

    await userEvent.click(screen.getByRole("radio", { name: new RegExp(t("folderDropInto")) }));
    expect((yes() as HTMLButtonElement).disabled).toBe(true);

    await userEvent.selectOptions(
      screen.getByRole("combobox", { name: t("folderDropInto") }),
      "side",
    );
    expect((yes() as HTMLButtonElement).disabled).toBe(false);
    await userEvent.click(yes());

    expect(await answer).toEqual({ contents: "into", into: "side" });
  });

  it("offers as a place only folders outside the one going, not the one above, not the archived", async () => {
    const { answer } = await ask();

    await userEvent.click(screen.getByRole("radio", { name: new RegExp(t("folderDropInto")) }));

    const places = screen
      .getAllByRole("option")
      .map((one) => (one as HTMLOptionElement).value)
      .filter(Boolean);
    expect(places).toEqual(["~root", "side"]);
    await userEvent.click(screen.getByRole("button", { name: t("cancel") }));
    await answer;
  });

  it("answers the top of the library as no folder at all", async () => {
    const { answer } = await ask();

    await userEvent.click(screen.getByRole("radio", { name: new RegExp(t("folderDropInto")) }));
    await userEvent.selectOptions(
      screen.getByRole("combobox", { name: t("folderDropInto") }),
      "~root",
    );
    await userEvent.click(yes());

    expect(await answer).toEqual({ contents: "into", into: undefined });
  });

  it("offers only to move it up when what it holds is only handed up", async () => {
    const { answer } = await ask({ handsOnly: true });

    expect(screen.getAllByRole("radio")).toHaveLength(1);
    expect(screen.queryByRole("radio", { name: new RegExp(t("folderDropAll")) })).toBeNull();
    await userEvent.click(yes());
    expect(await answer).toEqual({ contents: "up" });
  });

  it("answers null when it is cancelled, and when the window is closed", async () => {
    const { answer: first } = await ask();
    await userEvent.click(screen.getByRole("button", { name: t("cancel") }));
    expect(await first).toBeNull();
    expect(screen.queryByRole("dialog")).toBeNull();

    let second: Promise<FolderAnswer | null> = Promise.resolve({ contents: "up" });
    act(() => {
      second = folderDropAsked(asked());
    });
    await screen.findByRole("dialog");
    await userEvent.keyboard("{Escape}");
    expect(await second).toBeNull();
  });

  it("starts from the first choice every time it asks", async () => {
    const { answer: first } = await ask();
    await userEvent.click(screen.getByRole("radio", { name: new RegExp(t("folderDropAll")) }));
    await userEvent.click(screen.getByRole("button", { name: t("cancel") }));
    await first;

    act(() => {
      void folderDropAsked(asked({ folder: inside, up: going }));
    });
    await screen.findByRole("dialog");

    expect((screen.getAllByRole("radio")[0] as HTMLInputElement).checked).toBe(true);
  });

  it("never offers a folder of Agents to move what it holds into", async () => {
    const { answer } = await ask();

    await userEvent.click(screen.getByRole("radio", { name: new RegExp(t("folderDropInto")) }));
    const picker = screen.getByRole("combobox") as HTMLSelectElement;
    const named = [...picker.options].map((one) => one.textContent);

    expect(named.some((one) => /Claude/.test(one ?? ""))).toBe(false);
    expect(named.some((one) => /Casa/.test(one ?? ""))).toBe(true);

    await userEvent.click(screen.getByRole("button", { name: t("cancel") }));
    await answer;
  });
});
