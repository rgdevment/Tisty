import { Document, Font, Image, Link, Page, StyleSheet, Text, View } from "@react-pdf/renderer";
import type { Paper } from "../core";

export const SIZES: Record<Paper, [number, number]> = {
  a4: [595.28, 841.89],
  letter: [612, 792],
  tabloid: [792, 1224],
};

export const MARGIN = 50;
export const TYPE = 10.5;
export const LEADING = 1.55;

export interface Run {
  text: string;
  bold?: boolean;
  italic?: boolean;
  code?: boolean;
  href?: string;
}

export type Shape =
  | { kind: "heading"; level: number; runs: Run[] }
  | { kind: "para"; runs: Run[] }
  | { kind: "quote"; runs: Run[] }
  | { kind: "code"; runs: Run[] }
  | { kind: "bullet"; mark: string; runs: Run[]; deep: number }
  | { kind: "image"; src: string; alt?: string }
  | { kind: "table"; rows: Run[][][] }
  | { kind: "rule" };

const sheet = StyleSheet.create({
  page: {
    paddingTop: MARGIN,
    paddingBottom: MARGIN,
    paddingHorizontal: MARGIN,
    fontSize: TYPE,
  },
  h1: { fontSize: 20, marginBottom: 10, marginTop: 4 },
  h2: { fontSize: 15, marginBottom: 7, marginTop: 14 },
  h3: { fontSize: 12.5, marginBottom: 5, marginTop: 12 },
  para: { marginBottom: 9, lineHeight: LEADING },
  quote: {
    marginBottom: 9,
    paddingLeft: 12,
    borderLeftWidth: 2,
    borderLeftColor: "#a1a1aa",
    color: "#3f3f46",
    lineHeight: 1.55,
  },
  code: {
    marginBottom: 9,
    padding: 9,
    backgroundColor: "#f4f4f5",
    fontFamily: "Courier",
    fontSize: 9,
    lineHeight: 1.4,
  },
  row: { flexDirection: "row", marginBottom: 4 },
  mark: { width: 16 },
  image: { marginVertical: 10, width: "100%", maxHeight: 360, objectFit: "contain" },
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

const drawn = (runs: Run[]) =>
  runs.map((run, at) => {
    const style = [
      run.bold ? dressed.bold : null,
      run.italic ? dressed.italic : null,
      run.code ? sheet.inline : null,
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

const shaped = (one: Shape, at: number) => {
  const key = `${one.kind}:${at}`;
  switch (one.kind) {
    case "heading":
      return (
        <Text key={key} style={one.level === 1 ? sheet.h1 : one.level === 2 ? sheet.h2 : sheet.h3}>
          {drawn(one.runs)}
        </Text>
      );
    case "quote":
      return (
        <Text key={key} style={sheet.quote}>
          {drawn(one.runs)}
        </Text>
      );
    case "code":
      return (
        <View key={key} style={sheet.code}>
          {one.runs.map((run) => (
            <Text key={`${run.text}`}>{run.text}</Text>
          ))}
        </View>
      );
    case "bullet":
      return (
        <View key={key} style={[sheet.row, { paddingLeft: one.deep * 14 }]} wrap={false}>
          <Text style={sheet.mark}>{one.mark}</Text>
          <Text style={{ flex: 1, lineHeight: 1.5 }}>{drawn(one.runs)}</Text>
        </View>
      );
    case "image":
      return one.src ? (
        <Image key={key} style={sheet.image} src={one.src} />
      ) : (
        <Text key={key} style={sheet.missing}>
          {one.alt || "?"}
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
      return (
        <Text key={key} style={sheet.para}>
          {drawn(one.runs)}
        </Text>
      );
  }
};

export const Papered = ({ shapes, leaf }: { shapes: Shape[]; leaf: Paper }) => {
  const size = SIZES[leaf];

  return (
    <Document>
      <Page size={size} style={sheet.page}>
        {shapes.map(shaped)}
      </Page>
    </Document>
  );
};

export const registered = (): void => {
  Font.registerHyphenationCallback((word) => [word]);
};
