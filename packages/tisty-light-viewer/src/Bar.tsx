import { Show } from "solid-js";
import { Back, Dots, Lines, Logo, Search, ToDark, ToLight } from "./icons";
import type { Words } from "./words";

export default function Bar(props: {
  readonly words: Words;
  readonly name: string;
  readonly place: string;
  readonly onFind: () => void;
  readonly onTheme: () => void;
  readonly onMore: () => void;
  readonly onSide: () => void;
  readonly onBack?: () => void;
}) {
  const { say } = props.words;
  return (
    <header class="bar">
      <Show when={props.onBack}>
        <button
          class="tool"
          type="button"
          aria-label={say("back")}
          title={say("back")}
          onClick={() => props.onBack?.()}
        >
          <Back />
        </button>
      </Show>
      <span class="brand">
        <Logo />
        <span class="brand-name">
          {props.name}
          <span class="brand-by">{props.place}</span>
        </span>
      </span>
      <div class="bar-end">
        <button
          class="tool tool-side"
          type="button"
          aria-label={say("sidePanel")}
          title={say("sidePanel")}
          onClick={props.onSide}
        >
          <Lines />
        </button>
        <button
          class="tool"
          type="button"
          aria-label={say("find")}
          title={say("find")}
          onClick={props.onFind}
        >
          <Search />
        </button>
        <button
          class="tool"
          type="button"
          aria-label={say("theme")}
          title={say("theme")}
          onClick={props.onTheme}
        >
          <ToDark />
          <ToLight />
        </button>
        <button
          class="tool tool-more"
          type="button"
          aria-label={say("more")}
          title={say("more")}
          onClick={props.onMore}
        >
          <Dots />
        </button>
      </div>
    </header>
  );
}
