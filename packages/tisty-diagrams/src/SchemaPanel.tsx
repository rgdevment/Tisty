import { useState } from "react";
import { useHands, useSay } from "./handing";
import { caption, field, Said, Section, Ticked } from "./panelbits";
import { ENDS, type End, KEYS, type Key, type Schema } from "./schema";
import { toDbml, toSql } from "./schemaout";
import {
  type Altered,
  columnAdded,
  columnChanged,
  columnRemoved,
  type Picked,
  relationChanged,
  relationRemoved,
  relationsOf,
  tableOf,
  tableRemoved,
  tableRenamed,
} from "./scheming";
import type { Word } from "./words";

const END_WORDS: Record<End, Word> = {
  one: "schemaEndOne",
  "zero-one": "schemaEndZeroOne",
  "zero-many": "schemaEndZeroMany",
  "one-many": "schemaEndOneMany",
};

const TYPES = [
  "int",
  "bigint",
  "string",
  "text",
  "varchar(255)",
  "uuid",
  "bool",
  "date",
  "datetime",
  "decimal(10,2)",
];

/** What is chosen on the board, to rename and reshape, and the schema to copy as SQL or DBML. */
export default function SchemaPanel({
  schema,
  picked,
  onChange,
}: Readonly<{ schema: Schema; picked: Picked; onChange: (made: Altered) => void }>) {
  const t = useSay();
  const { button } = useHands().looks;
  const { copy: toClipboard } = useHands();
  const [copied, setCopied] = useState<string | null>(null);
  const copy = (said: string, what: string) =>
    void toClipboard(said)
      .then(() => setCopied(what))
      .catch(() => setCopied(null));

  const table = picked?.kind === "table" ? tableOf(schema, picked.name) : undefined;
  const owner = picked?.kind === "column" ? tableOf(schema, picked.table) : undefined;
  const column = picked?.kind === "column" ? owner?.columns[picked.index] : undefined;
  const relation = picked?.kind === "relation" ? relationsOf(schema)[picked.index] : undefined;

  return (
    <div className="flex flex-col gap-3">
      {!table && !column && !relation && (
        <p className="m-0 text-[12.5px] text-faint">{t("schemaPick")}</p>
      )}
      {table && (
        <Section title={t("schemaTable")}>
          <Said
            id="schema-table"
            label={t("schemaName")}
            value={table.name}
            onSay={(said) => onChange(tableRenamed(schema, table.name, said))}
          />
          <div className="flex gap-1.5">
            <button
              type="button"
              className={`${button} flex-1`}
              onClick={() => onChange(columnAdded(schema, table.name))}
            >
              {t("schemaAddColumn")}
            </button>
            <button
              type="button"
              className={`${button} text-urgent`}
              onClick={() => onChange(tableRemoved(schema, table.name))}
            >
              {t("schemaRemoveTable")}
            </button>
          </div>
        </Section>
      )}
      {column && owner && picked?.kind === "column" && (
        <Section title={`${t("schemaColumn")} · ${owner.name}`}>
          <Said
            id="schema-column"
            label={t("schemaName")}
            value={column.name}
            onSay={(name) => onChange(columnChanged(schema, owner.name, picked.index, { name }))}
          />
          <Said
            id="schema-type"
            label={t("schemaType")}
            value={column.type}
            onSay={(type) =>
              onChange(
                columnChanged(schema, owner.name, picked.index, { type: type.trim() || "string" }),
              )
            }
          />
          <div className="flex flex-wrap gap-1">
            {TYPES.map((one) => (
              <button
                key={one}
                type="button"
                className={`${button} font-mono text-[11px]`}
                onClick={() =>
                  onChange(columnChanged(schema, owner.name, picked.index, { type: one }))
                }
              >
                {one}
              </button>
            ))}
          </div>
          <div className="flex flex-wrap gap-3">
            {KEYS.map((key: Key) => (
              <Ticked
                key={key}
                label={key}
                on={column.keys.includes(key)}
                onFlip={() => {
                  const keys = column.keys.includes(key)
                    ? column.keys.filter((one) => one !== key)
                    : KEYS.filter((one) => one === key || column.keys.includes(one));
                  onChange(columnChanged(schema, owner.name, picked.index, { keys }));
                }}
              />
            ))}
            <Ticked
              label={t("schemaRequired")}
              on={column.required}
              onFlip={() =>
                onChange(
                  columnChanged(schema, owner.name, picked.index, { required: !column.required }),
                )
              }
            />
          </div>
          <Said
            id="schema-note"
            label={t("schemaNote")}
            value={column.comment ?? ""}
            onSay={(comment) =>
              onChange(
                columnChanged(schema, owner.name, picked.index, {
                  comment: comment.trim() || undefined,
                }),
              )
            }
          />
          <button
            type="button"
            className={`${button} self-start text-urgent`}
            onClick={() => onChange(columnRemoved(schema, owner.name, picked.index))}
          >
            {t("schemaRemoveColumn")}
          </button>
        </Section>
      )}
      {relation && picked?.kind === "relation" && (
        <Section title={t("schemaRelation")}>
          {(["fromEnd", "toEnd"] as const).map((side) => (
            <label key={side} className="flex flex-col gap-1">
              <span className={caption}>
                {t("schemaSide", side === "fromEnd" ? relation.from : relation.to)}
              </span>
              <select
                className={field}
                value={relation[side]}
                onChange={(e) =>
                  onChange(relationChanged(schema, picked.index, { [side]: e.target.value as End }))
                }
              >
                {ENDS.map((end) => (
                  <option key={end} value={end}>
                    {t(END_WORDS[end])}
                  </option>
                ))}
              </select>
            </label>
          ))}
          <Said
            id="schema-label"
            label={t("schemaLabel")}
            value={relation.label}
            onSay={(label) =>
              onChange(relationChanged(schema, picked.index, { label: label.trim() }))
            }
          />
          <Ticked
            label={t("schemaIdentifying")}
            on={relation.identifying}
            onFlip={() =>
              onChange(
                relationChanged(schema, picked.index, { identifying: !relation.identifying }),
              )
            }
          />
          <button
            type="button"
            className={`${button} self-start text-urgent`}
            onClick={() => onChange(relationRemoved(schema, picked.index))}
          >
            {t("schemaRemoveRelation")}
          </button>
        </Section>
      )}
      <div className="flex items-center gap-1.5">
        <button type="button" className={button} onClick={() => copy(toSql(schema), "SQL")}>
          {t("schemaCopySql")}
        </button>
        <button type="button" className={button} onClick={() => copy(toDbml(schema), "DBML")}>
          {t("schemaCopyDbml")}
        </button>
        {copied && <output className="text-[12px] text-faint">{t("schemaCopied", copied)}</output>}
      </div>
    </div>
  );
}
