import type { Task } from "../core";
import type { Signing } from "../crediting";
import { fill, t } from "../locales";
import { proofOf } from "../owners";
import { clientNamed } from "../who";

interface Props {
  task: Task;
  signing: Signing;
  onConfirm: (task: Task) => void;
  onKeepOpen: (task: Task) => void;
}

export const agentWords = (task: Task, signing: Signing): string | undefined => {
  const said = task.resolved;
  if (!said) return undefined;
  if (signing === "mark") return t(said.drop ? "saidMarkNotDoing" : "saidMarkDone");
  const named = signing === "named" ? clientNamed(said.via) : undefined;
  if (named) return fill(said.drop ? "agentNamedSaidNotDoing" : "agentNamedSaidDone", named);
  return t(said.drop ? "agentSaidNotDoing" : "agentSaidDone");
};

export const confirmWord = (task: Task): string =>
  task.resolved?.drop ? t(task.repeat ? "endRepeat" : "discardIt") : t("markDone");

export default function AgentWord({ task, signing, onConfirm, onKeepOpen }: Props) {
  if (!task.resolved || task.status !== "open") return null;
  const drop = Boolean(task.resolved.drop);
  const seat =
    "flex shrink-0 items-center gap-1 whitespace-nowrap rounded-md px-1.5 py-px hover:bg-line";
  const proof = proofOf(task);
  const landed = [
    task.resolved.branch && `⎇ ${task.resolved.branch}`,
    task.resolved.pr && fill("saidPr", task.resolved.pr),
  ].filter(Boolean);
  return (
    <div className="mt-1 text-[11.5px]">
      {(proof.length > 0 || landed.length > 0) && (
        <div data-proof="" className="mb-1 border-l-2 border-hue-teal/40 pl-2 text-soft">
          {proof.map((line) => (
            <p key={line} className="truncate">
              {line}
            </p>
          ))}
          {landed.length > 0 && (
            <p className="truncate font-mono text-[10.5px] text-faint">{landed.join(" · ")}</p>
          )}
        </div>
      )}
      <div className="flex flex-wrap items-center gap-x-1.5 gap-y-1">
        <span
          className="mr-1 text-hue-teal"
          title={signing === "mark" ? agentWords(task, "agent") : undefined}
        >
          {signing === "mark" && <span aria-hidden="true">◆ </span>}
          {agentWords(task, signing)}
        </span>
        <button
          type="button"
          tabIndex={-1}
          title={drop && task.repeat ? t("endRepeatWhy") : undefined}
          onKeyDown={(e) => e.stopPropagation()}
          onClick={(e) => {
            e.stopPropagation();
            onConfirm(task);
          }}
          className={`${seat} font-medium text-accent`}
        >
          <span aria-hidden="true">{drop ? "⊘" : "✓"}</span> {confirmWord(task)}
        </button>
        <button
          type="button"
          tabIndex={-1}
          title={fill("stillOpenIt", task.title)}
          onKeyDown={(e) => e.stopPropagation()}
          onClick={(e) => {
            e.stopPropagation();
            onKeepOpen(task);
          }}
          className={`${seat} text-soft hover:text-ink`}
        >
          <span aria-hidden="true">↩</span> {t("stillOpen")}
        </button>
      </div>
    </div>
  );
}
