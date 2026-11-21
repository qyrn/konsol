import { imageUrl } from "../images";
import type { Game } from "../types";

export function Backdrop({ game }: { game: Game | null }) {
  const heroUrl = imageUrl(game?.hero ?? null);
  const coverUrl = imageUrl(game?.cover ?? null);

  return (
    <div className="backdrop" aria-hidden="true">
      {heroUrl ? (
        <img key={heroUrl} className="backdrop-image" src={heroUrl} alt="" />
      ) : coverUrl ? (
        <img key={coverUrl} className="backdrop-image backdrop-image-blurred" src={coverUrl} alt="" />
      ) : null}
      <div className="backdrop-shade" />
      <div className="backdrop-glow" />
    </div>
  );
}
