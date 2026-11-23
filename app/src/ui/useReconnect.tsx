import { type ReactNode, useState } from "react";
import { typed } from "../clouds";
import { unstuck } from "../coming";
import {
  type CloudStatus,
  type Connected,
  cloudChoose,
  cloudConnect,
  cloudConnectCancel,
  cloudForget,
  cloudStatus,
} from "../core";
import { t } from "../locales";
import type { Run, Which, Word } from "./Card";
import DavForm from "./DavForm";

interface Asked {
  card: Which;
  status: CloudStatus | null;
  provider: string;
  held: boolean;
  run: Run;
  fail: (word?: Word) => void;
  carry: () => void;
  onSigned: () => void;
}

interface Reconnecting {
  /** In the browser while stopping it still means something, settling once the sign-in went through. */
  signing: "browser" | "settling" | null;
  reconnect: () => void;
  form: ReactNode;
  waiting: ReactNode;
}

/** Signing in again to the cloud already chosen, for any card that finds its access gone. */
export function useReconnect({
  card,
  status,
  provider,
  held,
  run,
  fail,
  carry,
  onSigned,
}: Asked): Reconnecting {
  const [askingPassword, setAskingPassword] = useState(false);
  const [signing, setSigning] = useState<Reconnecting["signing"]>(null);

  // A copy this machine already carried stays with its account; one never carried may change it.
  const settled = async (account: Connected) => {
    const now = status ?? (await cloudStatus().catch(() => null));
    const was = now?.account;
    if (was && was !== account.account && now.joined) {
      await cloudForget(account.account, account.provider).catch(() => undefined);
      throw { code: "cloudElsewhere", name: account.email };
    }
    await cloudChoose(account.account, account.email, account.provider);
    if (was && was !== account.account) await cloudForget(was, now.provider).catch(() => undefined);
    onSigned();
    unstuck();
  };

  const reconnected = (account: Connected) => {
    setSigning("settling");
    return settled(account).finally(() => setSigning(null));
  };

  const reconnect = () => {
    if (typed(provider)) return setAskingPassword(true);
    setSigning("browser");
    run(
      card,
      cloudConnect(provider)
        .then(reconnected)
        .then(
          () => true,
          (e) => {
            if (e?.code === "cloudDropped") return false;
            throw e;
          },
        )
        .finally(() => setSigning(null)),
      (went) => went && carry(),
    );
  };

  const passwordGiven = (account: Connected) => {
    setAskingPassword(false);
    run(card, reconnected(account), () => carry());
  };

  return {
    signing,
    reconnect,
    form: askingPassword && status && (
      <div className="mt-2.5 rounded-[10px] border border-hair px-3 py-2">
        <DavForm
          account={status.account}
          busy={held}
          onConnected={passwordGiven}
          onTrouble={(text) => text && fail({ card, text })}
        />
      </div>
    ),
    waiting: signing && (
      <p
        role="status"
        aria-live="polite"
        className="mt-2 text-[11.5px] text-faint motion-safe:animate-pulse"
      >
        {t(signing === "browser" ? "cloudWaiting" : "cloudSettling")}
        {signing === "browser" && (
          <button
            type="button"
            onClick={() => void cloudConnectCancel().catch(() => undefined)}
            className="ml-1 text-accent hover:underline"
          >
            {t("cancel")}
          </button>
        )}
      </p>
    ),
  };
}
