import { Show } from "solid-js";
import { type Opened, reveal } from "./host";
import type { Words } from "./words";

const saying = (words: Words, opened: Opened): string => {
  if (opened.kind === "drawn") return words.say(opened.own ? "emptyDoc" : "empty");
  if (opened.refusal.code === "documentTooBig") return words.say("tooBig");
  if (opened.own && opened.refusal.code === "noSuchDoc") return words.say("noSuchDoc");
  return words.say(opened.own ? "unreadableDoc" : "unreadable");
};

export default function Problem(props: {
  readonly words: Words;
  readonly opened: Opened;
  readonly revealed: string | null;
  readonly onRetry: () => void;
  readonly onEdit: () => void;
  readonly fail: (problem: unknown) => void;
}) {
  const { say } = props.words;
  return (
    <div class="problem">
      <p>{saying(props.words, props.opened)}</p>
      <div class="problem-acts">
        <Show
          when={props.opened.kind === "missed"}
          fallback={
            <button class="action action-main" type="button" onClick={props.onEdit}>
              {say("editOne")}
            </button>
          }
        >
          <Show when={props.revealed}>
            {(path) => (
              <button
                class="action"
                type="button"
                onClick={() => void reveal(path()).catch(props.fail)}
              >
                {say("revealIt")}
              </button>
            )}
          </Show>
        </Show>
        <button class="action" type="button" onClick={props.onRetry}>
          {say("retry")}
        </button>
      </div>
    </div>
  );
}
