import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { Head } from "tisty-editor/read";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { fill, t } from "../locales";
import Beside from "../ui/Beside";

const ipc = vi.hoisted(() => ({
  card: null as unknown,
  facts: { bytes: 7, pages: 0, shareable: "yes" } as unknown,
  asked: [] as string[],
}));

// Reset before each test, so one that fails halfway does not leave its facts to the next.
beforeEach(() => {
  ipc.facts = { bytes: 7, pages: 0, shareable: "yes" };
});

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    ipc.asked.push(cmd);
    if (cmd === "doc_facts") return Promise.resolve(ipc.facts);
    return Promise.resolve(cmd === "share_card" ? ipc.card : null);
  },
}));

const headed = (many: number): Head[] =>
  Array.from({ length: many }, (_, at) => ({
    key: `h${at}`,
    level: 2,
    text: `Sección ${at + 1}`,
    go: () => {},
  }));

const beside = (heads: Head[], more: { onCopy?: () => void } = {}, paper = "dev_a-0001") => (
  <Beside
    onError={() => {}}
    title="Notas"
    paper={paper}
    body="# Notas"
    tags={[]}
    kept={0}
    heads={heads}
    pages={[]}
    leaf="a4"
    onLeaf={() => {}}
    making={false}
    onCopy={more.onCopy ?? (() => {})}
    onTakeOut={() => {}}
    onShut={() => {}}
    onApart={() => {}}
  />
);

const panel = (heads: Head[], more: { onCopy?: () => void } = {}) => render(beside(heads, more));

// A row of the column: its details element, and a press on its summary as the person makes it.
const row = (key: string) =>
  document.querySelector(`details[data-row=${key}]`) as HTMLDetailsElement;
// The document's facts arrive after the first draw and draw the column again, so a press waits for them.
const press = async (key: string) => {
  await act(async () => {});
  await userEvent.click(row(key).querySelector("summary") as HTMLElement);
};

describe("the document's column", () => {
  it("opens the details with a document whose outline is short", () => {
    panel(headed(2));
    expect(row("details").open).toBe(true);
  });

  it("keeps the details folded when the outline needs the room", () => {
    panel(headed(12));
    expect(row("details").open).toBe(false);
  });

  it("folds the details once a long outline arrives after the page is drawn", async () => {
    const view = panel([]);
    view.rerender(beside(headed(12)));
    // The outline is followed once the words rest, not on every key.
    await waitFor(() => expect(row("details").open).toBe(false));
  });

  it("leaves what the person opened alone when the outline grows", async () => {
    const view = panel(headed(12));
    await press("details");
    expect(row("details").open).toBe(true);
    view.rerender(beside(headed(14)));
    expect(row("details").open).toBe(true);
  });

  it("opens one part at a time", async () => {
    panel(headed(2));
    await press("shared");
    expect(screen.getByText(t("sharedWhy"))).toBeTruthy();
    await waitFor(() => expect(row("details").open).toBe(false));
  });

  it("gathers every way out of the document under one button, the web page among them", async () => {
    const onCopy = vi.fn();
    panel(headed(2), { onCopy });
    await userEvent.click(screen.getByRole("button", { name: t("exportIt") }));
    for (const way of [t("previewPdf"), t("toPdf"), t("toHtml"), t("toWord"), t("saveAsMarkdown")])
      expect(screen.getByRole("menuitem", { name: new RegExp(way) })).toBeTruthy();
    await userEvent.click(screen.getByRole("menuitem", { name: new RegExp(t("copyPlain")) }));
    expect(onCopy).toHaveBeenCalledTimes(1);
  });
});

describe("the export button", () => {
  it("closes its menu when pressed again", async () => {
    panel(headed(2));
    // Asked for afresh each time: the column may have drawn the button again between the presses.
    const button = () => screen.getByRole("button", { name: t("exportIt") });
    await userEvent.click(button());
    expect(screen.queryByRole("menu")).toBeTruthy();
    expect(button().isConnected).toBe(true);
    await userEvent.click(button());
    expect(screen.queryByRole("menu")).toBeNull();
  });

  it("opens from the keyboard too", async () => {
    panel(headed(2));
    screen.getByRole("button", { name: t("exportIt") }).focus();
    await userEvent.keyboard("{Enter}");
    expect(screen.queryByRole("menu")).toBeTruthy();
  });

  it("says the PDF is being made instead of offering a second one", async () => {
    render(
      <Beside
        onError={() => {}}
        title="Notas"
        paper="dev_a-0001"
        body="# Notas"
        tags={[]}
        kept={0}
        heads={[]}
        pages={[]}
        leaf="a4"
        onLeaf={() => {}}
        making
        onCopy={() => {}}
        onTakeOut={() => {}}
        onShut={() => {}}
        onApart={() => {}}
      />,
    );
    await userEvent.click(screen.getByRole("button", { name: t("makingPdf") }));
    expect(screen.queryByRole("menuitem", { name: new RegExp(t("previewPdf")) })).toBeNull();
    expect(screen.queryByRole("menuitem", { name: new RegExp(t("toPdf")) })).toBeNull();
    expect(screen.getByRole("menuitem", { name: new RegExp(t("toHtml")) })).toBeTruthy();
    expect(screen.getByRole("button", { name: t("makingPdf") }).title).toBe(t("makingPdf"));
  });
});

describe("the details coming back to a document", () => {
  it("decide afresh for it, whatever another document left them as", async () => {
    const view = render(beside(headed(12)));
    await press("details");
    view.rerender(beside(headed(2), {}, "dev_a-0002"));
    view.rerender(beside(headed(12)));
    expect(row("details").open).toBe(false);
  });
});

describe("a page's column", () => {
  it("takes the person back to the document the page belongs to", async () => {
    const onPage = vi.fn();
    const up = {
      id: "dev_a-0001",
      file: "dev_a-0001",
      title: "Guía",
      folder: null,
      archived: false,
      away: false,
    };
    render(
      <Beside
        onError={() => {}}
        title="La Rina"
        paper="dev_a-0002"
        body="# La Rina"
        tags={[]}
        kept={0}
        heads={[]}
        pages={[]}
        up={up}
        onPage={onPage}
        leaf="a4"
        onLeaf={() => {}}
        making={false}
        onCopy={() => {}}
        onTakeOut={() => {}}
        onShut={() => {}}
        onApart={() => {}}
      />,
    );

    expect(screen.getByText(t("besidePageOf"))).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Guía" }));
    expect(onPage).toHaveBeenCalledWith(up);
  });
});

describe("the shared row", () => {
  it("offers to share a document nobody shared yet", async () => {
    ipc.card = null;
    panel(headed(2));
    await press("shared");
    expect(await screen.findByRole("button", { name: t("shareIt") })).toBeTruthy();
  });

  it("offers nothing on someone else's document, and says why", async () => {
    ipc.card = null;
    ipc.facts = { bytes: 7, pages: 0, author: "Tisty", shareable: "notMine" };
    panel(headed(2));
    await waitFor(() =>
      expect(row("shared").querySelector(".side-gist")?.textContent).toBe(t("sharedNotMine")),
    );
    await press("shared");
    expect(screen.getByText(t("sharedNotMineWhy"))).toBeTruthy();
    expect(screen.queryByRole("button", { name: t("shareIt") })).toBeNull();
  });

  it("asks for a key before sharing, since nothing goes out unsigned", async () => {
    ipc.card = null;
    ipc.facts = { bytes: 7, pages: 0, shareable: "noKey" };
    panel(headed(2));
    await waitFor(() =>
      expect(row("shared").querySelector(".side-gist")?.textContent).toBe(t("sharedNoKey")),
    );
    await press("shared");
    expect(screen.getByText(t("sharedNoKeyWhy"))).toBeTruthy();
    expect(screen.queryByRole("button", { name: t("shareIt") })).toBeNull();
  });

  it("waits for the new phrase when it began again on another computer", async () => {
    ipc.card = null;
    ipc.facts = { bytes: 7, pages: 0, shareable: "moved" };
    panel(headed(2));
    await waitFor(() =>
      expect(row("shared").querySelector(".side-gist")?.textContent).toBe(t("sharedMoved")),
    );
    await press("shared");
    expect(screen.getByText(t("sharedMovedWhy"))).toBeTruthy();
    expect(screen.queryByRole("button", { name: t("shareIt") })).toBeNull();
  });

  it("hands out the links of a shared document, says how it stands, and stops sharing it", async () => {
    ipc.card = {
      read: "https://tisty.cl/k7Qm2xPa#llave",
      edit: "https://tisty.cl/k7Qm2xPa#llave.token",
      gated: true,
      version: 2,
      info: {
        latest: 2,
        reads: 3,
        ends: 1_800_000_000,
        paused: false,
        blocked: false,
        gated: true,
      },
    };
    ipc.asked = [];
    panel(headed(2));
    await screen.findByText("https://tisty.cl/k7Qm2xPa#llave.token");
    expect(row("shared").querySelector(".side-gist")?.textContent).toContain(t("sharedGated"));
    expect(row("shared").querySelector(".side-gist")?.textContent).toContain(
      fill("sharedReads", "3"),
    );
    await press("shared");
    await userEvent.click(screen.getByRole("button", { name: t("shareStop") }));
    await waitFor(() => expect(ipc.asked).toContain("share_stop"));
    await waitFor(() =>
      expect(row("shared").querySelector(".side-gist")?.textContent).toBe(t("sharedNot")),
    );
  });
});
