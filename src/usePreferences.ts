import { useCallback, useEffect, useState } from "react";
import { errorMessage, getSettings, savePreferences } from "./api";
import type { Preferences } from "./types";

export const DEFAULT_PREFERENCES: Preferences = {
  launchAtLogin: false,
  minimizeDuringSession: false,
  accentFollowsGame: true,
  reduceMotion: false,
};

export function usePreferences(onError: (message: string) => void) {
  const [preferences, setPreferences] = useState<Preferences>(DEFAULT_PREFERENCES);

  useEffect(() => {
    getSettings()
      .then((settings) => setPreferences(settings.preferences))
      .catch((error: unknown) => onError(errorMessage(error)));
  }, [onError]);

  const update = useCallback(
    async (next: Preferences) => {
      const previous = preferences;
      setPreferences(next);
      try {
        await savePreferences(next);
      } catch (error) {
        setPreferences(previous);
        onError(errorMessage(error));
      }
    },
    [preferences, onError],
  );

  return { preferences, update };
}
