import { Show } from "solid-js";
import { type Opened, reveal } from "./host";
import type { Words } from "./words";

const saying = (words: Words, opened: Opened): string => {
  if (opened.kind === "drawn") return words.say("empty");
  return words.say(opened.refusal.code === "documentTooBig" ? "tooBig" : "unreadable");
};

export default function Problem(props: {
  readonly words: Words;
  readonly opened: Opened;
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
          <button
            class="action"
            type="button"
            onClick={() => void reveal(props.opened.path).catch(props.fail)}
          >
            {say("revealIt")}
          </button>
        </Show>
        <button class="action" type="button" onClick={props.onRetry}>
          {say("retry")}
        </button>
      </div>
    </div>
  );
}
