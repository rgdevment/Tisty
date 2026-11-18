import { useEffect, useRef } from "react";
import { type Network, OPENED } from "../brainsty";
import { useDesk } from "../desk";
import { t } from "../locales";
import { drawNetwork, type HueOf } from "./brainsty/drawing";
import { layOut } from "./brainsty/layingOut";
import { paletteNow } from "./brainsty/looks";
import { edgesOf, seeded } from "./brainsty/settling";
import { followTheme } from "./theming";

export default function Brainsty({ network }: { readonly network: Network }) {
  const { papers, data } = useDesk();
  const host = useRef<HTMLElement>(null);
  const hues = useRef<HueOf>(() => undefined);
  const tinted = new Map<string, string>();
  for (const one of papers.folders) if (one.color) tinted.set(one.id, one.color);
  for (const one of data.every ?? data.lists) if (one.color) tinted.set(one.id, one.color);
  hues.current = (group) => (group ? tinted.get(group) : undefined);

  useEffect(() => {
    const container = host.current;
    if (!container) return;
    const spots = seeded(network.nodes);
    const drawn = drawNetwork(
      container,
      network,
      { x: spots.x.slice(), y: spots.y.slice() },
      (group) => hues.current(group),
      paletteNow(),
      getComputedStyle(container).fontFamily,
    );
    const near = network.near;
    let following = near !== undefined;
    let frame = 0;
    const follow = () => {
      if (following && near !== undefined) drawn.centre(near);
    };
    // sigma takes the new places on its next frame, so the camera follows after it.
    const followSoon = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(follow);
    };
    const letGo = () => {
      following = false;
    };
    follow();
    container.dataset.drawnMs = String(Math.round(sinceOpened()));
    container.addEventListener("pointerdown", letGo);
    container.addEventListener("wheel", letGo, { passive: true });
    const stop = layOut({ ...spots, edges: edgesOf(network.edges) }, (step) => {
      drawn.place(step);
      followSoon();
      if (!step.done) return;
      container.dataset.laidMs = String(Math.round(step.ms));
      container.dataset.iterations = String(step.iterations);
    });
    const unfollow = followTheme(() => drawn.retheme(paletteNow()));
    return () => {
      cancelAnimationFrame(frame);
      stop();
      unfollow();
      container.removeEventListener("pointerdown", letGo);
      container.removeEventListener("wheel", letGo);
      drawn.kill();
    };
  }, [network]);

  return <figure ref={host} aria-label={t("brainstyNetwork")} className="absolute inset-0 m-0" />;
}

const sinceOpened = (): number => {
  const marks = performance.getEntriesByName(OPENED);
  const mark = marks[marks.length - 1];
  performance.clearMarks(OPENED);
  return mark ? performance.now() - mark.startTime : 0;
};
