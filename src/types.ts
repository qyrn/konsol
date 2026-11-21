export type GameSource = "steam" | "gog" | "epic" | "amazon" | "rockstar" | "desktop" | "manual";

export type ImageSource = { kind: "file"; value: string } | { kind: "url"; value: string };

export interface Game {
  id: string;
  title: string;
  source: GameSource;
  cover: ImageSource | null;
  hero: ImageSource | null;
  capsule: ImageSource | null;
  accent: string | null;
  hidden: boolean;
  installed: boolean;
  achievements: AchievementSummary | null;
  details: GameDetails;
}

export interface TagWeight {
  id: number;
  name: string;
  weight: number;
}

export interface GameDetails {
  playtimeMinutes: number | null;
  lastPlayed: number | null;
  sizeBytes: number | null;
  developer: string | null;
  year: number | null;
  genre: string | null;
  tags: TagWeight[];
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
  keepRunningInBackground: boolean;
  accentFollowsGame: boolean;
  reduceMotion: boolean;
}

export interface GameAchievements {
  gameId: string;
  unlocked: number;
  total: number | null;
  recent: Achievement[];
}

export interface Collection {
  name: string;
  gameIds: string[];
}

export interface SessionSpan {
  gameId: string;
  startedAt: number;
  endedAt: number;
}

export interface Discovery {
  appId: number;
  name: string;
  capsule: string | null;
  reviewLabel: string | null;
  because: string;
}

export interface Settings {
  hasSteamGridDbKey: boolean;
  hasSteamWebApiKey: boolean;
  preferences: Preferences;
  gameFolders: string[];
}

export interface NewManualGame {
  title: string;
  executable: string;
  arguments: string;
  accent: string | null;
  cover: string | null;
}
