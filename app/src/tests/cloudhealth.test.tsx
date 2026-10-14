import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { adopt, fill, t } from "../locales";
import Clouding from "../ui/Clouding";

const ipc = vi.hoisted(() => ({
  calls: [] as string[],
  health: [] as string[],
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    ipc.calls.push(cmd);
    if (cmd === "cloud_health") return Promise.resolve(ipc.health.shift() ?? "reachable");
    if (cmd === "cloud_connect") return new Promise(() => {});
    return Promise.resolve(null);
  },
}));

vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: () => Promise.resolve() }));

const GOOGLE = "Google Drive";

const opened = () =>
  render(<Clouding busy={false} onTrouble={vi.fn()} onBack={vi.fn()} onDone={vi.fn()} />);

beforeEach(() => {
  ipc.calls = [];
  ipc.health = [];
  adopt("en");
});

describe("the way to Google Drive, looked at before connecting", () => {
  it("is asked once on opening, and says nothing when Google answers", async () => {
    opened();

    await waitFor(() => expect(ipc.calls.filter((one) => one === "cloud_health")).toHaveLength(1));
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("says in plain words that something blocks it, and lets the person look again", async () => {
    ipc.health = ["blocked", "reachable"];
    opened();

    const said = await screen.findByRole("alert");
    expect(said.textContent).toContain(fill("healthBlocked", GOOGLE));

    await userEvent.click(screen.getByRole("button", { name: t("healthAgain") }));
    await waitFor(() => expect(screen.queryByRole("alert")).toBeNull());
    expect(ipc.calls.filter((one) => one === "cloud_health")).toHaveLength(2);
  });

  it("tells a filter that reads secure connections from a blocked way", async () => {
    ipc.health = ["inspected"];
    opened();

    expect((await screen.findByRole("alert")).textContent).toContain(
      fill("healthInspected", GOOGLE),
    );
  });

  it("tells Google having trouble from the person's network", async () => {
    ipc.health = ["troubled"];
    opened();

    expect((await screen.findByRole("alert")).textContent).toContain(
      fill("healthTroubled", GOOGLE),
    );
  });

  it("never stands in the way of connecting", async () => {
    ipc.health = ["blocked"];
    opened();
    await screen.findByRole("alert");

    await userEvent.click(screen.getByRole("button", { name: t("cloudOpenGoogle") }));

    expect(ipc.calls).toContain("cloud_connect");
  });

  it("speaks Spanish with tú", async () => {
    adopt("es");
    ipc.health = ["blocked"];
    opened();

    expect((await screen.findByRole("alert")).textContent).toContain(
      "Algo está bloqueando la conexión con Google Drive",
    );
  });
});
