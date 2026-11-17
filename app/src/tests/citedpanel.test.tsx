import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Citing } from "../citing";
import { t } from "../locales";
import Beside from "../ui/Beside";

const bus = vi.hoisted(() => ({
  heard: new Map<string, () => void>(),
  citing: null as Citing[] | null,
  asked: 0,
  held: null as ((citing: Citing[] | null) => void) | null,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    if (cmd === "doc_cited_by") {
      bus.asked += 1;
      if (bus.asked === 1 && bus.held === null && bus.citing?.[0]?.id === "slow")
        return new Promise((done) => {
          bus.held = done;
        });
      return Promise.resolve(bus.citing);
    }
    return Promise.resolve(cmd === "doc_facts" ? { bytes: 7, pages: 0 } : null);
  },
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: (named: string, fn: () => void) => {
    bus.heard.set(named, fn);
    return Promise.resolve(() => bus.heard.delete(named));
  },
}));

const TRIPS: Citing = {
  id: "dev_a-0002",
  kind: "doc",
  title: "Viajes",
  card: false,
  state: "open",
  under: [
    { id: "hoteles", title: "Hoteles" },
    { id: "comida", title: "Comida" },
  ],
};
const ERRAND: Citing = { id: "01K", kind: "task", title: "Llamar", card: false, state: "done" };

const shown = (onCited = vi.fn()) =>
  render(
    <Beside
      title="Veterinario"
      paper="dev_a-0001"
      body="# Veterinario"
      tags={[]}
      kept={0}
      heads={[]}
      pages={[]}
      leaf="a4"
      onLeaf={() => {}}
      making={false}
      onCopy={() => {}}
      onTakeOut={() => {}}
      onShut={() => {}}
      onApart={() => {}}
      onCited={onCited}
    />,
  );

const row = () => document.querySelector("details[data-row=cited]") as HTMLDetailsElement | null;

describe("who links to the open document", () => {
  beforeEach(() => {
    bus.heard.clear();
    bus.citing = null;
    bus.asked = 0;
    bus.held = null;
  });

  it("lists each document and task with the sections that link, and opens the one pressed", async () => {
    bus.citing = [TRIPS, ERRAND];
    const onCited = vi.fn();
    shown(onCited);

    await waitFor(() => expect(row()?.open).toBe(true));
    expect(row()?.querySelector(".side-gist")?.textContent).toBe("2");
    expect(screen.getByText("In: Hoteles · Comida")).toBeTruthy();
    expect(document.querySelectorAll(".is-cited.is-away")).toHaveLength(1);
    await userEvent.click(screen.getByText("Viajes"));
    expect(onCited).toHaveBeenCalledWith(TRIPS);
  });

  it("says the links are still being read rather than that nobody links here", async () => {
    shown();

    await waitFor(() => expect(screen.getByText(t("citedNotReady"))).toBeTruthy());
    expect(screen.queryByText(t("citedNobody"))).toBeNull();
  });

  it("asks again when the brain has moved, so a link written elsewhere shows up", async () => {
    bus.citing = [];
    shown();
    await waitFor(() => expect(screen.getByText(t("citedNobody"))).toBeTruthy());

    bus.citing = [TRIPS];
    await act(async () => bus.heard.get("brain")?.());

    await waitFor(() => expect(screen.getByText("Viajes")).toBeTruthy());
    expect(bus.asked).toBe(2);
  });

  it("keeps the latest answer when an older one arrives after it", async () => {
    bus.citing = [{ ...TRIPS, id: "slow", title: "Viejo" }];
    shown();
    await waitFor(() => expect(bus.held).not.toBeNull());

    bus.citing = [TRIPS];
    await act(async () => bus.heard.get("brain")?.());
    await waitFor(() => expect(screen.getByText("Viajes")).toBeTruthy());
    await act(async () => bus.held?.([{ ...TRIPS, id: "slow", title: "Viejo" }]));

    expect(screen.queryByText("Viejo")).toBeNull();
    expect(screen.getByText("Viajes")).toBeTruthy();
  });

  it("leaves the panel as drawn when a round moved nothing it shows", async () => {
    bus.citing = [TRIPS];
    shown();
    await waitFor(() => expect(screen.getByText("Viajes")).toBeTruthy());
    const drawn = document.querySelector(".is-cited");

    bus.citing = [{ ...TRIPS }];
    await act(async () => bus.heard.get("brain")?.());

    expect(bus.asked).toBe(2);
    expect(document.querySelector(".is-cited")).toBe(drawn);
  });
});
