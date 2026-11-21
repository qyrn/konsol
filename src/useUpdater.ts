import { relaunch } from "@tauri-apps/plugin-process";
import { type Update, check } from "@tauri-apps/plugin-updater";
import { useCallback, useEffect, useState } from "react";
import { errorMessage } from "./api";

export type UpdateState =
  | { kind: "idle" }
  | { kind: "checking" }
  | { kind: "up-to-date" }
  | { kind: "downloading"; version: string }
  | { kind: "ready"; version: string }
  | { kind: "failed"; message: string };

async function downloadUpdate(update: Update) {
  await update.downloadAndInstall();
}

export interface UpdaterControls {
  state: UpdateState;
  checkNow: () => void;
  restart: () => void;
}

export function useUpdater(): UpdaterControls {
  const [state, setState] = useState<UpdateState>({ kind: "idle" });

  const checkForUpdate = useCallback(async (silent: boolean) => {
    setState({ kind: "checking" });
    try {
      const update = await check();
      if (!update) {
        setState({ kind: "up-to-date" });
        return;
      }
      setState({ kind: "downloading", version: update.version });
      await downloadUpdate(update);
      setState({ kind: "ready", version: update.version });
    } catch (error) {
      setState(silent ? { kind: "idle" } : { kind: "failed", message: errorMessage(error) });
    }
  }, []);

  useEffect(() => {
    void checkForUpdate(true);
  }, [checkForUpdate]);

  return {
    state,
    checkNow: () => void checkForUpdate(false),
    restart: () => void relaunch(),
  };
}
