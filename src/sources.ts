import type { GameSource } from "./types";

export const SOURCE_ORDER: readonly GameSource[] = ["steam", "gog", "epic", "amazon", "script", "manual"];

export const SOURCE_LABELS: Record<GameSource, string> = {
  steam: "Steam",
  gog: "GOG",
  epic: "Epic",
  amazon: "Amazon",
  script: "Scripts",
  manual: "Ajoutés à la main",
};
