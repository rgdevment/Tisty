import { convertFileSrc } from "@tauri-apps/api/core";
import { type RefObject, useEffect, useMemo, useRef, useState } from "react";
import { type Network, type Neuron, OPENED } from "../brainsty";
import { useDesk } from "../desk";
import { serve } from "../inUse";
import { t } from "../locales";
import { pictured } from "../previews";
import { type Drawing, drawNetwork, type HueOf } from "./brainsty/drawing";
import { layOut } from "./brainsty/layingOut";
import { degreesOf, paletteNow } from "./brainsty/looks";
import { type Overlays, overlaysOn } from "./brainsty/overlays";
import Preview from "./brainsty/Preview";
import { edgesOf, seeded } from "./brainsty/settling";
import { brainShaped } from "./brainsty/shape";
import { EVERYTHING, found, hiddenBy, type Sift } from "./brainsty/sifting";
import Toolbar from "./brainsty/Toolbar";
import { walker } from "./brainsty/walking";
import { followTheme } from "./theming";

export type Escaping = RefObject<(() => boolean) | null>;

interface Props {
  readonly network: Network;
  readonly onOpen: (neuron: Neuron) => void;
  readonly escaping: Escaping;
}

export default function Brainsty({ network, onOpen, escaping }: Props) {
  const { papers, data } = useDesk();
  const host = useRef<HTMLElement>(null);
  const finder = useRef<HTMLInputElement>(null);
  const drawing = useRef<{ drawn: Drawing; overlays: Overlays } | null>(null);
  const [ready, setReady] = useState(false);
  const [picked, setPicked] = useState<number | null>(null);
  const [query, setQuery] = useState("");
  const [sift, setSift] = useState<Sift>(EVERYTHING);
  const hues = useRef<HueOf>(() => undefined);
  const tinted = new Map<string, string>();
  const named = new Map<string, string>();
  for (const one of papers.folders) {
    named.set(one.id, one.name);
    if (one.color) tinted.set(one.id, one.color);
  }
  for (const one of data.every ?? data.lists) {
    named.set(one.id, one.name);
    if (one.color) tinted.set(one.id, one.color);
  }
  hues.current = (group) => (group ? tinted.get(group) : undefined);
  const hidden = useMemo(() => hiddenBy(network, sift, new Date()), [network, sift]);
  const hits = useMemo(() => found(network, query, hidden), [network, query, hidden]);
  const degrees = useMemo(() => degreesOf(network.nodes.length, network.edges), [network]);
  const tags = useMemo(
    () =>
      network.nodes
        .flatMap((one, node) => (one.kind === "tag" ? [node] : []))
        .sort((one, other) => degrees[other] - degrees[one]),
    [network, degrees],
  );
  const live = useRef({ picked, hidden, onOpen });
  live.current = { picked, hidden, onOpen };
  // The keys walk the network from the network itself, so the search hands the focus back once it is done.
  const backToNetwork = () => host.current?.focus();
  escaping.current = () => {
    if (query) {
      setQuery("");
      backToNetwork();
    } else if (picked !== null) setPicked(null);
    else return false;
    return true;
  };

  useEffect(() => {
    const container = host.current;
    if (!container) return;
    const spots = seeded(network.nodes);
    const shaped = brainShaped(spots);
    const hueOf: HueOf = (group) => hues.current(group);
    const palette = paletteNow();
    const drawn = drawNetwork(
      container,
      network,
      shaped,
      hueOf,
      palette,
      getComputedStyle(container).fontFamily,
    );
    const overlays = overlaysOn(container, drawn, network, hueOf, palette, PICTURES);
    overlays.reshape(shaped);
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
    const pick = (node: number) => {
      letGo();
      setPicked(node);
      drawn.focus(node);
    };
    follow();
    container.dataset.drawnMs = String(Math.round(sinceOpened()));
    container.addEventListener("pointerdown", letGo);
    container.addEventListener("wheel", letGo, { passive: true });
    const unheed = drawn.heed({
      pick: (node) => {
        letGo();
        setPicked(node);
      },
      open: (node) => live.current.onOpen(network.nodes[node]),
      clear: () => setPicked(null),
    });
    const keyed = walker(drawn, network.nodes.length, near, {
      picked: () => live.current.picked,
      hidden: () => live.current.hidden,
      pick,
      open: (node) => live.current.onOpen(network.nodes[node]),
      find: () => finder.current?.focus(),
    });
    window.addEventListener("keydown", keyed);
    const stop = layOut({ ...spots, edges: edgesOf(network.edges) }, (step) => {
      const placed = brainShaped(step);
      drawn.place(placed);
      overlays.reshape(placed);
      followSoon();
      if (!step.done) return;
      container.dataset.laidMs = String(Math.round(step.ms));
      container.dataset.iterations = String(step.iterations);
    });
    const unfollow = followTheme(() => {
      const next = paletteNow();
      drawn.retheme(next);
      overlays.retheme(next);
    });
    drawing.current = { drawn, overlays };
    setReady(true);
    return () => {
      cancelAnimationFrame(frame);
      stop();
      unfollow();
      unheed();
      window.removeEventListener("keydown", keyed);
      container.removeEventListener("pointerdown", letGo);
      container.removeEventListener("wheel", letGo);
      overlays.stop();
      drawing.current = null;
      drawn.kill();
    };
  }, [network]);

  useEffect(() => {
    if (!ready || !drawing.current) return;
    drawing.current.drawn.hide(hidden);
    drawing.current.overlays.sift(hidden);
    setPicked((was) => (was !== null && hidden[was] ? null : was));
  }, [ready, hidden]);

  useEffect(() => {
    const drawn = drawing.current?.drawn;
    if (!ready || !drawn) return;
    drawn.light(picked === null ? null : new Set([picked, ...drawn.beside(picked)]));
  }, [ready, picked]);

  const choose = (node: number) => {
    setQuery("");
    setPicked(node);
    drawing.current?.drawn.focus(node);
    backToNetwork();
  };
  const shown = picked === null ? null : network.nodes[picked];
  return (
    <div className="absolute inset-0">
      <figure
        ref={host}
        tabIndex={-1}
        aria-label={t("brainstyNetwork")}
        className="absolute inset-0 m-0 outline-none focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-solid focus-visible:outline-accent"
      />
      <Toolbar
        network={network}
        query={query}
        onQuery={setQuery}
        hits={hits}
        onChoose={choose}
        sift={sift}
        onSift={setSift}
        tags={tags}
        finder={finder}
      />
      {shown && picked !== null && (
        <Preview
          neuron={shown}
          ties={degrees[picked]}
          zone={shown.group ? named.get(shown.group) : undefined}
          onOpen={() => onOpen(shown)}
          onClose={() => setPicked(null)}
        />
      )}
      <p className="pointer-events-none absolute bottom-3 left-3 text-[11.5px] text-faint">
        {t("brainstyKeys")}
      </p>
    </div>
  );
}

const PICTURES = {
  shows: pictured,
  urlOf: (id: string) => serve(id).then(convertFileSrc),
};

const sinceOpened = (): number => {
  const marks = performance.getEntriesByName(OPENED);
  const mark = marks[marks.length - 1];
  performance.clearMarks(OPENED);
  return mark ? performance.now() - mark.startTime : 0;
};
