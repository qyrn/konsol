import { useEffect, useMemo, useState } from "react";
import { recentSessions } from "../api";
import { playtimeLabel } from "../format";
import { DEFAULT_ACCENT, cssVariable } from "../images";
import type { Game, SessionSpan } from "../types";

interface WeekRecapProps {
  games: Game[];
  refreshKey: number;
}

const DAYS = 7;
const TOP_GAMES = 5;
const dayFormatter = new Intl.DateTimeFormat("fr-FR", { weekday: "short" });

function startOfDay(timestamp: number): number {
  const date = new Date(timestamp);
  date.setHours(0, 0, 0, 0);
  return date.getTime();
}

export function WeekRecap({ games, refreshKey }: WeekRecapProps) {
  const [sessions, setSessions] = useState<SessionSpan[] | null>(null);

  useEffect(() => {
    let cancelled = false;
    recentSessions()
      .then((found) => {
        if (!cancelled) setSessions(found);
      })
      .catch(() => {
        if (!cancelled) setSessions([]);
      });
    return () => {
      cancelled = true;
    };
  }, [refreshKey]);

  const recap = useMemo(() => {
    const today = startOfDay(Date.now());
    const days = Array.from({ length: DAYS }, (_, index) => ({
      start: today - (DAYS - 1 - index) * 86_400_000,
      minutes: 0,
    }));
    const perGame = new Map<string, number>();
    for (const session of sessions ?? []) {
      const minutes = Math.max(0, (session.endedAt - session.startedAt) / 60);
      perGame.set(session.gameId, (perGame.get(session.gameId) ?? 0) + minutes);
      const day = days.find((entry) => entry.start === startOfDay(session.endedAt * 1000));
      if (day) day.minutes += minutes;
    }
    const gamesById = new Map(games.map((game) => [game.id, game]));
    const top = [...perGame.entries()]
      .flatMap(([gameId, minutes]) => {
        const game = gamesById.get(gameId);
        return game ? [{ game, minutes }] : [];
      })
      .sort((first, second) => second.minutes - first.minutes)
      .slice(0, TOP_GAMES);
    const total = [...perGame.values()].reduce((sum, minutes) => sum + minutes, 0);
    return { days, top, total, busiest: Math.max(1, ...days.map((day) => day.minutes)) };
  }, [games, sessions]);

  if (!sessions) return null;

  return (
    <section className="achievements-section week-recap">
      <h2>
        Ta semaine
        <span>
          {recap.total > 0
            ? `${playtimeLabel(Math.round(recap.total))} de jeu`
            : "Pas encore de partie cette semaine"}
        </span>
      </h2>
      {recap.total > 0 && (
        <div className="week-recap-body">
          <ol className="week-days" aria-label="Temps de jeu par jour">
            {recap.days.map((day) => (
              <li key={day.start}>
                <span
                  className="week-bar"
                  style={cssVariable("--fill", `${(day.minutes / recap.busiest) * 100}%`)}
                />
                <span className="week-day">{dayFormatter.format(new Date(day.start))}</span>
              </li>
            ))}
          </ol>
          <ul className="week-games">
            {recap.top.map(({ game, minutes }) => (
              <li key={game.id} style={cssVariable("--item-accent", game.accent ?? DEFAULT_ACCENT)}>
                <span className="week-game-title">{game.title}</span>
                <span className="week-game-time">{playtimeLabel(Math.round(minutes))}</span>
              </li>
            ))}
          </ul>
        </div>
      )}
    </section>
  );
}
