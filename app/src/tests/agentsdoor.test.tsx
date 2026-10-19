import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import Agents from "../ui/Agents";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve(null),
}));

const known = { agent: null, setAgent: () => {}, agents: [], setAgents: () => {}, hands: [] };

const shown = (binary?: string) =>
  render(
    <Agents
      busy={null}
      quietly={() => {}}
      tell={() => {}}
      fail={() => {}}
      binary={binary}
      known={known}
    />,
  );

describe("how to connect an assistant", () => {
  it("says the snippets carry the path when there is one beside the window", () => {
    shown("C:/Programs/Tisty/tisty.exe");

    expect(screen.getByText(t("agentsHow"))).toBeTruthy();
  });

  it("says they call the name alone when the Store's copy has no path to give", () => {
    shown();

    expect(screen.getByText(t("agentsHowAlias"))).toBeTruthy();
    expect(screen.queryByText(t("agentsHow"))).toBeNull();
  });
});
