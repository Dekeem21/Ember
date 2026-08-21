/** `128,45h` style playtime used across the dashboard. */
export function formatHours(seconds: number): string {
  const hours = seconds / 3600;
  if (hours < 1) return `${Math.round(seconds / 60)}m`;
  return `${hours.toFixed(2).replace(".", ",")}h`;
}

export function formatDuration(seconds: number): string {
  if (seconds <= 0) return "—";
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.round((seconds % 3600) / 60);
  if (hours === 0) return `${minutes}m`;
  return `${hours}h ${minutes}m`;
}

export function formatRelative(iso: string | null): string {
  if (!iso) return "Never played";
  const then = new Date(iso).getTime();
  if (Number.isNaN(then)) return "Never played";
  const days = Math.floor((Date.now() - then) / 86_400_000);
  if (days <= 0) return "Today";
  if (days === 1) return "Yesterday";
  if (days < 30) return `${days} days ago`;
  const months = Math.floor(days / 30);
  return months === 1 ? "1 month ago" : `${months} months ago`;
}

export function formatClock(date: Date): string {
  return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", hour12: false });
}

export const sourceLabel: Record<string, string> = {
  steam: "Steam",
  heroic: "Heroic",
  lutris: "Lutris",
  manual: "Local"
};

export const runnerLabel: Record<string, string> = {
  native: "Native",
  umu: "Proton (umu)",
  steam: "Steam client",
  heroic: "Heroic client",
  lutris: "Lutris client"
};
