import { moveDecimalPoint } from './moveDecimal'

export const toFormattedString = (strIn: string, symbol: string = ''): string => {
  const str = moveDecimalPoint(strIn, 18 * -1)
  return `${str} ${symbol}`
}