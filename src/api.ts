import { invoke } from "@tauri-apps/api/core";
import type { Game } from "./types";

export function listGames(): Promise<Game[]> {
  return invoke<Game[]>("list_games");
}

export function launchGame(id: string): Promise<void> {
  return invoke("launch_game", { id });
}

export function setGameHidden(id: string, hidden: boolean): Promise<void> {
  return invoke("set_game_hidden", { id, hidden });
}

export function addManualGame(title: string, executable: string): Promise<void> {
  return invoke("add_manual_game", { title, executable });
}

export function removeManualGame(id: string): Promise<void> {
  return invoke("remove_manual_game", { id });
}

export function errorMessage(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return "Un truc a planté, réessaie.";
}
