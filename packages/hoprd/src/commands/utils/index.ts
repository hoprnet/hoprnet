export * from './checkPeerId.js'
export * from './message.js'

const EXTRA_PADDING = 2

export function styleValue(value: any, _type?: any): string {
  return value // no-op[
}
export function getPaddingLength(items: string[], addExtraPadding: boolean = true): number {
  return Math.max(...items.map((str) => str.length)) + (addExtraPadding ? EXTRA_PADDING : 0)
}
