import type { Network } from "../brainsty";
import { t } from "../locales";

export default function Brainsty({ network }: { network: Network }) {
  return (
    <div
      role="img"
      aria-label={t("brainstyNetwork")}
      data-nodes={network.nodes.length}
      className="absolute inset-0"
    />
  );
}
