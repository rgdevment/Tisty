import { type Place, type Said, touched } from "../agentDocs";
import { folderShown } from "../agentsSync";
import { asLink } from "../copying";
import type { Folded } from "../core";
import { fill, t } from "../locales";
import { Action, AgentFacts, BackupsModal, FileModal, HereNote, Pill } from "./AgentCardParts";
import AgentProposal from "./AgentProposal";
import { SharedFact, SharedWindows } from "./AgentShared";
import Glyph from "./Glyph";
import { useSharedRow } from "./sharedRow";
import { type AgentShelf, type CardHands, type CardProps, useAgentCard } from "./useAgentCard";

const KIND = { rule: "agentRule", skill: "agentSkill", memory: "agentMemory" } as const;

const STATE: Record<Said["state"], { word: Parameters<typeof t>[0]; tone: string; glyph: string }> =
  {
    synced: { word: "agentSynced", tone: "text-hue-green bg-hue-green/10", glyph: "done" },
    pending: { word: "agentPending", tone: "text-soft bg-hover", glyph: "refresh" },
    held: { word: "agentHeldPill", tone: "text-hue-amber bg-hue-amber/10", glyph: "pause" },
    unled: { word: "agentPaused", tone: "text-hue-amber bg-hue-amber/10", glyph: "pause" },
    unproven: { word: "agentUnproven", tone: "text-hue-amber bg-hue-amber/10", glyph: "alert" },
    outside: { word: "agentOutside", tone: "text-hue-amber bg-hue-amber/10", glyph: "alert" },
    both: { word: "agentBoth", tone: "text-hue-red bg-hue-red/10", glyph: "alert" },
    off: { word: "agentPaused", tone: "text-soft bg-hover", glyph: "pause" },
    noConsent: { word: "agentNotAllowed", tone: "text-soft bg-hover", glyph: "lock" },
    empty: { word: "agentNothing", tone: "text-soft bg-hover", glyph: "alert" },
    linked: { word: "agentLinkedFile", tone: "text-hue-red bg-hue-red/10", glyph: "alert" },
    broken: { word: "agentBroken", tone: "text-hue-red bg-hue-red/10", glyph: "alert" },
    unreadable: { word: "agentUnreadable", tone: "text-hue-red bg-hue-red/10", glyph: "alert" },
  };

const SAID_OF: Partial<Record<Said["state"], "agentBothSaid" | "agentUnprovenSaid">> = {
  both: "agentBothSaid",
  unproven: "agentUnprovenSaid",
};

const ORDER: readonly Said["state"][] = [
  "both",
  "outside",
  "unproven",
  "held",
  "unled",
  "broken",
  "linked",
  "unreadable",
  "noConsent",
  "off",
  "empty",
  "pending",
  "synced",
];

const worst = (places: readonly Place[]): Said["state"] | null =>
  ORDER.find((state) => places.some((one) => one.said.state === state)) ?? null;

interface Props {
  readonly card: CardProps;
  readonly shelf: AgentShelf;
  readonly body: string;
}

function Outside({ hands, outside }: { hands: CardHands; outside: Place }) {
  const said = outside.said;
  return (
    <div
      role="alert"
      className="mt-3 flex gap-2 rounded-[10px] bg-hue-amber/10 px-3.5 py-2.5 text-[12.5px] text-hue-amber"
    >
      <Glyph name="alert" className="mt-0.5 h-[15px] w-[15px] shrink-0" />
      <div className="min-w-0">
        <p className="font-medium">{t(SAID_OF[said.state] ?? "agentOutsideSaid")}</p>
        <p className="mt-0.5">{fill("agentOutsideWhere", outside.at)}</p>
        <div className="mt-2 flex flex-wrap gap-2">
          {said.state === "outside" && said.mine && hands.owned(outside) && (
            <Action glyph="download" label={t("agentBring")} onClick={() => hands.bring(outside)} />
          )}
          {said.state === "both" && hands.owned(outside) && outside.shared === 1 && (
            <Action
              glyph="download"
              label={t("agentKeepFile")}
              onClick={() => hands.keepFile(outside)}
            />
          )}
          {!hands.own && (
            <Action
              glyph="upload"
              label={t("agentOverwrite")}
              onClick={() => hands.overwrite(outside)}
            />
          )}
          {!(said.state === "outside" && said.gone) && (
            <Action
              glyph="view"
              label={t("agentSeeFile")}
              onClick={() => hands.show(outside, null)}
            />
          )}
        </div>
      </div>
    </div>
  );
}

function Notices({ hands }: { hands: CardHands }) {
  const { doc } = hands.props;
  const proposal = doc.proposal;
  return (
    <>
      {doc.places.length === 0 && (doc.here === "applied" || doc.here === "away") && (
        <p className="mt-3 text-[12.5px] text-soft">{t(doc.twin ? "agentTwin" : "agentNowhere")}</p>
      )}
      {doc.folderOff && <p className="mt-3 text-[12.5px] text-soft">{t("agentFolderOff")}</p>}
      <HereNote here={doc.here} onTake={hands.take} onAgree={hands.agree} />
      {proposal && (
        <AgentProposal
          proposal={proposal}
          onApprove={hands.takes ? () => hands.approve(proposal) : undefined}
          onDiscard={() => hands.discard(proposal)}
        />
      )}
      {doc.places
        .filter((one) => one.said.state === "held")
        .map((one) => (
          <p key={one.at} className="mt-3 text-[12.5px] text-hue-amber">
            {fill("agentHeldSaid", one.heldBy.join(", "))}
          </p>
        ))}
      {doc.places
        .filter((one) => one.said.state === "unled")
        .map((one) => (
          <div key={one.at} className="mt-3 text-[12.5px] text-hue-amber">
            <p>{fill("agentUnledSaid", one.named)}</p>
            <div className="mt-2 flex">
              <Action glyph="refresh" label={t("agentsLeadAgain")} onClick={hands.lead} />
            </div>
          </div>
        ))}
      {doc.places.filter(touched).map((outside) => (
        <Outside key={outside.at} hands={hands} outside={outside} />
      ))}
    </>
  );
}

export default function AgentCard({ card, shelf, body }: Props) {
  const hands = useAgentCard(card, shelf);
  const shared = useSharedRow(card.id, hands.failed, card.onKeep);
  const { doc } = card;
  const state = worst(doc.places);
  return (
    <section
      className="agent-card mx-auto mt-1 w-full max-w-[760px] rounded-t-[10px] border border-b-0 border-line bg-sheet px-4 pt-4 pb-3"
      aria-label={t("agentCard")}
    >
      <div className="flex flex-wrap items-center gap-2">
        <Pill tone="text-hue-purple bg-hue-purple/10">{t(KIND[doc.kind])}</Pill>
        <Pill tone="text-hue-orange bg-hue-orange/10">{doc.named ?? t("agentGlobal")}</Pill>
        {state && (
          <Pill tone={STATE[state].tone} glyph={STATE[state].glyph}>
            {t(STATE[state].word)}
          </Pill>
        )}
        {doc.proposal && (
          <Pill tone="text-hue-purple bg-hue-purple/10" glyph="pencil">
            {t("agentProposalPill")}
          </Pill>
        )}
      </div>
      <h2 className="mt-2.5 text-[21px] font-medium text-ink">{doc.label}</h2>
      {doc.own.about && <p className="mt-0.5 text-[12.5px] text-soft">{doc.own.about}</p>}
      {hands.own && <p className="mt-1.5 text-[12.5px] text-soft">{t("agentMemoryOwn")}</p>}
      <AgentFacts doc={doc} body={body} onBackups={hands.seeBackups} onError={hands.failed}>
        <SharedFact shared={shared} />
      </AgentFacts>
      <Notices hands={hands} />
      <SharedWindows shared={shared} id={card.id} title={doc.label} onError={hands.failed} />
      {hands.kept && !hands.shown && (
        <BackupsModal
          place={hands.kept}
          restores={!hands.own}
          onSee={(backup) => hands.kept && hands.show(hands.kept, backup)}
          onRestore={(backup) => hands.kept && hands.restore(hands.kept, backup)}
          onClose={hands.closeBackups}
        />
      )}
      {hands.shown && (
        <FileModal
          shown={hands.shown}
          restores={!hands.own}
          onRestore={(backup) => hands.shown && hands.restore(hands.shown.place, backup)}
          onClose={hands.closeFile}
        />
      )}
    </section>
  );
}

interface Buttons {
  readonly card: CardProps;
  readonly shelf: AgentShelf;
  readonly onEarlier?: () => void;
}

// Under the words, so they stay at hand however long the file is.
export function AgentButtons({ card, shelf, onEarlier }: Buttons) {
  const hands = useAgentCard(card, shelf);
  const { doc, editing, locked, onEdit } = card;
  const single = hands.single;
  const turnable = doc.places.filter(hands.turnable);
  const pauses = (place: Place) => place.said.state !== "off";
  return (
    <nav
      aria-label={t("agentCardActions")}
      className="mx-auto mb-4 flex w-full max-w-[760px] flex-wrap gap-2 rounded-b-[10px] border border-line border-t-hair bg-sheet px-4 py-3"
    >
      {!hands.own && !locked && (
        <Action
          glyph={editing ? "done" : "pencil"}
          label={t(editing ? "agentDoneEditing" : "agentEdit")}
          onClick={() => onEdit(!editing)}
        />
      )}
      {single && single.kept.length > 0 && (
        <Action
          glyph="archive"
          label={t("agentBackupsAll")}
          onClick={() => hands.seeBackups(single)}
        />
      )}
      {onEarlier && <Action glyph="history" label={t("agentFileHistory")} onClick={onEarlier} />}
      <Action
        glyph="link"
        label={t("copyLink")}
        onClick={() => void asLink(card.file).catch(hands.failed)}
      />
      {turnable.map((place) => (
        <Action
          key={place.at}
          glyph={pauses(place) ? "pause" : "refresh"}
          label={
            single
              ? fill(pauses(place) ? "agentPauseSync" : "agentResumeSync", place.named)
              : fill(pauses(place) ? "agentPause" : "agentResume", place.named)
          }
          onClick={() => hands.turn(place)}
        />
      ))}
    </nav>
  );
}

interface Trail {
  readonly trail: readonly Folded[];
  readonly onFolder?: (id: string | null) => void;
}

export function AgentTrail({ trail, onFolder }: Trail) {
  return (
    <nav
      aria-label={t("agentTrail")}
      className="flex min-w-0 items-center gap-1 truncate text-[11.5px] font-normal text-faint"
    >
      {trail.map((one, n) => (
        <span key={one.id} className="flex min-w-0 items-center gap-1">
          {n > 0 && <span className="text-faint">/</span>}
          <button
            type="button"
            onClick={() => onFolder?.(one.id)}
            className="truncate hover:text-ink hover:underline"
          >
            {folderShown(one)}
          </button>
        </span>
      ))}
    </nav>
  );
}
