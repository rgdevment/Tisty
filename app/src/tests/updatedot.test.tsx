import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

const asked: string[] = [];
const refuse: { next: unknown } = { next: null };

import type { Ready } from "../core";
import About from "../ui/About";
import Sidebar from "../ui/Sidebar";

vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: () => Promise.resolve() }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    asked.push(cmd);
    if (refuse.next && cmd === "update_install") return Promise.reject(refuse.next);
    return Promise.resolve({
      version: "0.2.0",
      sandbox: null,
      repository: "https://example.invalid/tisty",
      license: "AGPL-3.0",
      store: "C:/store",
    });
  },
}));

const chosen = { named: "aboutScreen" } as Parameters<typeof Sidebar>[0]["chosen"];

function bar(waiting?: string) {
  render(
    <Sidebar
      lists={[]}
      papers={{ folders: [], docs: [] }}
      counts={{}}
      chosen={chosen}
      waiting={waiting}
      onChoose={vi.fn()}
      onFile={vi.fn()}
      onHere={vi.fn()}
      onMove={vi.fn()}
      onFolderMenu={vi.fn()}
      onDocMenu={vi.fn()}
      onDocsMenu={vi.fn()}
      onHereMenu={vi.fn()}
    />,
  );
}

describe("the update dot", () => {
  it("names itself for a reader that cannot see it", () => {
    bar("0.2.0");

    expect(screen.getByLabelText(/a newer version exists/)).toBeTruthy();
  });

  it("says nothing at all when this is the newest one", () => {
    bar();

    expect(screen.queryByLabelText(/a newer version exists/)).toBeNull();
    expect(screen.getByLabelText("About")).toBeTruthy();
  });
});

beforeEach(() => {
  asked.length = 0;
  refuse.next = null;
});

describe("what About suggests", () => {
  const ready = (route: Ready["route"], named: string | null = null): Ready => ({
    version: "0.3.0",
    route,
    package: named,
    installs: route === "download" || route === "brew",
  });

  it("tells a Store install it is coming, and how to have it sooner", async () => {
    render(<About ready={ready("store")} onError={vi.fn()} />);

    expect(await screen.findByText(/Tisty 0.3.0 is out/)).toBeTruthy();
    const said = screen.getByText(/Microsoft Store brings it to you/);
    expect(said.textContent).toContain("get updates");
    expect(screen.queryByRole("button", { name: "Update" })).toBeNull();
  });

  it("offers to do it for a copy that can replace itself", async () => {
    render(<About ready={ready("download")} onError={vi.fn()} />);

    expect(await screen.findByText("Do you want to update it?")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Update" })).toBeTruthy();
  });

  it("offers the same to a Homebrew install, which now keeps itself", async () => {
    render(<About ready={ready("brew", "tisty")} onError={vi.fn()} />);

    expect(await screen.findByRole("button", { name: "Update" })).toBeTruthy();
    expect(screen.queryByText(/brew upgrade/)).toBeNull();
  });

  it("gives the formula its own command, without the cask flag or a button", async () => {
    render(<About ready={ready("brewCli", "tisty-cli")} onError={vi.fn()} />);

    const said = await screen.findByText(/brew upgrade tisty-cli/);
    expect(said.textContent).toContain("brew update &&");
    expect(said.textContent).not.toContain("--cask");
    expect(screen.queryByRole("button", { name: "Update" })).toBeNull();
  });

  it("shows how far the download has got, and then that it is installing", async () => {
    const { rerender } = render(
      <About ready={ready("download")} step={{ stage: "getting", far: 25 }} onError={vi.fn()} />,
    );

    expect(await screen.findByText(/Getting it — 25 %/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Update" })).toBeNull();

    rerender(
      <About
        ready={ready("download")}
        step={{ stage: "installing", far: 100 }}
        onError={vi.fn()}
      />,
    );
    expect(screen.getByText(/Tisty will close and open again/)).toBeTruthy();
  });

  it("asks the core to install, and says nothing more while it is at it", async () => {
    render(<About ready={ready("download")} onError={vi.fn()} />);

    await userEvent.click(await screen.findByRole("button", { name: "Update" }));

    expect(asked).toContain("update_install");
    expect(screen.getByRole("button", { name: "Update" })).toHaveProperty("disabled", true);
  });

  it("hands the button back when the install was refused, and clears what was underway", async () => {
    refuse.next = { code: "updateGone" };
    const onError = vi.fn();
    const onGaveUp = vi.fn();
    render(<About ready={ready("download")} onError={onError} onGaveUp={onGaveUp} />);

    await userEvent.click(await screen.findByRole("button", { name: "Update" }));

    await waitFor(() => expect(onError).toHaveBeenCalled());
    expect(onGaveUp).toHaveBeenCalled();
    expect(screen.getByRole("button", { name: "Update" })).toHaveProperty("disabled", false);
  });

  it("says nothing when this copy is the newest", async () => {
    render(<About ready={null} onError={vi.fn()} />);

    expect(await screen.findByText("AGPL-3.0")).toBeTruthy();
    expect(screen.queryByText(/is out/)).toBeNull();
  });
});
