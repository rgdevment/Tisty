import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { adopt, fill, t } from "../locales";
import Clouding from "../ui/Clouding";

const ipc = vi.hoisted(() => ({
  calls: [] as string[],
  health: [] as string[],
  signed: null as { account: string; email: string } | null,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    ipc.calls.push(cmd);
    if (cmd === "cloud_health") return Promise.resolve(ipc.health.shift() ?? "reachable");
    if (cmd === "cloud_connect")
      return ipc.signed ? Promise.resolve(ipc.signed) : new Promise(() => {});
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
  ipc.signed = null;
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

  it("tells a slow answer gently, as something to try past rather than a wall", async () => {
    ipc.health = ["slow"];
    opened();

    const said = await screen.findByRole("status");
    expect(said.textContent).toContain(fill("healthSlow", GOOGLE));
    expect(screen.queryByText(fill("healthBlocked", GOOGLE))).toBeNull();
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("is not asked again once the sign-in went through", async () => {
    ipc.signed = { account: "0123", email: "ana@example.com" };
    opened();
    await waitFor(() => expect(ipc.calls.filter((one) => one === "cloud_health")).toHaveLength(1));

    await userEvent.click(screen.getByRole("button", { name: t("cloudOpenGoogle") }));

    expect(await screen.findByText(t("cloudConnectedTitle"))).toBeTruthy();
    expect(ipc.calls.filter((one) => one === "cloud_health")).toHaveLength(1);
    expect(screen.queryByText(fill("healthChecking", GOOGLE))).toBeNull();
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

  it("says a slow answer in Spanish with tú too", async () => {
    adopt("es");
    ipc.health = ["slow"];
    opened();

    expect((await screen.findByRole("status")).textContent).toContain(
      "Google Drive tardó en responder; puede ser una red lenta o un filtro. Puedes intentar conectarte igual.",
    );
  });
});
