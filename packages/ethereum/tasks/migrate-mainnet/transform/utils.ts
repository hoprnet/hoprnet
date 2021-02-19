import type {
  HoprDistributorParams,
  Schedule,
  Allocations,
  HoprDistributorParamsRaw,
  ScheduleRaw,
  AllocationsRaw
} from './types'
import Web3 from 'web3'
import { durations } from '@hoprnet/hopr-utils'
import { toMultiplier } from '../../../utils'

const { toWei, toBN, toChecksumAddress } = Web3.utils

export const MULTIPLIER = String(10 ** 6)

export const transformHoprDistributorParams = (
  network: string,
  input: HoprDistributorParamsRaw
): HoprDistributorParams => {
  return {
    network,
    startTime: String(new Date(input.startTime).getTime()),
    maxMintAmount: toWei(input.maxMintAmount, 'ether'),
    multisig: toChecksumAddress(input.multisig)
  }
}

export const transformSchedule = (name: string, input: ScheduleRaw): Schedule => {
  if (input.durations.length !== input.percents.length) {
    throw Error('Durations and percents length is not the same')
  }

  return {
    name,
    durations: input.durations.map((days) => String(durations.days(Number(days)) / 1e3)),
    percents: input.percents.map((percent) => {
      const result = toMultiplier(percent, MULTIPLIER)
      if (toBN(result).gt(toBN(MULTIPLIER))) {
        throw Error('Multiplied result is higher than multiplier')
      }

      return result
    })
  }
}

export const transformAllocations = (name: string, input: AllocationsRaw): Allocations => {
  if (input.accounts.length !== input.amounts.length) {
    throw Error('Account and amounts length is not the same')
  }

  // count dups
  const dups = new Map<string, number>()

  // add duplicates
  const balances = new Map<string, string>()
  for (let i = 0; i < input.accounts.length; i++) {
    const account = toChecksumAddress(input.accounts[i])
    const amount = toWei(input.amounts[i], 'ether')
    const balance = toBN(balances.get(account) ?? 0).add(toBN(amount))

    // TEMPORARY CHECK
    const count = dups.get(account) ?? 0
    dups.set(account, count + 1)
    if (dups.get(account) > 7) {
      console.log(`found dub ${account}`)
    }

    balances.set(account, balance.toString())
  }

  return {
    name,
    accounts: Array.from(balances.keys()),
    amounts: Array.from(balances.values())
  }
}
