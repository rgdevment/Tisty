import { HEXAGON } from "./mindlayout";
import type { Shape } from "./mindmap";
import { IDEA_LINES } from "./mindshapes";

/** A shape of an idea drawn small, with the same lines the map draws the idea with. */
export function MindSketch({ shape }: Readonly<{ shape: Shape }>) {
  return (
    <span
      aria-hidden="true"
      className={`flex h-[18px] w-[30px] items-center justify-center border-current ${IDEA_LINES[shape]}`}
      style={shape === "hexagon" ? { clipPath: HEXAGON } : undefined}
    >
      {shape === "plain" && <span className="text-[10px] leading-none">Aa</span>}
    </span>
  );
}
