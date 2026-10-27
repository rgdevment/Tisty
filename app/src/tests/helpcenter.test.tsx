import { readdirSync, readFileSync } from "node:fs";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { HelpEntry, HelpPictures } from "../guide/calls";
import {
  aimedAt,
  cameFrom,
  describedBy,
  dressed,
  searched,
  sectionsOf,
  suggestedFor,
} from "../guide/help";
import { inTheRepo } from "./repo";

const ENTRIES: HelpEntry[] = [
  { slug: "", title: "The Tisty wiki", section: "", order: 0, twin: "" },
  { slug: "tasks", title: "Tasks", section: "Tasks", order: 1, twin: "tareas" },
  { slug: "sync-and-backup", title: "Sync and backup", section: "Your data", order: 2, twin: "x" },
];

const BODIES: Record<string, string> = {
  "": "Home.",
  tasks: "A task is something to do. It closes and goes to the archive.",
  "sync-and-backup": "## Encrypting your cloud\n\nYour phrase opens the cloud on another computer.",
};

const ipc = vi.hoisted(() => ({
  pictures: { weighs: 2_100_000, missing: 2_100_000, kept: {} } as HelpPictures,
  fetched: 0,
}));

vi.mock("../core", async (original) => ({
  ...(await original<typeof import("../core")>()),
  openLink: () => Promise.resolve(),
}));

vi.mock("../guide/calls", async (original) => ({
  ...(await original<typeof import("../guide/calls")>()),
  helpPages: () => Promise.resolve(ENTRIES),
  helpPage: (_: boolean, slug: string) => Promise.resolve(BODIES[slug] ?? null),
  helpPictures: () => Promise.resolve(ipc.pictures),
  helpPicturesFetch: () => {
    ipc.fetched += 1;
    return Promise.resolve({ ...ipc.pictures, missing: 0 });
  },
  helpReference: () => Promise.resolve("# Guide\n\nIntro.\n\n## Pens\n\nA pen."),
  helpDrawn: (body: string) => Promise.resolve(`<p>${body.length}</p>`),
  helpAnchors: () => Promise.resolve([["guide"], ["pens"]]),
}));

vi.mock("../ui/Page", () => ({
  default: ({ label }: { label: string }) => (
    <section aria-label="page">
      <h2>{label}</h2>
    </section>
  ),
}));

import Help from "../ui/Help";

const help = inTheRepo("docs/help");
const pages = readdirSync(help).filter((one) => one.endsWith(".md"));
const slugOf = (page: string) =>
  page
    .normalize("NFD")
    .replace(/\p{M}/gu, "")
    .toLowerCase()
    .replace(/[^a-z0-9.]+/g, "-")
    .replace(/\./g, "")
    .replace(/^-+|-+$/g, "");
const fileOf = (slug: string) => pages.find((one) => slugOf(one.replace(/\.md$/, "")) === slug);

describe("what the help offers for where you were", () => {
  it("names the screen the help was asked from", () => {
    expect(cameFrom({ named: "keeping", tab: "security" })).toBe("keeping:security");
    expect(cameFrom({ named: "keeping" })).toBe("keeping:general");
    expect(cameFrom({ named: "docs" })).toBe("docs");
    expect(cameFrom({})).toBe("tasks");
  });

  it("points only at pages and sections the help really has, in both languages", () => {
    for (const from of [
      "keeping:sync",
      "keeping:security",
      "keeping:data",
      "keeping:agents",
      "keeping:general",
      "docs",
      "tasks",
      "archive",
    ]) {
      for (const spanish of [false, true]) {
        for (const one of suggestedFor(from, spanish)) {
          const file = fileOf(one.slug);
          expect(file, `${from} → ${one.slug}`).toBeTruthy();
          if (!one.heading || !file) continue;
          const text = readFileSync(`${help}/${file}`, "utf8");
          expect(text, `${one.slug} has no «${one.heading}»`).toMatch(
            new RegExp(`^#{2,4} ${one.heading}$`, "m"),
          );
        }
      }
    }
  });

  it("falls back to the tasks when the screen has nothing of its own", () => {
    expect(suggestedFor("nowhere", false)).toEqual(suggestedFor("tasks", false));
  });
});

describe("reading the help", () => {
  it("groups pages by section and leaves the home out", () => {
    expect(sectionsOf(ENTRIES).map(([section]) => section)).toEqual(["Tasks", "Your data"]);
  });

  it("finds a page whatever the accents and puts a title match first", () => {
    const found = searched(ENTRIES, BODIES, "frase phrase");
    expect(found).toEqual([]);
    expect(searched(ENTRIES, BODIES, "PHRASE")[0]?.entry.slug).toBe("sync-and-backup");
    expect(searched(ENTRIES, BODIES, "tasks")[0]?.entry.slug).toBe("tasks");
    expect(
      searched([{ ...ENTRIES[1], title: "Tareas" }], { tasks: "Fácil" }, "facil"),
    ).toHaveLength(1);
  });

  it("turns links to other pages into its own and stands in for a picture not yet here", () => {
    const pictures: HelpPictures = {
      weighs: 2,
      missing: 1,
      kept: { "https://x.invalid/a.png": "/tmp/a.png" },
    };
    const html = dressed(
      '<p><a href="help:tasks#closing">Tasks</a><img src="https://x.invalid/a.png" alt="A"><img src="https://x.invalid/b.png" alt="B"></p>',
      pictures,
      (path) => `asset://${path}`,
      (alt) => `Picture: ${alt}`,
    );
    expect(html).toContain('href="#help/tasks#closing"');
    expect(html).toContain('src="asset:///tmp/a.png"');
    expect(html).toContain('<span class="picture-away">Picture: B</span>');
    expect(aimedAt("help/tasks#closing")).toEqual({ slug: "tasks", anchor: "closing" });
    expect(aimedAt("colours")).toBeNull();
  });

  it("describes a page by its first plain sentence", () => {
    expect(describedBy("> [!NOTE]\n> skip\n\nThe [first](x) words, **here**.")).toBe(
      "The first words, here.",
    );
  });
});

describe("the help inside the window", () => {
  it("opens on its home, finds a page and opens it", async () => {
    render(<Help from="keeping:security" />);
    expect(
      await screen.findByRole("heading", { name: "What would you like to know?" }),
    ).toBeTruthy();
    expect(await screen.findByRole("button", { name: "Encrypting your cloud" })).toBeTruthy();
    await userEvent.type(screen.getByLabelText("Search the whole help, offline too"), "archive");
    const results = screen.getByRole("region", { name: "Search the whole help, offline too" });
    await userEvent.click(await within(results).findByRole("button", { name: /^Tasks/ }));
    expect(await screen.findByRole("heading", { name: "Tasks" })).toBeTruthy();
  });

  it("offers the pictures once, downloads them on asking and lets them wait", async () => {
    const { unmount } = render(<Help />);
    expect(await screen.findByText(/Download 2.1 MB of pictures/)).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Download" }));
    await waitFor(() => expect(ipc.fetched).toBe(1));
    await waitFor(() => expect(screen.queryByText(/of pictures/)).toBeNull());
    unmount();

    render(<Help />);
    await screen.findByText(/of pictures/);
    await userEvent.click(screen.getByRole("button", { name: "Not now" }));
    expect(screen.queryByText(/of pictures/)).toBeNull();
  });

  it("keeps Tisty's Markdown a click away and comes back from it", async () => {
    render(<Help />);
    await userEvent.click((await screen.findAllByRole("button", { name: "Tisty's Markdown" }))[0]);
    expect(await screen.findByRole("navigation", { name: /topics/i })).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "‹ Help" }));
    expect(
      await screen.findByRole("heading", { name: "What would you like to know?" }),
    ).toBeTruthy();
  });
});
