import { useEffect, useRef, useState } from "react";
import { ALIAS_AT_MOST, sign, signed, signTheRest } from "../core";
import { fill, t } from "../locales";
import type { Run, Which, Word } from "./Card";
import Modal from "./Modal";
import { Ask, Band, Line, off } from "./Rows";

interface Props {
  busy: Which | null;
  said?: Word;
  trouble?: Word;
  run: Run;
  tell: (word?: Word) => void;
  greeted?: number;
}

export default function Signing({ busy, said, trouble, run, tell, greeted }: Props) {
  const held = busy !== null;
  const [alias, setAlias] = useState("");
  const signed_as = useRef("");
  const [aliases, setAliases] = useState<string[]>([]);
  const [mine, setMine] = useState(0);
  const [asking, setAsking] = useState<string | null>(null);
  const before = aliases.filter((one) => one !== alias);

  useEffect(() => {
    signed()
      .then((one) => {
        setAlias(one.alias ?? "");
        signed_as.current = one.alias ?? "";
        setAliases(one.before);
        setMine(one.mine);
      })
      .catch(() => {});
  }, [greeted]);

  return (
    <>
      {asking && (
        <Modal title={fill("aliasNow", asking)} onClose={() => setAsking(null)}>
          <p className="mt-3 text-[12.5px] leading-relaxed text-soft">
            {mine === 1 ? t("aliasRestAskOne") : fill("aliasRestAsk", String(mine))}
          </p>
          <p className="mt-2 text-[11.5px] leading-relaxed text-faint">{t("aliasRestNever")}</p>
          <div className="mt-5 flex flex-wrap items-center justify-end gap-2 text-[12.5px]">
            <button
              type="button"
              onClick={() => setAsking(null)}
              className="rounded-[10px] px-3 py-1.5 text-faint hover:text-ink"
            >
              {t("aliasRestNo")}
            </button>
            <button
              type="button"
              disabled={held}
              onClick={() => {
                setAsking(null);
                run("signing", signTheRest(), (many) => {
                  setMine(0);
                  tell({
                    card: "signing",
                    text: many === 1 ? t("aliasRestDoneOne") : fill("aliasRestDone", String(many)),
                  });
                });
              }}
              className="cursor-pointer rounded-[10px] bg-accent px-3.5 py-1.5 text-bg disabled:opacity-60"
            >
              {t("aliasRestYes")}
            </button>
          </div>
        </Modal>
      )}
      <Band label={t("bandSigning")} />
      <div className="border-t border-hair">
        <Line
          title={
            <span className="flex items-center gap-1.5">
              {t("alias")}
              <Ask said={t("aliasWhy")} />
            </span>
          }
          why={t("aliasShort")}
          which="signing"
          said={said}
          trouble={trouble}
        >
          <input
            type="text"
            aria-label={t("alias")}
            value={alias}
            disabled={held}
            maxLength={ALIAS_AT_MOST}
            placeholder={t("aliasNone")}
            onChange={(e) => setAlias(e.target.value)}
            onBlur={() => {
              const said = alias.trim();
              if (
                said.localeCompare(signed_as.current, undefined, {
                  sensitivity: "accent",
                }) === 0
              ) {
                setAlias(signed_as.current);
                return;
              }
              run("signing", sign(said || undefined), (now) => {
                setAlias(now.alias ?? "");
                setAliases(now.before);
                setMine(now.mine);
                signed_as.current = now.alias ?? "";
                tell({
                  card: "signing",
                  text: now.alias ? fill("aliasKept", now.alias) : t("aliasGone"),
                });
                if (now.alias && now.mine > 0) setAsking(now.alias);
              });
            }}
            list={before.length > 0 ? "signed-before" : undefined}
            className={`w-40 rounded-[10px] border border-line bg-bg px-2 py-1 text-[12.5px] ${off}`}
          />
          {mine > 0 && (
            <button
              type="button"
              disabled={held}
              onClick={() => setAsking(alias)}
              className="rounded-[10px] border border-line px-2 py-1 text-[12.5px] hover:bg-hover disabled:opacity-60"
            >
              {t("aliasRest")}
            </button>
          )}
          {before.length > 0 && (
            <datalist id="signed-before">
              {before.map((one) => (
                <option key={one} value={one} />
              ))}
            </datalist>
          )}
        </Line>
      </div>
    </>
  );
}
