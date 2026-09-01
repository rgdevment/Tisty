import type { Filed } from "../core";
import { t } from "../locales";

interface Props {
  pages: Filed[];
  told: Set<string>;
  onOpen: (page: Filed) => void;
  onPut?: (page: Filed) => void;
}

export default function Contents({ pages, told, onOpen, onPut }: Props) {
  if (pages.length === 0) return null;
  const inside = pages.filter((one) => told.has(one.file));
  const loose = pages.filter((one) => !told.has(one.file));

  const row = (page: Filed, at: string) => (
    <li key={page.id} className="leaf-row">
      <button type="button" onClick={() => onOpen(page)} className="leaf-open">
        <span className="leaf-num">{at}</span>
        <span className="leaf-name">{page.title || t("untitledDoc")}</span>
      </button>
      {onPut && !told.has(page.file) && (
        <button type="button" onClick={() => onPut(page)} className="leaf-put">
          {t("putLeaf")}
        </button>
      )}
    </li>
  );

  return (
    <section aria-label={t("theseLeaves")} className="leaves">
      <h2 className="leaves-head">
        {t("theseLeaves")}
        <span className="leaves-many">{pages.length}</span>
      </h2>
      <p className="leaves-why">{loose.length > 0 ? t("someLoose") : t("allInside")}</p>
      <ul className="leaves-list">
        {inside.map((one, at) => row(one, String(at + 1).padStart(2, "0")))}
      </ul>
      {loose.length > 0 && (
        <>
          <div aria-hidden="true" className="leaves-split" />
          <ul className="leaves-list">{loose.map((one) => row(one, "—"))}</ul>
        </>
      )}
    </section>
  );
}
