import { useEffect, useState } from "react";

export function useSessionStarts(runningIds: ReadonlySet<string>): ReadonlyMap<string, number> {
  const [starts, setStarts] = useState<ReadonlyMap<string, number>>(new Map());

  useEffect(() => {
    setStarts((previous) => {
      const next = new Map<string, number>();
      for (const id of runningIds) next.set(id, previous.get(id) ?? Date.now());
      const unchanged = next.size === previous.size && [...next.keys()].every((id) => previous.has(id));
      return unchanged ? previous : next;
    });
  }, [runningIds]);

  return starts;
}
