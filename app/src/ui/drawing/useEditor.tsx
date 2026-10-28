import type { ReactNode } from "react";
import { t } from "../../locales";
import ClassPanel from "./ClassPanel";
import ClassView from "./ClassView";
import FlowPanel from "./FlowPanel";
import FlowView from "./FlowView";
import IdeaMap from "./IdeaMap";
import IdeaPanel from "./IdeaPanel";
import { renamed, reshaped, withChild, without, withSibling } from "./minding";
import SchemaBoard from "./SchemaBoard";
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
export const useEditor = (text: string, setText: (text: string) => void): Editor | null => {
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
      board: (
        <SchemaBoard
          schema={scheme.schema}
          picked={scheme.picked}
          fresh={t("schemaNewTable")}
          onPick={scheme.pick}
          onChange={scheme.change}
        />
      ),
      panel: scrolled(
        <SchemaPanel schema={scheme.schema} picked={scheme.picked} onChange={scheme.change} />,
      ),
    };
  if (flow)
    return { board: <FlowView editing={flow} />, panel: scrolled(<FlowPanel editing={flow} />) };
  if (classes)
    return {
      board: <ClassView editing={classes} />,
      panel: scrolled(<ClassPanel editing={classes} />),
    };
  return null;
};
