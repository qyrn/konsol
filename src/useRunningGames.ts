import { useEffect, useState } from "react";
import { runningGameIds } from "./api";

const POLL_INTERVAL_MS = 3000;

export function useRunningGames(): ReadonlySet<string> {
  const [runningIds, setRunningIds] = useState<ReadonlySet<string>>(new Set());

  useEffect(() => {
    let cancelled = false;
    async function refresh() {
      try {
        const ids = await runningGameIds();
        if (!cancelled) setRunningIds(new Set(ids));
      } catch {
        if (!cancelled) setRunningIds(new Set());
      }
    }
    void refresh();
    const interval = window.setInterval(() => void refresh(), POLL_INTERVAL_MS);
    return () => {
      cancelled = true;
      window.clearInterval(interval);
    };
  }, []);

  return runningIds;
}
