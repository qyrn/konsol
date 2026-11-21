import type { GameSource } from "./types";

export const SOURCE_ORDER: readonly GameSource[] = [
  "steam",
  "gog",
  "epic",
  "amazon",
  "rockstar",
  "desktop",
  "manual",
];

export const SOURCE_LABELS: Record<GameSource, string> = {
  steam: "Steam",
  gog: "GOG",
  epic: "Epic",
  amazon: "Amazon",
  rockstar: "Rockstar",
  desktop: "Sans launcher",
  manual: "Ajoutés à la main",
};

export const SOURCE_LAUNCHERS: Record<GameSource, string> = {
  steam: "Steam",
  gog: "Heroic",
  epic: "Heroic",
  amazon: "Heroic",
  rockstar: "Le launcher Rockstar",
  desktop: "Konsol",
  manual: "Konsol",
};
