import { Show } from "solid-js";
import { maximized, minimized } from "./host";
import { Back, Barred, Crossed, Dots, Lines, Logo, Search, Square, ToDark, ToLight } from "./icons";
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
  readonly onClose: () => void;
}) {
  const { say } = props.words;
  return (
    <header class="bar" data-tauri-drag-region>
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
      <span class="brand" data-tauri-drag-region>
        <Logo />
        <span class="brand-name" data-tauri-drag-region>
          {props.name}
          <span class="brand-by" data-tauri-drag-region>
            {props.place}
          </span>
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
        <div class="window">
          <button
            class="tool"
            type="button"
            aria-label={say("minimize")}
            onClick={() => void minimized()}
          >
            <Barred />
          </button>
          <button
            class="tool"
            type="button"
            aria-label={say("maximize")}
            onClick={() => void maximized()}
          >
            <Square />
          </button>
          <button class="tool" type="button" aria-label={say("close")} onClick={props.onClose}>
            <Crossed />
          </button>
        </div>
      </div>
    </header>
  );
}
