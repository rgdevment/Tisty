import { useEffect, useMemo, useState } from "react";
import type { Group, Network } from "../../brainsty";
import { fill, t } from "../../locales";
import Glyph from "../Glyph";
import { followTheme } from "../theming";
import { legendOf } from "./legendKeys";
import { paletteNow, type Shape } from "./looks";

interface Props {
  readonly network: Network;
  readonly groups: readonly Group[];
}

const OPEN = "tisty.brainstyLegend";
const FEW = 6;

const keptOpen = (): boolean => {
  try {
    return localStorage.getItem(OPEN) === "open";
  } catch {
    return false;
  }
};

const keepOpen = (open: boolean) => {
  try {
    if (open) localStorage.setItem(OPEN, "open");
    else localStorage.removeItem(OPEN);
  } catch {
    // Without storage the legend simply starts folded next time.
  }
};

export default function Legend({ network, groups }: Props) {
  const [open, setOpen] = useState(keptOpen);
  const [palette, setPalette] = useState(paletteNow);
  const [all, setAll] = useState(false);
  useEffect(() => followTheme(() => setPalette(paletteNow())), []);
  const legend = useMemo(() => legendOf(network, groups, palette), [network, groups, palette]);
  const shown = all ? legend.groups : legend.groups.slice(0, FEW);
  const more = legend.groups.length - shown.length;
  const turn = () => {
    keepOpen(!open);
    setOpen(!open);
  };

  return (
    <section
      aria-label={t("brainstyLegend")}
      className={`absolute bottom-9 left-3 z-[5] flex max-h-[calc(100%-120px)] flex-col overflow-hidden rounded-[10px] border border-hair bg-bg/95 text-[11.5px] text-soft shadow-lift ${
        open ? "w-[min(440px,calc(100%-24px))]" : ""
      }`}
    >
      <button
        type="button"
        onClick={turn}
        aria-expanded={open}
        className="flex items-center gap-1.5 px-2.5 py-1.5 text-left font-semibold text-soft hover:bg-hover hover:text-ink"
      >
        <Glyph
          name="chevron"
          className={`h-3 w-3 text-faint transition-transform duration-150 ${open ? "" : "-rotate-90"}`}
        />
        {t("brainstyLegend")}
      </button>
      {open && (
        <div className="scroller grid min-h-0 grid-cols-2 gap-x-5 gap-y-3 px-3 pb-3">
          <Part title={t("brainstyLegendKinds")}>
            {legend.kinds.map((one) => (
              <Row key={one.kind} label={t(`brainstyOne_${one.kind}`)}>
                <Dot shape={one.shape} size={one.size} color={one.color} />
              </Row>
            ))}
          </Part>
          {legend.groups.length > 0 && (
            <Part title={t("brainstyLegendColours")}>
              {shown.map((one) => (
                <Row key={one.id} label={one.name}>
                  <Dot shape="circle" size={5} color={one.color} />
                </Row>
              ))}
              {(more > 0 || all) && legend.groups.length > FEW && (
                <button
                  type="button"
                  onClick={() => setAll(!all)}
                  className="self-start rounded px-1 text-faint hover:bg-hover hover:text-ink"
                >
                  {all ? t("brainstyLegendFewer") : fill("brainstyLegendMore", String(more))}
                </button>
              )}
              <p className="text-faint">{t("brainstyLegendPlain")}</p>
            </Part>
          )}
          <Part title={t("brainstyLegendStandings")}>
            {legend.standings.map((one) => (
              <Row key={one.standing} label={t(`brainstyStanding_${one.standing}`)}>
                <Dot shape="circle" size={one.size} color={one.color} />
              </Row>
            ))}
          </Part>
          {(legend.lines.length > 0 || legend.alike) && (
            <Part title={t("brainstyLegendLines")}>
              {legend.lines.map((one) => (
                <Row key={one.tie} label={t(`brainstyTie_${one.tie}`)}>
                  <svg viewBox="0 0 24 8" aria-hidden="true" className="h-2 w-6 shrink-0">
                    <line
                      x1="1"
                      y1="4"
                      x2="23"
                      y2="4"
                      stroke={one.color}
                      strokeWidth={Math.max(1, one.size * 1.5)}
                      strokeDasharray={one.dashed ? "3 2" : undefined}
                    />
                  </svg>
                </Row>
              ))}
              {legend.alike && (
                <Row label={t("brainstyAlike")} title={t("brainstyAlikeWhat")}>
                  <svg viewBox="0 0 24 8" aria-hidden="true" className="h-2 w-6 shrink-0">
                    <path
                      d="M1 7 Q12 -2 23 7"
                      fill="none"
                      stroke={legend.alike.color}
                      strokeWidth={legend.alike.size}
                    />
                  </svg>
                </Row>
              )}
            </Part>
          )}
        </div>
      )}
    </section>
  );
}

function Part({ title, children }: { readonly title: string; readonly children: React.ReactNode }) {
  return (
    <div className="flex flex-col gap-1">
      <h3 className="text-[10.5px] font-semibold tracking-wide text-faint uppercase">{title}</h3>
      {children}
    </div>
  );
}

function Row({
  label,
  title,
  children,
}: {
  readonly label: string;
  readonly title?: string;
  readonly children: React.ReactNode;
}) {
  return (
    <p className="flex items-center gap-2" title={title}>
      <span className="grid w-6 shrink-0 place-items-center">{children}</span>
      <span className="truncate">{label}</span>
    </p>
  );
}

function Dot({
  shape,
  size,
  color,
}: {
  readonly shape: Shape;
  readonly size: number;
  readonly color: string;
}) {
  const across = Math.min(14, Math.max(5, size * 1.6));
  return (
    <span
      data-shape={shape}
      className={shape === "circle" ? "rounded-full" : "rounded-[2px]"}
      style={{
        width: across,
        height: across,
        background: color,
        transform: shape === "diamond" ? "rotate(45deg) scale(0.8)" : undefined,
      }}
    />
  );
}
