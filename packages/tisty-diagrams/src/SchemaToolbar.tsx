import { alignedMoves } from "./flowtools";
import type { Rect } from "./graphlayout";
import { schemaRelaid, tablesPlaced } from "./schemaplaces";
import type { Altered } from "./scheming";
import { AlignTools, Gap, RemoveTools, ToolBar, UndoTools, ViewTools } from "./tools";
import type { Scheming } from "./useSchema";
import { chosenTables } from "./useSchemaCanvas";

/** What can be done to the schema and to what is chosen in it, above the canvas. */
export default function SchemaToolbar({
  editing,
  laid,
  removal,
}: Readonly<{
  editing: Scheming;
  laid: ReadonlyMap<string, Rect>;
  removal: Altered | null;
}>) {
  const { schema, picked, change, undo, redo } = editing;
  const names = chosenTables(picked);

  return (
    <ToolBar>
      <UndoTools undo={undo} redo={redo} />
      <Gap />
      <RemoveTools
        removable={removal !== null}
        refused={undefined}
        onRemove={() => removal && change(removal)}
      />
      <Gap />
      <AlignTools
        disabled={names.length < 2}
        onAlign={(how) =>
          change(
            tablesPlaced(
              schema,
              alignedMoves(laid, names, how).map((one) => ({ name: one.id, x: one.x, y: one.y })),
              picked,
            ),
          )
        }
      />
      <Gap />
      <ViewTools
        laidOut={schema.tables.length > 0}
        onRelay={() => change(schemaRelaid(schema, picked))}
      />
    </ToolBar>
  );
}
