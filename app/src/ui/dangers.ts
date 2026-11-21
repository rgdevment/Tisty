import { fill, t } from "../locales";

export interface Danger {
  title: string;
  lead: string;
  points: string[];
  go: string;
  undone?: boolean;
}

export const DANGERS = {
  restore: (): Danger => ({
    title: t("carefulRestoreTitle"),
    lead: t("carefulRestoreLead"),
    points: [t("carefulRestoreOne"), t("carefulRestoreTwo")],
    go: t("carefulRestoreGo"),
    undone: true,
  }),
  syncOff: (): Danger => ({
    title: t("syncOffTitle"),
    lead: t("carefulSyncOffLead"),
    points: [t("carefulSyncOffOne"), t("carefulSyncOffTwo")],
    go: t("syncOffNow"),
  }),
  forget: (): Danger => ({
    title: t("cloudForgetHere"),
    lead: t("carefulForgetLead"),
    points: [t("carefulForgetOne"), t("carefulForgetTwo")],
    go: t("carefulForgetGo"),
  }),
  revoke: (): Danger => ({
    title: t("cloudRevokeAll"),
    lead: t("carefulRevokeLead"),
    points: [t("carefulRevokeOne"), t("carefulRevokeTwo")],
    go: t("carefulRevokeGo"),
  }),
  abandon: (asked: string): Danger => ({
    title: t("cloudAbandon"),
    lead: asked,
    points: [],
    go: t("cloudAbandon"),
    undone: true,
  }),
  wipe: (provider: string): Danger => ({
    title: t("carefulWipeTitle"),
    lead: fill("carefulWipeLead", provider),
    points: [t("carefulWipeOne"), t("carefulWipeTwo")],
    go: t("carefulWipeGo"),
    undone: true,
  }),
  unseal: (provider: string): Danger => ({
    title: t("sealOff"),
    lead: fill("carefulUnsealLead", provider),
    points: [t("carefulUnsealOne")],
    go: t("carefulUnsealGo"),
  }),
  drop: (name: string, last: string): Danger => ({
    title: t("carefulDropTitle"),
    lead: fill("carefulDropLead", name),
    points: [t("carefulDropOne"), t("carefulDropTwo"), last],
    go: t("carefulDropGo"),
    undone: true,
  }),
  door: (): Danger => ({
    title: t("agentsTurnOff"),
    lead: t("carefulDoorLead"),
    points: [t("carefulDoorOne"), t("carefulDoorTwo")],
    go: t("agentsTurnOff"),
  }),
  stranded: (name: string): Danger => ({
    title: fill("carefulDocTitle", name),
    lead: t("carefulDocLead"),
    points: [t("carefulDocOne")],
    go: t("carefulDocGo"),
    undone: true,
  }),
  loose: (many: number): Danger => ({
    title: many === 1 ? t("carefulLooseTitleOne") : fill("carefulLooseTitle", String(many)),
    lead: t("carefulLooseLead"),
    points: [t("carefulLooseOne")],
    go: t("carefulLooseGo"),
  }),
  missing: (name: string): Danger => ({
    title: fill("carefulDocTitle", name),
    lead: t("carefulDocMissingLead"),
    points: [t("carefulDocOne")],
    go: t("carefulDocGo"),
    undone: true,
  }),
};
