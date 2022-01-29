import Hopr from '@hoprnet/hopr-core'
import { Balance, moveDecimalPoint, NativeBalance } from '@hoprnet/hopr-utils'
import { isError } from '.'

type WithdrawArgs = {
  amount: string
  weiAmount: string
  currency: 'NATIVE' | 'HOPR'
  recipient: string
}

const validateWithdrawArgs = async ({
  amount,
  currency,
  recipient
}: {
  amount: string
  currency: string
  recipient: string
}): Promise<WithdrawArgs | Error> => {
  const validCurrency = currency.toUpperCase() as 'NATIVE' | 'HOPR'
  if (!['NATIVE', 'HOPR'].includes(validCurrency)) {
    return new Error('incorrectCurrency')
  }

  if (isNaN(Number(amount))) {
    return new Error('incorrectAmount')
  }

  // @TODO: validate recipient address

  const weiAmount =
    validCurrency === 'NATIVE'
      ? moveDecimalPoint(amount, NativeBalance.DECIMALS)
      : moveDecimalPoint(amount, Balance.DECIMALS)

  return {
    amount,
    weiAmount,
    currency: validCurrency,
    recipient
  }
}

export const withdraw = async ({
  rawCurrency,
  rawRecipient,
  rawAmount,
  node
}: {
  rawCurrency: string
  rawRecipient: string
  rawAmount: string
  node: Hopr
}) => {
  const validation = await validateWithdrawArgs({
    amount: rawAmount,
    currency: rawCurrency,
    recipient: rawRecipient
  })
  if (isError(validation)) {
    return validation
  }

  const { weiAmount, recipient, currency } = validation
  const receipt = await node.withdraw(currency, recipient, weiAmount)

  return receipt
}
