import Graph from "graphology";
import Sigma, { DEFAULT_STYLES } from "sigma";
import {
  layerDashed,
  layerFill,
  layerPlain,
  pathLine,
  sdfCircle,
  sdfSquare,
} from "sigma/rendering";
import { type Network, TIES } from "../../brainsty";
import { DASH, type LineLook, lineLook, nodeLook, type Palette } from "./looks";
import type { Spots } from "./settling";

export type HueOf = (group?: string) => string | undefined;

export interface Drawing {
  place: (spots: Spots) => void;
  retheme: (palette: Palette) => void;
  centre: (on: number) => void;
  kill: () => void;
}

const CLEAR = "rgba(0, 0, 0, 0)";
const NEAR_RATIO = 0.35;

const lineAttributes = (look: LineLook) => ({
  size: look.size,
  color: look.color,
  solid: look.gap > 0 ? CLEAR : look.color,
  gap: look.gap,
});

export function drawNetwork(
  container: HTMLElement,
  network: Network,
  spots: Spots,
  hueOf: HueOf,
  palette: Palette,
  font: string,
): Drawing {
  const graph = new Graph({ type: "directed", multi: true });
  const seen = { palette };
  network.nodes.forEach((one, n) => {
    graph.addNode(String(n), {
      x: spots.x[n],
      y: spots.y[n],
      label: one.title,
      ...nodeLook(one, hueOf(one.group), palette),
    });
  });
  for (const [from, to, code] of network.edges) {
    const tie = TIES[code];
    if (tie)
      graph.addEdge(String(from), String(to), { tie, ...lineAttributes(lineLook(tie, palette)) });
  }
  const sigma = new Sigma(graph, container, {
    primitives: {
      nodes: { shapes: [sdfCircle(), sdfSquare()], layers: [layerFill()] },
      edges: {
        paths: [pathLine()],
        variables: {
          solid: { type: "color", default: CLEAR },
          gap: { type: "number", default: 0 },
        },
        layers: [
          layerPlain({ color: { attribute: "solid" } }),
          layerDashed({ dashSize: { value: DASH }, gapSize: { attribute: "gap", default: 0 } }),
        ],
      },
    },
    styles: {
      nodes: [
        DEFAULT_STYLES.nodes,
        {
          shape: { attribute: "shape", defaultValue: "circle" },
          labelColor: () => seen.palette.ink,
          labelFont: font,
        },
      ],
      edges: [DEFAULT_STYLES.edges, { solid: { attribute: "solid" }, gap: { attribute: "gap" } }],
    },
    settings: { hideEdgesOnMove: true, labelRenderedSizeThreshold: 9, allowInvalidContainer: true },
  });
  return {
    place: ({ x, y }) => {
      if (x.length !== graph.order) return;
      graph.updateEachNodeAttributes(
        (key, at) => {
          const n = Number(key);
          return { ...at, x: x[n], y: y[n] };
        },
        { attributes: ["x", "y"] },
      );
    },
    retheme: (next) => {
      seen.palette = next;
      graph.updateEachNodeAttributes((key, at) => {
        const one = network.nodes[Number(key)];
        return { ...at, ...nodeLook(one, hueOf(one.group), next) };
      });
      graph.updateEachEdgeAttributes((_, at) => ({
        ...at,
        ...lineAttributes(lineLook(at.tie, next)),
      }));
      sigma.refresh();
    },
    centre: (on) => {
      const shown = sigma.getNodeDisplayData(String(on));
      if (shown) sigma.getCamera().setState({ x: shown.x, y: shown.y, ratio: NEAR_RATIO });
    },
    kill: () => sigma.kill(),
  };
}
