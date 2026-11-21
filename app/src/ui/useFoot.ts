import { useEffect, useState } from "react";
import { useComing, useStuck } from "../coming";
import type { Carrying, CloudStatus } from "../core";
import { useLeftForLater } from "../deciding";
import { footing } from "../footing";
import { useKnocking } from "../knocking";
import { offlineNow } from "../syncSaid";

export function useFoot(state: Carrying | null, status: CloudStatus | null, mine = false) {
  const [offline, setOffline] = useState(offlineNow);
  const coming = useComing();
  const stuck = useStuck();
  const knocked = useKnocking().waiting.length > 0;
  const putOff = useLeftForLater();
  useEffect(() => {
    const online = () => setOffline(offlineNow());
    window.addEventListener("online", online);
    window.addEventListener("offline", online);
    return () => {
      window.removeEventListener("online", online);
      window.removeEventListener("offline", online);
    };
  }, []);
  const foot = footing({ state, status, coming, mine, offline, knocked, putOff, stuck });
  return { foot, coming, stuck };
}
