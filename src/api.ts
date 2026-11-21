import { invoke } from "@tauri-apps/api/core";
import type {
  Achievement,
  Collection,
  Discovery,
  Game,
  GameAchievements,
  NewManualGame,
  Preferences,
  SessionSpan,
  Settings,
} from "./types";

export const LIBRARY_EVENT = "library-changed";
export const ACHIEVEMENT_EVENT = "achievement-unlocked";

export function listGames(): Promise<Game[]> {
  return invoke<Game[]>("list_games");
}

export function launchGame(id: string): Promise<void> {
  return invoke("launch_game", { id });
}

export function setGameHidden(id: string, hidden: boolean): Promise<void> {
  return invoke("set_game_hidden", { id, hidden });
}

export function runningGameIds(): Promise<string[]> {
  return invoke<string[]>("running_game_ids");
}

export function renameGame(id: string, title: string): Promise<void> {
  return invoke("rename_game", { id, title });
}

export function addManualGame(game: NewManualGame): Promise<string> {
  return invoke<string>("add_manual_game", { game });
}

export function saveGameFolders(folders: string[]): Promise<void> {
  return invoke("save_game_folders", { folders });
}

export function uninstallGame(id: string): Promise<void> {
  return invoke("uninstall_game", { id });
}

export function achievementOverview(): Promise<GameAchievements[]> {
  return invoke<GameAchievements[]>("achievement_overview");
}

export function getCollections(): Promise<Collection[]> {
  return invoke<Collection[]>("get_collections");
}

export function saveCollections(collections: Collection[]): Promise<void> {
  return invoke("save_collections", { collections });
}

export function addPlaytime(id: string, minutes: number): Promise<void> {
  return invoke("add_playtime", { id, minutes });
}

export function recentSessions(): Promise<SessionSpan[]> {
  return invoke<SessionSpan[]>("recent_sessions");
}

export function discoverGames(): Promise<Discovery[]> {
  return invoke<Discovery[]>("discover_games");
}

export function openStorePage(appId: number): Promise<void> {
  return invoke("open_store_page", { appId });
}

export function stopGame(id: string): Promise<void> {
  return invoke("stop_game", { id });
}

export function savePreferences(preferences: Preferences): Promise<void> {
  return invoke("save_preferences", { preferences });
}

export function removeManualGame(id: string): Promise<void> {
  return invoke("remove_manual_game", { id });
}

export function getAchievements(id: string): Promise<Achievement[]> {
  return invoke<Achievement[]>("get_achievements", { id });
}

export function getSettings(): Promise<Settings> {
  return invoke<Settings>("get_settings");
}

export function saveSettings(steamGridDbKey: string | null, steamWebApiKey: string | null): Promise<void> {
  return invoke("save_settings", { steamGridDbKey, steamWebApiKey });
}

export function errorMessage(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return "Un truc a planté, réessaie.";
}
