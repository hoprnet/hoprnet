import BigNumber from 'bignumber.js'

export const toDecimalPoint = (amount: BigNumber | string | number, decimalPoint: number): string => {
  return new BigNumber(amount).multipliedBy(new BigNumber(10).pow(decimalPoint)).toString()
}
