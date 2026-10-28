import { useMemo } from "react";
import type { WireEdge } from "./FlowWire";
import { useSay } from "./handing";
import { type TableNode, tableHandles } from "./SchemaNodes";
import { schemaJointsOf, schemaLaid } from "./schemalayout";
import { tablesPlaced } from "./schemaplaces";
import { type Picked, relationChanged, tableRenamed } from "./scheming";
import { useMoving } from "./useMoving";
import type { Scheming } from "./useSchema";

export type SchemaNaming =
  | { kind: "table"; name: string }
  | { kind: "relation"; index: number }
  | null;

/** The tables chosen, one or several. */
export const chosenTables = (picked: Picked): readonly string[] => {
  if (picked?.kind === "table") return [picked.name];
  return picked?.kind === "tables" ? picked.names : [];
};

// A schema has no lanes; its tables are dragged alone or together.
const NO_LANES = new Map();

/** What React Flow draws of a schema, and its tables as they are dragged. */
export const useSchemaCanvas = (
  editing: Scheming,
  naming: SchemaNaming,
  setNaming: (naming: SchemaNaming) => void,
) => {
  const t = useSay();
  const { schema, picked, change, pick } = editing;
  const still = useMemo(() => schemaLaid(schema), [schema]);
  const { moving, laid, moved } = useMoving({
    still,
    lanes: NO_LANES,
    groupOf: () => null,
    chosen: chosenTables(picked),
    choose: (choice) =>
      pick(
        "several" in choice
          ? { kind: "tables", names: choice.several }
          : { kind: "table", name: choice.one },
      ),
    settle: (spots) => {
      const moves = [...spots].map(([name, spot]) => ({ name, x: spot.x, y: spot.y }));
      const kept: Picked =
        picked?.kind === "tables" ? picked : { kind: "table", name: moves[0]?.name ?? "" };
      change(tablesPlaced(schema, moves, kept));
    },
    resize: () => {},
  });

  // The same objects while another table is dragged, so only the one moving is drawn again.
  const placedCards = useMemo(
    (): TableNode[] =>
      schema.tables.flatMap((table) => {
        const rect = still.get(table.name);
        if (!rect) return [];
        return [
          {
            id: table.name,
            type: "table",
            position: { x: rect.x, y: rect.y },
            width: rect.wide,
            height: rect.tall,
            handles: tableHandles(rect.wide, rect.tall, table.columns.length),
            selected: chosenTables(picked).includes(table.name),
            data: {
              table,
              wide: rect.wide,
              tall: rect.tall,
              column:
                picked?.kind === "column" && picked.table === table.name ? picked.index : null,
              naming: naming?.kind === "table" && naming.name === table.name,
              label: t("schemaName"),
              linkTitle: t("schemaLink"),
              onPick: () => pick({ kind: "table", name: table.name }),
              onPickColumn: (index) => pick({ kind: "column", table: table.name, index }),
              onName: () => setNaming({ kind: "table", name: table.name }),
              onNamed: (text) => {
                if (text !== null && text.trim() !== table.name)
                  change(tableRenamed(schema, table.name, text));
                setNaming(null);
              },
            },
          },
        ];
      }),
    [schema, still, picked, naming, t, pick, change, setNaming],
  );
  const cards = placedCards.map((one) => {
    const spot = moving.get(one.id);
    return spot ? { ...one, position: { x: spot.x, y: spot.y } } : one;
  });
  const wires = useMemo(
    (): WireEdge[] =>
      schemaJointsOf(schema).map((one, index) => ({
        id: one.key,
        source: one.from,
        target: one.to,
        type: "wire",
        selected: picked?.kind === "relation" && picked.index === index,
        data: {
          label: one.label,
          title: `${one.from} → ${one.to}`,
          dashed: one.dashed,
          thick: false,
          startLabel: one.startLabel,
          endLabel: one.endLabel,
          naming: naming?.kind === "relation" && naming.index === index,
          field: t("schemaLabel"),
          onPick: () => pick({ kind: "relation", index }),
          onName: () => setNaming({ kind: "relation", index }),
          onNamed: (label) => {
            if (label !== null && label.trim() !== one.label)
              change(relationChanged(schema, index, { label: label.trim() }));
            setNaming(null);
          },
        },
      })),
    [schema, picked, naming, t, pick, change, setNaming],
  );

  return { nodes: cards, edges: wires, laid, moved };
};
