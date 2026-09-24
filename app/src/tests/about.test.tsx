import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { saidPlainly } from "../refusal";
import About from "../ui/About";

const opened = vi.hoisted(() => ({ urls: [] as string[] }));

vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: (url: string) => {
    opened.urls.push(url);
    return Promise.resolve();
  },
}));

const ipc = vi.hoisted(() => ({
  tries: 0,
  answer: (_cmd: string): Promise<unknown> => Promise.resolve(null),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    if (cmd === "about") ipc.tries += 1;
    return ipc.answer(cmd);
  },
}));

const build = {
  version: "0.1.0",
  license: "AGPL-3.0",
  repository: "https://github.com/rgdevment/tisty",
  store: "C:/Users/x/AppData/Roaming/tisty",
};

beforeEach(() => {
  opened.urls = [];
  ipc.tries = 0;
  ipc.answer = () => Promise.resolve(build);
});

describe("the about screen", () => {
  it("says what this build is", async () => {
    render(<About ready={null} onError={() => {}} />);

    expect(await screen.findByText("0.1.0")).toBeTruthy();
  });

  it("recovers from its own failure", async () => {
    ipc.answer = () => Promise.reject(new Error("the store is not readable"));
    render(<About ready={null} onError={() => {}} />);

    expect(await screen.findByRole("alert")).toBeTruthy();
    expect(screen.getByText(/not readable/)).toBeTruthy();

    ipc.answer = () => Promise.resolve(build);
    await userEvent.click(screen.getByRole("button", { name: /try again/i }));

    expect(await screen.findByText("0.1.0")).toBeTruthy();
    await waitFor(() => expect(ipc.tries).toBe(2));
    expect(screen.queryByRole("alert")).toBeNull();
  });
});

describe("the other tools", () => {
  it("points each one at its own repository", async () => {
    render(<About ready={null} onError={() => {}} />);
    await screen.findByText("0.1.0");

    await userEvent.click(screen.getByRole("button", { name: /CopyPaste/ }));
    await userEvent.click(screen.getByRole("button", { name: /LinkUnbound/ }));

    expect(opened.urls).toEqual([
      "https://github.com/rgdevment/CopyPaste",
      "https://github.com/rgdevment/LinkUnbound",
    ]);
  });

  it("draws an icon for each, not a bullet", async () => {
    const { container } = render(<About ready={null} onError={() => {}} />);
    await screen.findByText("0.1.0");

    expect(container.querySelectorAll("img").length).toBe(2);
  });

  it("opens the coffee page for anyone who wants to help", async () => {
    render(<About ready={null} onError={() => {}} />);
    await screen.findByText("0.1.0");

    await userEvent.click(screen.getByRole("button", { name: /coffee/i }));

    expect(opened.urls).toEqual(["https://buymeacoffee.com/rgdevment"]);
  });

  it("opens Tisty's own repository as a url", async () => {
    render(<About ready={null} onError={() => {}} />);
    await screen.findByText("0.1.0");

    await userEvent.click(screen.getByRole("button", { name: /repository/i }));

    expect(opened.urls).toContain("https://github.com/rgdevment/tisty");
  });
});

describe("the notice every bundled licence asks for", () => {
  it("is shown from the window, and asked for only when it is", async () => {
    let asked = 0;
    ipc.answer = (cmd) => {
      if (cmd !== "notices") return Promise.resolve(build);
      asked += 1;
      return Promise.resolve("MIT License\n\nCopyright (c) alguien");
    };
    render(<About ready={null} onError={() => {}} />);

    const button = await screen.findByText("Third-party notices");
    expect(asked).toBe(0);

    await userEvent.click(button);

    expect(await screen.findByText(/Copyright \(c\) alguien/)).toBeTruthy();
    expect(asked).toBe(1);

    await userEvent.click(button);
    await waitFor(() => {
      expect(screen.queryByText(/Copyright \(c\) alguien/)).toBeNull();
    });
  });

  it("draws the notice as prose, not as the markdown it is written in", async () => {
    ipc.answer = (cmd) =>
      cmd === "notices"
        ? Promise.resolve(
            "## In the window\n\n| Package | Licence |\n| --- | --- |\n| jiff | MIT |",
          )
        : Promise.resolve(build);
    render(<About ready={null} onError={() => {}} />);

    await userEvent.click(await screen.findByText("Third-party notices"));

    const row = await screen.findByRole("cell", { name: "jiff" });
    expect(row.closest("table")).toBeTruthy();
    expect(screen.queryByText(/\| --- \|/)).toBeNull();
    expect(screen.queryByText(/^## /)).toBeNull();
  });
});

// Only the Store speaks for a copy it keeps: what reaches here is what the Store itself has,
// and that one this copy can take without leaving the window.
describe("a newer version the Store itself offers", () => {
  it("says who was asked when there is nothing, instead of calling the copy the newest", async () => {
    ipc.answer = (cmd) =>
      Promise.resolve(cmd === "about" ? { ...build, keptByTheStore: true } : null);
    render(<About ready={null} onError={() => {}} />);

    await screen.findByText("0.1.0");
    expect(screen.getByText(/Microsoft Store has nothing newer/i)).toBeTruthy();
    expect(screen.queryByText(/newest version/i)).toBeNull();
  });

  // «Could not ask» and «nothing for you» read the same on screen, and that is what hid a Store
  // that had not answered for whole versions.
  it("reports a Store that did not answer instead of leaving «nothing newer» standing", async () => {
    const problems: unknown[] = [];
    ipc.answer = (cmd) =>
      cmd === "update_ready"
        ? Promise.reject({ code: "updateUnanswered" })
        : Promise.resolve({ ...build, keptByTheStore: true });
    render(<About ready={null} onError={(problem) => problems.push(problem)} />);

    await screen.findByText("0.1.0");
    await userEvent.click(screen.getByRole("button", { name: /check for updates/i }));

    await waitFor(() => expect(problems.length).toBe(1));
    expect(saidPlainly(problems[0])).toMatch(/did not answer/i);
  });

  const waiting = { version: "1.15.0", route: "store" as const, package: null, installs: true };

  // The Store says nothing about an update it is already fetching, so the only thing the person
  // can be told is that one is on the way — with no number to put on it and nothing to press.
  it("says the Store is bringing one when the Store is bringing one", async () => {
    render(
      <About
        ready={{
          version: "",
          route: "store",
          package: null,
          installs: false,
          coming: true,
        }}
        onError={vi.fn()}
      />,
    );

    expect(await screen.findByText("The Store is bringing a new version")).toBeTruthy();
    expect(screen.getByText(/in the Store.s own queue/i)).toBeTruthy();
    expect(
      screen.queryByRole("button", { name: /update/i }),
      "there is nothing to press: the Store is already on it",
    ).toBeNull();
  });

  // A copy left closed for weeks remembers an offer the feed has moved past; the button must not
  // fail the same way for ever, but look again and offer what is out now.
  it("looks again when the offer it clicked is off the feed, and offers what is out now", async () => {
    const problems: unknown[] = [];
    const seen: string[] = [];
    ipc.answer = (cmd) => {
      seen.push(cmd);
      if (cmd === "update_install") return Promise.reject({ code: "updateMoved", name: "1.17.0" });
      if (cmd === "update_ready")
        return Promise.resolve({
          version: "1.17.0",
          route: "download",
          package: null,
          installs: true,
        });
      return Promise.resolve(build);
    };
    render(
      <About
        ready={{ version: "1.15.1", route: "download", package: null, installs: true }}
        onError={(problem) => problems.push(problem)}
      />,
    );

    await screen.findByText("0.1.0");
    expect(screen.getByText(/1\.15\.1/)).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: /update/i }));

    await screen.findByText(/1\.17\.0/);
    expect(screen.queryByText(/1\.15\.1/)).toBeNull();
    expect(seen.filter((cmd) => cmd === "update_ready")).toEqual(["update_ready"]);
    expect(problems.map((one) => saidPlainly(one))).toEqual([
      "That version is no longer offered; 1.17.0 is.",
    ]);
  });

  it("is taken with the one button, and never through a door to the Store", async () => {
    render(<About ready={waiting} onError={() => {}} />);

    await screen.findByText("0.1.0");

    expect(screen.queryByRole("button", { name: /open the store/i })).toBeNull();
    expect(screen.getByRole("button", { name: /update/i })).toBeTruthy();
    expect(opened.urls).toEqual([]);
  });
});

describe("a copy the Store keeps", () => {
  it("is offered the rating beside the star, and neither is offered twice", async () => {
    ipc.answer = (cmd) =>
      Promise.resolve(cmd === "about" ? { ...build, keptByTheStore: true } : null);
    render(<About ready={null} onError={() => {}} />);
    await screen.findByText("0.1.0");

    await userEvent.click(screen.getByRole("button", { name: /rate it in the store/i }));
    await userEvent.click(screen.getByRole("button", { name: /star on github/i }));

    expect(opened.urls).toEqual([
      "ms-windows-store://review/?ProductId=9PGVWXD8X93N",
      "https://github.com/rgdevment/Tisty",
    ]);
  });

  it("keeps the rating out of a copy the Store does not keep", async () => {
    render(<About ready={null} onError={() => {}} />);
    await screen.findByText("0.1.0");

    expect(screen.queryByRole("button", { name: /rate it in the store/i })).toBeNull();
    expect(screen.getByRole("button", { name: /star on github/i })).toBeTruthy();
  });
});
