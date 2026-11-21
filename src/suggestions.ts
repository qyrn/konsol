import type { Game } from "./types";

const MAX_SUGGESTIONS = 10;
const BARELY_PLAYED_MINUTES = 60;

export interface Suggestion {
  game: Game;
  because: Game;
}

function playtime(game: Game): number {
  return game.details.playtimeMinutes ?? 0;
}

export function suggestGames(games: Game[]): Suggestion[] {
  const played = games.filter((game) => playtime(game) >= BARELY_PLAYED_MINUTES);
  const genreWeight = new Map<string, number>();
  for (const game of played) {
    for (const genre of game.details.genres) {
      genreWeight.set(genre, (genreWeight.get(genre) ?? 0) + Math.log1p(playtime(game)));
    }
  }
  if (genreWeight.size === 0) return [];

  return games
    .filter(
      (game) => !game.hidden && playtime(game) < BARELY_PLAYED_MINUTES && game.details.genres.length > 0,
    )
    .map((game) => {
      const score = game.details.genres.reduce((total, genre) => total + (genreWeight.get(genre) ?? 0), 0);
      const because = played
        .filter((other) => other.details.genres.some((genre) => game.details.genres.includes(genre)))
        .sort((first, second) => playtime(second) - playtime(first))[0];
      return { game, because, score: score + (game.installed ? 0.5 : 0) };
    })
    .filter(
      (entry): entry is Suggestion & { score: number } => entry.score > 0 && entry.because !== undefined,
    )
    .sort((first, second) => second.score - first.score)
    .slice(0, MAX_SUGGESTIONS)
    .map(({ game, because }) => ({ game, because }));
}

export interface GenreShelf {
  genre: string;
  games: Game[];
}

export const OTHER_GENRE = "Autres";

export function shelvesByGenre(games: Game[]): GenreShelf[] {
  const byGenre = new Map<string, Game[]>();
  for (const game of games) {
    const genre = game.details.genre ?? OTHER_GENRE;
    byGenre.set(genre, [...(byGenre.get(genre) ?? []), game]);
  }
  return [...byGenre.entries()]
    .map(([genre, shelfGames]) => ({ genre, games: shelfGames }))
    .sort((first, second) => {
      if (first.genre === OTHER_GENRE) return 1;
      if (second.genre === OTHER_GENRE) return -1;
      return second.games.length - first.games.length || first.genre.localeCompare(second.genre, "fr");
    });
}
