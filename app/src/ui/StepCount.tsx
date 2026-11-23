import type { Volume } from "../core";
import { stepsWaiting } from "./stepsWaiting";

/// Ticked of all, and apart the steps an agent said done that still wait on the person.
export default function StepCount({ volume }: { volume: Volume }) {
  if (!volume.steps) return null;
  const said = volume.steps_said ?? 0;
  return (
    <>
      {`${volume.steps_done ?? 0}/${volume.steps}`}
      {said > 0 && (
        <span title={stepsWaiting(String(said))} className="ml-1 text-hue-teal">
          <span aria-hidden="true">◆{said}</span>
          <span className="sr-only"> · {stepsWaiting(String(said))}</span>
        </span>
      )}
    </>
  );
}
