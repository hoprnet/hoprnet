import BigNumber from 'bignumber.js'

export const toDecimalPoint = (amount: ConstructorParameters<typeof BigNumber>[0], decimalPoint: number): string => {
  return new BigNumber(amount).multipliedBy(new BigNumber(10).pow(decimalPoint)).toString()
}
