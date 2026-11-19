import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it } from "vitest";
import type { Group, Network } from "../brainsty";
import { t } from "../locales";
import Legend from "../ui/brainsty/Legend";
import { legendOf } from "../ui/brainsty/legendKeys";
import { lineLook, nodeLook, type Palette } from "../ui/brainsty/looks";

const palette: Palette = {
  bg: "#ffffff",
  ink: "#111111",
  soft: "#555555",
  faint: "#999999",
  accent: "#2266ff",
  tag: "#aa44ff",
  file: "#888888",
  hues: { teal: "#11aa99", orange: "#ff8800", pink: "#ff66aa", amber: "#ffbb00" },
};

const network: Network = {
  nodes: [
    { id: "a.md", kind: "doc", title: "A", group: "f1" },
    { id: "b.md", kind: "doc", title: "B", group: "f1" },
    { id: "01A", kind: "task", title: "bread", group: "l1", state: "open" },
    { id: "01B", kind: "task", title: "milk", group: "plain", state: "done" },
    { id: "#home", kind: "tag", title: "home" },
  ],
  edges: [
    [0, 1, 1],
    [2, 4, 3],
  ],
};

const groups: Group[] = [
  { id: "l1", name: "Casa", color: "orange" },
  { id: "f1", name: "Proyectos", color: "teal" },
  { id: "plain", name: "Sin color", color: null },
  { id: "empty", name: "Vacía", color: "pink" },
];

describe("what the legend says", () => {
  const legend = legendOf(network, groups, palette);

  it("names only the kinds in the network, drawn as the network draws them", () => {
    expect(legend.kinds.map((one) => one.kind)).toEqual(["doc", "task", "tag"]);
    const task = nodeLook({ id: "", kind: "task", title: "" }, undefined, palette);
    expect(legend.kinds[1]).toEqual({ kind: "task", ...task });
    expect(legend.kinds[0].shape).toBe("square");
  });

  it("names the coloured folders and lists in the network, the fullest first", () => {
    expect(legend.groups).toEqual([
      { id: "f1", name: "Proyectos", color: "#11aa99", count: 2 },
      { id: "l1", name: "Casa", color: "#ff8800", count: 1 },
    ]);
  });

  it("shows each state fading as the network fades it", () => {
    expect(legend.standings.map((one) => one.standing)).toEqual(["live", "done", "archived"]);
    expect(legend.standings[0].color).toBe("rgba(34, 102, 255, 1)");
    expect(legend.standings[2].size).toBeLessThan(legend.standings[0].size);
  });

  it("names only the lines in the network, dashed where the network dashes them", () => {
    expect(legend.alike).toBeNull();
    expect(legend.lines).toEqual([
      { tie: "mention", color: lineLook("mention", palette).color, size: 1, dashed: true },
      { tie: "tag", color: lineLook("tag", palette).color, size: 0.5, dashed: false },
    ]);
  });
});

describe("the alike curve in the legend", () => {
  it("is named, amber, when the network carries guesses", () => {
    const guessed = legendOf({ ...network, alike: [[0, 2, 700, "bread"]] }, groups, palette);

    expect(guessed.alike).toEqual({ color: "rgba(255, 187, 0, 0.95)", size: 1.5 });
  });
});

describe("the legend over the network", () => {
  beforeEach(() => {
    localStorage.clear();
    for (const [hue, colour] of Object.entries(palette.hues))
      document.documentElement.style.setProperty(`--tisty-hue-${hue}`, colour);
  });

  it("starts folded, opens on asking with kinds, colours, states and lines, and stays as left", async () => {
    const user = userEvent.setup();
    const { unmount } = render(<Legend network={network} groups={groups} />);
    const button = () => screen.getByRole("button", { name: t("brainstyLegend") });

    expect(button().getAttribute("aria-expanded")).toBe("false");
    expect(screen.queryByText("Proyectos")).toBeNull();
    await user.click(button());

    expect(screen.getByText(t("brainstyOne_doc"))).toBeTruthy();
    expect(screen.getByText("Proyectos")).toBeTruthy();
    expect(screen.queryByText("Vacía")).toBeNull();
    expect(screen.getByText(t("brainstyStanding_archived"))).toBeTruthy();
    expect(screen.getByText(t("brainstyTie_mention"))).toBeTruthy();
    unmount();

    const again = render(<Legend network={network} groups={groups} />);
    expect(button().getAttribute("aria-expanded")).toBe("true");
    await user.click(button());
    expect(screen.queryByText("Proyectos")).toBeNull();
    again.unmount();

    render(<Legend network={network} groups={groups} />);
    expect(button().getAttribute("aria-expanded")).toBe("false");
  });

  it("names the amber curve and what it means", () => {
    localStorage.setItem("tisty.brainstyLegend", "open");
    render(<Legend network={{ ...network, alike: [[0, 2, 700, "bread"]] }} groups={groups} />);

    expect(screen.getByText(t("brainstyAlike")).closest("p")?.getAttribute("title")).toBe(
      t("brainstyAlikeWhat"),
    );
  });

  it("shows the first colours and the rest on asking", async () => {
    localStorage.setItem("tisty.brainstyLegend", "open");
    const user = userEvent.setup();
    const many: Group[] = Array.from({ length: 10 }, (_, at) => ({
      id: `g${at}`,
      name: `Lista ${at}`,
      color: "teal",
    }));
    const crowded: Network = {
      nodes: many.map((one) => ({ id: one.id, kind: "task", title: one.name, group: one.id })),
      edges: [],
    };
    render(<Legend network={crowded} groups={many} />);

    expect(screen.queryByText("Lista 9")).toBeNull();
    await user.click(screen.getByRole("button", { name: /^4/ }));
    expect(screen.getByText("Lista 9")).toBeTruthy();
    await user.click(screen.getByRole("button", { name: t("brainstyLegendFewer") }));
    expect(screen.queryByText("Lista 9")).toBeNull();
  });
});
