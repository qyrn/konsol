export function titleSizeClass(title: string): string {
  const length = title.length;
  if (length <= 10) return "title-size-xl";
  if (length <= 16) return "title-size-l";
  if (length <= 24) return "title-size-m";
  return "title-size-s";
}
