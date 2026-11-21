export type GameSource = "steam" | "gog" | "epic" | "amazon" | "desktop" | "manual";

export type ImageSource = { kind: "file"; value: string } | { kind: "url"; value: string };

export interface Game {
  id: string;
  title: string;
  source: GameSource;
  cover: ImageSource | null;
  hero: ImageSource | null;
  accent: string | null;
  hidden: boolean;
  achievements: AchievementSummary | null;
}

export interface AchievementSummary {
  unlocked: number;
  total: number | null;
}

export interface Achievement {
  id: string;
  name: string;
  description: string;
  iconUrl: string | null;
  unlocked: boolean;
  unlockedAt: number | null;
}

export interface Settings {
  hasSteamGridDbKey: boolean;
  hasSteamWebApiKey: boolean;
}
