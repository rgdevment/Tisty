import { useEffect, useState } from "react";
import tisty from "../assets/tisty.svg";
import { openLink } from "../core";
import { t } from "../locales";
import { headline, newsSeen, newsWaiting, toldFor } from "../news";
import Modal from "./Modal";
import { go } from "./RelocatingSteps";

const RELEASES = "https://github.com/rgdevment/Tisty/releases";

export default function News() {
  const [versions, setVersions] = useState<string[]>([]);

  useEffect(() => {
    newsWaiting()
      .then((found) => setVersions(Array.isArray(found) ? found : []))
      .catch(() => undefined);
  }, []);

  const told = toldFor(versions);
  if (told.length === 0) return null;

  const close = () => {
    setVersions([]);
    void newsSeen().catch(() => undefined);
  };

  return (
    <Modal
      title={headline(versions[0])}
      wide
      onClose={close}
      art={
        <div className="mb-3 flex flex-col items-center gap-2">
          <img
            src={tisty}
            alt=""
            className="h-[72px] w-auto dark:drop-shadow-[0_0_1px_rgb(248_250_255/0.9)]"
          />
          <span className="flex items-center gap-2 text-[10.5px] font-semibold tracking-[0.06em] text-faint uppercase">
            {t("newsOver")}
            <span className="rounded-full bg-accent/10 px-2 py-0.5 font-mono text-accent normal-case tracking-normal">
              {versions[0]}
            </span>
          </span>
        </div>
      }
    >
      <div className="scroller -mr-3 max-h-[46vh] pr-3">
        {told.map((one, at) => (
          <section key={one.version} aria-label={one.version}>
            {at > 0 && (
              <h3 className="mt-4 mb-1 font-mono text-[11.5px] text-faint">{one.version}</h3>
            )}
            <ul>
              {one.told.map((said) => (
                <li
                  key={`${one.version}-${said.title}`}
                  className="relative border-b border-hair py-2.5 pl-4 text-[12.5px] leading-relaxed text-soft last:border-b-0 before:absolute before:top-[17px] before:left-0 before:size-1.5 before:rounded-full before:bg-accent"
                >
                  <b className="font-semibold text-ink">{said.title}</b>: {said.said}
                </li>
              ))}
            </ul>
          </section>
        ))}
      </div>
      <div className="mt-4 flex items-center gap-4">
        <button
          type="button"
          onClick={() => void openLink(RELEASES).catch(() => undefined)}
          className="text-[12.5px] text-accent hover:underline"
        >
          {t("newsAll")}
        </button>
        <button type="button" onClick={close} className={`ml-auto ${go}`}>
          {t("newsOk")}
        </button>
      </div>
    </Modal>
  );
}
