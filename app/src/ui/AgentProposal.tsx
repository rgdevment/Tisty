import type { Proposal } from "../agentDocs";
import { stamped } from "../format";
import { fill, t } from "../locales";
import { Action } from "./AgentCardParts";
import ComparedLines from "./ComparedLines";
import Glyph from "./Glyph";

interface Props {
  readonly proposal: Proposal;
  readonly onApprove?: () => void;
  readonly onDiscard: () => void;
}

export default function AgentProposal({ proposal, onApprove, onDiscard }: Props) {
  const said = proposal.via ? fill("agentProposalBy", proposal.via) : t("agentProposalSomeone");
  return (
    <section
      aria-label={t("agentProposalLabel")}
      className="mt-3 rounded-[10px] bg-hue-purple/10 px-3.5 py-2.5 text-[12.5px] text-ink"
    >
      <p className="flex items-center gap-1.5 font-medium text-hue-purple">
        <Glyph name="pencil" className="h-[14px] w-[14px] shrink-0" />
        {said}
        <span className="font-normal text-faint">· {stamped(proposal.at)}</span>
      </p>
      <p className="mt-1 text-soft">{t("agentProposalLead")}</p>
      {proposal.stale && !proposal.unfit && (
        <p className="mt-1 text-hue-amber">{t("agentProposalStale")}</p>
      )}
      {proposal.unfit && <p className="mt-1 text-hue-amber">{t(proposal.unfit)}</p>}
      <ComparedLines
        lines={proposal.lines}
        gone={t("agentProposalApproved")}
        came={t("agentProposalProposed")}
      />
      <div className="mt-2.5 flex flex-wrap gap-2">
        {onApprove && !proposal.unfit && (
          <Action glyph="done" label={t("agentApproveChange")} onClick={onApprove} />
        )}
        <Action glyph="close" label={t("agentDiscardChange")} onClick={onDiscard} />
      </div>
    </section>
  );
}
