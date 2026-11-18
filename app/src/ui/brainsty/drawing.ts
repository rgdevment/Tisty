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
import {
  bigOnes,
  DASH,
  degreesOf,
  type LineLook,
  lineLook,
  nodeLook,
  type Palette,
  rgba,
} from "./looks";
import type { Spots } from "./settling";
import type { Point } from "./zones";
import { allTitles, bigTitles, lerp, lineDetail, quantized, zoomOf } from "./zoom";

export type HueOf = (group?: string) => string | undefined;

export interface Drawing {
  place: (spots: Spots) => void;
  retheme: (palette: Palette) => void;
  centre: (on: number) => void;
  zoom: () => number;
  toScreen: (point: Point) => Point;
  nodeOnScreen: (node: number) => Point | undefined;
  onFrame: (listener: () => void) => () => void;
  light: (nodes: ReadonlySet<number> | null) => void;
  hide: (hidden: readonly boolean[]) => void;
  beside: (node: number) => number[];
  heed: (hands: Hands) => () => void;
  focus: (node: number) => void;
  kill: () => void;
}

export interface Hands {
  pick: (node: number) => void;
  open: (node: number) => void;
  clear: () => void;
}

interface Seen {
  zoom: number;
  hasHighlighted: boolean;
}

interface Lit {
  isHovered?: boolean;
  isHighlighted?: boolean;
}

interface Line {
  tone: string;
  alpha: number;
  size: number;
  gap: number;
}

const CLEAR = "rgba(0, 0, 0, 0)";
const NEAR_RATIO = 0.35;
const FOCUS_RATIO = 0.12;
const FAR_SHARE = 0.35;
const DIM = 0.12;

const lineAttributes = (look: LineLook): Line => ({
  tone: look.tone,
  alpha: look.alpha,
  size: look.size,
  gap: look.gap,
});

const lineColour = (line: Line, zoom: number): string =>
  rgba(line.tone, lerp(line.alpha * FAR_SHARE, line.alpha, lineDetail(zoom)));

const dashed = (line: Line, zoom: number): boolean => line.gap > 0 && lineDetail(zoom) >= 0.5;

const titleShown = (big: boolean, zoom: number): number =>
  big ? Math.max(bigTitles(zoom), allTitles(zoom)) : allTitles(zoom);

const split = <T>(all: readonly T[], test: (one: T) => boolean): [T[], T[]] => {
  const yes: T[] = [];
  const no: T[] = [];
  for (const one of all) (test(one) ? yes : no).push(one);
  return [yes, no];
};

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
  const degrees = degreesOf(network.nodes.length, network.edges);
  const big = bigOnes(degrees);
  network.nodes.forEach((one, n) => {
    graph.addNode(String(n), {
      x: spots.x[n],
      y: spots.y[n],
      label: one.kind === "tag" ? `#${one.title}` : one.title,
      big: big[n],
      ...nodeLook(one, hueOf(one.group), palette, degrees[n]),
    });
  });
  for (const [from, to, code] of network.edges) {
    const tie = TIES[code];
    if (tie)
      graph.addEdge(String(from), String(to), { tie, ...lineAttributes(lineLook(tie, palette)) });
  }
  const sigma = new Sigma(graph, container, {
    customGraphState: { zoom: 0 },
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
          labelFont: font,
          color: (at, state: Lit, shared: Seen) =>
            shared.hasHighlighted && !state.isHighlighted ? rgba(String(at.color), DIM) : at.color,
          labelVisibility: (at, state: Lit, shared: Seen) => {
            if (state.isHovered || state.isHighlighted) return "visible";
            if (shared.hasHighlighted) return "hidden";
            return titleShown(Boolean(at.big), shared.zoom) > 0 ? "auto" : "hidden";
          },
          labelColor: (at, _state, shared: Seen) =>
            rgba(seen.palette.ink, titleShown(Boolean(at.big), shared.zoom) || 1),
        },
      ],
      edges: [
        DEFAULT_STYLES.edges,
        {
          color: (at, state: Lit, shared: Seen) =>
            shared.hasHighlighted && !state.isHighlighted
              ? rgba((at as Line).tone, (at as Line).alpha * DIM)
              : lineColour(at as Line, shared.zoom),
          size: (at, _state, shared: Seen) =>
            lerp((at as Line).size * 0.6, (at as Line).size, lineDetail(shared.zoom)),
          solid: (at, _state, shared: Seen) =>
            dashed(at as Line, shared.zoom) ? CLEAR : lineColour(at as Line, shared.zoom),
          gap: (at, _state, shared: Seen) =>
            dashed(at as Line, shared.zoom) ? (at as Line).gap : 0,
        },
      ],
    },
    settings: { hideEdgesOnMove: true, labelRenderedSizeThreshold: 1, allowInvalidContainer: true },
  });
  const camera = sigma.getCamera();
  let zoom = quantized(zoomOf(camera.ratio));
  sigma.setGraphState({ zoom });
  const zoomed = () => {
    const next = quantized(zoomOf(camera.ratio));
    if (next === zoom) return;
    zoom = next;
    sigma.setGraphState({ zoom });
  };
  camera.on("updated", zoomed);
  const lit: { nodes: string[]; edges: string[] } = { nodes: [], edges: [] };
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
        const n = Number(key);
        const one = network.nodes[n];
        return { ...at, ...nodeLook(one, hueOf(one.group), next, degrees[n]) };
      });
      graph.updateEachEdgeAttributes((_, at) => ({
        ...at,
        ...lineAttributes(lineLook(at.tie, next)),
      }));
      sigma.refresh();
    },
    centre: (on) => {
      const shown = sigma.getNodeDisplayData(String(on));
      if (shown) camera.setState({ x: shown.x, y: shown.y, ratio: NEAR_RATIO });
    },
    zoom: () => zoom,
    toScreen: ([x, y]) => {
      const at = sigma.graphToViewport({ x, y });
      return [at.x, at.y];
    },
    nodeOnScreen: (node) => {
      const key = String(node);
      if (!graph.hasNode(key)) return undefined;
      const at = sigma.graphToViewport({
        x: graph.getNodeAttribute(key, "x"),
        y: graph.getNodeAttribute(key, "y"),
      });
      return [at.x, at.y];
    },
    onFrame: (listener) => {
      sigma.on("afterRender", listener);
      return () => sigma.off("afterRender", listener);
    },
    light: (nodes) => {
      sigma.setNodesState(lit.nodes, { isHighlighted: false });
      sigma.setEdgesState(lit.edges, { isHighlighted: false });
      lit.nodes = nodes ? [...nodes].map(String).filter((key) => graph.hasNode(key)) : [];
      lit.edges = nodes
        ? graph.filterEdges((_, __, from, to) => nodes.has(Number(from)) && nodes.has(Number(to)))
        : [];
      sigma.setNodesState(lit.nodes, { isHighlighted: true });
      sigma.setEdgesState(lit.edges, { isHighlighted: true });
      sigma.setGraphState({ hasHighlighted: lit.nodes.length > 0 });
    },
    hide: (hidden) => {
      const [gone, kept] = split(graph.nodes(), (key) => Boolean(hidden[Number(key)]));
      sigma.setNodesState(gone, { isHidden: true });
      sigma.setNodesState(kept, { isHidden: false });
      const [cut, whole] = split(graph.edges(), (edge) =>
        graph.extremities(edge).some((key) => hidden[Number(key)]),
      );
      sigma.setEdgesState(cut, { isHidden: true });
      sigma.setEdgesState(whole, { isHidden: false });
    },
    beside: (node) =>
      graph.hasNode(String(node)) ? graph.neighbors(String(node)).map(Number) : [],
    heed: (hands) => {
      const picked = ({ node }: { node: string }) => hands.pick(Number(node));
      const opened = (said: { node: string; preventSigmaDefault: () => void }) => {
        said.preventSigmaDefault();
        hands.open(Number(said.node));
      };
      sigma.on("clickNode", picked);
      sigma.on("doubleClickNode", opened);
      sigma.on("clickStage", hands.clear);
      return () => {
        sigma.off("clickNode", picked);
        sigma.off("doubleClickNode", opened);
        sigma.off("clickStage", hands.clear);
      };
    },
    focus: (node) => {
      const shown = sigma.getNodeDisplayData(String(node));
      if (!shown) return;
      void camera.animate(
        { x: shown.x, y: shown.y, ratio: Math.min(camera.ratio, FOCUS_RATIO) },
        { duration: 300 },
      );
    },
    kill: () => {
      camera.off("updated", zoomed);
      sigma.kill();
    },
  };
}
