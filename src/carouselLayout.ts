const SCALES = [1, 0.76, 0.62, 0.52];
const OPACITIES = [1, 0.7, 0.4, 0.16];
const FIRST_STEP = 236 / 280;
const NEXT_STEP = 150 / 280;

export interface CarouselSlot {
  offset: number;
  scale: number;
  opacity: number;
}

export function carouselSlot(distance: number): CarouselSlot {
  const rank = Math.abs(distance);
  return {
    offset: rank === 0 ? 0 : Math.sign(distance) * (FIRST_STEP + (rank - 1) * NEXT_STEP),
    scale: SCALES[Math.min(rank, SCALES.length - 1)] ?? 1,
    opacity: OPACITIES[rank] ?? 0,
  };
}
