import { useEffect, useRef, useState } from "react";
import {
  type AgentDoc,
  agentAgree,
  agentApprove,
  agentBring,
  agentDiscard,
  agentHeld,
  agentRestore,
  agentTake,
  type Place,
  type Proposal,
  type Said,
  touched,
} from "../agentDocs";
import { agentsKeep, agentsTurn } from "../agentsSync";
import { useDesk } from "../desk";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { sure } from "../sure";
import {
  Action,
  BackupsModal,
  FileModal,
  HereNote,
  Pill,
  PlaceRow,
  type Shown,
} from "./AgentCardParts";
import AgentProposal from "./AgentProposal";
import { AGENTS_CHANGED } from "./AgentsKeeper";
import Glyph from "./Glyph";

interface Props {
  readonly doc: AgentDoc;
  readonly file: string;
  readonly editing: boolean;
  readonly locked: boolean;
  readonly onEdit: (editing: boolean) => void;
  readonly onSettle: () => Promise<void>;
  readonly onTaken: () => Promise<void>;
  readonly onError: (said: string) => void;
}

const KIND = { rule: "agentRule", skill: "agentSkill", memory: "agentMemory" } as const;

const STATE: Record<Said["state"], { word: Parameters<typeof t>[0]; tone: string; glyph: string }> =
  {
    synced: { word: "agentSynced", tone: "text-hue-green bg-hue-green/10", glyph: "done" },
    pending: { word: "agentPending", tone: "text-soft bg-hover", glyph: "refresh" },
    held: { word: "agentHeldPill", tone: "text-hue-amber bg-hue-amber/10", glyph: "pause" },
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

export default function AgentCard({
  doc,
  file,
  editing,
  locked,
  onEdit,
  onSettle,
  onTaken,
  onError,
}: Props) {
  const [shown, setShown] = useState<Shown | null>(null);
  const [kept, setKept] = useState<Place | null>(null);
  const own = doc.kind === "memory";
  const state = worst(doc.places);
  const changed = doc.places.filter(touched);
  const single = doc.places.length === 1 ? doc.places[0] : null;
  const turnable = (place: Place) => !doc.folderOff && place.said.state !== "noConsent";
  // Taking the file's words in replaces the page's, so it waits until the page is no longer being written.
  const takes = !locked && !editing;
  // A file speaks for the document only when it is that assistant's own: a Global rule is never rewritten from one of them.
  const owned = (place: Place) => takes && doc.level === place.assistant;

  const { papersChanged } = useDesk();
  // The page is read again only while it still shows the document the words were brought into.
  const showing = useRef<string | null>(file);
  useEffect(() => {
    showing.current = file;
    return () => {
      showing.current = null;
    };
  }, [file]);
  const failed = (e: unknown) => onError(saidPlainly(e));
  const done = () => window.dispatchEvent(new Event(AGENTS_CHANGED));
  const show = (place: Place, backup: number | null) =>
    agentHeld(file, place.at, backup)
      .then((text) => setShown({ place, backup, text }))
      .catch(failed);
  // Words still waiting to be saved land first, so the document the file comes over is the one on the page.
  const bring = (place: Place) =>
    Promise.resolve(onSettle())
      .then(() => agentBring(file, place.at, doc.print))
      .then(() => (showing.current === file ? onTaken() : undefined))
      .then(papersChanged)
      .catch(failed)
      .finally(done);
  // Each action asks first, and a refusal anywhere along the way is shown, never left unheard.
  const asked = (said: string, then: () => Promise<unknown>) =>
    sure(said)
      .then((yes) => (yes ? then() : undefined))
      .catch(failed);
  const overwrite = (place: Place) =>
    asked(fill("agentOverwriteSure", place.at), () => agentsKeep(place.at, true).then(done));
  const keepFile = (place: Place) => asked(fill("agentKeepFileSure", place.at), () => bring(place));
  const restore = (place: Place, backup: number) =>
    asked(fill("agentRestoreSure", place.at), () =>
      agentRestore(file, place.at, backup).then(() => {
        setShown(null);
        setKept(null);
        done();
      }),
    );
  const take = (here: boolean) => agentTake(file, here).then(done).catch(failed);
  // Approving sends what the page says now, so words still waiting to be saved land first.
  const agree = () =>
    asked(t("agentAgreeSure"), () =>
      Promise.resolve(onSettle())
        .then(() => agentAgree(file, doc.print))
        .finally(done),
    );
  // Approving writes the proposal over the page, so words still waiting to be saved land first and the page is read again.
  const approve = (proposal: Proposal) =>
    asked(t("agentApproveSure"), () =>
      Promise.resolve(onSettle())
        .then(() => agentApprove(file, proposal.print, doc.print))
        .then(() => (showing.current === file ? onTaken() : undefined))
        .then(papersChanged)
        .finally(done),
    );
  const discard = (proposal: Proposal) =>
    asked(t("agentDiscardSure"), () => agentDiscard(file, proposal.print).finally(done));
  const turn = (place: Place) =>
    agentsTurn(place.assistant, place.said.state === "off")
      .then(done)
      .catch(failed);

  return (
    <section
      className="agent-card mx-auto mb-5 w-full max-w-[760px] px-1"
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
      <p className="mt-2.5 text-[21px] font-medium text-ink">{doc.label}</p>
      <p className="mb-3 truncate text-[11.5px] text-faint" title={doc.name}>
        {doc.name}
      </p>
      {own && <p className="mb-3 text-[12.5px] text-soft">{t("agentMemoryOwn")}</p>}
      <div className="rounded-[10px] bg-panel px-3.5 py-1">
        {doc.places.length === 0 && (doc.here === "applied" || doc.here === "away") && (
          <p className="py-2 text-[12.5px] text-soft">
            {t(doc.twin ? "agentTwin" : "agentNowhere")}
          </p>
        )}
        {doc.places.map((place) => (
          <PlaceRow
            key={place.at}
            place={place}
            code={doc.code}
            onBackups={() => setKept(place)}
            onTurn={!single && turnable(place) ? () => turn(place) : undefined}
          />
        ))}
      </div>
      {doc.folderOff && <p className="mt-3 text-[12.5px] text-soft">{t("agentFolderOff")}</p>}
      <HereNote here={doc.here} onTake={take} onAgree={agree} />
      {doc.proposal && (
        <AgentProposal
          proposal={doc.proposal}
          onApprove={takes ? () => doc.proposal && approve(doc.proposal) : undefined}
          onDiscard={() => doc.proposal && discard(doc.proposal)}
        />
      )}
      {doc.places
        .filter((one) => one.said.state === "held")
        .map((one) => (
          <p key={one.at} className="mt-3 text-[12.5px] text-hue-amber">
            {fill("agentHeldSaid", one.heldBy.join(", "))}
          </p>
        ))}
      {changed.map((outside) => (
        <div
          key={outside.at}
          role="alert"
          className="mt-3 flex gap-2 rounded-[10px] bg-hue-amber/10 px-3.5 py-2.5 text-[12.5px] text-hue-amber"
        >
          <Glyph name="alert" className="mt-0.5 h-[15px] w-[15px] shrink-0" />
          <div className="min-w-0">
            <p className="font-medium">{t(SAID_OF[outside.said.state] ?? "agentOutsideSaid")}</p>
            <p className="mt-0.5">{fill("agentOutsideWhere", outside.at)}</p>
            <div className="mt-2 flex flex-wrap gap-2">
              {outside.said.state === "outside" && outside.said.mine && owned(outside) && (
                <Action glyph="download" label={t("agentBring")} onClick={() => bring(outside)} />
              )}
              {outside.said.state === "both" && owned(outside) && outside.shared === 1 && (
                <Action
                  glyph="download"
                  label={t("agentKeepFile")}
                  onClick={() => keepFile(outside)}
                />
              )}
              {!own && (
                <Action
                  glyph="upload"
                  label={t("agentOverwrite")}
                  onClick={() => overwrite(outside)}
                />
              )}
              {!(outside.said.state === "outside" && outside.said.gone) && (
                <Action
                  glyph="view"
                  label={t("agentSeeFile")}
                  onClick={() => show(outside, null)}
                />
              )}
            </div>
          </div>
        </div>
      ))}
      <div className="mt-3 flex flex-wrap gap-2">
        {!own && !locked && (
          <Action
            glyph={editing ? "done" : "pencil"}
            label={t(editing ? "agentDoneEditing" : "agentEdit")}
            onClick={() => onEdit(!editing)}
          />
        )}
        {single && single.kept.length > 0 && (
          <Action glyph="history" label={t("agentBackupsAll")} onClick={() => setKept(single)} />
        )}
        {single && turnable(single) && (
          <Action
            glyph={single.said.state === "off" ? "refresh" : "pause"}
            label={fill(
              single.said.state === "off" ? "agentResumeSync" : "agentPauseSync",
              single.named,
            )}
            onClick={() => turn(single)}
          />
        )}
      </div>
      {kept && !shown && (
        <BackupsModal
          place={kept}
          restores={!own}
          onSee={(backup) => show(kept, backup)}
          onRestore={(backup) => restore(kept, backup)}
          onClose={() => setKept(null)}
        />
      )}
      {shown && (
        <FileModal
          shown={shown}
          restores={!own}
          onRestore={(backup) => restore(shown.place, backup)}
          onClose={() => setShown(null)}
        />
      )}
    </section>
  );
}
