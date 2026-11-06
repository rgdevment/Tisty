import { openUrl } from "@tauri-apps/plugin-opener";
import { useCallback, useEffect, useRef, useState } from "react";
import { copied } from "../core";
import { weigh } from "../format";
import { helpDrawn } from "../guide/calls";
import { fill, locale, t } from "../locales";
import {
  docShared,
  type ShareCard,
  type ShareLook,
  shareLook,
  shareStop,
  sharingNow,
  type TermsNow,
  termsAccepted,
  termsNow,
  toldPlainly,
} from "../sharing";
import Modal from "./Modal";

const SECRETS_SHOWN = 3;
const FOLIO_SITE = "folio.tisty.app";
const DAYS_FALLBACK = [1, 3, 7, 15];
const TERMS_ON_THE_WEB: Record<string, string> = {
  es: "https://github.com/rgdevment/Folio-by-Tisty/blob/main/docs/TERMINOS.md",
  en: "https://github.com/rgdevment/Folio-by-Tisty/blob/main/docs/TERMS.md",
};

const codeOf = (problem: unknown) => (problem as { code?: string } | null)?.code;

const endsAfter = (days: number) =>
  days === 1 ? t("shareEndsDay") : fill("shareEndsDays", String(days));

const countOf = (carried: number, most?: number) =>
  most === undefined ? String(carried) : fill("shareCarriedOf", String(carried), String(most));

const weightOf = (carried: number, most?: number) =>
  most === undefined ? weigh(carried) : fill("shareCarriedOf", weigh(carried), weigh(most));

const onTheWeb = () => TERMS_ON_THE_WEB[locale()] ?? TERMS_ON_THE_WEB.en;

function Terms({
  terms,
  onAccepted,
  onLater,
  onError,
}: Readonly<{
  terms: Extract<TermsNow, { state: "toAccept" }>;
  onAccepted: () => void;
  onLater: () => void;
  onError?: (problem: unknown) => void;
}>) {
  const [agreed, setAgreed] = useState(false);
  const [drawn, setDrawn] = useState("");
  const box = useRef<HTMLDivElement>(null);

  useEffect(() => {
    let live = true;
    helpDrawn(terms.text)
      .then((html) => {
        if (live) setDrawn(html);
      })
      .catch((problem) => onError?.(problem));
    return () => {
      live = false;
    };
  }, [terms.text, onError]);

  useEffect(() => {
    const here = box.current;
    if (!here) return;
    here.innerHTML = drawn;
    for (const link of here.querySelectorAll<HTMLAnchorElement>("a[href]")) {
      link.addEventListener("click", (event) => {
        event.preventDefault();
        const url = new URL(link.getAttribute("href") ?? "", onTheWeb());
        if (url.protocol === "https:") void openUrl(url.href).catch(() => {});
      });
    }
  }, [drawn]);

  const accept = () => {
    termsAccepted(terms.version)
      .then(onAccepted)
      .catch((problem) => onError?.(problem));
  };

  return (
    <Modal
      title={t(terms.acceptedBefore ? "termsChangedTitle" : "termsFirstTitle")}
      onClose={onLater}
      wide
    >
      <p className="-mt-2 mb-4 text-[12.5px] text-soft">
        {terms.acceptedBefore
          ? fill("termsChangedWhy", terms.acceptedBefore, terms.version)
          : fill("termsFirstWhy", terms.version)}
      </p>
      <div className="flex flex-col gap-4">
        <div
          ref={box}
          className="scroller max-h-[306px] rounded-[10px] border border-hair bg-sheet px-4 py-3.5 text-[12.5px] leading-relaxed"
        />
        <label className="flex cursor-pointer items-start gap-2 text-[12.5px]">
          <input
            type="checkbox"
            className="mt-0.5"
            checked={agreed}
            onChange={(e) => setAgreed(e.target.checked)}
          />
          <span>{t("termsAgree")}</span>
        </label>
        <div className="flex items-center justify-end gap-2">
          <button
            type="button"
            onClick={() => void openUrl(onTheWeb()).catch(() => {})}
            className="mr-auto text-[11.5px] text-accent"
          >
            {t("termsOnTheWeb")}
          </button>
          <button
            type="button"
            onClick={onLater}
            className="h-8 rounded-md border border-line px-3 text-[12.5px] hover:bg-hover"
          >
            {t("termsLater")}
          </button>
          <button
            type="button"
            disabled={!agreed}
            onClick={accept}
            className="h-8 rounded-md bg-accent px-3 text-[12.5px] font-semibold text-white disabled:opacity-50"
          >
            {t("termsGo")}
          </button>
        </div>
      </div>
    </Modal>
  );
}

function NotOpenYet({ onClose }: Readonly<{ onClose: () => void }>) {
  return (
    <Modal title={t("shareClosedTitle")} onClose={onClose}>
      <div className="flex flex-col gap-4">
        <p className="text-[13px] leading-relaxed">
          {t("shareClosedWhat")}
          <br />
          {t("shareClosedYet")}{" "}
          <button
            type="button"
            onClick={() => void openUrl(`https://${FOLIO_SITE}`).catch(() => {})}
            className="text-accent"
          >
            {FOLIO_SITE}
          </button>
          .
        </p>
        <div className="flex justify-end">
          <button
            type="button"
            onClick={onClose}
            className="h-8 rounded-md bg-accent px-3 text-[12.5px] font-semibold text-white"
          >
            {t("shareClosedOk")}
          </button>
        </div>
      </div>
    </Modal>
  );
}

function NoAnswer({ onRetry, onClose }: Readonly<{ onRetry: () => void; onClose: () => void }>) {
  return (
    <Modal title={t("shareUnreachedTitle")} onClose={onClose}>
      <div className="flex flex-col gap-4">
        <p className="text-[13px] leading-relaxed">{t("shareUnreachedWhy")}</p>
        <p className="text-[11.5px] text-soft">{t("shareUnreachedSent")}</p>
        <div className="flex justify-end gap-2">
          <button
            type="button"
            onClick={onClose}
            className="h-8 rounded-md border border-line px-3 text-[12.5px] hover:bg-hover"
          >
            {t("shareUnreachedClose")}
          </button>
          <button
            type="button"
            onClick={onRetry}
            className="h-8 rounded-md bg-accent px-3 text-[12.5px] font-semibold text-white"
          >
            {t("shareUnreachedRetry")}
          </button>
        </div>
      </div>
    </Modal>
  );
}

function Links({ card }: Readonly<{ card: ShareCard }>) {
  return (
    <div className="flex flex-col gap-3">
      <LinkLine label={t("shareLinkRead")} url={card.read} />
      {card.edit && (
        <div className="flex flex-col gap-1">
          <LinkLine label={t("shareLinkEdit")} url={card.edit} />
          <p className="text-[11.5px] leading-relaxed text-faint">{t("sharedDoneEdit")}</p>
        </div>
      )}
    </div>
  );
}

function LinkLine({ label, url }: Readonly<{ label: string; url: string }>) {
  return (
    <div className="flex flex-col gap-1">
      <div className="flex items-baseline justify-between gap-2 text-[12.5px] font-medium">
        <span>{label}</span>
        <button
          type="button"
          onClick={() => void copied(url).catch(() => {})}
          className="text-[11.5px] font-normal text-accent"
        >
          {t("shareCopy")}
        </button>
      </div>
      <span className="truncate rounded-md bg-hover px-2 py-1.5 font-mono text-[11.5px] text-soft">
        {url}
      </span>
    </div>
  );
}

function LeftOut({ told }: Readonly<{ told: ShareCard["told"] }>) {
  if (told.length === 0) return null;
  return (
    <ul className="flex flex-col gap-1 rounded-[10px] bg-hue-amber/10 px-3.5 py-2.5 text-[12.5px]">
      {told.map((one) => {
        const said = toldPlainly(one);
        return <li key={said}>{said}</li>;
      })}
    </ul>
  );
}

function Done({ card, onClose }: Readonly<{ card: ShareCard; onClose: () => void }>) {
  return (
    <Modal title={t("sharedDone")} onClose={onClose} wide>
      <p className="-mt-2 mb-4 text-[12.5px] text-soft">{t("sharedDoneCopied")}</p>
      <div className="flex flex-col gap-4">
        <Links card={card} />
        <p className="text-[11.5px] leading-relaxed text-faint">{t("sharedDoneKey")}</p>
        <LeftOut told={card.told} />
        <div className="flex justify-end gap-2">
          <button
            type="button"
            onClick={() => void openUrl(card.read).catch(() => {})}
            className="h-8 rounded-md border border-line px-3 text-[12.5px] hover:bg-hover"
          >
            {t("shareOpenWeb")}
          </button>
          <button
            type="button"
            onClick={onClose}
            className="h-8 rounded-md bg-accent px-3 text-[12.5px] font-semibold text-white"
          >
            {t("shareFinish")}
          </button>
        </div>
      </div>
    </Modal>
  );
}

function Going({ until, onCancel }: Readonly<{ until: number | null; onCancel: () => void }>) {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    if (until === null) return;
    const ticking = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(ticking);
  }, [until]);
  const left = until === null ? 0 : Math.max(0, Math.ceil((until - now) / 1000));
  const said = `${Math.floor(left / 60)}:${String(left % 60).padStart(2, "0")}`;
  return (
    <Modal title={t("shareGoing")} wide>
      <div className="flex flex-col gap-4">
        <output className="text-[13px]">
          {until === null ? t("shareGoingWhy") : t("shareWaiting")}
        </output>
        {until !== null && (
          <p className="text-[11.5px] text-soft">{fill("shareWaitingWhy", said)}</p>
        )}
        {until !== null && (
          <div className="flex justify-end">
            <button
              type="button"
              onClick={onCancel}
              className="h-8 rounded-md border border-line px-3 text-[12.5px] hover:bg-hover"
            >
              {t("shareCancel")}
            </button>
          </div>
        )}
      </div>
    </Modal>
  );
}

const past = (carried: number, most?: number) => most !== undefined && carried > most;

const pastTheLimits = ({ carrying, limits }: ShareLook) =>
  past(carrying.pictures, limits.pictures_at_most) ||
  past(carrying.picturesBytes, limits.pictures_bytes_at_most) ||
  past(carrying.largestPicture, limits.picture_bytes_at_most) ||
  past(carrying.files, limits.files_at_most) ||
  past(carrying.filesBytes, limits.files_bytes_at_most);

function Choosing({
  doc,
  title,
  look,
  changed,
  onShare,
  onClose,
  onLookAgain,
  onError,
}: Readonly<{
  doc: string;
  title: string;
  look: ShareLook;
  changed: boolean;
  onShare: (chosen: Parameters<typeof docShared>[1]) => void;
  onClose: () => void;
  onLookAgain: () => void;
  onError?: (problem: unknown) => void;
}>) {
  const [editable, setEditable] = useState(false);
  const [signs, setSigns] = useState<"alias" | "name">(look.alias ? "alias" : "name");
  const [asked, setAsked] = useState("");
  const [days, setDays] = useState(look.limits.ttl_days?.[0] ?? DAYS_FALLBACK[0]);
  const [cites, setCites] = useState(false);
  const secrets = cites ? [...look.secrets, ...look.citedSecrets] : look.secrets;
  const others = look.shared.filter((one) => one.doc !== doc);
  const already = others.length !== look.shared.length;
  const most = look.limits.shares_at_most;
  const full = !already && most !== undefined && others.length >= most;
  const named = look.alias || look.name;
  const chosenName = signs === "alias" ? look.alias : look.name;
  const signedAs = named ? (chosenName ?? undefined) : asked;
  const over = pastTheLimits(look);
  const ready = !full && !over && Boolean(signedAs?.trim());

  const stopOne = (one: string) => {
    shareStop(one)
      .then(onLookAgain)
      .catch((problem) => onError?.(problem));
  };

  return (
    <Modal title={t("shareTitle")} onClose={onClose} wide>
      <p className="-mt-2 mb-4 truncate text-[12.5px] text-soft">{title}</p>
      <div className="flex flex-col gap-5">
        {full && (
          <div
            role="alert"
            className="flex flex-col gap-1.5 rounded-[10px] bg-hue-amber/10 px-3.5 py-2.5"
          >
            <span className="text-[13px] font-semibold text-hue-amber">
              {fill("shareFullHead", String(others.length))}
            </span>
            <ul className="flex flex-col gap-1 text-[12.5px]">
              {others.map((one) => (
                <li key={one.doc} className="flex items-center justify-between gap-3">
                  <span className="truncate">{one.title}</span>
                  <button
                    type="button"
                    onClick={() => stopOne(one.doc)}
                    className="h-7 shrink-0 rounded-md border border-line px-2.5 text-[11.5px] hover:bg-hover"
                  >
                    {t("shareStop")}
                  </button>
                </li>
              ))}
            </ul>
            <span className="text-[11.5px] leading-relaxed text-soft">
              {fill("shareFullWhy", String(most))}
            </span>
          </div>
        )}
        {changed && <output className="text-[12.5px] text-soft">{t("folioChanged")}</output>}
        <LeftOut told={look.told} />
        {over && (
          <div
            role="alert"
            className="flex flex-col gap-1 rounded-[10px] bg-hue-amber/10 px-3.5 py-2.5"
          >
            <span className="text-[13px] font-semibold text-hue-amber">{t("shareOverHead")}</span>
            <span className="text-[11.5px] leading-relaxed text-soft">{t("shareOverWhy")}</span>
          </div>
        )}
        {secrets.length > 0 && (
          <div
            role="alert"
            className="flex flex-col gap-1 rounded-[10px] bg-hue-amber/10 px-3.5 py-2.5"
          >
            <span className="text-[13px] font-semibold text-hue-amber">{t("shareSecrets")}</span>
            <ul className="text-[12.5px]">
              {secrets.slice(0, SECRETS_SHOWN).map((one) => (
                <li key={`${one.under}:${one.named}`}>
                  {one.under ? fill("shareSecretUnder", one.under) : t("shareSecretTop")}
                  {one.named && <code className="ml-1.5 font-mono text-[11.5px]">{one.named}</code>}
                </li>
              ))}
              {secrets.length > SECRETS_SHOWN && (
                <li>{fill("shareSecretsMore", String(secrets.length - SECRETS_SHOWN))}</li>
              )}
            </ul>
            <span className="text-[11.5px] leading-relaxed text-soft">{t("shareSecretsWhy")}</span>
          </div>
        )}

        <fieldset className="flex flex-col gap-2">
          <legend className="mb-2 text-[12.5px] font-semibold">{t("shareWhat")}</legend>
          <div className="grid grid-cols-2 gap-1 rounded-[10px] bg-hover p-1">
            {[false, true].map((one) => (
              <label
                key={String(one)}
                className="flex h-8 cursor-pointer items-center justify-center gap-2 rounded-md text-[12.5px] has-[:checked]:bg-bg has-[:checked]:font-semibold has-[:checked]:shadow-lift"
              >
                <input
                  type="radio"
                  name="what"
                  className="sr-only"
                  checked={editable === one}
                  onChange={() => setEditable(one)}
                />
                {t(one ? "shareCanEdit" : "shareReadOnly")}
              </label>
            ))}
          </div>
          <p className="text-[11.5px] leading-relaxed text-faint">
            {t(editable ? "shareEditWhy" : "shareReadWhy")}
          </p>
        </fieldset>

        <fieldset className="flex flex-col gap-1.5">
          <legend className="mb-2 text-[12.5px] font-semibold">{t("shareSign")}</legend>
          {look.alias && (
            <label className="flex cursor-pointer items-center gap-2 text-[12.5px]">
              <input
                type="radio"
                name="signs"
                checked={signs === "alias"}
                onChange={() => setSigns("alias")}
              />
              {fill("shareSignAlias", look.alias)}
            </label>
          )}
          {look.name && look.name !== look.alias && (
            <label className="flex cursor-pointer items-center gap-2 text-[12.5px]">
              <input
                type="radio"
                name="signs"
                checked={signs === "name"}
                onChange={() => setSigns("name")}
              />
              {fill("shareSignName", look.name)}
            </label>
          )}
          {!named && (
            <label className="flex flex-col gap-1 text-[12.5px]">
              {t("shareSignAsk")}
              <input
                value={asked}
                onChange={(e) => setAsked(e.target.value)}
                className="h-8 rounded-md border border-line bg-bg px-2.5 text-[13px]"
              />
            </label>
          )}
          <p className="text-[11.5px] leading-relaxed text-faint">
            {t(named ? "shareSignWhy" : "shareSignAskWhy")}
          </p>
        </fieldset>

        <label className="flex items-center justify-between gap-3 text-[13px] font-medium">
          {t("shareEnds")}
          <select
            value={days}
            onChange={(e) => setDays(Number(e.target.value))}
            className="h-8 rounded-md border border-line bg-bg px-2 text-[12.5px] font-normal"
          >
            {(look.limits.ttl_days ?? DAYS_FALLBACK).map((one) => (
              <option key={one} value={one}>
                {endsAfter(one)}
              </option>
            ))}
          </select>
        </label>

        <div className="flex flex-col gap-2">
          <span className="text-[12.5px] font-semibold">{t("shareCarries")}</span>
          <dl className="grid grid-cols-[auto_1fr] gap-x-3.5 gap-y-1 rounded-[10px] border border-hair px-3.5 py-2.5 text-[12.5px]">
            <dt className="text-faint">{t("shareCarriesPages")}</dt>
            <dd className="text-right">{look.carrying.pages}</dd>
            <dt className="text-faint">{t("shareCarriesPictures")}</dt>
            <dd className="text-right">
              {countOf(look.carrying.pictures, look.limits.pictures_at_most)}
              {" · "}
              {weightOf(look.carrying.picturesBytes, look.limits.pictures_bytes_at_most)}
            </dd>
            <dt className="text-faint">{t("shareCarriesFiles")}</dt>
            <dd className="text-right">
              {countOf(look.carrying.files, look.limits.files_at_most)}
              {" · "}
              {weightOf(look.carrying.filesBytes, look.limits.files_bytes_at_most)}
            </dd>
          </dl>
          {look.cited.length > 0 && (
            <>
              <label className="flex cursor-pointer items-center gap-2 text-[12.5px]">
                <input
                  type="checkbox"
                  checked={cites}
                  onChange={(e) => setCites(e.target.checked)}
                />
                {look.cited.length === 1
                  ? t("shareCitesOne")
                  : fill("shareCites", String(look.cited.length))}
              </label>
              <p className="text-[11.5px] leading-relaxed text-faint">{t("shareCitesWhy")}</p>
            </>
          )}
        </div>

        <div className="flex justify-end gap-2">
          <button
            type="button"
            onClick={onClose}
            className="h-8 rounded-md border border-line px-3 text-[12.5px] hover:bg-hover"
          >
            {t("shareCancel")}
          </button>
          <button
            type="button"
            disabled={!ready}
            onClick={() =>
              onShare({
                cited: cites ? look.cited : [],
                signedAs: signedAs?.trim(),
                editable,
                ttlDays: days,
                seen: look.seen ?? undefined,
              })
            }
            className="h-8 rounded-md bg-accent px-3 text-[12.5px] font-semibold text-white disabled:opacity-50"
          >
            {t(secrets.length > 0 ? "shareGoAnyway" : "shareGo")}
          </button>
        </div>
      </div>
    </Modal>
  );
}

type Step =
  | { at: "reading" }
  | { at: "closed" }
  | { at: "unreached" }
  | { at: "terms"; terms: Extract<TermsNow, { state: "toAccept" }> }
  | { at: "choosing"; look: ShareLook }
  | { at: "going"; until: number | null }
  | { at: "done"; card: ShareCard };

export default function Sharing({
  doc,
  title,
  termsOnly = false,
  onShared,
  onClose,
  onError,
}: Readonly<{
  doc: string;
  title: string;
  termsOnly?: boolean;
  onShared: (card: ShareCard) => void;
  onClose: () => void;
  onError?: (problem: unknown) => void;
}>) {
  const [step, setStep] = useState<Step>({ at: "reading" });
  const live = useRef(true);
  const retry = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  const [changed, setChanged] = useState(false);
  const asked = useRef({ onClose, onError, termsOnly });
  asked.current = { onClose, onError, termsOnly };

  const failed = useCallback((problem: unknown) => {
    asked.current.onClose();
    asked.current.onError?.(problem);
  }, []);

  const lookOver = useCallback(() => {
    setStep({ at: "reading" });
    shareLook(doc)
      .then((look) => {
        if (live.current) setStep({ at: "choosing", look });
      })
      .catch(failed);
  }, [doc, failed]);

  const termsOver = useCallback(() => {
    termsNow()
      .then((terms) => {
        if (!live.current) return;
        if (terms.state === "toAccept") setStep({ at: "terms", terms });
        else if (asked.current.termsOnly) asked.current.onClose();
        else lookOver();
      })
      .catch(failed);
  }, [failed, lookOver]);

  const begin = useCallback(() => {
    if (asked.current.termsOnly) {
      termsOver();
      return;
    }
    setStep({ at: "reading" });
    sharingNow()
      .then((now) => {
        if (!live.current) return;
        if (now === "open") termsOver();
        else setStep({ at: now });
      })
      .catch(failed);
  }, [failed, termsOver]);

  useEffect(() => {
    live.current = true;
    begin();
    return () => {
      live.current = false;
      clearTimeout(retry.current);
    };
  }, [begin]);

  const share = (chosen: Parameters<typeof docShared>[1]) => {
    setStep({ at: "going", until: null });
    docShared(doc, chosen)
      .then((card) => {
        if (!live.current) return;
        void copied(card.read).catch(() => {});
        onShared(card);
        setStep({ at: "done", card });
      })
      .catch((problem) => {
        if (!live.current) return;
        const until = Number((problem as { name?: string } | null)?.name);
        if (codeOf(problem) === "folioWait" && until > 0) {
          setStep({ at: "going", until });
          retry.current = setTimeout(() => share(chosen), Math.max(0, until - Date.now()));
          return;
        }
        if (codeOf(problem) === "folioTermsToAccept") {
          termsOver();
          return;
        }
        if (codeOf(problem) === "folioClosed") {
          setStep({ at: "closed" });
          return;
        }
        if (codeOf(problem) === "folioChanged") {
          setChanged(true);
          lookOver();
          return;
        }
        failed(problem);
      });
  };

  switch (step.at) {
    case "reading":
      return (
        <Modal title={t("shareTitle")} onClose={onClose} wide>
          <output className="text-[13px] text-soft">{t("shareLooking")}</output>
        </Modal>
      );
    case "closed":
      return <NotOpenYet onClose={onClose} />;
    case "unreached":
      return <NoAnswer onRetry={begin} onClose={onClose} />;
    case "terms":
      return (
        <Terms
          terms={step.terms}
          onAccepted={termsOnly ? onClose : lookOver}
          onLater={onClose}
          onError={failed}
        />
      );
    case "choosing":
      return (
        <Choosing
          doc={doc}
          title={title}
          look={step.look}
          changed={changed}
          onShare={share}
          onClose={onClose}
          onLookAgain={lookOver}
          onError={failed}
        />
      );
    case "going":
      return <Going until={step.until} onCancel={onClose} />;
    case "done":
      return <Done card={step.card} onClose={onClose} />;
  }
}
