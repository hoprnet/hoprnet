import BigNumber from 'bignumber.js'

export function moveDecimalPoint(amount: BigNumber | string | number, position: number): string {
  return new BigNumber(amount).multipliedBy(new BigNumber(10).pow(position)).toString(10)
}
