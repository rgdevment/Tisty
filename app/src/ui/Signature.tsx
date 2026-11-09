import type { Signing } from "../crediting";
import { agentNamed, hostedOn, signedBy, signedByAnAgent, wroteWhere } from "../who";

interface Props {
  by: string | undefined;
  via: string | undefined | null;
  signing: Signing;
  lead?: string;
  className?: string;
}

export default function Signature({ by, via, signing, lead, className = "" }: Props) {
  if (!agentNamed(by)) return null;
  const where = wroteWhere(by, via);
  return (
    <span
      className={`text-hue-teal ${className}`}
      title={signing === "named" ? hostedOn(by) : where}
    >
      {lead}
      {signing === "mark" ? (
        <span role="img" aria-label={where}>
          ◆
        </span>
      ) : signing === "named" ? (
        signedBy(by, via)
      ) : (
        signedByAnAgent(by)
      )}
    </span>
  );
}
