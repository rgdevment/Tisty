import { type Classes, readClasses } from "./classes";
import { type Flow, readFlow } from "./flow";
import { LINE, linesOf, outlineOf } from "./flowoutlines";
import { endWordAt, endWordBox, type Route, routesOf } from "./flowroutes";
import { type Shape as FlowShape, pictured } from "./flowshapes";
import { looksOf, type Look as Painted } from "./flowstyles";
import { kindOf } from "./framing";
import type { Rect } from "./graphlayout";
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
  pillSize,
  straight,
  stroke,
  writing,
} from "./picture";
import { CLASS_HEAD, CLASS_LINE, classScene, flowScene, type Scene } from "./scenes";
import { readSchema, type Schema } from "./schema";
import { HEAD, ROW, schemaScene } from "./schemalayout";

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
  const scene = schemaScene(schema);
  const byName = new Map(schema.tables.map((one) => [one.name, one]));
  return framedPicture(scene, {
    size: scene.laid.size,
    drawn: (palette) =>
      graph(scene, palette, (name, box) => {
        const rows = (byName.get(name)?.columns ?? []).map((column, index) => {
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
          boxed({ ...box, tall: HEAD }, { fill: palette.hover }),
          stroke(
            straight({ x: box.x, y: box.y + HEAD }, { x: box.x + box.wide, y: box.y + HEAD }),
            palette.line,
            1,
          ),
          writing({
            x: box.x + 12,
            y: box.y + HEAD / 2,
            text: fitted(name, box.wide - 24, 8),
            size: 12.5,
            fill: palette.ink,
            bold: true,
            anchor: "start",
          }),
          ...rows,
          boxed(box, { fill: "none", stroke: palette.line, radius: 8 }),
        ].join("");
      }),
  });
};

const routed = new WeakMap<Scene, { one: Scene["joints"][number]; way: Route }[]>();

const joinedIn = (scene: Scene) => {
  const kept = routed.get(scene);
  if (kept) return kept;
  const { at } = scene.laid;
  const known = scene.joints.flatMap((one) => {
    const from = at.get(one.from);
    const to = at.get(one.to);
    return from && to ? [{ one, from, to }] : [];
  });
  const routes = routesOf(
    known.map(({ one, from, to }) => ({
      from,
      to,
      label: one.label,
      startLabel: one.startLabel,
      endLabel: one.endLabel,
      fromMiddle: one.fromMiddle,
      toMiddle: one.toMiddle,
      fromKey: one.from,
      toKey: one.to,
    })),
    [...at.values()],
  );
  const joined = known.map(({ one }, index) => ({ one, way: routes[index] }));
  routed.set(scene, joined);
  return joined;
};

const PICTURE_ROOM = 8;

const labelBox = (way: Route, said: string): Rect => {
  const { wide, tall } = pillSize(said);
  return { x: way.labelX - wide / 2, y: way.labelY - tall / 2, wide, tall };
};

export const pictureBounds = (scene: Scene): Rect => {
  const boxes = [
    ...scene.laid.at.values(),
    ...scene.groups.map((group) => group.rect),
    ...joinedIn(scene).flatMap(({ one, way }) => [
      ...way.way.map((point) => ({ ...point, wide: 0, tall: 0 })),
      ...(one.label ? [labelBox(way, one.label)] : []),
      ...(one.startLabel ? [endWordBox(way.from, way.fromSide, one.startLabel)] : []),
      ...(one.endLabel ? [endWordBox(way.to, way.toSide, one.endLabel)] : []),
    ]),
  ];
  const left = Math.min(0, ...boxes.map((one) => one.x - PICTURE_ROOM));
  const top = Math.min(0, ...boxes.map((one) => one.y - PICTURE_ROOM));
  const right = Math.max(
    scene.laid.size.wide,
    ...boxes.map((one) => one.x + one.wide + PICTURE_ROOM),
  );
  const bottom = Math.max(
    scene.laid.size.tall,
    ...boxes.map((one) => one.y + one.tall + PICTURE_ROOM),
  );
  return { x: left, y: top, wide: right - left, tall: bottom - top };
};

// An arrow stepping aside or a long word may reach past the shapes' margin: the picture grows rather than cut it.
const framedPicture = (scene: Scene, picture: Picture): Picture => {
  const bounds = pictureBounds(scene);
  const same =
    !bounds.x &&
    !bounds.y &&
    bounds.wide === picture.size.wide &&
    bounds.tall === picture.size.tall;
  if (same) return picture;
  return {
    size: { wide: bounds.wide, tall: bounds.tall },
    drawn: (palette) =>
      `<g transform="translate(${-bounds.x},${-bounds.y})">${picture.drawn(palette)}</g>`,
  };
};

// Groups behind, then lines, then their words, then the boxes over the ends of the lines.
const graph = (scene: Scene, palette: Palette, box: (id: string, rect: Rect) => string) => {
  const { at } = scene.laid;
  const joined = joinedIn(scene);
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
        { said: one.startLabel, end: endWordAt(way.from, way.fromSide) },
        { said: one.endLabel, end: endWordAt(way.to, way.toSide) },
      ].flatMap(({ said, end }) =>
        said ? [writing({ ...end, text: said, size: 11, fill: palette.soft })] : [],
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
  return framedPicture(scene, {
    size: scene.laid.size,
    drawn: (palette) =>
      graph(scene, palette, (id, rect) => {
        const one = byId.get(id);
        const lines = linesOf(one?.text ?? id);
        const shape = one?.shape ?? "process";
        const drop = outlineOf(shape, rect.wide, rect.tall).drop;
        const top = rect.y + rect.tall / 2 + drop - ((lines.length - 1) * LINE) / 2;
        const look = looks.get(id);
        const ink = look?.color ?? palette.ink;
        const words = lines.map((line, at) =>
          writing({
            x: rect.x + rect.wide / 2,
            y: top + at * LINE,
            text: fitted(line, rect.wide - 24, 7.5),
            size: 13,
            fill: ink,
          }),
        );
        return `${node(shape, rect, palette, look)}${words.join("")}`;
      }),
  });
};

const classPicture = (diagram: Classes): Picture => {
  const scene = classScene(diagram);
  const byName = new Map(diagram.classes.map((one) => [one.name, one]));
  return framedPicture(scene, {
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
  });
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
