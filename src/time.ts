export const durations = {
  seconds(seconds: number) {
    return seconds * 1e3
  },
  minutes(minutes: number) {
    return minutes * durations.seconds(60)
  },
  hours(hours: number) {
    return hours * durations.minutes(60)
  }
}