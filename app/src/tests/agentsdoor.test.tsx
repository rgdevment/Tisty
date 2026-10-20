import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import Agents, { cursorLink, mcpbLink, vscodeLink } from "../ui/Agents";

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

describe("connecting an assistant in one click", () => {
  it("hands VS Code the server it starts, with the path on this machine", () => {
    const link = vscodeLink("C:/Programs/Tisty/tisty.exe");
    const [scheme, said] = link.split("?");

    expect(scheme).toBe("vscode:mcp/install");
    expect(JSON.parse(decodeURIComponent(said))).toEqual({
      name: "tisty",
      type: "stdio",
      command: "C:/Programs/Tisty/tisty.exe",
      args: ["mcp"],
    });
  });

  it("hands Cursor the same server, even from a folder with accents in its name", () => {
    const link = new URL(cursorLink("C:/Users/Mário/Tisty/tisty.exe"));
    const config = link.searchParams.get("config") ?? "";
    const bytes = Uint8Array.from(atob(config), (one) => one.charCodeAt(0));

    expect(link.searchParams.get("name")).toBe("tisty");
    expect(JSON.parse(new TextDecoder().decode(bytes))).toEqual({
      command: "C:/Users/Mário/Tisty/tisty.exe",
      args: ["mcp"],
    });
  });

  it("calls the name alone when the Store's copy has no path to give", () => {
    expect(JSON.parse(decodeURIComponent(vscodeLink().split("?")[1])).command).toBe("tisty");
  });

  it("downloads the bundle of this very version, and the last stable one from a candidate", () => {
    const repository = "https://github.com/rgdevment/Tisty";

    expect(mcpbLink({ repository, version: "1.31.0" })).toBe(
      `${repository}/releases/download/v1.31.0/tisty-mcp-1.31.0.mcpb`,
    );
    expect(mcpbLink({ repository, version: "1.31.0-rc.1" })).toBe(`${repository}/releases/latest`);
  });
});

describe("what the settings say about the door", () => {
  const withAgent = (on: boolean, byDefault: boolean) =>
    render(
      <Agents
        busy={null}
        quietly={() => {}}
        tell={() => {}}
        fail={() => {}}
        known={{ ...known, agent: { on, byDefault, called: "espino 3", code: "12 34" } }}
      />,
    );

  it("only says it refuses everything while the door is closed", () => {
    withAgent(false, false);
    expect(screen.getByText(t("agentsHowShut"))).toBeTruthy();
  });

  it("says Tisty opened it and how to close it, and no longer that it refuses", () => {
    withAgent(true, true);

    expect(screen.getByText(t("agentsByDefault"))).toBeTruthy();
    expect(screen.queryByText(t("agentsHowShut"))).toBeNull();
  });

  it("says nothing about a default once the person chose", () => {
    withAgent(true, false);
    expect(screen.queryByText(t("agentsByDefault"))).toBeNull();
  });
});
