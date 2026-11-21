import { beforeEach, describe, expect, it } from "vitest";
import {
  attention,
  counted,
  firstSteps,
  type Known,
  QUIET_DAYS,
  spotOf,
  stepsShown,
  tiles,
  whole,
} from "../attention";
import type { About, Carrying, Settings, Standing } from "../core";
import { adopt } from "../locales";

const NOW = new Date(2026, 10, 21, 10, 0, 0);
const daysAgo = (days: number) => new Date(NOW.getTime() - days * 86_400_000).toISOString();

const carrying = (over: Partial<Carrying> = {}): Carrying => ({
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
  backedUpAt: daysAgo(2),
  ...over,
});

const standing = (over: Partial<Standing> = {}): Standing => ({
  phrase: "ok",
  door: null,
  looked: daysAgo(0),
  ...over,
});

const build = (over: Partial<Standing> = {}): About => ({
  version: "1.49.0",
  sandbox: null,
  repository: "",
  license: "AGPL-3.0-only",
  store: "",
  candidates: false,
  candidatesApply: true,
  keptByTheStore: false,
  standing: standing(over),
});

const kept: Settings = {
  quiet: [],
  holds: "everywhere",
  shares: true,
  onlySharedAbove: 0,
  historyDays: 90,
};

const known = (over: Partial<Known> = {}): Known => ({
  state: carrying(),
  foot: null,
  build: build(),
  kept,
  keys: "Ctrl+Alt+T",
  alias: "Mario",
  ready: null,
  checked: null,
  wired: null,
  ...over,
});

beforeEach(() => adopt("es"));

describe("what asks for the person's attention", () => {
  it("is not said at all until what the page reads on opening has arrived", () => {
    expect(attention(known({ state: null }), NOW)).toBeNull();
    expect(attention(known({ waiting: true }), NOW)).toBeNull();
  });

  it("leaves out only what a failed read would have said, and never claims the whole", () => {
    const items = attention(
      known({
        kept: null,
        keys: undefined,
        build: null,
        state: carrying({ backedUpAt: daysAgo(47) }),
      }),
      NOW,
    );
    expect(items?.map((one) => one.key)).toEqual(["backup"]);
    expect(whole(known({ keys: undefined }))).toBe(false);
    expect(whole(known({ build: null }))).toBe(false);
    expect(whole(known())).toBe(true);
  });

  it("is an empty list, not a missing one, when everything is in order", () => {
    expect(attention(known(), NOW)).toEqual([]);
  });

  it("lists each real source with the page, row and words that fix it", () => {
    const items = attention(
      known({
        ready: { version: "1.50.0", route: "download", installs: true },
        state: carrying({ backedUpAt: daysAgo(47) }),
        build: build({ phrase: "loose" }),
        keys: null,
        checked: { many: 2, at: daysAgo(0), waiting: ["Mac mini"] },
        wired: [
          {
            id: "cursor",
            name: "Cursor",
            at: "C:/Users/someone/.cursor/mcp.json",
            wired: true,
            astray: true,
            maybe: false,
            byHand: false,
            manual: { file: "", snippet: "" },
          },
        ],
      }),
      NOW,
    );
    expect(items?.map((one) => [one.key, one.spot, one.said, one.act])).toEqual([
      ["update", { page: "about", row: "update" }, "Tisty 1.50.0 ya está disponible", "Actualizar"],
      [
        "phrase",
        { page: "security", row: "identity" },
        "Tu frase todavía no está confirmada",
        "Confirmarla",
      ],
      [
        "machine-Mac mini",
        { page: "security", row: "machines" },
        "Mac mini escribe sin que hayas confirmado su clave",
        "Confirmar",
      ],
      [
        "backup",
        { page: "copies", row: "backup" },
        expect.stringMatching(/^Tu último respaldo es de /),
        "Crear respaldo",
      ],
      [
        "health",
        { page: "health", row: "review" },
        "2 cosas por revisar en tu historial",
        "Revisar",
      ],
      [
        "keys",
        { page: "notices", row: "quick" },
        "La captura rápida se quedó sin atajo",
        "Elegir otro",
      ],
      [
        "astray-cursor",
        { page: "agents", shelf: "wired", row: "wired" },
        "Cursor apunta a un Tisty que ya no existe",
        "Corregir",
      ],
    ]);
  });

  it("names a waiting computer from Security's read alone, before Health was checked", () => {
    const items = attention(known({ machines: ["Mac mini"] }), NOW);
    expect(items?.map((one) => one.key)).toEqual(["machine-Mac mini"]);
    expect(tiles(known({ machines: ["Mac mini"] }), { named: "" }, NOW).security?.dot).toBe("warn");
  });

  it("says one thing to look at in the singular", () => {
    const items = attention(known({ checked: { many: 1, at: daysAgo(0), waiting: [] } }), NOW);
    expect(items?.[0]?.said).toBe("1 cosa por revisar en tu historial");
  });

  it("leaves a shortcut turned off on purpose alone", () => {
    expect(attention(known({ keys: null, kept: { ...kept, quickKeys: "off" } }), NOW)).toEqual([]);
  });

  it("names an update the Store has without a number", () => {
    const items = attention(
      known({ ready: { version: "", route: "store", installs: false, coming: "queued" } }),
      NOW,
    );
    expect(items?.[0]?.said).toBe("La Store tiene una versión nueva");
    expect(items?.[0]?.act).toBe("Ver");
  });

  it("raises sync only for the footings that need the person, and days of silence", () => {
    const synced = carrying({ mode: "folder", chosen: "C:/Users/someone/OneDrive/Tisty" });
    for (const [kind, said] of [
      ["lost", "Tu nube pide que vuelvas a iniciar sesión"],
      ["paused", "La sincronización está en pausa en este equipo"],
    ] as const) {
      const items = attention(known({ state: synced, foot: { kind } }), NOW);
      expect(items).toEqual([
        { key: "sync", spot: { page: "sync", row: "syncing" }, said, act: "Ver" },
      ]);
    }
    const looking = attention(known({ state: synced, foot: { kind: "look", putOff: false } }), NOW);
    expect(looking?.[0]?.said).toBe("La sincronización tiene algo que revisar");
    expect(attention(known({ state: synced, foot: { kind: "offline" } }), NOW)).toEqual([]);
    const silent = carrying({ ...synced, heard: daysAgo(QUIET_DAYS) });
    expect(attention(known({ state: silent, foot: { kind: "ok" } }), NOW)?.[0]?.said).toBe(
      "Llevas días sin recibir nada de tus otros equipos",
    );
  });

  it("counts what each page holds for the bar", () => {
    const items = attention(
      known({
        build: build({ phrase: "loose" }),
        checked: { many: 1, at: daysAgo(0), waiting: ["Mac mini"] },
      }),
      NOW,
    );
    expect(counted(items)).toEqual({ security: 2, health: 1 });
    expect(counted(null)).toEqual({});
  });

  it("sends a computer encrypting the cloud to Security, and every other footing to Sync", () => {
    expect(spotOf({ kind: "sealing", turn: "on" })).toEqual({ page: "security", row: "sealing" });
    expect(spotOf({ kind: "lost" })).toEqual({ page: "sync", row: "syncing" });
  });
});

describe("the first steps", () => {
  it("wait for the alias and the phrase before saying anything", () => {
    expect(firstSteps(known({ alias: undefined }))).toBeNull();
    expect(firstSteps(known({ build: null }))).toBeNull();
    expect(firstSteps(known({ state: null }))).toBeNull();
  });

  it("show on a fresh install, with nothing done", () => {
    const steps = firstSteps(
      known({
        state: carrying({ asked: false, backedUpAt: undefined }),
        build: build({ phrase: "none" }),
        alias: null,
      }),
    );
    expect(steps?.map((one) => one.done)).toEqual([false, false, false, false, false]);
    expect(stepsShown(steps)).toBe(true);
  });

  it("go once the four that are not optional are done, staying on this computer included", () => {
    const steps = firstSteps(known());
    expect(steps?.map((one) => one.done)).toEqual([true, true, true, true, false]);
    expect(stepsShown(steps)).toBe(false);
  });

  it("tick the optional assistant only once one is known to be connected, not for an open door", () => {
    const door = known({ build: build({ door: "espino 3" }) });
    expect(firstSteps(door)?.[4]?.done).toBe(false);
    const wired = known({
      wired: [
        {
          id: "claude-code",
          name: "Claude Code",
          at: "",
          wired: true,
          astray: false,
          maybe: false,
          byHand: false,
          manual: { file: "", snippet: "" },
        },
      ],
    });
    expect(firstSteps(wired)?.[4]?.done).toBe(true);
  });

  it("count the way of syncing as chosen only once the person was asked", () => {
    expect(firstSteps(known({ state: carrying({ asked: false }) }))?.[0]?.done).toBe(false);
    expect(firstSteps(known({ state: carrying({ asked: true }) }))?.[0]?.done).toBe(true);
  });
});

describe("the status tiles", () => {
  const sync = { named: "OneDrive", line: "Al día · 10:00" };

  it("stay neutral while their source has not arrived", () => {
    const shown = tiles(known({ state: null, build: null }), sync, NOW);
    expect(shown.sync).toBeNull();
    expect(shown.copies).toBeNull();
    expect(shown.security).toBeNull();
    expect(shown.agents).toBeNull();
    expect(shown.about).toBeNull();
  });

  it("say Health was not checked rather than guess", () => {
    expect(tiles(known(), sync, NOW).health).toEqual({
      page: "health",
      dot: "off",
      head: "Sin revisar todavía",
      detail: "se revisa al abrir Salud",
    });
  });

  it("read each page's state", () => {
    const shown = tiles(
      known({
        state: carrying({ mode: "folder", chosen: "C:/OneDrive/Tisty", backedUpAt: daysAgo(47) }),
        foot: { kind: "ok" },
        build: build({ phrase: "loose", door: "espino 3" }),
        checked: { many: 0, at: daysAgo(0), waiting: [] },
        ready: { version: "1.50.0", route: "download", installs: true },
      }),
      sync,
      NOW,
    );
    expect(shown.sync).toMatchObject({ dot: "ok", head: "En OneDrive", detail: "Al día · 10:00" });
    expect(shown.copies).toMatchObject({ dot: "warn", head: "Último respaldo" });
    expect(shown.security).toMatchObject({ dot: "warn", head: "Frase sin confirmar" });
    expect(shown.agents).toMatchObject({
      dot: "ok",
      head: "Puerta abierta",
      detail: "firma como «espino 3»",
    });
    expect(shown.health).toMatchObject({ dot: "ok", head: "Todo en orden" });
    expect(shown.about).toMatchObject({
      dot: "warn",
      head: "Hay una 1.50.0",
      detail: "tienes la 1.49.0",
    });
  });

  it("read a store that stays here, with nothing set up", () => {
    const shown = tiles(
      known({
        state: carrying({ backedUpAt: undefined }),
        build: build({ phrase: "none", looked: null }),
        checked: { many: 3, at: daysAgo(0), missed: "x", waiting: [] },
      }),
      sync,
      NOW,
    );
    expect(shown.sync).toMatchObject({
      dot: "off",
      head: "Desactivada",
      detail: "solo en este equipo",
    });
    expect(shown.copies).toMatchObject({
      dot: "off",
      head: "Sin respaldo",
      detail: "ninguna todavía",
    });
    expect(shown.security).toMatchObject({ dot: "off", head: "Sin frase todavía" });
    expect(shown.agents).toMatchObject({ dot: "off", head: "Puerta cerrada" });
    expect(shown.health).toMatchObject({ dot: "warn", head: "3 cosas por revisar" });
    expect(shown.about).toMatchObject({
      dot: "off",
      head: "Tisty 1.49.0",
      detail: "aún no busca versiones nuevas",
    });
  });

  it("say up to date only for an answer from today, by the calendar day here", () => {
    const at = (day: number, hour: number, minute: number) =>
      new Date(2026, 10, day, hour, minute).toISOString();
    const about = (looked: string, now: Date) =>
      tiles(known({ build: build({ looked }) }), sync, now).about;
    expect(about(at(20, 23, 59), new Date(2026, 10, 21, 0, 1))).toMatchObject({
      dot: "off",
      detail: expect.stringMatching(/^buscó por última vez /),
    });
    expect(about(at(21, 0, 1), new Date(2026, 10, 21, 23, 59))).toMatchObject({
      dot: "ok",
      detail: "estás al día",
    });
    expect(about(at(21, 11, 0), NOW)).toMatchObject({ dot: "ok", detail: "estás al día" });
    expect(about(at(22, 9, 0), NOW)).toMatchObject({
      dot: "off",
      detail: expect.stringMatching(/^buscó por última vez /),
    });
    expect(about(daysAgo(30), NOW)?.dot).toBe("off");
  });

  it("color sync green only for a round that ended well, and never guess an unread cloud", () => {
    const folder = carrying({ mode: "folder", chosen: "C:/OneDrive/Tisty" });
    const dot = (over: Partial<Known>) =>
      tiles(known({ state: folder, ...over }), sync, NOW).sync?.dot;
    expect(dot({ foot: { kind: "ok" } })).toBe("ok");
    expect(dot({ foot: { kind: "syncing", done: 1, whole: 3 } })).toBe("ok");
    expect(dot({ foot: { kind: "offline" } })).toBe("off");
    expect(dot({ foot: { kind: "lost" } })).toBe("warn");
    expect(dot({ foot: null })).toBe("off");
    const cloud = carrying({ mode: "cloud", cloudProvider: "google" });
    expect(
      tiles(known({ state: cloud, foot: { kind: "ok" }, status: null }), sync, NOW).sync,
    ).toMatchObject({
      dot: "off",
      detail: "todavía no se pudo leer el estado de la nube",
    });
  });

  it("say a way of syncing a newer Tisty chose is not off", () => {
    expect(tiles(known({ state: carrying({ mode: "later" }) }), sync, NOW).sync).toEqual({
      page: "sync",
      dot: "warn",
      head: "La eligió un Tisty más nuevo",
      detail: "actualiza este para verla",
    });
  });

  it("warn on sync left quiet, a phrase elsewhere, a partial check and a stray assistant", () => {
    const shown = tiles(
      known({
        state: carrying({ mode: "folder", chosen: "C:/OneDrive/Tisty", heard: daysAgo(9) }),
        build: build({ phrase: "elsewhere", door: "espino 3" }),
        checked: { many: 0, at: daysAgo(0), missed: "x", waiting: [] },
        wired: [
          {
            id: "cursor",
            name: "Cursor",
            at: "",
            wired: true,
            astray: true,
            maybe: false,
            byHand: false,
            manual: { file: "", snippet: "" },
          },
        ],
      }),
      sync,
      NOW,
    );
    expect(shown.sync).toMatchObject({
      dot: "warn",
      detail: expect.stringMatching(/^sin noticias desde /),
    });
    expect(shown.security).toMatchObject({ dot: "warn", head: "Frase en otro equipo" });
    expect(shown.health).toMatchObject({
      dot: "warn",
      head: "Algunas revisiones no se pudieron hacer",
    });
    expect(shown.agents).toMatchObject({ dot: "warn" });
    expect(shown.about).toMatchObject({ dot: "ok", detail: "estás al día" });
    const old = tiles(known({ build: build({ looked: daysAgo(30) }) }), sync, NOW);
    expect(old.about?.detail).toMatch(/^buscó por última vez /);
  });
});
