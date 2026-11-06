import { For, onCleanup, Show } from "solid-js";
import { reveal, source } from "./host";
import type { Words } from "./words";

export default function MoreMenu(props: {
  readonly words: Words;
  readonly path: string;
  readonly onClose: () => void;
  readonly onFind: () => void;
  readonly onZoom: (step: number) => void;
  readonly onPrint: () => void;
  readonly fail: (problem: unknown) => void;
}) {
  const { say } = props.words;
  const choices = [
    { label: say("find"), hint: "Ctrl+F", pick: props.onFind },
    { label: say("bigger"), hint: "Ctrl+=", pick: () => props.onZoom(1) },
    { label: say("smaller"), hint: "Ctrl+−", pick: () => props.onZoom(-1) },
    { label: say("plain"), hint: "Ctrl+0", pick: () => props.onZoom(0) },
    {
      label: say("copy"),
      pick: () =>
        void source()
          .then((said) => navigator.clipboard.writeText(said))
          .catch(props.fail),
    },
    { label: say("print"), hint: "Ctrl+P", pick: props.onPrint },
    { label: say("revealIt"), pick: () => void reveal(props.path).catch(props.fail) },
  ];
  const away = (e: PointerEvent) => {
    if (!(e.target as HTMLElement).closest(".menu, .tool-more")) props.onClose();
  };
  window.addEventListener("pointerdown", away);
  onCleanup(() => window.removeEventListener("pointerdown", away));
  return (
    <div class="menu" role="menu" aria-label={say("more")}>
      <For each={choices}>
        {(one) => (
          <button
            type="button"
            role="menuitem"
            onClick={() => {
              props.onClose();
              one.pick();
            }}
          >
            <span>{one.label}</span>
            <Show when={one.hint}>
              <kbd>{one.hint}</kbd>
            </Show>
          </button>
        )}
      </For>
    </div>
  );
}
