import { useCallback, useEffect, useState } from "react";
import { about, revealed, served, type About as Build } from "../core";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";

export default function About({ onError }: { onError: (problem: unknown) => void }) {
  const [build, setBuild] = useState<Build | null>(null);
  // In the card, like its sister screen: a version that cannot be read is not a
  // reason to leave the page blank with no way to ask again.
  const [trouble, setTrouble] = useState<string | null>(null);

  const look = useCallback(() => {
    setTrouble(null);
    about()
      .then(setBuild)
      .catch((problem) => setTrouble(saidPlainly(problem)));
  }, []);

  useEffect(look, [look]);

  return (
    <main className="flex flex-col overflow-hidden">
      <div data-tauri-drag-region className="h-9 shrink-0" />
      <div className="scroller mx-auto w-full max-w-[560px] px-6 pb-12">
        <h2 className="mb-3.5 text-[21px] font-semibold">{t("aboutScreen")}</h2>

        {build?.sandbox && (
          <p className="mb-3 rounded-[10px] bg-mark-priority px-4 py-3 text-[12.5px] text-ink">
            {fill("inSandbox", build.sandbox)}
          </p>
        )}

        <Card title="Tisty">
          <p className="text-[12.5px] leading-relaxed text-soft">{t("aboutWhat")}</p>
          <p className="mt-2 text-[11.5px] leading-relaxed text-faint">{t("aboutPrivacy")}</p>
        </Card>

        {trouble && (
          <Card title={t("aboutBuild")}>
            <p role="alert" className="text-[12.5px] leading-relaxed text-urgent">
              {t("aboutFailed")}
            </p>
            <p className="mt-1 text-[11.5px] leading-relaxed text-faint">{trouble}</p>
            <button type="button" onClick={look} className={`mt-2.5 ${mild}`}>
              {t("tryAgain")}
            </button>
          </Card>
        )}

        {build && (
          <>
            <Card title={t("aboutBuild")}>
              <dl className="grid grid-cols-[auto_minmax(0,1fr)] gap-x-4 gap-y-1 text-[12.5px]">
                <dt className="text-faint">{t("aboutVersion")}</dt>
                <dd className="tabular-nums text-soft">{build.version}</dd>
                <dt className="text-faint">{t("aboutLicense")}</dt>
                <dd className="text-soft">{build.license}</dd>
              </dl>
              <div className="mt-2.5">
                <button type="button" onClick={() => served(build.repository).catch(onError)} className={mild}>
                  {t("aboutRepo")}
                </button>
              </div>
            </Card>

            <Card title={t("aboutStore")}>
              <p className="truncate text-[12.5px] text-soft" title={build.store}>
                {build.store}
              </p>
              <div className="mt-2.5">
                <button type="button" onClick={() => revealed(build.store).catch(onError)} className={mild}>
                  {t("aboutReveal")}
                </button>
              </div>
            </Card>
          </>
        )}
      </div>
    </main>
  );
}

const mild =
  "rounded-[7px] border border-line px-2.5 py-1 text-[12.5px] hover:bg-hover disabled:opacity-50";

function Card({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="mb-3 rounded-[10px] border border-hair px-4 py-3.5">
      <h3 className="mb-0.5 text-[13.5px] font-semibold">{title}</h3>
      {children}
    </section>
  );
}
