import { convertFileSrc } from "@tauri-apps/api/core";
import { type RefObject, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { type Group, type Network, type Neuron, OPENED } from "../brainsty";
import { serve } from "../inUse";
import { t } from "../locales";
import { pictured } from "../previews";
import Corner from "./brainsty/Corner";
import { tiesAt } from "./brainsty/cardFacts";
import { type Curves, curvesOn } from "./brainsty/curves";
import { type Drawing, drawNetwork, type HueOf } from "./brainsty/drawing";
import { groupsKey } from "./brainsty/grouping";
import { alikeOf } from "./brainsty/guessing";
import { type Icons, iconsOn } from "./brainsty/icons";
import type { Motions } from "./brainsty/Legend";
import { layOut } from "./brainsty/layingOut";
import { type Living, livingOn } from "./brainsty/living";
import { degreesOf, nodeLook, paletteNow } from "./brainsty/looks";
import { type Overlays, overlaysOn } from "./brainsty/overlays";
import Preview from "./brainsty/Preview";
import { edgesOf, seeded, weightsOf } from "./brainsty/settling";
import { brainShaped } from "./brainsty/shape";
import { EVERYTHING, found, hiddenBy, type Sift } from "./brainsty/sifting";
import Toolbar from "./brainsty/Toolbar";
import { turnedByHand } from "./brainsty/turning";
import { walker } from "./brainsty/walking";
import { followTheme } from "./theming";

export type Escaping = RefObject<(() => boolean) | null>;

interface Props {
  readonly network: Network;
  readonly groups: readonly Group[];
  readonly onOpen: (neuron: Neuron) => void;
  readonly escaping: Escaping;
}

export default function Brainsty({ network, groups, onOpen, escaping }: Props) {
  const host = useRef<HTMLElement>(null);
  const finder = useRef<HTMLInputElement>(null);
  const drawing = useRef<{
    drawn: Drawing;
    overlays: Overlays;
    curves: Curves;
    icons: Icons;
    living: Living;
    recolour: () => void;
  } | null>(null);
  const [ready, setReady] = useState(false);
  const [picked, setPicked] = useState<number | null>(null);
  const [query, setQuery] = useState("");
  const [sift, setSift] = useState<Sift>(EVERYTHING);
  const [motions, setMotions] = useState(motionsAtFirst);
  const [costShown, setCostShown] = useState(false);
  const [agentsShown, setAgentsShown] = useState(() => kept(AGENTS) === "shown");
  const hues = useRef<HueOf>(() => undefined);
  const outside = useRef(new Set<string>());
  outside.current = new Set(groups.flatMap((one) => (one.agents ? [one.id] : [])));
  const tinted = new Map<string, string>();
  const named = new Map<string, string>();
  for (const one of groups) {
    named.set(one.id, one.name);
    if (one.color) tinted.set(one.id, one.color);
  }
  hues.current = (group) => (group ? tinted.get(group) : undefined);
  const looked = groupsKey(groups);
  const theirs = JSON.stringify([...outside.current]);
  const agentsFolders = useMemo(() => new Set<string>(JSON.parse(theirs)), [theirs]);
  const leftOut = agentsShown ? NONE : agentsFolders;
  const legendGroups = useMemo(
    () => (agentsShown ? groups : groups.filter((one) => !one.agents)),
    [agentsShown, groups],
  );
  const hasAgents = useMemo(
    () => network.nodes.some((one) => one.group !== undefined && agentsFolders.has(one.group)),
    [network, agentsFolders],
  );
  const hidden = useMemo(
    () => hiddenBy(network, sift, new Date(), leftOut),
    [network, sift, leftOut],
  );
  const hits = useMemo(() => found(network, query, hidden), [network, query, hidden]);
  const alike = useMemo(
    () => (picked === null || !sift.alike ? [] : alikeOf(network, picked, hidden)),
    [network, picked, hidden, sift.alike],
  );
  const degrees = useMemo(() => degreesOf(network.nodes.length, network.edges), [network]);
  const tied = useMemo(
    () => (picked === null ? [] : tiesAt(network.edges, picked)),
    [network, picked],
  );
  const indexOf = useMemo(
    () => new Map(network.nodes.map((one, node) => [one.id, node])),
    [network],
  );
  const tags = useMemo(
    () =>
      network.nodes
        .flatMap((one, node) => (one.kind === "tag" ? [node] : []))
        .sort((one, other) => degrees[other] - degrees[one]),
    [network, degrees],
  );
  const live = useRef({ picked, hidden, onOpen, guessing: sift.alike, motions, looked });
  live.current = { picked, hidden, onOpen, guessing: sift.alike, motions, looked };
  const colouredBy = useRef(looked);
  // The keys walk the network from the network itself, so the search hands the focus back once it is done.
  const backToNetwork = () => host.current?.focus();
  escaping.current = () => {
    if (query || (finder.current !== null && document.activeElement === finder.current)) {
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
    const outsideOf = (group?: string) => (group ? outside.current.has(group) : false);
    const palette = paletteNow();
    const drawn = drawNetwork(
      container,
      network,
      shaped,
      hueOf,
      palette,
      getComputedStyle(container).fontFamily,
      outsideOf,
    );
    const curves = curvesOn(container, drawn, network, palette);
    curves.show(live.current.guessing);
    curves.reshape(shaped);
    const overlays = overlaysOn(container, drawn, network, hueOf, palette, PICTURES);
    overlays.reshape(shaped);
    let colours = palette;
    const colourOf = (node: number) => {
      const one = network.nodes[node];
      return nodeLook(one, hueOf(one.group), colours, 0, outsideOf(one.group)).color;
    };
    const icons = iconsOn(container, drawn, network, colourOf, PICTURES.shows);
    const living = livingOn(container, drawn, network, colourOf, {
      motions: live.current.motions,
    });
    const unturned = turnedByHand(container, living.turnBy);
    const near = network.near;
    const pick = (node: number) => {
      setPicked(node);
      drawn.focus(node);
    };
    container.dataset.drawnMs = String(Math.round(sinceOpened()));
    const unheed = drawn.heed({
      pick: (node) => setPicked(node),
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
    const { edges } = network;
    const stop = layOut({ ...spots, edges: edgesOf(edges), weights: weightsOf(edges) }, (step) => {
      const placed = brainShaped(step);
      drawn.place(placed);
      overlays.reshape(placed);
      curves.reshape(placed);
      if (!step.done) return;
      living.ready();
      container.dataset.laidMs = String(Math.round(step.ms));
      container.dataset.iterations = String(step.iterations);
    });
    const recolour = () => {
      const next = paletteNow();
      drawn.retheme(next);
      overlays.retheme(next);
      curves.retheme(next);
      colours = next;
      icons.retheme();
    };
    const unfollow = followTheme(recolour);
    colouredBy.current = live.current.looked;
    drawing.current = { drawn, overlays, curves, icons, living, recolour };
    setReady(true);
    return () => {
      stop();
      unfollow();
      unheed();
      window.removeEventListener("keydown", keyed);
      overlays.stop();
      curves.stop();
      icons.stop();
      unturned();
      living.stop();
      drawing.current = null;
      drawn.kill();
    };
  }, [network]);

  useEffect(() => {
    if (!ready || !drawing.current) return;
    drawing.current.drawn.hide(hidden);
    drawing.current.overlays.sift(hidden);
    drawing.current.curves.sift(hidden);
    drawing.current.icons.sift(hidden);
    drawing.current.living.sift(hidden);
    setPicked((was) => (was !== null && hidden[was] ? null : was));
  }, [ready, hidden]);

  useEffect(() => {
    const drawn = drawing.current?.drawn;
    if (!ready || !drawn) return;
    const lit =
      picked === null
        ? null
        : new Set([picked, ...drawn.beside(picked), ...alike.map((one) => one.node)]);
    drawn.light(lit, picked);
    drawing.current?.icons.light(lit);
    drawing.current?.curves.pick(picked);
    drawing.current?.living.hold(picked !== null);
  }, [ready, picked, alike]);

  useEffect(() => {
    if (ready) drawing.current?.curves.show(sift.alike);
  }, [ready, sift.alike]);

  // A folder painted another colour shows at once, without laying the network out again.
  useEffect(() => {
    if (!ready || colouredBy.current === looked) return;
    colouredBy.current = looked;
    drawing.current?.recolour();
  }, [ready, looked]);

  useEffect(() => {
    const living = drawing.current?.living;
    if (!ready || !living) return;
    living.travel(motions.travel);
    living.spin(motions.spin);
  }, [ready, motions]);

  useEffect(() => {
    const shown = (e: KeyboardEvent) => {
      if (!(e.ctrlKey && e.shiftKey && e.key.toLowerCase() === "m")) return;
      e.preventDefault();
      setCostShown((was) => !was);
    };
    window.addEventListener("keydown", shown);
    return () => window.removeEventListener("keydown", shown);
  }, []);

  const cost = useCallback(() => drawing.current?.living.measured() ?? { ms: 0, fps: 0 }, []);

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
        // biome-ignore lint/a11y/noNoninteractiveTabindex: the arrows walk the network, so Tab has to reach it
        tabIndex={0}
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
        agents={
          hasAgents
            ? {
                shown: agentsShown,
                onShow: (shown) => {
                  keep(AGENTS, shown ? "shown" : null);
                  setAgentsShown(shown);
                },
              }
            : undefined
        }
      />
      {shown && picked !== null && (
        <Preview
          neuron={shown}
          ties={degrees[picked]}
          tied={tied}
          nodeOf={(id) => indexOf.get(id)}
          zone={shown.group ? named.get(shown.group) : undefined}
          alike={alike.map((one) => ({ ...one, neuron: network.nodes[one.node] }))}
          onPick={choose}
          onOpen={() => onOpen(shown)}
          onClose={() => setPicked(null)}
        />
      )}
      <Corner
        network={network}
        groups={legendGroups}
        motions={motions}
        onMotions={(next) => {
          keep(TRAVEL, next.travel ? null : "off");
          keep(SPIN, next.spin ? null : "off");
          setMotions(next);
        }}
        costShown={costShown}
        cost={cost}
      />
      <p className="pointer-events-none absolute bottom-3 left-3 max-w-[calc(100%-24px)] truncate rounded-md bg-bg/70 px-1.5 text-[11.5px] text-faint">
        {t("brainstyKeys")}
      </p>
    </div>
  );
}

const TRAVEL = "tisty.brainstyTravellers";
const SPIN = "tisty.brainstySpin";
const AGENTS = "tisty.brainstyAgents";
const NONE: ReadonlySet<string> = new Set();

const kept = (key: string): string | null => {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
};

const keep = (key: string, value: string | null) => {
  try {
    if (value === null) localStorage.removeItem(key);
    else localStorage.setItem(key, value);
  } catch {
    // Without storage the network simply starts as it always does.
  }
};

// Both come on until the person turns one off in the legend; the system's «reduce motion» no longer decides for them.
const motionsAtFirst = (): Motions => ({
  travel: kept(TRAVEL) !== "off",
  spin: kept(SPIN) !== "off",
});

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
