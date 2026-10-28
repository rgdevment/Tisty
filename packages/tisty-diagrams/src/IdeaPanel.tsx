import { useEffect, useState } from "react";
import { useHands, useSay } from "./handing";
import { MindSketch } from "./MindSketch";
import { type Idea, SHAPES, type Shape } from "./mindmap";
import { SHAPE_WORDS } from "./mindshapes";

/** The chosen idea's words, shape and the ways to grow or prune the map from it. */
export default function IdeaPanel({
  idea,
  root,
  onRename,
  onShape,
  onChild,
  onSibling,
  onRemove,
}: Readonly<{
  idea: Idea;
  root: boolean;
  onRename: (text: string) => void;
  onShape: (shape: Shape) => void;
  onChild: () => void;
  onSibling: () => void;
  onRemove: () => void;
}>) {
  const t = useSay();
  const { button } = useHands().looks;
  // The field may be empty while the person retypes it; the map keeps its last words meanwhile.
  const [draft, setDraft] = useState(idea.text);
  useEffect(() => setDraft((was) => (was.trim() ? idea.text : was)), [idea.text]);
  return (
    <div className="flex flex-col gap-2.5 border-b border-hair pb-4">
      <label htmlFor="diagram-idea" className="text-[12px] font-semibold text-soft">
        {t("diagramIdea")}
      </label>
      <input
        id="diagram-idea"
        value={draft}
        onChange={(e) => {
          setDraft(e.target.value);
          if (e.target.value.trim()) onRename(e.target.value);
        }}
        className="rounded-md border border-line bg-desk px-2.5 py-2 text-[13px] text-ink outline-none focus:border-accent"
      />
      <div className="flex gap-1.5">
        <button type="button" className={`${button} flex-1`} onClick={onChild}>
          {t("diagramChild")}
        </button>
        <button type="button" className={`${button} flex-1`} onClick={onSibling}>
          {t("diagramSibling")}
        </button>
        <button
          type="button"
          className={`${button} text-urgent`}
          disabled={root}
          onClick={onRemove}
        >
          {t("diagramRemove")}
        </button>
      </div>
      <span className="text-[12px] font-semibold text-soft">{t("diagramShape")}</span>
      <div className="flex flex-wrap gap-1.5">
        {SHAPES.map((shape) => (
          <button
            key={shape}
            type="button"
            aria-pressed={idea.shape === shape}
            aria-label={t(SHAPE_WORDS[shape])}
            title={t(SHAPE_WORDS[shape])}
            className={`${button} ${idea.shape === shape ? "border-accent text-accent" : ""}`}
            onClick={() => onShape(shape)}
          >
            <MindSketch shape={shape} />
          </button>
        ))}
      </div>
    </div>
  );
}
