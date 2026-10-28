import {
  type ClassAltered,
  classesPinned,
  classesPlaced,
  kindKept,
  linkChanged,
  sameKind,
} from "./classing";
import { KINDS } from "./classkinds";
import { classesRelaid } from "./classspaces";
import { nodeStyled } from "./flowstyles";
import { alignedMoves } from "./flowtools";
import type { Rect } from "./graphlayout";
import { useSay } from "./handing";
import {
  AlignTools,
  Gap,
  PaintTools,
  RemoveTools,
  Tool,
  ToolBar,
  UndoTools,
  ViewTools,
} from "./tools";
import { chosenClasses } from "./useClassCanvas";
import type { Classing } from "./useClasses";

/** What can be done to the class diagram and to what is chosen in it, above the canvas. */
export default function ClassToolbar({
  editing,
  laid,
  removal,
}: Readonly<{
  editing: Classing;
  laid: ReadonlyMap<string, Rect>;
  removal: ClassAltered | null;
}>) {
  const t = useSay();
  const { diagram, picked, change, undo, redo } = editing;
  const names = chosenClasses(picked);
  const link = picked?.kind === "link" ? diagram.links[picked.index] : undefined;

  return (
    <ToolBar>
      <UndoTools undo={undo} redo={redo} />
      <Gap />
      <RemoveTools
        removable={removal !== null}
        refused={undefined}
        onRemove={() => removal && change(removal)}
        onDuplicate={null}
      />
      <Gap />
      <PaintTools
        disabled={!names.length}
        onStyle={(look) => {
          const body = names.reduce(
            (lines, name) => nodeStyled({ body: lines }, name, look).body,
            diagram.body,
          );
          change({ body, places: diagram.places, picked });
        }}
      />
      <Gap />
      {KINDS.filter(([arrow]) => arrow !== "..").map(([arrow, word, icon]) => (
        <Tool
          key={arrow}
          label={t(word)}
          icon={icon}
          disabled={!link}
          pressed={link ? sameKind(link.arrow, arrow) : undefined}
          onClick={() =>
            link &&
            picked?.kind === "link" &&
            change(linkChanged(diagram, picked.index, { arrow: kindKept(link.arrow, arrow) }))
          }
        />
      ))}
      <Gap />
      <AlignTools
        disabled={names.length < 2}
        onAlign={(how) =>
          change(
            classesPlaced(
              classesPinned(diagram, laid),
              alignedMoves(laid, names, how).map((one) => ({ name: one.id, x: one.x, y: one.y })),
              picked,
            ),
          )
        }
      />
      <Gap />
      <ViewTools
        laidOut={diagram.classes.length > 0}
        onRelay={() => change(classesRelaid(diagram))}
      />
    </ToolBar>
  );
}
