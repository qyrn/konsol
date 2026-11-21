import type { Game } from "./types";

const MAX_SUGGESTIONS = 12;
const PLAYED_MINUTES = 60;
const MIN_SIMILARITY = 0.45;
const IGNORED_TAGS = new Set([
  "Solo",
  "Multijoueur",
  "Indépendant",
  "Excellente bande-son",
  "Atmosphérique",
  "Accès anticipé",
  "Free-to-play",
  "Coopération",
  "Coop en ligne",
  "Manette entièrement prise en charge",
  "Classique",
  "Culte",
  "Difficile",
  "Beau",
  "Bonne intrigue",
  "Rejouabilité",
  "Personnalisation du personnage",
  "Animé",
  "Coloré",
]);

export interface Suggestion {
  game: Game;
  because: Game;
}

function playtime(game: Game): number {
  return game.details.playtimeMinutes ?? 0;
}

function tagVector(game: Game): Map<string, number> {
  const tags = game.details.tags.filter((tag) => !IGNORED_TAGS.has(tag.name));
  const strongest = Math.max(1, ...tags.map((tag) => tag.weight));
  return new Map(tags.map((tag) => [tag.name, tag.weight / strongest]));
}

function cosine(first: Map<string, number>, second: Map<string, number>): number {
  let dot = 0;
  for (const [name, weight] of first) dot += weight * (second.get(name) ?? 0);
  const norm = (vector: Map<string, number>) =>
    Math.sqrt([...vector.values()].reduce((sum, value) => sum + value * value, 0));
  const denominator = norm(first) * norm(second);
  return denominator === 0 ? 0 : dot / denominator;
}

export function suggestGames(games: Game[]): Suggestion[] {
  const played = games
    .filter((game) => playtime(game) >= PLAYED_MINUTES && game.details.tags.length > 0)
    .map((game) => ({ game, vector: tagVector(game), pull: Math.log1p(playtime(game) / 60) }));
  if (played.length === 0) return [];

  return games
    .filter((game) => !game.hidden && playtime(game) < PLAYED_MINUTES && game.details.tags.length > 0)
    .flatMap((game) => {
      const vector = tagVector(game);
      const best = played
        .map((entry) => ({ entry, similarity: cosine(vector, entry.vector) }))
        .filter(({ similarity }) => similarity >= MIN_SIMILARITY)
        .sort(
          (first, second) => second.similarity * second.entry.pull - first.similarity * first.entry.pull,
        )[0];
      return best
        ? [{ game, because: best.entry.game, score: best.similarity * (1 + best.entry.pull / 10) }]
        : [];
    })
    .sort((first, second) => second.score - first.score)
    .slice(0, MAX_SUGGESTIONS)
    .map(({ game, because }) => ({ game, because }));
}

export interface Category {
  name: string;
  games: Game[];
}

export const OTHER_CATEGORY = "Autres";

export function categorize(games: Game[]): Category[] {
  const byCategory = new Map<string, Game[]>();
  for (const game of games) {
    const name = game.details.genre ?? OTHER_CATEGORY;
    byCategory.set(name, [...(byCategory.get(name) ?? []), game]);
  }
  return [...byCategory.entries()]
    .map(([name, categoryGames]) => ({
      name,
      games: [...categoryGames].sort(
        (first, second) =>
          Number(second.installed) - Number(first.installed) || first.title.localeCompare(second.title, "fr"),
      ),
    }))
    .sort((first, second) => {
      if (first.name === OTHER_CATEGORY) return 1;
      if (second.name === OTHER_CATEGORY) return -1;
      return second.games.length - first.games.length || first.name.localeCompare(second.name, "fr");
    });
}
