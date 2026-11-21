export function formatCount(value?: number): string {
  return value === undefined ? "—" : value.toLocaleString();
}
