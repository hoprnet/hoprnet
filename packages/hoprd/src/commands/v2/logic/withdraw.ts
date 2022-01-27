import Hopr from '@hoprnet/hopr-core'
import { Balance, moveDecimalPoint, NativeBalance } from '@hoprnet/hopr-utils'
import { isError } from '..'
import { styleValue } from '../../utils'

type WithdrawArgs = {
  amount: string
  weiAmount: string
  currency: 'NATIVE' | 'HOPR'
  recipient: string
}

const validateWithdrawArgs = async ({
  amount,
  currency,
  recipient,
  log
}: {
  amount: string
  currency: string
  recipient: string
  log?: (string) => void
}): Promise<WithdrawArgs | Error> => {
  const validCurrency = currency.toUpperCase() as 'NATIVE' | 'HOPR'
  if (!['NATIVE', 'HOPR'].includes(validCurrency)) {
    log &&
      log(
        styleValue(`Incorrect currency provided: '${validCurrency}', correct options are: 'native', 'hopr'.`, 'failure')
      )
    return new Error('incorrectCurrency')
  }

  if (isNaN(Number(amount))) {
    log && log(styleValue(`Incorrect amount provided: '${amount}'.`, 'failure'))
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
  node,
  log
}: {
  rawCurrency: string
  rawRecipient: string
  rawAmount: string
  node: Hopr
  log?: (string) => void
}) => {
  const validation = await validateWithdrawArgs({
    amount: rawAmount,
    currency: rawCurrency,
    recipient: rawRecipient
  })
  if (isError(validation)) {
    return validation
  }

  const { amount, weiAmount, recipient, currency } = validation
  const symbol = currency === 'NATIVE' ? NativeBalance.SYMBOL : Balance.SYMBOL
  const receipt = await node.withdraw(currency, recipient, weiAmount)

  log &&
    log(
      `Withdrawing ${styleValue(amount, 'number')} ${symbol} to ${styleValue(
        recipient,
        'peerId'
      )}, receipt ${styleValue(receipt, 'hash')}.`
    )

  return
}
