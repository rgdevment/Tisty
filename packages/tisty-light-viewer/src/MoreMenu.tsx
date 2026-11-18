import { For, onCleanup, Show } from "solid-js";
import { address, copied, plain, reveal } from "./host";
import type { Words } from "./words";

type Choice = { label: string; hint?: string; pick: () => void };

export default function MoreMenu(props: {
  readonly words: Words;
  readonly drawn: boolean;
  readonly searchable: boolean;
  readonly own: boolean;
  readonly revealed: string | null;
  readonly onClose: () => void;
  readonly onFind: () => void;
  readonly onZoom: (step: number) => void;
  readonly onPrint: () => void;
  readonly fail: (problem: unknown) => void;
}) {
  const { say } = props.words;
  const copying = (label: string, said: () => Promise<string>): Choice => ({
    label,
    pick: () => void said().then(copied).catch(props.fail),
  });
  const find: Choice = { label: say("find"), hint: "Ctrl+F", pick: props.onFind };
  const choices: Choice[] = [
    ...(props.searchable ? [find] : []),
    { label: say("bigger"), hint: "Ctrl+=", pick: () => props.onZoom(1) },
    { label: say("smaller"), hint: "Ctrl+−", pick: () => props.onZoom(-1) },
    { label: say("plain"), hint: "Ctrl+0", pick: () => props.onZoom(0) },
    ...(props.drawn ? [copying(say("copy"), plain)] : []),
    ...(props.drawn && props.own ? [copying(say("copyLink"), address)] : []),
    { label: say("print"), hint: "Ctrl+P", pick: props.onPrint },
  ];
  const path = props.revealed;
  if (path !== null) {
    choices.push({ label: say("revealIt"), pick: () => void reveal(path).catch(props.fail) });
  }
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
