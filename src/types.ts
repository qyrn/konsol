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
  details: GameDetails;
}

export interface GameDetails {
  playtimeMinutes: number | null;
  lastPlayed: number | null;
  sizeBytes: number | null;
  developer: string | null;
  year: number | null;
  genre: string | null;
  location: string | null;
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
  rarityTenths: number | null;
}

export interface Preferences {
  launchAtLogin: boolean;
  minimizeDuringSession: boolean;
  accentFollowsGame: boolean;
  reduceMotion: boolean;
}

export interface Settings {
  hasSteamGridDbKey: boolean;
  hasSteamWebApiKey: boolean;
  preferences: Preferences;
}

export interface NewManualGame {
  title: string;
  executable: string;
  arguments: string;
  accent: string | null;
  cover: string | null;
}
