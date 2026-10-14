import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { cloudStanding } from "../cloudState";
import { choose, ended } from "../coming";
import type { CloudStatus } from "../core";
import { adopt, fill, t } from "../locales";
import Apart from "../ui/Apart";
import Coming from "../ui/Coming";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

afterEach(() => {
  act(() => {
    ended(null);
    choose(false);
  });
  adopt("en");
});

const OTHER = "01M3QX8K2Z5V7N9B4C6D8F0G1H";

const status = (code: string): CloudStatus => ({
  provider: "google",
  account: "0123",
  email: "ana@example.com",
  joined: true,
  unitsSpent: 0,
  unitsADay: 20000,
  bytesSpent: 0,
  bytesADay: 0,
  pausedHere: false,
  haltedUntil: null,
  owed: 0,
  last: { at: Math.floor(Date.now() / 1000), code, name: OTHER, left: 0, names: [] },
});

describe("the doors name the place they stand in", () => {
  it("say Google Drive when the other history is in the cloud", () => {
    render(
      <Apart
        kin="strangers"
        cloud={t("cloudGoogle")}
        onPick={vi.fn()}
        onElse={vi.fn()}
        onClose={vi.fn()}
      />,
    );

    expect(screen.getByText("Your Google Drive already holds another Tisty")).toBeTruthy();
    expect(screen.getByText("Take what Google Drive has")).toBeTruthy();
    expect(screen.queryByText(/that folder/i)).toBeNull();
  });

  it("say it in Spanish too, with tú", () => {
    adopt("es");
    render(
      <Apart
        kin="strangers"
        cloud={t("cloudGoogle")}
        onPick={vi.fn()}
        onElse={vi.fn()}
        onClose={vi.fn()}
      />,
    );

    expect(screen.getByText("Tu Google Drive ya guarda otro Tisty")).toBeTruthy();
    expect(screen.getByText("Quedarme con lo de Google Drive")).toBeTruthy();
  });

  it("keep saying folder for a folder", () => {
    adopt("es");
    render(<Apart kin="strangers" onPick={vi.fn()} onElse={vi.fn()} onClose={vi.fn()} />);

    expect(screen.getByText("Esa carpeta ya guarda otro Tisty")).toBeTruthy();
    expect(screen.getByText("Quedarme con lo de la carpeta")).toBeTruthy();
  });

  it("name the cloud even when they only ask to take its name", async () => {
    const pick = vi.fn();
    render(
      <Apart
        kin="sameLineage"
        cloud={t("cloudGoogle")}
        onPick={pick}
        onElse={vi.fn()}
        onClose={vi.fn()}
      />,
    );

    await userEvent.click(
      screen.getByRole("button", { name: fill("apartHomeDoCloud", t("cloudGoogle")) }),
    );
    expect(pick).toHaveBeenCalledWith("merge");
  });
});

describe("the sidebar while a place holds another history", () => {
  it("never shows the other store's id", () => {
    render(<Coming />);
    act(() => ended({ code: "otherStore", name: OTHER }));

    expect(screen.getByRole("alert").textContent).toBe(t("stuckChoosing"));
    expect(document.body.textContent).not.toContain(OTHER.slice(0, 4));
  });

  it("says the same plain thing for either refusal", () => {
    render(<Coming />);
    act(() => ended({ code: "wouldReset", name: OTHER }));

    expect(screen.getByRole("alert").textContent).toBe(t("stuckChoosing"));
  });

  it("names a copy gone back on as itself, not as another Tisty", () => {
    render(<Coming />);
    act(() => ended({ code: "restoredApart", name: "Google Drive" }));

    expect(screen.getByRole("alert").textContent).toBe(fill("restoredApart", "Google Drive"));
  });

  it("still names any other trouble as it always did", () => {
    render(<Coming />);
    act(() => ended({ code: "syncFailed" }));

    expect(screen.getByRole("alert").textContent).not.toBe(t("stuckChoosing"));
  });
});

describe("the cloud card while a place holds another history", () => {
  it("waits on the person and offers to choose, naming Google Drive and no id", () => {
    for (const code of ["wouldReset", "otherStore"]) {
      const standing = cloudStanding(status(code));

      expect(standing?.said).toBe("stateApart");
      expect(standing?.exit).toBe("choose");
      expect(standing?.name).toBe(t("cloudGoogle"));
    }
  });

  it("says nothing of a choice when this machine takes the other history on its own", () => {
    expect(cloudStanding({ ...status("otherStore"), adopts: true })).toBeNull();
  });

  it("tells a copy gone back on in its own words, still offering to choose", () => {
    const standing = cloudStanding(status("restoredApart"));

    expect(standing?.said).toBe("stateRestoredApart");
    expect(standing?.told).toBe(fill("stateRestoredApartWhy", t("cloudGoogle")));
    expect(standing?.exit).toBe("choose");
  });

  it("leaves every other ending as it was", () => {
    expect(cloudStanding(status("cloudFull"))?.said).toBe("stateFull");
    expect(cloudStanding(status("ok"))).toBeNull();
  });
});
