import { useEffect, useRef, useState } from "react";
import {
  type AgentDoc,
  agentAgree,
  agentApprove,
  agentBring,
  agentDiscard,
  agentHeld,
  agentSettle,
  agentTake,
  type Place,
  type Proposal,
} from "../agentDocs";
import { agentsKeep, agentsLead, agentsTurn } from "../agentsSync";
import { useDesk } from "../desk";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { sure } from "../sure";
import type { Shown } from "./AgentCardParts";
import { AGENTS_CHANGED } from "./AgentsKeeper";

export interface CardProps {
  readonly doc: AgentDoc;
  readonly file: string;
  readonly editing: boolean;
  readonly locked: boolean;
  readonly onEdit: (editing: boolean) => void;
  readonly onSettle: () => Promise<void>;
  readonly onTaken: () => Promise<void>;
  readonly onError: (said: string) => void;
  readonly id?: string;
  readonly onKeep?: () => Promise<unknown> | undefined;
}

export type CardHands = ReturnType<typeof useAgentCard>;

interface Opened {
  readonly doc: AgentDoc | null;
  readonly editing: boolean;
  readonly onEdit: (editing: boolean) => void;
}

interface Page {
  readonly locked: boolean;
  readonly flush: () => Promise<unknown> | undefined;
  readonly held: { readonly current: unknown };
  readonly reread: (file: string) => Promise<void>;
  readonly onError: (said: string) => void;
}

interface Paper {
  readonly id: string;
  readonly file: string;
}

export const cardFor = (agent: Opened, paper: Paper | undefined, page: Page): CardProps | null =>
  agent.doc && paper
    ? {
        doc: agent.doc,
        file: paper.file,
        id: paper.id,
        onKeep: page.flush,
        editing: agent.editing,
        locked: page.locked,
        onEdit: agent.onEdit,
        onSettle: async () => {
          await page.flush();
          if (page.held.current) return Promise.reject({ code: "unsavedStays" });
        },
        onTaken: () => page.reread(paper.file),
        onError: page.onError,
      }
    : null;

type Kept<T> = { file: string; one: T } | null;

// Held above the card and the buttons under the words, and dropped as soon as another file opens.
export function useAgentShelf(file: string | null) {
  const [shown, setShown] = useState<Kept<Shown>>(null);
  useEffect(() => {
    setShown(null);
  }, [file]);
  return {
    shown: shown && shown.file === file ? shown.one : null,
    setShown: (one: Shown | null) => setShown(one && file ? { file, one } : null),
  };
}

export type AgentShelf = ReturnType<typeof useAgentShelf>;

export function useAgentCard(props: CardProps, shelf: AgentShelf) {
  const { doc, file, editing, locked, onSettle, onTaken, onError } = props;
  const { shown, setShown } = shelf;
  const single = doc.places.length === 1 ? doc.places[0] : null;
  // Taking the file's words in replaces the page's, so it waits until the page is no longer being written.
  const takes = !locked && !editing;
  const { papersChanged } = useDesk();
  const showing = useRef<string | null>(file);
  useEffect(() => {
    showing.current = file;
    return () => {
      showing.current = null;
    };
  }, [file]);
  const failed = (e: unknown) => onError(saidPlainly(e));
  const done = () => window.dispatchEvent(new Event(AGENTS_CHANGED));
  const show = (place: Place) =>
    agentHeld(file, place.at, null)
      .then((text) => setShown({ place, text }))
      .catch(failed);
  const settled = () => Promise.resolve(onSettle());
  const reread = () => (showing.current === file ? onTaken() : undefined);
  const bring = (place: Place) =>
    settled()
      .then(() => agentBring(file, place.at, doc.print))
      .then(reread)
      .then(papersChanged)
      .catch(failed)
      .finally(done);
  const asked = (said: string, then: () => Promise<unknown>) =>
    sure(said)
      .then((yes) => (yes ? then() : undefined))
      .catch(failed);
  return {
    props,
    own: doc.kind === "memory",
    single,
    takes,
    shown,
    owned: (place: Place) => takes && doc.level === place.assistant,
    turnable: (place: Place) => !doc.folderOff && place.said.state !== "noConsent",
    show,
    bring,
    closeFile: () => setShown(null),
    overwrite: (place: Place) =>
      asked(fill("agentOverwriteSure", place.at), () => agentsKeep(place.at, true).then(done)),
    keepFile: (place: Place) => asked(fill("agentKeepFileSure", place.at), () => bring(place)),
    take: (here: boolean) => agentTake(file, here).then(done).catch(failed),
    settle: () =>
      settled()
        .then(() => agentSettle(file))
        .then(reread)
        .catch(failed)
        .finally(done),
    agree: () =>
      asked(t("agentAgreeSure"), () =>
        settled()
          .then(() => agentAgree(file, doc.print))
          .finally(done),
      ),
    approve: (proposal: Proposal) =>
      asked(t("agentApproveSure"), () =>
        settled()
          .then(() => agentApprove(file, proposal.print, doc.print))
          .then(reread)
          .then(papersChanged)
          .finally(done),
      ),
    discard: (proposal: Proposal) =>
      asked(t("agentDiscardSure"), () => agentDiscard(file, proposal.print).finally(done)),
    lead: () => agentsLead().then(done).catch(failed),
    turn: (place: Place) =>
      agentsTurn(place.assistant, place.said.state === "off")
        .then(done)
        .catch(failed),
    failed,
  };
}
