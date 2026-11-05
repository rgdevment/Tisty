export declare const VERSION: "1.2.0";

export type PictureKind = "image/png" | "image/jpeg";

export type EnvelopeKind = "main" | "files";

export type PartName = "first" | "rest" | "picture" | "file";

export type SheetsWord = "on";

export type SaidPriority = "do" | "decide" | "delegate" | "minor";

export type Verdict = "holds" | "retired" | "doubtful" | "refused" | "unreadable";

export type Paper = "a4" | "letter" | "folio" | "tabloid";

export type Orientation = "portrait" | "landscape";

export type Status = "open" | "done" | "dropped";

export type Priority = "do" | "decide" | "delegate" | "unset" | "minor";

/** A file a document names. A picture travels in its main envelope and is drawn with it. Any other file travels in its files envelope, shows as a card and is asked for the first time it is opened. */
export interface Attachment {
  at: number;
  name: string;
  kind?: string;
  size?: number;
}

/** An attachment that is drawn. Its bytes are pixels the writer wrote itself, so an image is nothing but an image. Its size is known before it is drawn, so its place on the sheet is held and no cut moves when it arrives. */
export interface Picture {
  at: number;
  name: string;
  kind: PictureKind;
  size: number;
  width: number;
  height: number;
}

/** A small isolated view inside a document, which can run its own script. Tisty runs it. Whoever does not implement this contract never runs it, and shows a card that says it is seen in Tisty. */
export interface Widget {
  title?: string;
  body: string;
}

/** A document is the parent: what a person writes and keeps. It holds its words, with all they contain, in one language, and it can hold pages. */
export interface Document {
  id: string;
  title?: string;
  tags?: string[];
  lang?: string;
  sheet?: Sheet;
  signed_as?: string;
  owner_name?: string;
  made?: string;
  wrote?: string;
  body: string;
  html: string;
  pages?: Page[];
  pictures?: Picture[];
  files?: Attachment[];
  files_hash?: string;
  cited?: Cited[];
  languages?: Translation[];
}

export interface DocumentReader {
  show(document: Document): void;
  setPaper(paper: Paper): void;
  setSheets(on: boolean): void;
  setOrientation(orientation: Orientation): void;
  index(): string[];
  details(): Document;
  verdict(file: Uint8Array, served: Served): Promise<Verdict>;
  goToPage(at: number): void;
  setLanguage(lang: string): void;
  editInTisty(): void;
  copyLink(): string;
}

/** How a document travels, as one file the server stores unopened. Per language, the main envelope holds its words, all its pages and its pictures, with references ready, so reading them takes no other request. A second one holds its other files, and exists only when it has any. These fields are the manifest, written in clear, and all the server reads. */
export interface Envelope {
  contract: string;
  kind: EnvelopeKind;
  id: string;
  version: number;
  from?: number;
  sealed: boolean;
  head?: string;
  commitment?: string;
  owner?: Owner;
  parts: EnvelopePart[];
}

export interface EnvelopeWriter {
  seal(document: Document, pictures: Uint8Array[], files: Uint8Array[], sealing: Sealing): Promise<Uint8Array[]>;
}

export interface EnvelopeReader {
  open(file: Uint8Array, opening: Opening): Promise<Uint8Array[]>;
}

/** What a writer needs to seal a document, beyond the document and its bytes. */
export interface Sealing {
  id: string;
  version: number;
  from?: number;
  link_key?: string;
  files_hash?: string;
  files_key?: string;
  owner?: Owner;
  token?: string;
}

/** What a reader asks for when it opens an envelope. */
export interface Opening {
  id: string;
  kind: EnvelopeKind;
  link_key?: string;
}

/** One part of an envelope, as the manifest tells it. The server holds its limits to these sizes. */
export interface EnvelopePart {
  name: PartName;
  compressed: boolean;
  sealed_size: number;
  open_size: number;
}

export interface SyncWriter {
  sync(): void;
}

/** The properties at the start of a document, between two lines of three dashes. The first line is ---, the words start right under it, and --- or ... closes them. Past their limits they stay as written and none is read. A value that says nothing Tisty knows stays written too and does nothing, so no document fails for its properties: the writer reads them and gives the document what they mean. This holds what they mean, once read. */
export interface FrontMatter {
  title?: string;
  tags?: string[];
  paper?: Paper;
  sheets?: SheetsWord;
  orientation?: Orientation;
  lang?: string;
  translationKey?: string;
  task?: string;
  scheduled?: string;
  due?: string;
  remind?: string[];
  list?: string;
  priority?: SaidPriority;
  repeat?: string;
}

/** A page is a document like any other, with one difference: it says which document it belongs to. There is one level and no more, so a page never has pages of its own. It travels inside its document's envelopes, so what they hold is said once. */
export interface Page {
  id: string;
  title?: string;
  tags?: string[];
  lang?: string;
  sheet?: Sheet;
  made?: string;
  wrote?: string;
  body: string;
  html: string;
  page_of: string;
  at: number;
}

/** Where the same document is in another language. */
export interface Translation {
  lang: string;
  document: string;
  title?: string;
  key?: string;
}

/** A word a document or a task is kept under, in the one form Tisty keeps it, as a tag_word. Capitals and accents come off and a space or an underscore turns into a dash, so #Camión and #camion are one tag. In a sentence, #word needs two characters and a letter, and it does not count glued to the text before it, inside code or in [[#2]]. */
export interface Tag {
  name: string;
}

/** Who answers for an envelope, and through it for the document. The root key is who a person is, and it never changes: a version under another root is someone else. The machine key signs, under a certificate from the root, and it can be replaced. It travels in clear, so the server checks it without opening anything. */
export interface Signature {
  root: string;
  machine: string;
  since: string;
  certificate: string;
  signed: string;
  permit_signed?: string;
}

/** Who owns what is sealed, and who may edit it. It answers for the document and never for its words: an editor signs each version, and the owner's signature says only that this root owns this id and let this permit edit it. A version holds if its signature is under this root, or if it brings a permit_signed that holds against this permit. */
export interface Owner {
  root: string;
  machine: string;
  since: string;
  certificate: string;
  permit?: string;
  name_tag: string;
  signed: string;
}

/** A person's root saying it no longer answers for one of their machine keys, since a moment, as when a machine is lost or retired. It deletes nothing and stops nothing already shared, and whoever holds that machine can still read what it holds. Everything it decides is decided by whoever receives it. */
export interface Revocation {
  root: string;
  machine: string;
  at: string;
  signed: string;
}

/** What the server serves beside an envelope, in clear, so a reader can weigh its signature. A reader trusts it as far as it trusts the server: it says according to the server, and a server that hides a revocation cannot be caught on the web. */
export interface Served {
  received?: string;
  revocations?: Revocation[];
}

/** Another document shared along with this one. It keeps its own envelopes, because it is not part of this document. */
export interface Cited {
  document: string;
  key?: string;
  title?: string;
}

/** How a document or a page is laid on paper: its size and its orientation. A sheet holds nothing of its own, and measures on screen what it measures on paper. A line of three dashes, the page break, cuts a sheet and never a page. */
export interface Sheet {
  paper?: Paper;
  on?: boolean;
  orientation?: Orientation;
}

/** A task is something to do. It ends, and is kept whole. Only whoever needs tasks implements this. */
export interface Task {
  id: string;
  title: string;
  status: Status;
  priority?: Priority;
  description?: string;
  date?: string;
  deadline?: string;
  list?: string;
  tags?: string[];
  repeat?: string;
  part_of?: string;
  after?: string;
  source?: string;
  steps?: Step[];
  notes?: Note[];
  reminders?: Reminder[];
}

/** One line of a task's checklist. */
export interface Step {
  text: string;
  done: boolean;
}

/** Something recorded on a task, with when. */
export interface Note {
  at: string;
  body: string;
}

/** When a task rings, written as a task's date is. */
export interface Reminder {
  at: string;
}

export type ContractName = "text" | "table" | "code" | "diagram" | "formula" | "attachment" | "picture" | "widget" | "document" | "envelope" | "sealing" | "opening" | "envelope_part" | "sync" | "link" | "front_matter" | "page" | "translation" | "tag" | "signature" | "owner" | "revocation" | "served" | "cited" | "sheet" | "style" | "task" | "step" | "note" | "reminder";
export declare const CONTRACTS: Record<ContractName, unknown>;
export declare const ALLOWED: Record<string, { attributes: string[]; classes: string[]; class_starts: string[]; class_ends: string[] }>;
/** What a person may write by hand, for whoever cleans a document. */
export declare const PASSTHROUGH: { tags: string[]; attributes: Record<string, string[]> };
export declare const STYLES: { on: string; property: string; unit?: string; sizes?: string[]; color?: boolean; values?: string[] }[];
export declare const URLS: { schemes: string[]; attributes: string[]; references: string[]; paints: string[] };
/** What is wrong with a value under this contract: nothing when it holds. */
export declare function check(contract: ContractName, value: unknown): string[];
/** What in the HTML of a document is outside the contracts. It is no cleaner: it says whether a document speaks this language. */
export declare function checkHtml(html: string): string[];
export declare function referencesIn(html: string): { kind: string; at: number }[];
export declare function picturesIn(html: string): { at: number; width?: string; height?: string }[];
