import { ARROWS } from "./classes";
import {
  classKindChanged,
  classRemoved,
  classRenamed,
  linkChanged,
  linkRemoved,
  memberAdded,
  memberChanged,
  memberRemoved,
  visibilityOf,
  withVisibility,
} from "./classing";
import { KINDS, SORTS, VISIBILITY_WORDS } from "./classkinds";
import { useHands, useSay } from "./handing";
import { Chosen, Said, Section } from "./panelbits";
import type { Classing } from "./useClasses";

/** The chosen class with its members, or the chosen relation with its kind, ends and words. */
export default function ClassPanel({ editing }: Readonly<{ editing: Classing }>) {
  const t = useSay();
  const { button } = useHands().looks;
  const { diagram, picked, change } = editing;
  const name = picked?.kind === "class" || picked?.kind === "member" ? picked.name : null;
  const chosen = name ? diagram.classes.find((one) => one.name === name) : undefined;
  const link = picked?.kind === "link" ? diagram.links[picked.index] : undefined;
  const sort = chosen?.note ?? "";
  // A kind written by hand that the list does not offer is still shown, as it is written.
  const sorts = [
    ...SORTS.map(([note, word]) => ({ value: note ?? "", label: t(word) })),
    ...(SORTS.some(([note]) => (note ?? "") === sort) ? [] : [{ value: sort, label: `«${sort}»` }]),
  ];
  const visibilities = VISIBILITY_WORDS.map(([mark, word]) => ({ value: mark, label: t(word) }));
  const kinds = [
    ...KINDS.map(([arrow, word]) => ({ value: arrow, label: `${t(word)}  ${arrow}` })),
    ...ARROWS.filter((one) => !KINDS.some(([arrow]) => arrow === one)).map((one) => ({
      value: one,
      label: one,
    })),
  ];

  return (
    <div className="flex flex-col gap-3">
      {!chosen && !link && (
        <p className="m-0 text-[12.5px] text-faint">
          {picked?.kind === "classes"
            ? t("flowPickedMany", String(picked.names.length))
            : t("classPick")}
        </p>
      )}
      {chosen && (
        <Section title={t("classTitle")}>
          <Said
            id="class-name"
            label={t("schemaName")}
            value={chosen.name}
            onSay={(said) => change(classRenamed(diagram, chosen.name, said))}
          />
          <Chosen
            id="class-sort"
            label={t("classSort")}
            value={sort}
            options={sorts}
            onChoose={(note) => change(classKindChanged(diagram, chosen.name, note || null))}
          />
          <span className="text-[12px] font-semibold text-soft">{t("classMembers")}</span>
          {chosen.members.map((member, index) => (
            <div key={member.line} className="flex items-end gap-1.5">
              <Chosen
                id={`class-visibility-${index}`}
                label={t("classVisibility")}
                value={visibilityOf(member.text)}
                options={visibilities}
                onChoose={(mark) =>
                  change(
                    memberChanged(diagram, chosen.name, index, withVisibility(member.text, mark)),
                  )
                }
              />
              <div className="flex flex-1 flex-col gap-1">
                <Said
                  id={`class-member-${index}`}
                  label={member.text.includes("(") ? t("classMethod") : t("classAttribute")}
                  value={member.text}
                  onSay={(said) => change(memberChanged(diagram, chosen.name, index, said))}
                />
              </div>
              <button
                type="button"
                className={`${button} text-urgent`}
                aria-label={t("classRemoveMember")}
                onClick={() => change(memberRemoved(diagram, chosen.name, index))}
              >
                ×
              </button>
            </div>
          ))}
          <div className="flex gap-1.5">
            <button
              type="button"
              className={`${button} flex-1`}
              onClick={() => change(memberAdded(diagram, chosen.name, t("classNewAttribute")))}
            >
              {t("classAddAttribute")}
            </button>
            <button
              type="button"
              className={`${button} flex-1`}
              onClick={() => change(memberAdded(diagram, chosen.name, t("classNewMethod")))}
            >
              {t("classAddMethod")}
            </button>
          </div>
          <button
            type="button"
            className={`${button} self-start text-urgent`}
            onClick={() => change(classRemoved(diagram, chosen.name))}
          >
            {t("classRemove")}
          </button>
        </Section>
      )}
      {link && picked?.kind === "link" && (
        <Section title={`${t("classLinkTitle")} · ${link.left} – ${link.right}`}>
          <Chosen
            id="class-kind"
            label={t("classKind")}
            value={link.arrow}
            options={kinds}
            onChoose={(arrow) => change(linkChanged(diagram, picked.index, { arrow }))}
          />
          <Said
            id="class-left"
            label={`${t("classCard")} · ${link.left}`}
            value={link.leftCard}
            onSay={(leftCard) => change(linkChanged(diagram, picked.index, { leftCard }))}
          />
          <Said
            id="class-right"
            label={`${t("classCard")} · ${link.right}`}
            value={link.rightCard}
            onSay={(rightCard) => change(linkChanged(diagram, picked.index, { rightCard }))}
          />
          <Said
            id="class-label"
            label={t("flowLabel")}
            value={link.label}
            onSay={(label) => change(linkChanged(diagram, picked.index, { label }))}
          />
          <button
            type="button"
            className={`${button} self-start text-urgent`}
            onClick={() => change(linkRemoved(diagram, picked.index))}
          >
            {t("classRemoveLink")}
          </button>
        </Section>
      )}
    </div>
  );
}
