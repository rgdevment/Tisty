import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

const asked: string[] = [];
const refuse: { next: unknown } = { next: null };

import type { Ready } from "../core";
import type { KnownAgents } from "../knownAgents";
import { t } from "../locales";
import About from "../ui/About";
import RailNotices from "../ui/RailNotices";
import Sidebar from "../ui/Sidebar";
import { knownFor, lostOne } from "./knownFor";

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
      candidates: false,
      candidatesApply: true,
      autoUpdates: false,
      autoUpdatesApply: true,
    });
  },
}));

const chosen = { named: "tasks" } as Parameters<typeof Sidebar>[0]["chosen"];

function bar(waiting?: string, more: Partial<Parameters<typeof Sidebar>[0]> = {}) {
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
      {...more}
    />,
  );
}

const finder = (agents: KnownAgents) => ({
  attention: null,
  finding: "",
  setFinding: () => {},
  agents,
});

describe("the agents that can no longer find Tisty", () => {
  const lost = () => finder(knownFor({ agent: { on: true }, agents: [lostOne("Codex")] }));

  it("are said in the sidebar and on the gear, and lead to their page", () => {
    const onChoose = vi.fn();
    bar(undefined, { finder: lost(), onChoose });

    expect(screen.getByText("Codex can no longer find Tisty")).toBeTruthy();
    expect(screen.getByRole("button", { name: /Some agents need fixing/ })).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "See and fix" }));
    expect(onChoose.mock.calls[0][0]).toMatchObject({
      named: "keeping",
      tab: "agents",
      shelf: "wired",
    });
  });

  it("are not said twice inside Settings, nor at all when nothing is lost", () => {
    bar(undefined, { finder: lost(), chosen: { named: "keeping" } as typeof chosen });
    expect(screen.queryByText("Codex can no longer find Tisty")).toBeNull();
    cleanup();
    bar(undefined, { finder: finder(knownFor({ agent: { on: true } })) });
    expect(screen.queryByRole("button", { name: /Some agents need fixing/ })).toBeNull();
  });

  it("light the folded bar's notice", () => {
    render(<RailNotices agents />);
    expect(screen.getByRole("button", { name: t("sideNotices") })).toBeTruthy();
  });
});

describe("the update dot", () => {
  it("names itself for a reader that cannot see it", () => {
    bar("0.2.0");

    expect(screen.getByLabelText(/a newer version exists/)).toBeTruthy();
  });

  it("says nothing at all when this is the newest one", () => {
    bar();

    expect(screen.queryByLabelText(/a newer version exists/)).toBeNull();
    expect(screen.queryByLabelText("About")).toBeNull();
  });
});

beforeEach(() => {
  asked.length = 0;
  refuse.next = null;
});

describe("what About suggests", () => {
  const ready = (route: Ready["route"]): Ready => ({
    version: "0.3.0",
    route,
    installs: true,
  });

  // An offer a Store copy sees is one the Store itself made, so it is taken like any other.
  it("offers a Store install what the Store has, with the same button", async () => {
    render(<About ready={ready("store")} onError={vi.fn()} />);

    expect(await screen.findByText(/Tisty 0.3.0 is out/)).toBeTruthy();
    expect(screen.getByRole("button", { name: "Update" })).toBeTruthy();
  });

  it("offers to do it for a copy that can replace itself", async () => {
    render(<About ready={ready("download")} onError={vi.fn()} />);

    expect(await screen.findByText("Do you want to update it?")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Update" })).toBeTruthy();
  });

  it("offers the same to a Homebrew install, which now keeps itself", async () => {
    render(<About ready={ready("brew")} onError={vi.fn()} />);

    expect(await screen.findByRole("button", { name: "Update" })).toBeTruthy();
    expect(screen.queryByText(/brew upgrade/)).toBeNull();
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
    refuse.next = { code: "updateFailed", name: "no room left" };
    const onError = vi.fn();
    const onGaveUp = vi.fn();
    render(<About ready={ready("download")} onError={onError} onGaveUp={onGaveUp} />);

    await userEvent.click(await screen.findByRole("button", { name: "Update" }));

    await waitFor(() =>
      expect(document.querySelector("[data-setting=update] [data-note=err]")).toBeTruthy(),
    );
    expect(onError).not.toHaveBeenCalled();
    expect(onGaveUp).toHaveBeenCalled();
    expect(screen.getByRole("button", { name: "Update" })).toHaveProperty("disabled", false);
  });

  it("takes the candidates only when somebody asks for them, and says what that means", async () => {
    render(<About ready={null} onError={vi.fn()} />);

    const asks = await screen.findByRole("checkbox", { name: /release candidates/i });
    expect(asks).toHaveProperty("checked", false);
    expect(screen.getByText(/less tested/i)).toBeTruthy();

    await userEvent.click(asks);

    expect(asked).toContain("update_candidates");
  });

  it("says nothing when this copy is the newest", async () => {
    render(<About ready={null} onError={vi.fn()} />);

    expect(await screen.findByText("AGPL-3.0")).toBeTruthy();
    expect(screen.queryByText(/is out/)).toBeNull();
  });
});
