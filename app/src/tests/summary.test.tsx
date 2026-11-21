import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Ready } from "../core";
import { adopt } from "../locales";
import { useSettingsDesk } from "../settingsDesk";
import { landedOn } from "../settingsRows";
import Settings from "../ui/Keeping";
import SettingsNav from "../ui/SettingsNav";
import type { Chosen } from "../views";

const ipc = vi.hoisted(() => ({
  calls: [] as string[],
  state: {} as Record<string, unknown>,
  standing: {} as Record<string, unknown>,
  alias: null as string | null,
  keys: "Ctrl+Alt+T" as string | null,
  held: new Set<string>(),
  failing: new Set<string>(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    ipc.calls.push(cmd);
    if (ipc.held.has(cmd)) return new Promise(() => {});
    if (ipc.failing.has(cmd)) return Promise.reject(new Error(cmd));
    switch (cmd) {
      case "sync_state":
        return Promise.resolve({ ...ipc.state });
      case "signed":
        return Promise.resolve({
          alias: ipc.alias,
          name: null,
          before: [],
          mine: 0,
          aliasImitates: false,
          nameImitates: false,
        });
      case "settings":
        return Promise.resolve({
          quiet: [],
          holds: "everywhere",
          shares: true,
          onlySharedAbove: 0,
          historyDays: 90,
        });
      case "waking":
        return Promise.resolve({ offered: true, wakes: false, theirs: false });
      case "shortcut":
        return Promise.resolve(ipc.keys);
      case "shortcuts_offered":
        return Promise.resolve(["Ctrl+Alt+T"]);
      case "about":
        return Promise.resolve({
          version: "1.49.0",
          sandbox: null,
          repository: "",
          license: "AGPL-3.0-only",
          store: "",
          candidates: false,
          candidatesApply: true,
          keptByTheStore: false,
          standing: { ...ipc.standing },
        });
      case "identity_shown":
        return Promise.resolve({
          held: true,
          elsewhere: false,
          fingerprint: "60065 50166",
          suffix: null,
          confirmed: true,
          alias: "Mario",
          machines: [],
        });
      case "agent":
        return Promise.resolve({ on: false });
      case "wiring":
      case "assistants":
        return Promise.resolve([]);
      default:
        return Promise.resolve(null);
    }
  },
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: () => Promise.resolve(() => {}),
  emit: () => Promise.resolve(),
}));

const day = 86_400_000;

beforeEach(() => {
  adopt("es");
  ipc.calls = [];
  ipc.held = new Set();
  ipc.failing = new Set();
  ipc.alias = "Mario";
  ipc.keys = "Ctrl+Alt+T";
  ipc.state = {
    asked: true,
    open: 3,
    archived: 1,
    lists: 1,
    attachments: 0,
    weight: 1_000,
    carries: 8_000_000_000,
    mode: "alone",
    cloudOffered: true,
    clouds: [],
    freesSpace: false,
    backedUpAt: new Date(Date.now() - 2 * day).toISOString(),
  };
  ipc.standing = { phrase: "ok", door: null, looked: new Date().toISOString() };
});

function Window({
  start = "summary",
  ready = null,
}: {
  start?: Chosen["tab"];
  ready?: Ready | null;
}) {
  const desk = useSettingsDesk();
  const [chosen, setChosen] = useState<Chosen>({ named: "keeping", tab: start });
  const tab = chosen.tab ?? "summary";
  return (
    <>
      <nav>
        <SettingsNav
          tab={tab}
          onTab={(next) => setChosen({ named: "keeping", tab: next })}
          onBack={() => {}}
          keeping={desk}
        />
      </nav>
      <Settings
        tab={tab}
        onTab={(next) => setChosen({ named: "keeping", tab: next })}
        onPack={() => {}}
        onUnpack={() => {}}
        onChanged={() => {}}
        onDoc={() => {}}
        ready={ready}
        desk={desk}
        onLand={(spot) => setChosen(landedOn(spot))}
        shelf={chosen.shelf}
        row={chosen.row}
        landedAt={chosen.landedAt}
      />
    </>
  );
}

const bar = () => within(screen.getByRole("navigation"));

describe("opening the settings", () => {
  it("lands on the overview and asks the backend nothing part 1 did not", async () => {
    render(<Window />);
    expect(await screen.findByRole("heading", { name: "Resumen" })).toBeTruthy();
    expect(await screen.findByText("Nada pide tu atención.")).toBeTruthy();
    expect([...new Set(ipc.calls)].sort()).toEqual([
      "about",
      "reachable",
      "settings",
      "shortcut",
      "signed",
      "sync_state",
      "waking",
    ]);
  });

  it("never says nothing asks for attention before what it reads has arrived", async () => {
    ipc.held = new Set(["about"]);
    render(<Window />);
    expect(await screen.findByRole("heading", { name: "Resumen" })).toBeTruthy();
    await act(() => new Promise((done) => setTimeout(done, 30)));
    expect(screen.queryByText("Nada pide tu atención.")).toBeNull();
    expect(screen.queryByText(/^Por atender/)).toBeNull();
    expect(screen.getByRole("button", { name: /^Seguridad/ }).getAttribute("aria-busy")).toBe(
      "true",
    );
  });
});

describe("the overview", () => {
  it("still lists what it knows when a read fails, without claiming nothing asks", async () => {
    ipc.failing = new Set(["shortcut"]);
    render(<Window ready={{ version: "1.50.0", route: "download", installs: true }} />);
    expect(await screen.findByText("Por atender · 1")).toBeTruthy();
    expect(screen.queryByText(/sin atajo/)).toBeNull();
  });

  it("says nothing asks only once every read answered", async () => {
    ipc.failing = new Set(["about"]);
    render(<Window />);
    expect(await screen.findByRole("heading", { name: "Resumen" })).toBeTruthy();
    await act(() => new Promise((done) => setTimeout(done, 30)));
    expect(screen.queryByText("Nada pide tu atención.")).toBeNull();
  });

  it("walks a fresh install through its first steps", async () => {
    ipc.alias = null;
    ipc.state = { ...ipc.state, asked: false, backedUpAt: undefined };
    ipc.standing = { phrase: "none", door: null, looked: null };
    render(<Window />);
    expect(await screen.findByText("Deja Tisty a tu medida · 0 de 5")).toBeTruthy();
    for (const step of [
      "Elige dónde van tus copias",
      "Crea tu frase de 24 palabras",
      "Haz tu primer respaldo",
      "Ponle tu alias a lo que escribes",
      "Conecta un asistente",
    ])
      expect(screen.getByText(step)).toBeTruthy();
    expect(screen.queryByText("Nada pide tu atención.")).toBeNull();
  });

  it("lists what needs the person, numbers each page in the bar, and goes there", async () => {
    ipc.keys = null;
    ipc.standing = { phrase: "loose", door: null, looked: null };
    render(<Window ready={{ version: "1.50.0", route: "download", installs: true }} />);

    expect(await screen.findByText("Por atender · 3")).toBeTruthy();
    expect(screen.getByText("Tisty 1.50.0 ya está disponible")).toBeTruthy();
    expect(await bar().findByRole("tab", { name: "Seguridad · 1" })).toBeTruthy();
    expect(bar().getByRole("tab", { name: "Tareas y avisos · 1" })).toBeTruthy();
    expect(bar().getByRole("tab", { name: "Ayuda y acerca de · 1" })).toBeTruthy();
    expect(screen.getByText("Hay una 1.50.0")).toBeTruthy();

    await userEvent.click(screen.getByRole("button", { name: /se quedó sin atajo/ }));

    expect(await screen.findByRole("heading", { name: "Tareas y avisos" })).toBeTruthy();
    await waitFor(() =>
      expect(document.querySelector('[data-setting="quick"]')?.hasAttribute("data-landed")).toBe(
        true,
      ),
    );
  });

  it("reads the phrase and the door again on coming back from another page", async () => {
    ipc.standing = { phrase: "none", door: null, looked: null };
    render(<Window />);
    expect(await screen.findByText("Sin frase todavía")).toBeTruthy();
    await userEvent.click(bar().getByRole("tab", { name: "Seguridad" }));
    await screen.findByRole("heading", { name: "Seguridad" });
    ipc.standing = { phrase: "loose", door: null, looked: null };

    await userEvent.click(bar().getByRole("tab", { name: "Resumen" }));

    expect(await screen.findByText("Tu frase todavía no está confirmada")).toBeTruthy();
    expect(ipc.calls.filter((one) => one === "about")).toHaveLength(3);
  });

  it("drops the bar's number once the phrase is confirmed on its own page", async () => {
    ipc.standing = { phrase: "loose", door: null, looked: null };
    render(<Window />);
    expect(await bar().findByRole("tab", { name: "Seguridad · 1" })).toBeTruthy();
    ipc.standing = { phrase: "ok", door: null, looked: null };

    await userEvent.click(bar().getByRole("tab", { name: "Seguridad · 1" }));

    expect(await bar().findByRole("tab", { name: "Seguridad" })).toBeTruthy();
  });

  it("opens a tile's page", async () => {
    render(<Window />);
    await userEvent.click(await screen.findByRole("button", { name: /^Copias/ }));
    expect(await screen.findByRole("heading", { name: "Copias" })).toBeTruthy();
  });
});

describe("searching a setting", () => {
  it("shows what matches, goes to a row inside an assistants tab, and clears the box", async () => {
    render(<Window />);
    await screen.findByRole("heading", { name: "Resumen" });
    const box = bar().getByRole("searchbox", { name: "Buscar un ajuste" });

    await userEvent.type(box, "asistentes en este");

    expect(await screen.findByRole("heading", { name: "«asistentes en este»" })).toBeTruthy();
    expect(screen.getByText("1 ajuste")).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: /Asistentes en este equipo/ }));

    expect(await screen.findByRole("tab", { name: "Conectados", selected: true })).toBeTruthy();
    expect((box as HTMLInputElement).value).toBe("");
    await waitFor(() =>
      expect(document.querySelector('[data-setting="wired"]')?.hasAttribute("data-landed")).toBe(
        true,
      ),
    );
  });

  it("says when nothing matches, and Esc clears the box first", async () => {
    render(<Window />);
    await screen.findByRole("heading", { name: "Resumen" });
    const box = bar().getByRole("searchbox", { name: "Buscar un ajuste" });

    await userEvent.type(box, "zzzz");

    expect(
      await screen.findByText("Nada con esas palabras. Prueba con «respaldo», «atajo» o «idioma»."),
    ).toBeTruthy();
    expect(screen.getByText("0 ajustes")).toBeTruthy();
    expect(bar().queryByRole("tab", { selected: true })).toBeNull();

    await userEvent.keyboard("{Escape}");

    expect((box as HTMLInputElement).value).toBe("");
    expect(await screen.findByRole("heading", { name: "Resumen" })).toBeTruthy();
  });
});
