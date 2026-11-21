import { imageUrl } from "../images";
import type { Game } from "../types";

interface BackdropProps {
  games: Game[];
  selectedId: string | null;
  veil: number;
}

function BackdropLayer({ game, active }: { game: Game; active: boolean }) {
  const heroUrl = imageUrl(game.hero);
  const coverUrl = imageUrl(game.cover);
  const source = heroUrl ?? coverUrl;
  if (!source) return null;

  return (
    <img
      className="backdrop-layer"
      data-active={active}
      data-blurred={!heroUrl}
      src={source}
      alt=""
      draggable={false}
    />
  );
}

export function Backdrop({ games, selectedId, veil }: BackdropProps) {
  return (
    <div className="backdrop" aria-hidden="true">
      {games.map((game) => (
        <BackdropLayer key={game.id} game={game} active={game.id === selectedId} />
      ))}
      <div className="backdrop-vignette" />
      <div className="backdrop-glow" />
      <div className="backdrop-veil" style={{ opacity: veil }} />
    </div>
  );
}
