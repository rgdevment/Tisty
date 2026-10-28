import type { ReactNode } from "react";
import ClassCanvas from "./ClassCanvas";
import ClassPanel from "./ClassPanel";
import FlowCanvas from "./FlowCanvas";
import FlowPanel from "./FlowPanel";
import { useSay } from "./handing";
import IdeaMap from "./IdeaMap";
import IdeaPanel from "./IdeaPanel";
import { renamed, reshaped, withChild, without, withSibling } from "./minding";
import SchemaCanvas from "./SchemaCanvas";
import SchemaPanel from "./SchemaPanel";
import { useClasses } from "./useClasses";
import { useFlow } from "./useFlow";
import { useMind } from "./useMind";
import { useSchema } from "./useSchema";

export interface Editor {
  readonly board: ReactNode;
  readonly panel: ReactNode;
}

const scrolled = (panel: ReactNode) => (
  <div className="scroller max-h-[60%] shrink-0 overflow-auto border-b border-hair pb-3">
    {panel}
  </div>
);

/** The canvas and panel for the kind the text is, while the window can keep it; `null` leaves Mermaid's drawing. */
export const useEditor = (
  text: string,
  setText: (text: string) => void,
  dark: boolean,
): Editor | null => {
  const t = useSay();
  const map = useMind(text, setText);
  const scheme = useSchema(text, setText);
  const flow = useFlow(text, setText);
  const classes = useClasses(text, setText);
  if (map) {
    const fresh = t("diagramNewIdea");
    return {
      board: (
        <IdeaMap
          mind={map.mind}
          chosen={map.chosen}
          fresh={fresh}
          undo={map.undo}
          redo={map.redo}
          onChoose={map.choose}
          onChange={map.change}
        />
      ),
      panel: (
        <IdeaPanel
          idea={map.idea}
          root={map.chosen.length === 0}
          onRename={(words) => map.change(renamed(map.mind, map.chosen, words))}
          onShape={(shape) => map.change(reshaped(map.mind, map.chosen, shape))}
          onChild={() => map.change(withChild(map.mind, map.chosen, fresh))}
          onSibling={() => map.change(withSibling(map.mind, map.chosen, fresh))}
          onRemove={() => map.change(without(map.mind, map.chosen))}
        />
      ),
    };
  }
  if (scheme)
    return {
      board: <SchemaCanvas editing={scheme} dark={dark} />,
      panel: scrolled(
        <SchemaPanel schema={scheme.schema} picked={scheme.picked} onChange={scheme.change} />,
      ),
    };
  if (flow)
    return {
      board: <FlowCanvas editing={flow} dark={dark} />,
      panel: scrolled(<FlowPanel editing={flow} />),
    };
  if (classes)
    return {
      board: <ClassCanvas editing={classes} dark={dark} />,
      panel: scrolled(<ClassPanel editing={classes} />),
    };
  return null;
};
