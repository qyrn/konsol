import { useEffect, useState } from "react";
import { runningGameIds } from "./api";

const POLL_INTERVAL_MS = 3000;

function sameIds(previous: ReadonlySet<string>, ids: string[]): boolean {
  return previous.size === ids.length && ids.every((id) => previous.has(id));
}

export function useRunningGames(): ReadonlySet<string> {
  const [runningIds, setRunningIds] = useState<ReadonlySet<string>>(new Set());

  useEffect(() => {
    let cancelled = false;
    async function refresh() {
      try {
        const ids = await runningGameIds();
        if (!cancelled) setRunningIds((previous) => (sameIds(previous, ids) ? previous : new Set(ids)));
      } catch {
        if (!cancelled) setRunningIds((previous) => (previous.size === 0 ? previous : new Set()));
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
