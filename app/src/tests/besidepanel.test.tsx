import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { cloneElement } from "react";
import type { Head } from "tisty-editor/read";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { measureFor, measureOf } from "../breadth";
import { t } from "../locales";
import Beside from "../ui/Beside";

const linking = vi.hoisted(() => ({ files: [] as string[], refuses: null as unknown }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => Promise.resolve(cmd === "doc_facts" ? { bytes: 7, pages: 0 } : null),
}));

vi.mock("../copying", () => ({
  asPlain: () => Promise.resolve(),
  asLink: (file: string) => {
    linking.files.push(file);
    return linking.refuses ? Promise.reject(linking.refuses) : Promise.resolve();
  },
}));

const headed = (many: number): Head[] =>
  Array.from({ length: many }, (_, at) => ({
    key: `h${at}`,
    level: 2,
    text: `Sección ${at + 1}`,
    go: () => {},
  }));

type Hands = { onCopy?: () => void; onError?: (problem: unknown) => void };

const beside = (heads: Head[], more: Hands = {}, paper = "dev_a-0001") => (
  <Beside
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
    onError={more.onError}
    onTakeOut={() => {}}
    onShut={() => {}}
    onApart={() => {}}
  />
);

const panel = (heads: Head[], more: Hands = {}) => render(beside(heads, more));

// A row of the column: its details element, and a press on its summary as the person makes it.
const row = (key: string) =>
  document.querySelector(`details[data-row=${key}]`) as HTMLDetailsElement;
// The document's facts arrive after the first draw and draw the column again, so a press waits for them.
const press = async (key: string) => {
  await act(async () => {});
  await userEvent.click(row(key).querySelector("summary") as HTMLElement);
};

describe("the document's column", () => {
  it("hands over the design picked, by its name", async () => {
    const onDesign = vi.fn();
    render(cloneElement(beside([]), { design: "bosque", onDesign }));
    const row = screen.getByRole("group", { name: t("designIs") });
    const pick = (name: string) => within(row).getByRole("button", { name });
    expect(pick(t("designBosque")).getAttribute("aria-pressed")).toBe("true");
    await userEvent.click(pick(t("designFiordo")));
    expect(onDesign).toHaveBeenCalledWith("fiordo");
  });

  it("offers back the design of its own a document had, and never another document's", async () => {
    const onDesign = vi.fn();
    const own = "own/arena/vino/cafe/lora" as const;
    const shown = (paper: string, design: typeof own | "coral") =>
      cloneElement(beside([]), { paper, design, onDesign });
    const view = render(shown("a", own));
    view.rerender(shown("a", "coral"));
    const pick = () => screen.getByRole("button", { name: t("designOwn") });
    await userEvent.click(pick());
    expect(onDesign).toHaveBeenLastCalledWith(own);
    view.rerender(shown("b", "coral"));
    await userEvent.click(pick());
    expect(onDesign).toHaveBeenLastCalledWith("own/papel/grafito/tinta/sans");
  });

  it("offers no design where the document cannot be written", () => {
    panel([]);
    expect(screen.queryByRole("group", { name: t("designIs") })).toBeNull();
  });

  it("offers the widths of the text and writes the one picked", async () => {
    const onMeasure = vi.fn((one) => measureFor("dev_a-0001", one));
    render(cloneElement(beside([]), { measure: "normal", onMeasure }));
    const row = screen.getByRole("group", { name: t("measure") });
    const names = within(row)
      .getAllByRole("button")
      .map((one) => one.textContent);
    expect(names).toEqual([
      t("measureNarrow"),
      t("measureNormal"),
      t("measureWide"),
      t("measureFull"),
    ]);
    expect(within(row).getByRole("button", { name: t("measureNormal") }).ariaPressed).toBe("true");
    await userEvent.click(within(row).getByRole("button", { name: t("measureWide") }));
    expect(onMeasure).toHaveBeenCalledWith("wide");
    expect(measureOf("dev_a-0001", JSON.parse(localStorage.getItem("tisty.measure") ?? "{}"))).toBe(
      "wide",
    );
  });

  it("holds the widths while the sheets are on, and says why", async () => {
    const onMeasure = vi.fn();
    render(cloneElement(beside([]), { measure: "normal", onMeasure, sheets: true }));
    const row = screen.getByRole("group", { name: t("measure") });
    expect(row.getAttribute("title")).toBe(t("measureNeedsNoSheets"));
    await userEvent.click(within(row).getByRole("button", { name: t("measureWide") }));
    expect(onMeasure).not.toHaveBeenCalled();
  });

  it("offers no widths where the text's width means nothing", () => {
    panel([]);
    expect(screen.queryByRole("group", { name: t("measure") })).toBeNull();
  });

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

describe("copying the link from the export menu", () => {
  beforeEach(() => {
    linking.files.length = 0;
    linking.refuses = null;
  });

  const copyLink = async () => {
    await userEvent.click(screen.getByRole("button", { name: t("exportIt") }));
    await userEvent.click(screen.getByRole("menuitem", { name: new RegExp(t("copyLink")) }));
  };

  it("copies the link of the document the column is beside", async () => {
    const onError = vi.fn();
    render(beside(headed(2), { onError }, "dev_a-0007"));

    await copyLink();

    expect(linking.files).toEqual(["dev_a-0007"]);
    expect(onError).not.toHaveBeenCalled();
  });

  it("hands the problem to the window when the link could not be copied", async () => {
    linking.refuses = { code: "noClipboard" };
    const onError = vi.fn();
    panel(headed(2), { onError });

    await copyLink();

    await waitFor(() => expect(onError).toHaveBeenCalledWith({ code: "noClipboard" }));
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

describe("sharing from the column", () => {
  const column = (body: string) =>
    render(
      <Beside
        title="Lluvia de ideas"
        paper="dev_a-0002"
        doc="01JBRAINSTORM0000000000000"
        body={body}
        tags={[]}
        kept={0}
        heads={[]}
        pages={[]}
        leaf="a4"
        onLeaf={() => {}}
        making={false}
        onCopy={() => {}}
        onTakeOut={() => {}}
        onShut={() => {}}
        onApart={() => {}}
      />,
    );

  it("offers a document with an idea to Folio", async () => {
    column("# Huerto\n\n> [!IDEA] 2026-11-12 · yellow\n> Regar de noche\n");
    await act(async () => {});
    expect(row("shared")).not.toBeNull();
  });

  it("never offers Brainstorm itself to Folio", async () => {
    column("---\nview: notes\nbrainstorm: on\n---\n\n# Lluvia de ideas\n");
    await act(async () => {});
    expect(row("shared")).toBeNull();
    expect(screen.queryByText(t("shareIt"))).toBeNull();
  });
});
