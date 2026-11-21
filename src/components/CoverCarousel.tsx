import type { CSSProperties } from "react";
import { carouselSlot } from "../carouselLayout";
import { DEFAULT_ACCENT } from "../images";
import type { Game } from "../types";
import { CoverImage } from "./CoverImage";

interface CoverCarouselProps {
  games: Game[];
  selectedId: string;
  onSelect: (game: Game) => void;
  onLaunch: (game: Game) => void;
  onPrevious: () => void;
  onNext: () => void;
}

function placement(distance: number, accent: string): CSSProperties {
  const slot = carouselSlot(distance);
  return {
    "--offset": slot.offset,
    "--scale": slot.scale,
    "--item-accent": accent,
    opacity: slot.opacity,
    zIndex: 30 - Math.abs(distance),
  } as CSSProperties;
}

export function CoverCarousel({
  games,
  selectedId,
  onSelect,
  onLaunch,
  onPrevious,
  onNext,
}: CoverCarouselProps) {
  const selectedIndex = Math.max(
    0,
    games.findIndex((game) => game.id === selectedId),
  );

  return (
    <div className="carousel" aria-label="Jeux">
      {games.map((game, index) => {
        const distance = index - selectedIndex;
        return (
          <button
            key={game.id}
            type="button"
            className="carousel-item"
            aria-label={game.title}
            aria-current={distance === 0}
            data-hidden={game.hidden}
            tabIndex={Math.abs(distance) > 3 ? -1 : 0}
            style={placement(distance, game.accent ?? DEFAULT_ACCENT)}
            onClick={() => onSelect(game)}
            onDoubleClick={() => onLaunch(game)}
          >
            <CoverImage game={game} />
          </button>
        );
      })}
      {games.length > 1 && (
        <>
          <button
            type="button"
            className="carousel-arrow carousel-arrow-previous glass"
            aria-label="Jeu précédent"
            onClick={onPrevious}
          >
            ‹
          </button>
          <button
            type="button"
            className="carousel-arrow carousel-arrow-next glass"
            aria-label="Jeu suivant"
            onClick={onNext}
          >
            ›
          </button>
        </>
      )}
    </div>
  );
}
