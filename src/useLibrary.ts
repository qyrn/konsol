import { useCallback, useEffect, useState } from "react";
import { errorMessage, listGames } from "./api";
import type { Game } from "./types";

interface LibraryState {
  games: Game[];
  loading: boolean;
  error: string | null;
}

export function useLibrary() {
  const [state, setState] = useState<LibraryState>({ games: [], loading: true, error: null });

  const reload = useCallback(async () => {
    setState((previous) => ({ ...previous, loading: true }));
    try {
      const games = await listGames();
      setState({ games, loading: false, error: null });
    } catch (error) {
      setState((previous) => ({ ...previous, loading: false, error: errorMessage(error) }));
    }
  }, []);

  useEffect(() => {
    void reload();
  }, [reload]);

  return { ...state, reload };
}
