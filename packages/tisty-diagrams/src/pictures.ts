import { type Classes, readClasses } from "./classes";
import { type Flow, readFlow } from "./flow";
import { outlineOf } from "./flowoutlines";
import { type Route, routeOf } from "./flowroutes";
import { type Shape as FlowShape, pictured } from "./flowshapes";
import { looksOf, type Look as Painted } from "./flowstyles";
import { kindOf } from "./framing";
import { edgeOf, type Rect } from "./graphlayout";
import { curve, HEXAGON, HUES, laidOut, type Placed } from "./mindlayout";
import { type Mind, type Shape as MindShape, readMind } from "./mindmap";
import {
  boxed,
  cut,
  fitted,
  type Look,
  type Palette,
  type Picture,
  pill,
  straight,
  stroke,
  writing,
} from "./picture";
import { CLASS_HEAD, CLASS_LINE, classScene, flowScene, type Scene } from "./scenes";
import { readSchema, type Schema } from "./schema";
import { bend, drawnOf, HEAD, ROW, relationWords } from "./schemalayout";

const RADII: Record<MindShape, number> = {
  plain: 6,
  square: 3,
  rounded: 8,
  circle: 999,
  cloud: 999,
  bang: 2,
  hexagon: 0,
};

const idea = (one: Placed, palette: Palette): string => {
  const hue = one.hue < 0 ? palette.accent : HUES[one.hue % HUES.length];
  const rect: Rect = one;
  const text = fitted(one.idea.text, one.wide - 16, one.depth === 0 ? 9 : 7.4);
  const middle = { x: one.x + one.wide / 2, y: one.y + one.tall / 2 };
  if (one.depth === 0)
    return `${boxed(rect, { fill: hue, radius: one.tall / 2 })}${writing({ ...middle, text, size: 15, fill: "#ffffff", bold: true })}`;
  const shape = one.idea.shape;
  const look: Look = {
    fill: one.depth === 1 ? palette.panel : palette.desk,
    stroke: one.depth === 1 || shape !== "plain" ? hue : undefined,
    width: shape === "bang" ? 2 : 1,
    radius: Math.min(RADII[shape], one.tall / 2),
    dashed: shape === "cloud" || shape === "bang" ? "4 3" : undefined,
  };
  const drawn = shape === "hexagon" ? cut(HEXAGON, rect, look) : boxed(rect, look);
  const words = one.depth === 1 ? { fill: palette.ink, bold: true } : { fill: palette.soft };
  return `${drawn}${writing({ ...middle, text, size: 13, ...words })}`;
};

const mindPicture = (mind: Mind): Picture => {
  const laid = laidOut(mind);
  return {
    size: laid.size,
    drawn: (palette) =>
      [
        ...laid.lines.map((line) => stroke(curve(line), HUES[line.hue % HUES.length], 2)),
        ...laid.ideas.map((one) => idea(one, palette)),
      ].join(""),
  };
};

const schemaPicture = (schema: Schema): Picture => {
  const drawn = drawnOf(schema);
  return {
    size: drawn.size,
    drawn: (palette) =>
      [
        ...drawn.links.map((one) =>
          stroke(bend(one), palette.accent, 1.6, one.relation.identifying ? undefined : "6 4"),
        ),
        ...drawn.links.map((one) =>
          pill(
            (one.from.x + one.to.x) / 2,
            (one.from.y + one.to.y) / 2,
            relationWords(one.relation),
            palette,
            palette.accent,
          ),
        ),
        ...drawn.boxes.map((box) => {
          const head = { x: box.x, y: box.y, wide: box.wide, tall: HEAD };
          const rows = box.table.columns.map((column, index) => {
            const y = box.y + HEAD + index * ROW + ROW / 2;
            const named = `${column.name}${column.required ? " *" : ""}`;
            const room = box.wide - 56 - column.type.length * 7;
            return [
              writing({
                x: box.x + 12,
                y,
                text: column.keys.join(","),
                size: 10,
                fill: palette.accent,
                bold: true,
                anchor: "start",
              }),
              writing({
                x: box.x + 44,
                y,
                text: fitted(named, room, 7),
                size: 12.5,
                fill: palette.ink,
                anchor: "start",
              }),
              writing({
                x: box.x + box.wide - 12,
                y,
                text: column.type,
                size: 11.5,
                fill: palette.faint,
                anchor: "end",
                mono: true,
              }),
            ].join("");
          });
          return [
            boxed(box, { fill: palette.panel, radius: 8 }),
            boxed(head, { fill: palette.hover }),
            stroke(
              straight({ x: box.x, y: box.y + HEAD }, { x: box.x + box.wide, y: box.y + HEAD }),
              palette.line,
              1,
            ),
            writing({
              x: box.x + 12,
              y: box.y + HEAD / 2,
              text: fitted(box.table.name, box.wide - 24, 8),
              size: 12.5,
              fill: palette.ink,
              bold: true,
              anchor: "start",
            }),
            ...rows,
            boxed(box, { fill: "none", stroke: palette.line, radius: 8 }),
          ].join("");
        }),
      ].join(""),
  };
};

const straightRoute = (from: Rect, to: Rect): Route => {
  const a = edgeOf(from, to);
  const b = edgeOf(to, from);
  return { path: straight(a, b), labelX: (a.x + b.x) / 2, labelY: (a.y + b.y) / 2, from: a, to: b };
};

// Groups behind, then lines, then their words, then the boxes over the ends of the lines.
const graph = (
  scene: Scene,
  palette: Palette,
  box: (id: string, rect: Rect) => string,
  route: (from: Rect, to: Rect) => Route = straightRoute,
) => {
  const { at } = scene.laid;
  const joined = scene.joints.flatMap((one) => {
    const from = at.get(one.from);
    const to = at.get(one.to);
    return from && to ? [{ one, way: route(from, to) }] : [];
  });
  return [
    ...scene.groups.flatMap((group) => [
      boxed(group.rect, { fill: palette.hover, stroke: palette.line, radius: 10, dashed: "5 4" }),
      writing({
        x: group.rect.x + 8,
        y: group.rect.y + 12,
        text: fitted(group.title, group.rect.wide - 16, 6.6),
        size: 11.5,
        fill: palette.soft,
        bold: true,
        anchor: "start",
      }),
    ]),
    ...joined.map(({ one, way }) =>
      stroke(way.path, palette.accent, one.thick ? 2.6 : 1.6, one.dashed ? "6 4" : undefined, {
        start: one.start,
        end: one.end,
      }),
    ),
    ...joined.flatMap(({ one, way }) =>
      one.label ? [pill(way.labelX, way.labelY, one.label, palette, palette.accent)] : [],
    ),
    // A relation's cardinalities sit just off its ends, as on the canvas.
    ...joined.flatMap(({ one, way }) =>
      [
        { said: one.startLabel, end: way.from },
        { said: one.endLabel, end: way.to },
      ].flatMap(({ said, end }) =>
        said
          ? [
              writing({
                x: end.x + 6,
                y: end.y - 10,
                text: said,
                size: 11,
                fill: palette.soft,
                anchor: "start",
              }),
            ]
          : [],
      ),
    ),
    ...[...at].map(([id, rect]) => box(id, rect)),
  ].join("");
};

// The window draws each shape from this same outline, so the picture matches it.
const node = (
  shape: FlowShape,
  rect: Rect,
  palette: Palette,
  look: Painted | undefined,
): string => {
  const outline = outlineOf(shape, rect.wide, rect.tall);
  const line = look?.stroke ?? palette.line;
  const body = outline.body
    ? `<path d="${outline.body}" fill="${look?.fill ?? palette.panel}" stroke="${line}" stroke-width="1"/>`
    : "";
  const lines = outline.lines.map((d) => stroke(d, line, 1)).join("");
  return `<g transform="translate(${rect.x},${rect.y})">${body}${lines}</g>`;
};

const flowPicture = (flow: Flow): Picture => {
  const scene = flowScene(flow);
  const byId = new Map(flow.nodes.map((one) => [one.id, one]));
  const looks = looksOf(flow);
  return {
    size: scene.laid.size,
    drawn: (palette) =>
      graph(
        scene,
        palette,
        (id, rect) => {
          const one = byId.get(id);
          const text = fitted(one?.text ?? id, rect.wide - 24, 7.5);
          const shape = one?.shape ?? "process";
          const drop = outlineOf(shape, rect.wide, rect.tall).drop;
          const middle = { x: rect.x + rect.wide / 2, y: rect.y + rect.tall / 2 + drop };
          const look = looks.get(id);
          const ink = look?.color ?? palette.ink;
          return `${node(shape, rect, palette, look)}${writing({ ...middle, text, size: 13, fill: ink })}`;
        },
        routeOf,
      ),
  };
};

const classPicture = (diagram: Classes): Picture => {
  const scene = classScene(diagram);
  const byName = new Map(diagram.classes.map((one) => [one.name, one]));
  return {
    size: scene.laid.size,
    drawn: (palette) =>
      graph(scene, palette, (name, rect) => {
        const one = byName.get(name);
        const top = CLASS_HEAD + (one?.note ? 14 : 0);
        const center = rect.x + rect.wide / 2;
        const note = one?.note
          ? writing({
              x: center,
              y: rect.y + 12,
              text: `«${one.note}»`,
              size: 10.5,
              fill: palette.faint,
            })
          : "";
        const members = (one?.members ?? []).map((member, index) =>
          writing({
            x: rect.x + 12,
            y: rect.y + top + 6 + index * CLASS_LINE + CLASS_LINE / 2,
            text: fitted(member.text, rect.wide - 24, 6.9),
            size: 11.5,
            fill: palette.soft,
            anchor: "start",
            mono: true,
          }),
        );
        return [
          boxed(rect, { fill: palette.panel, radius: 6 }),
          boxed({ ...rect, tall: top }, { fill: palette.hover }),
          stroke(
            straight({ x: rect.x, y: rect.y + top }, { x: rect.x + rect.wide, y: rect.y + top }),
            palette.line,
            1,
          ),
          note,
          writing({
            x: center,
            y: rect.y + top - CLASS_HEAD / 2,
            text: fitted(name, rect.wide - 24, 7.8),
            size: 13,
            fill: palette.ink,
            bold: true,
          }),
          ...members,
          boxed(rect, { fill: "none", stroke: palette.line, radius: 6 }),
        ].join("");
      }),
  };
};

// Null for the kinds the window leaves to Mermaid, whose own drawing is the one people see.
export const pictureOf = (source: string): Picture | null => {
  const kind = kindOf(source);
  if (kind === "mindmap") {
    const mind = readMind(source);
    return mind && mindPicture(mind);
  }
  if (kind === "erDiagram") {
    const schema = readSchema(source);
    return schema && schemaPicture(schema);
  }
  if (kind === "flowchart") {
    const flow = readFlow(source);
    // Only Mermaid draws icons and images.
    return flow && !flow.nodes.some((one) => pictured(one.keys)) ? flowPicture(flow) : null;
  }
  if (kind === "classDiagram") {
    const diagram = readClasses(source);
    return diagram && classPicture(diagram);
  }
  return null;
};
