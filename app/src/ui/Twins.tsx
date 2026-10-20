import { convertFileSrc } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { type Joined, opened, type Pair, type Twin } from "../core";
import { weigh } from "../format";
import { serve } from "../inUse";
import { fill, t } from "../locales";
import { pictured, previewOf } from "../previews";
import { mild, strong } from "./Rows";

function Seen({ at }: Readonly<{ at: string }>) {
  const [src, setSrc] = useState<string>();
  const shown = previewOf(at);
  const plays = shown?.as === "video" || shown?.as === "audio";
  const looks = pictured(at);

  useEffect(() => {
    if (!looks && !plays) return;
    let live = true;
    serve(at)
      .then((path) => live && setSrc(convertFileSrc(path)))
      .catch(() => undefined);
    return () => {
      live = false;
    };
  }, [at, looks, plays]);

  if (src && looks) return <img src={src} alt="" className="max-h-28 rounded-md object-contain" />;
  if (src && shown?.as === "video")
    // biome-ignore lint/a11y/useMediaCaption: a person's own recording carries no captions to offer
    return <video src={src} controls preload="metadata" className="max-h-32 w-full rounded-md" />;
  if (src && shown?.as === "audio")
    // biome-ignore lint/a11y/useMediaCaption: a person's own recording carries no captions to offer
    return <audio src={src} controls preload="metadata" className="w-full" />;
  return (
    <button type="button" onClick={() => void opened(at).catch(() => undefined)} className={mild}>
      {t("twinsOpen")}
    </button>
  );
}

function Copy({
  one,
  stays,
  onStay,
  onDoc,
}: Readonly<{ one: Twin; stays: boolean; onStay: () => void; onDoc?: (id: string) => void }>) {
  return (
    <li
      className={`flex flex-col gap-1.5 rounded-[10px] border p-2.5 ${
        stays ? "border-hue-green/50 bg-hue-green/5" : "border-hair"
      }`}
    >
      <span className="font-mono text-[11.5px] break-all text-soft">
        {one.at.replace("attachments/", "")}
      </span>
      <Seen at={one.at} />
      <span className="text-[11.5px] text-faint">
        {one.links.length === 0 ? t("twinsNowhere") : `${t("twinsLinked")} `}
        {one.links.map((link, n) => (
          <span key={`${link.kind}-${link.id}`}>
            {n > 0 && " · "}
            {link.kind === "doc" && onDoc ? (
              <button
                type="button"
                onClick={() => onDoc(link.id)}
                className="text-accent hover:underline"
              >
                {link.title}
              </button>
            ) : (
              link.title
            )}
          </span>
        ))}
      </span>
      {stays ? (
        <span className="text-[11.5px] font-medium text-hue-green">{t("twinsStays")}</span>
      ) : (
        <button type="button" onClick={onStay} className={`self-start ${mild}`}>
          {t("twinsKeepThis")}
        </button>
      )}
    </li>
  );
}

export default function Twins({
  pair,
  busy,
  onJoin,
  onDoc,
}: Readonly<{
  pair: Pair;
  busy: boolean;
  onJoin: (keep: string, go: string[]) => Promise<Joined>;
  onDoc?: (id: string) => void;
}>) {
  const [stays, setStays] = useState(pair.stays);
  const [later, setLater] = useState(false);
  const [done, setDone] = useState<Joined>();
  const going = pair.copies.filter((one) => one.at !== stays);
  const locked = going.find((one) => one.locked)?.locked;

  if (later) return null;
  if (done)
    return (
      <li role="status" className="text-[12.5px] text-soft">
        {fill("twinsJoined", weigh(done.freed), String(done.relinked))}
      </li>
    );
  return (
    <li className="flex flex-col gap-2">
      <span className="flex items-center gap-2 text-[12.5px]">
        <span className="tabular-nums text-faint">{weigh(pair.bytes)}</span>
        <span className="text-hue-green">{t("twinsSame")}</span>
      </span>
      <ul className="grid gap-2 sm:grid-cols-2">
        {pair.copies.map((one) => (
          <Copy
            key={one.at}
            one={one}
            stays={one.at === stays}
            onStay={() => setStays(one.at)}
            onDoc={onDoc}
          />
        ))}
      </ul>
      {locked && <p className="text-[11.5px] text-urgent">{fill("twinsLockedBy", locked)}</p>}
      <span className="flex gap-2">
        <button
          type="button"
          disabled={busy || Boolean(locked)}
          onClick={() =>
            void onJoin(
              stays,
              going.map((one) => one.at),
            ).then(setDone, () => undefined)
          }
          className={strong}
        >
          {t("twinsJoin")}
        </button>
        <button type="button" disabled={busy} onClick={() => setLater(true)} className={mild}>
          {t("twinsLater")}
        </button>
      </span>
    </li>
  );
}
