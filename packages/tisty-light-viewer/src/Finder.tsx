import { createSignal, onCleanup, onSettled, Show } from "solid-js";
import { Crossed, Down, Up } from "./icons";
import { type Sought, seeker, unsought } from "./seek";
import type { Words } from "./words";

export default function Finder(props: {
  readonly words: Words;
  readonly within: HTMLElement;
  readonly onClose: () => void;
}) {
  const { say } = props.words;
  const seeking = seeker(props.within);
  const [found, setFound] = createSignal<Sought | null>(null);
  let field!: HTMLInputElement;
  onSettled(() => field.focus());
  onCleanup(unsought);
  return (
    <search class="finder">
      <input
        ref={field}
        type="search"
        placeholder={say("find")}
        aria-label={say("find")}
        onInput={(e) =>
          setFound(e.currentTarget.value ? seeking.seek(e.currentTarget.value) : null)
        }
        onKeyDown={(e) => {
          if (e.key === "Enter") setFound(seeking.step(e.shiftKey ? -1 : 1));
        }}
      />
      <Show when={found()}>
        <output class="finder-count">
          {found()?.of ? `${found()?.at} / ${found()?.of}` : say("findNone")}
        </output>
      </Show>
      <button
        class="tool"
        type="button"
        aria-label={say("findBack")}
        onClick={() => setFound(seeking.step(-1))}
      >
        <Up />
      </button>
      <button
        class="tool"
        type="button"
        aria-label={say("findNext")}
        onClick={() => setFound(seeking.step(1))}
      >
        <Down />
      </button>
      <button class="tool" type="button" aria-label={say("close")} onClick={props.onClose}>
        <Crossed />
      </button>
    </search>
  );
}
