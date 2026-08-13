import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { Ready } from "../core";
import About from "../ui/About";
import Sidebar from "../ui/Sidebar";

vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: () => Promise.resolve() }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () =>
    Promise.resolve({
      version: "0.2.0",
      sandbox: null,
      repository: "https://example.invalid/tisty",
      license: "AGPL-3.0",
      store: "C:/store",
    }),
}));

const chosen = { named: "aboutScreen" } as Parameters<typeof Sidebar>[0]["chosen"];

function bar(ready: boolean) {
  render(
      <Sidebar
        lists={[]}
        papers={{ folders: [], docs: [] }}
        counts={{}}
        chosen={chosen}
        ready={ready}
        onChoose={vi.fn()}
        onFile={vi.fn()}
        onHere={vi.fn()}
        onMove={vi.fn()}
        onFolderMenu={vi.fn()}
        onDocMenu={vi.fn()}
        onDocsMenu={vi.fn()}
      />,
    );
}

describe("the update dot", () => {
  it("names itself for a reader that cannot see it", () => {
    bar(true);

    expect(screen.getByLabelText(/a newer version exists/)).toBeTruthy();
  });

  it("says nothing at all when this is the newest one", () => {
    bar(false);

    expect(screen.queryByLabelText(/a newer version exists/)).toBeNull();
    expect(screen.getByLabelText("About")).toBeTruthy();
  });
});

describe("what About suggests", () => {
  const ready = (route: Ready["route"], named: string | null = null): Ready => ({
    version: "0.3.0",
    route,
    url: "https://example.invalid/releases",
    package: named,
  });

  /// Nothing to ask of somebody whose store does it for them.
  it("asks nothing of a Store install", async () => {
    render(<About ready={ready("store")} onError={vi.fn()} />);

    expect(await screen.findByText(/Tisty 0.3.0 is out/)).toBeTruthy();
    expect(screen.getByText(/Microsoft Store installs it for you/)).toBeTruthy();
    expect(screen.queryByText(/Open the releases page/)).toBeNull();
  });

  it("gives the command to a Homebrew install", async () => {
    render(<About ready={ready("brew", "tisty")} onError={vi.fn()} />);

    expect(await screen.findByText(/brew upgrade --cask tisty/)).toBeTruthy();
  });

  /// A candidate installs under its own name, and the stable command upgrades
  /// nothing there.
  it("names the package a candidate was installed under", async () => {
    render(<About ready={ready("brew", "tisty-beta")} onError={vi.fn()} />);

    expect(await screen.findByText(/brew upgrade --cask tisty-beta/)).toBeTruthy();
  });

  it("gives the formula its own command, without the cask flag", async () => {
    render(<About ready={ready("brewCli", "tisty-cli")} onError={vi.fn()} />);

    const said = await screen.findByText(/brew upgrade tisty-cli/);
    expect(said.textContent).not.toContain("--cask");
  });

  it("offers the page to everyone else", async () => {
    render(<About ready={ready("download")} onError={vi.fn()} />);

    expect(await screen.findByText(/Open the releases page/)).toBeTruthy();
  });

  it("says nothing when this copy is the newest", async () => {
    render(<About ready={null} onError={vi.fn()} />);

    expect(await screen.findByText("AGPL-3.0")).toBeTruthy();
    expect(screen.queryByText(/is out/)).toBeNull();
  });
});
