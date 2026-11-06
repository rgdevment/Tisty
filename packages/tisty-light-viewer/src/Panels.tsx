import { createSignal, For, onCleanup, Show } from "solid-js";
import type { Facts, Head } from "./host";
import type { Words } from "./words";

const KILOBYTE = 1024;
const LISTED_UP_TO = 3;

const weigh = (bytes: number): string => {
  if (bytes < KILOBYTE) return `${bytes} B`;
  if (bytes < KILOBYTE * KILOBYTE) return `${Math.round(bytes / KILOBYTE)} KB`;
  return `${(bytes / (KILOBYTE * KILOBYTE)).toFixed(1)} MB`;
};

export default function Panels(props: {
  readonly words: Words;
  readonly open: boolean;
  readonly heads: readonly Head[];
  readonly place: string;
  readonly about: Facts | null;
  readonly fields: readonly (readonly [string, string])[];
  readonly sourced: boolean;
  readonly onEdit: () => void;
  readonly onSource: () => void;
  readonly onGo: (id: string) => void;
}) {
  const { say, locale } = props.words;
  const listed = () => props.heads.filter((one) => one.level <= LISTED_UP_TO);
  const [current, setCurrent] = createSignal<string | null>(null);

  const follow = () => {
    const root = document.documentElement;
    const atEnd =
      window.scrollY > 0 && window.scrollY + window.innerHeight >= root.scrollHeight - 1;
    const line = window.innerHeight / 3;
    let reached: string | null = null;
    for (const one of listed()) {
      const top = document.getElementById(one.id)?.getBoundingClientRect().top;
      if (top !== undefined && top <= line) reached = one.id;
    }
    setCurrent(atEnd ? (listed().at(-1)?.id ?? null) : reached);
  };

  let queued = false;
  const scrolled = () => {
    if (queued) return;
    queued = true;
    requestAnimationFrame(() => {
      queued = false;
      follow();
    });
  };
  window.addEventListener("scroll", scrolled, { passive: true });
  onCleanup(() => window.removeEventListener("scroll", scrolled));

  const wrote = () => {
    const at = props.about?.wrote;
    return at ? new Date(at * 1000).toLocaleDateString(locale) : null;
  };

  return (
    <aside class="panels" data-open={props.open ? "" : undefined} aria-label={say("details")}>
      <section class="panel panel-actions">
        <button class="action action-main" type="button" onClick={props.onEdit}>
          {say("editOne")}
        </button>
        <button class="action" type="button" onClick={props.onSource}>
          {say(props.sourced ? "sourceOff" : "sourceOn")}
        </button>
        <p class="actions-note">{say("editNote")}</p>
      </section>
      <Show when={listed().length > 0}>
        <section class="panel">
          <h2>{say("index")}</h2>
          <ol class="index outline">
            <For each={listed()}>
              {(one) => (
                <li data-level={one.level}>
                  <button
                    type="button"
                    aria-current={current() === one.id ? "location" : undefined}
                    onClick={() => props.onGo(one.id)}
                  >
                    {one.said}
                  </button>
                </li>
              )}
            </For>
          </ol>
        </section>
      </Show>
      <section class="panel">
        <h2>{say("details")}</h2>
        <dl class="facts">
          <Show when={props.place}>
            <div>
              <dt>{say("folder")}</dt>
              <dd>{props.place}</dd>
            </div>
          </Show>
          <Show when={props.about}>
            <div>
              <dt>{say("weight")}</dt>
              <dd>{weigh(props.about?.bytes ?? 0)}</dd>
            </div>
          </Show>
          <Show when={wrote()}>
            <div>
              <dt>{say("wrote")}</dt>
              <dd>{wrote()}</dd>
            </div>
          </Show>
          <For each={props.fields}>
            {(one) => (
              <div>
                <dt>{one[0]}</dt>
                <dd>{one[1]}</dd>
              </div>
            )}
          </For>
        </dl>
      </section>
    </aside>
  );
}
