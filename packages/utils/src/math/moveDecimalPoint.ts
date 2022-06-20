import BigNumber from 'bignumber.js'

export function moveDecimalPoint(amount: BigNumber.BigNumber | string | number, position: number): string {
  return new BigNumber.default(amount).multipliedBy(new BigNumber.default(10).pow(position)).toString(10)
}
