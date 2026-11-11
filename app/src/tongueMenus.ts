import type { Filed } from "./core";
import type { Hands } from "./docMenus";
import { fill, locale, t } from "./locales";
import { docTongue, docTranslate, tongueName, tonguesOf } from "./tongues";
import type { Choice } from "./ui/Menu";

// What a person most often writes in, with the app's own language first.
const COMMON = ["es", "en", "pt", "fr", "de", "it", "ca", "nl", "ja", "zh", "ko", "ru"];

const languages = (but: readonly (string | null | undefined)[]): string[] => {
  const app = locale().split("-")[0];
  return [app, ...COMMON.filter((one) => one !== app)].filter((one) => !but.includes(one));
};

// Each question names the document it asks about, so it is never a guess which one.
const asked = (doc: Filed) => fill("tongueOf", doc.title || t("untitledDoc"));

const named = (lang: string, onPick: () => void, into?: Choice["into"]): Choice => ({
  key: lang,
  icon: "◌",
  label: tongueName(lang),
  onPick: into ? undefined : onPick,
  into,
});

// The language a document says, or a step that asks for it before going on.
const saying = (
  doc: Filed,
  ask: string,
  but: readonly (string | null | undefined)[],
  then: (said?: string) => Choice,
): Choice =>
  doc.lang
    ? then()
    : {
        ...then(),
        onPick: undefined,
        into: {
          label: ask,
          choices: languages(but).map((lang) => named(lang, () => then(lang).onPick?.())),
        },
      };

export const tongueChoices = (doc: Filed, h: Hands): Choice[] => {
  const tongues = tonguesOf(h.papers.docs, doc);
  const said = tongues.length ? tongues.map((one) => one.lang) : [doc.lang];
  // A language added under someone else's edit link waits for its owner before readers see it (166).
  const anchor = tongues.find((one) => one.id === doc.tongue?.[0]) ?? doc;
  const lone = doc.away || !!doc.pageOf;
  const translate = (lang: string, from?: string) =>
    docTranslate(doc.id, lang, from)
      .then((made) => {
        h.changed();
        h.open(made.id);
        if (anchor.editing) h.noted(t("translateProposed"));
      })
      .catch(h.fail);
  const join = (other: Filed, lang?: string, theirs?: string) =>
    docTongue(doc.id, other.id, lang, theirs).then(h.changed).catch(h.fail);
  // Its languages stand in one folder, so the others are looked for beside it and one folder up.
  const above = h.papers.folders.find((one) => one.id === doc.folder)?.parent;
  const near = (folder: string | null) =>
    folder === (doc.folder ?? null) || (above !== undefined && folder === above);
  const others = h.papers.docs.filter(
    (one) =>
      one.id !== doc.id &&
      !one.pageOf &&
      !one.away &&
      near(one.folder ?? null) &&
      (!doc.tongue || one.tongue?.[0] !== doc.tongue[0]),
  );
  return [
    {
      key: "translate",
      icon: "◌",
      label: t("translateTo"),
      hint: anchor.editing ? t("translateProposed") : undefined,
      off: lone || (!doc.lang && doc.locked),
      into: {
        label: t("translateWhich"),
        choices: languages(said).map((lang) =>
          saying(doc, asked(doc), [lang], (from) => named(lang, () => translate(lang, from))),
        ),
      },
    },
    {
      key: "joinTongues",
      icon: "◌",
      label: t("joinTongues"),
      off: lone || doc.locked || others.length === 0,
      into: {
        label: t("joinWhich"),
        choices: others.map((other) => {
          const theirs = (lang?: string): Choice =>
            saying(other, asked(other), [lang], (their) => ({
              key: other.id,
              icon: "▤",
              label: other.title || t("untitledDoc"),
              onPick: () => join(other, lang, their),
            }));
          return doc.lang
            ? theirs()
            : {
                key: other.id,
                icon: "▤",
                label: other.title || t("untitledDoc"),
                into: {
                  label: asked(doc),
                  choices: languages([other.lang]).map((lang) => {
                    const next = theirs(lang);
                    return named(lang, () => next.onPick?.(), next.into);
                  }),
                },
              };
        }),
      },
    },
    {
      key: "partTongues",
      icon: "◌",
      label: t("partTongues"),
      off: !doc.tongue || doc.away,
      onPick: () => docTongue(doc.id).then(h.changed).catch(h.fail),
    },
  ];
};
