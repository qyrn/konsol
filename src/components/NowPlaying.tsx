import { useNow } from "../useNow";
import type { Game } from "../types";
import { CoverImage } from "./CoverImage";

interface NowPlayingProps {
  game: Game;
  startedAt: number;
}

function twoDigits(value: number): string {
  return String(Math.floor(value)).padStart(2, "0");
}

function sessionDuration(milliseconds: number): string {
  const seconds = Math.max(0, milliseconds / 1000);
  return `${twoDigits(seconds / 3600)}:${twoDigits((seconds / 60) % 60)}:${twoDigits(seconds % 60)}`;
}

export function NowPlaying({ game, startedAt }: NowPlayingProps) {
  const now = useNow(1000);

  return (
    <section className="now-playing" aria-live="polite">
      <div className="now-playing-cover">
        <span className="now-playing-halo" aria-hidden="true" />
        <CoverImage game={game} />
      </div>
      <p className="now-playing-status">
        <span className="pulse-dot" aria-hidden="true" />
        En jeu
      </p>
      <p className="now-playing-timer">{sessionDuration(now - startedAt)}</p>
      {game.achievements && (
        <p className="now-playing-hint">Tes succès s'affichent en notification dès que tu les débloques !</p>
      )}
    </section>
  );
}
