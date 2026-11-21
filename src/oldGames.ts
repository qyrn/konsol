import type { Game } from "./types";

const OLD_AFTER_SECONDS = 5 * 365 * 86_400;

export function isOldGame(game: Game, nowSeconds: number): boolean {
  const lastPlayed = game.details.lastPlayed;
  return lastPlayed !== null && nowSeconds - lastPlayed > OLD_AFTER_SECONDS;
}
