import { Document, Font, Image, Link, Page, StyleSheet, Text, View } from "@react-pdf/renderer";
import type { Paper } from "../core";
import { led } from "../leading";

export const SIZES: Record<Paper, [number, number]> = {
  a4: [595.28, 841.89],
  letter: [612, 792],
  tabloid: [792, 1224],
};

export const MARGIN = 50;
export const CODE = 9;
const PITCH = 0.6;
const PAD = 9;
export const TYPE = 10.5;
export const LEADING = 1.4;

export interface Run {
  text: string;
  bold?: boolean;
  italic?: boolean;
  code?: boolean;
  lit?: string;
  href?: string;
}

export type Shape =
  | { kind: "heading"; level: number; runs: Run[] }
  | { kind: "para"; runs: Run[] }
  | { kind: "quote"; runs: Run[] }
  | { kind: "code"; runs: Run[]; deep: number }
  | { kind: "bullet"; mark: string; runs: Run[]; deep: number }
  | { kind: "image"; src: string; alt?: string }
  | { kind: "file"; name: string; said: string }
  | { kind: "table"; rows: Run[][][] }
  | { kind: "rule" };

const sheet = StyleSheet.create({
  page: {
    paddingTop: MARGIN,
    paddingBottom: MARGIN,
    paddingHorizontal: MARGIN,
    fontSize: TYPE,
  },
  h1: { fontSize: 20, fontWeight: 700, marginBottom: 10, marginTop: 4 },
  h2: { fontSize: 15, fontWeight: 700, marginBottom: 7, marginTop: 14 },
  h3: { fontSize: 12.5, fontWeight: 700, marginBottom: 5, marginTop: 12 },
  title: {
    paddingBottom: 10,
    marginBottom: 14,
    borderBottomWidth: 1,
    borderBottomColor: "#e4e4e7",
  },
  para: { marginBottom: 9, lineHeight: LEADING },
  flow: { marginBottom: 9, flexDirection: "row", flexWrap: "wrap" },
  piece: { lineHeight: LEADING },
  said: { flex: 1, lineHeight: LEADING },
  saidFlow: { flex: 1, flexDirection: "row", flexWrap: "wrap" },
  quote: {
    marginBottom: 9,
    paddingLeft: 12,
    borderLeftWidth: 2,
    borderLeftColor: "#a1a1aa",
    color: "#3f3f46",
    lineHeight: LEADING,
  },
  code: {
    marginBottom: 9,
    padding: 9,
    backgroundColor: "#f4f4f5",
    borderRadius: 6,
    fontFamily: "Courier",
    fontSize: CODE,
    lineHeight: 1.4,
  },
  row: { flexDirection: "row", marginBottom: 4 },
  mark: { width: 20 },
  frame: {
    marginVertical: 10,
    padding: 7,
    backgroundColor: "#fafafa",
    borderWidth: 1,
    borderColor: "#e4e4e7",
    borderRadius: 12,
  },
  image: {
    width: "100%",
    maxHeight: 360,
    objectFit: "contain",
  },
  link: { color: "#1d4ed8", textDecoration: "underline" },
  inline: { fontFamily: "Courier", fontSize: 9.5, backgroundColor: "#f4f4f5" },
  table: {
    marginBottom: 10,
    borderWidth: 1,
    borderColor: "#d4d4d8",
    borderRightWidth: 0,
    borderBottomWidth: 0,
  },
  tr: { flexDirection: "row" },
  td: {
    flex: 1,
    padding: 5,
    borderRightWidth: 1,
    borderBottomWidth: 1,
    borderColor: "#d4d4d8",
    fontSize: 9.5,
    lineHeight: 1.35,
  },
  th: { backgroundColor: "#f4f4f5", fontWeight: 700 },
  card: {
    marginVertical: 8,
    padding: "9 11",
    borderWidth: 1,
    borderColor: "#d4d4d8",
    borderRadius: 6,
    backgroundColor: "#fafafa",
    flexDirection: "row",
    alignItems: "center",
  },
  cardLeaf: {
    width: 20,
    height: 25,
    marginRight: 10,
    padding: 3,
    borderWidth: 1,
    borderColor: "#a1a1aa",
    borderRadius: 2,
    backgroundColor: "#ffffff",
    justifyContent: "center",
  },
  cardLine: { height: 1.4, backgroundColor: "#c4c4c8", marginVertical: 1.4 },
  cardName: { fontSize: 10.5, fontWeight: 700 },
  cardSaid: { fontSize: 9, color: "#71717a", marginTop: 2 },
  foot: {
    position: "absolute",
    bottom: 24,
    left: MARGIN,
    right: MARGIN,
    textAlign: "center",
    fontSize: 8.5,
    color: "#a1a1aa",
  },
  missing: {
    marginVertical: 8,
    padding: 8,
    borderWidth: 1,
    borderColor: "#d4d4d8",
    borderStyle: "dashed",
    color: "#71717a",
    fontSize: 9,
  },
});

const dressed = StyleSheet.create({
  bold: { fontWeight: 700 },
  italic: { fontStyle: "italic" },
});

const PENS: Record<string, string> = {
  yellow: "#fdf0c3",
  green: "#d5f0e0",
  blue: "#d8e8fb",
  pink: "#f9dcea",
};

const pens = StyleSheet.create(
  Object.fromEntries(Object.entries(PENS).map(([name, tint]) => [name, { backgroundColor: tint }])),
);

const WORDS = /\S+\s*/g;
const SEAMS = /[^/\-_.=&?:]*[/\-_.=&?:]?/g;
const LONG = 40;

const torn = (runs: Run[]): Run[] =>
  runs.flatMap((run) => {
    const cuts =
      run.href && run.text.length > LONG
        ? (run.text.match(SEAMS) ?? [])
        : (run.text.match(WORDS) ?? []);
    return cuts.filter(Boolean).map((text) => ({ ...run, text }));
  });

const stringy = (runs: Run[]): boolean =>
  runs.some((run) => Boolean(run.href) && run.text.length > LONG);

const MARKS: Record<string, string> = { "☑": "[x]", "☐": "[ ]" };

const folded = (text: string, columns: number): string[] => {
  const lines: string[] = [];
  let rest = text;
  while (rest.length > columns) {
    const slice = rest.slice(0, columns);
    const seam = Math.max(slice.lastIndexOf("/"), slice.lastIndexOf("-"), slice.lastIndexOf(" "));
    const at = seam > columns / 2 ? seam + 1 : columns;
    lines.push(rest.slice(0, at));
    rest = rest.slice(at);
  }
  lines.push(rest);
  return lines;
};

const drawn = (runs: Run[], flowing = false) =>
  runs.map((run, at) => {
    const style = [
      flowing ? sheet.piece : null,
      run.bold ? dressed.bold : null,
      run.italic ? dressed.italic : null,
      run.code ? sheet.inline : null,
      run.lit ? (pens[run.lit] ?? pens.yellow) : null,
      run.href ? sheet.link : null,
    ].filter((one) => one !== null);

    const key = `${at}:${run.text.slice(0, 12)}`;
    return run.href ? (
      <Link key={key} src={run.href} style={style}>
        {run.text}
      </Link>
    ) : (
      <Text key={key} style={style}>
        {run.text}
      </Text>
    );
  });

// No font here draws an emoji, so a title wearing one would print a hollow box.
const plain = (runs: Run[]): Run[] => {
  const [first, ...rest] = runs;
  if (!first) return runs;
  const worn = led(first.text);
  return worn.mark ? [{ ...first, text: worn.rest }, ...rest] : runs;
};

const shaped = (one: Shape, at: number, room: number) => {
  const key = `${one.kind}:${at}`;
  switch (one.kind) {
    case "heading": {
      const rank = one.level === 1 ? sheet.h1 : one.level === 2 ? sheet.h2 : sheet.h3;
      return (
        <Text key={key} style={at === 0 ? [rank, sheet.title] : rank}>
          {drawn(at === 0 ? plain(one.runs) : one.runs)}
        </Text>
      );
    }
    case "quote":
      return (
        <Text key={key} style={sheet.quote}>
          {drawn(one.runs)}
        </Text>
      );
    case "code": {
      const columns = Math.floor((room - one.deep * 14 - PAD * 2) / (CODE * PITCH));
      const lines = one.runs.flatMap((run, line) =>
        folded(run.text, columns).map((text, cut) => ({ text, id: `${line}.${cut}` })),
      );
      return (
        <View key={key} style={[sheet.code, { marginLeft: one.deep * 14 }]}>
          {lines.map((line) => (
            <Text key={line.id}>{line.text}</Text>
          ))}
        </View>
      );
    }
    case "bullet":
      return (
        <View key={key} style={[sheet.row, { paddingLeft: one.deep * 14 }]} wrap={false}>
          <Text style={sheet.mark}>{MARKS[one.mark] ?? one.mark}</Text>
          {stringy(one.runs) ? (
            <View style={sheet.saidFlow}>{drawn(torn(one.runs), true)}</View>
          ) : (
            <Text style={sheet.said}>{drawn(one.runs)}</Text>
          )}
        </View>
      );
    case "file":
      return (
        <View key={key} style={sheet.card} wrap={false}>
          <View style={sheet.cardLeaf}>
            <View style={sheet.cardLine} />
            <View style={sheet.cardLine} />
            <View style={[sheet.cardLine, { width: "60%" }]} />
          </View>
          <View>
            <Text style={sheet.cardName}>{one.name}</Text>
            <Text style={sheet.cardSaid}>{one.said}</Text>
          </View>
        </View>
      );
    case "image":
      return one.src ? (
        <View key={key} style={sheet.frame} wrap={false}>
          <Image style={sheet.image} src={one.src} />
        </View>
      ) : (
        <Text key={key} style={sheet.missing}>
          {`◻ ${one.alt || "?"}`}
        </Text>
      );
    case "table":
      return (
        <View key={key} style={sheet.table}>
          {one.rows.map((row, at) => {
            const said = row.map((cell) => cell.map((run) => run.text).join("|")).join("¦");
            return (
              <View key={said} style={sheet.tr} wrap={false}>
                {row.map((cell) => (
                  <Text
                    key={`${said}:${cell.map((run) => run.text).join("")}`}
                    style={at === 0 ? [sheet.td, sheet.th] : sheet.td}
                  >
                    {drawn(cell)}
                  </Text>
                ))}
              </View>
            );
          })}
        </View>
      );
    case "rule":
      return <View key={key} break />;
    default:
      return stringy(one.runs) ? (
        <View key={key} style={sheet.flow}>
          {drawn(torn(one.runs), true)}
        </View>
      ) : (
        <Text key={key} style={sheet.para}>
          {drawn(one.runs)}
        </Text>
      );
  }
};

export const Papered = ({ sheets, leaf }: { sheets: Shape[][]; leaf: Paper }) => {
  const size = SIZES[leaf];

  return (
    <Document>
      {sheets.map((shapes, sheet_at) => (
        <Page key={`sheet:${sheet_at}`} size={size} style={sheet.page}>
          {shapes.map((one, at) => shaped(one, at, size[0] - MARGIN * 2))}
          <Text
            fixed
            style={sheet.foot}
            render={({ pageNumber, totalPages }) => (totalPages > 1 ? `${pageNumber}` : "")}
          />
        </Page>
      ))}
    </Document>
  );
};

export const registered = (): void => {
  Font.registerHyphenationCallback((word) => [word]);
};
