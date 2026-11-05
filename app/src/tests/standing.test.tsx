import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import Standing from "../Standing";

const store = vi.hoisted(() => ({ broke: [] as unknown[][], label: "main" }));

vi.mock("../broke", () => ({
  broke: (...said: unknown[]) => store.broke.push(said),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ label: store.label }),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(false) }));

const WINDOWS: Record<string, string> = {
  App: "../App",
  Moving: "../Moving",
  Quick: "../Quick",
  Viewing: "../Viewing",
};

const loadedWith = (broken?: string) => {
  const loaded: string[] = [];
  for (const [name, path] of Object.entries(WINDOWS))
    vi.doMock(path, () => {
      if (name === broken) throw new Error(`${name} cannot be loaded`);
      loaded.push(name);
      return { default: () => name };
    });
  vi.doMock("../finishing", () => {
    loaded.push("finishing");
    return { finishWhatIsAsked: () => Promise.resolve() };
  });
  return loaded;
};

const started = async (label: string) => {
  store.label = label;
  document.body.innerHTML = '<div id="root"></div>';
  await import("../main");
};

describe("the sheet comes when it is wanted", () => {
  const sheetIn = async (wanted: boolean) => {
    const { useSheet } = await import("../ui/useSheet");
    render(
      <Standing>
        <Showing useIt={useSheet} wanted={wanted} />
      </Standing>,
    );
  };

  beforeEach(() => {
    vi.resetModules();
    vi.spyOn(console, "error").mockImplementation(() => {});
  });

  afterEach(() => {
    vi.doUnmock("../ui/Sheet");
  });

  it("stays away while nothing asks for it", async () => {
    const loaded: string[] = [];
    vi.doMock("../ui/Sheet", () => {
      loaded.push("Sheet");
      return { default: () => "the sheet" };
    });

    await sheetIn(false);
    await new Promise((done) => setTimeout(done, 0));

    expect(screen.getByText("no sheet yet")).toBeTruthy();
    expect(loaded).toEqual([]);
  });

  it("is drawn once it has arrived", async () => {
    vi.doMock("../ui/Sheet", () => ({ default: () => "the sheet" }));

    await sheetIn(true);

    expect(await screen.findByText("the sheet")).toBeTruthy();
  });

  it("takes the window down when it cannot be loaded, as anything drawn in it would", async () => {
    vi.doMock("../ui/Sheet", () => {
      throw new Error("the sheet cannot be loaded");
    });

    await sheetIn(true);

    expect(await screen.findByText(t("windowFell"))).toBeTruthy();
  });
});

describe("a window loads its own code and no other's", () => {
  beforeEach(() => {
    vi.resetModules();
    vi.spyOn(console, "error").mockImplementation(() => {});
  });

  afterEach(() => {
    document.body.innerHTML = "";
  });

  it.each([
    ["main", "App"],
    ["quick", "Quick"],
    ["moving", "Moving"],
    ["view-3", "Viewing"],
  ])("%s draws %s and loads nothing else", async (label, name) => {
    const loaded = loadedWith();

    await started(label);

    expect(await screen.findByText(name)).toBeTruthy();
    expect(loaded).toEqual([name]);
  });

  it("a window that only finishes a document loads no window at all", async () => {
    const loaded = loadedWith();

    await started("finisher-1");

    await waitFor(() => expect(loaded).toEqual(["finishing"]));
    expect(document.getElementById("root")?.childElementCount).toBe(0);
  });

  it("a window whose code cannot be loaded says so and offers to draw it again", async () => {
    loadedWith("Quick");

    await started("quick");

    expect(await screen.findByText(t("windowFell"))).toBeTruthy();
    expect(screen.getByRole("button", { name: t("windowFellAgain") })).toBeTruthy();
  });
});

const Showing = ({ useIt, wanted }: { useIt: (wanted: boolean) => unknown; wanted: boolean }) => {
  const Sheet = useIt(wanted) as (() => string) | null;
  return Sheet ? <Sheet /> : "no sheet yet";
};

const Falls = () => {
  throw new TypeError("Cannot read properties of undefined (reading 'replace')");
};

describe("a window that stops drawing", () => {
  beforeEach(() => {
    store.broke = [];
    vi.spyOn(console, "error").mockImplementation(() => {});
  });

  it("says so and offers to draw it again, instead of going black", () => {
    render(
      <Standing>
        <Falls />
      </Standing>,
    );

    expect(screen.getByText(t("windowFell"))).toBeTruthy();
    expect(screen.getByRole("button", { name: t("windowFellAgain") })).toBeTruthy();
  });

  it("writes down what broke, with what it said", () => {
    render(
      <Standing>
        <Falls />
      </Standing>,
    );

    expect(store.broke).toHaveLength(1);
    expect(store.broke[0][0]).toBe("TypeError");
    expect(store.broke[0][1]).toContain("reading 'replace'");
  });

  it("draws the window again when asked", async () => {
    const reload = vi.fn();
    Object.defineProperty(window, "location", {
      value: { ...window.location, reload },
      writable: true,
    });

    render(
      <Standing>
        <Falls />
      </Standing>,
    );
    await userEvent.click(screen.getByRole("button", { name: t("windowFellAgain") }));

    expect(reload).toHaveBeenCalled();
  });

  it("stays out of the way while nothing is wrong", () => {
    render(
      <Standing>
        <p>lo de siempre</p>
      </Standing>,
    );

    expect(screen.getByText("lo de siempre")).toBeTruthy();
    expect(store.broke).toHaveLength(0);
  });
});
