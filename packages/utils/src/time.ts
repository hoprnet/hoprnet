export const durations = {
  seconds(seconds: number) {
    return seconds * 1e3
  },
  minutes(minutes: number) {
    return minutes * durations.seconds(60)
  },
  hours(hours: number) {
    return hours * durations.minutes(60)
  },
  days(days: number) {
    return days * durations.hours(24)
  }
}

/**
 * Compares timestamps to find out if "value" has expired.
 * @param value timestamp to compare with
 * @param now timestamp example: `new Date().getTime()`
 * @param ttl in milliseconds
 * @returns true if it's expired
 */
export function isExpired(value: number, now: number, ttl: number): boolean {
  return value + ttl < now
}
