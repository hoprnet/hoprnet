export const toMultiplier = (percent: string, multiplier: string): string => {
  return String(Math.floor(Number(percent) * Number(multiplier)))
}
