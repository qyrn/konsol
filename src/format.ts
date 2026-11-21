const DAY_MS = 86_400_000;
const monthFormatter = new Intl.DateTimeFormat("fr-FR", { month: "long" });
const sizeFormatter = new Intl.NumberFormat("fr-FR", { maximumFractionDigits: 1 });

export function playtimeLabel(minutes: number | null): string | null {
  if (!minutes) return null;
  if (minutes < 60) return `${minutes} min`;
  return `${Math.round(minutes / 60)} h`;
}

function startOfDay(timestamp: number): number {
  const date = new Date(timestamp);
  date.setHours(0, 0, 0, 0);
  return date.getTime();
}

export function lastPlayedLabel(lastPlayed: number | null, now: number): string | null {
  if (!lastPlayed) return null;
  const playedAt = lastPlayed * 1000;
  const days = Math.round((startOfDay(now) - startOfDay(playedAt)) / DAY_MS);
  if (days <= 0) return "Aujourd'hui";
  if (days === 1) return "Hier";
  if (days < 7) return `Il y a ${days} jours`;
  if (days < 14) return "La semaine dernière";
  if (days < 30) return `Il y a ${Math.floor(days / 7)} semaines`;
  const played = new Date(playedAt);
  const today = new Date(now);
  const monthsAgo = (today.getFullYear() - played.getFullYear()) * 12 + today.getMonth() - played.getMonth();
  if (monthsAgo <= 1) return "Le mois dernier";
  if (played.getFullYear() === today.getFullYear()) return `En ${monthFormatter.format(played)}`;
  const yearsAgo = today.getFullYear() - played.getFullYear();
  return yearsAgo === 1 ? "L'an dernier" : `Il y a ${yearsAgo} ans`;
}

export function sizeLabel(bytes: number | null): string | null {
  if (!bytes) return null;
  const gigabytes = bytes / 1024 ** 3;
  if (gigabytes >= 1) return `${sizeFormatter.format(gigabytes)} Go`;
  return `${sizeFormatter.format(bytes / 1024 ** 2)} Mo`;
}

export function rarityLabel(rarityTenths: number | null): string | null {
  if (rarityTenths === null) return null;
  return `${sizeFormatter.format(rarityTenths / 10)} % des joueurs`;
}

export function byMostRecent<T extends { title: string; details: { lastPlayed: number | null } }>(
  first: T,
  second: T,
): number {
  const difference = (second.details.lastPlayed ?? 0) - (first.details.lastPlayed ?? 0);
  return difference !== 0 ? difference : first.title.localeCompare(second.title, "fr");
}
