import type Hopr from '@hoprnet/hopr-core'
import { getBalances } from './api/v2/paths/account/balances'
import { getInfo } from './api/v2/paths/node/info'

const COMMANDS = ['info', 'balance', 'daemonize'] as const

export const isSupported = (command: any): boolean => {
  return COMMANDS.includes(command)
}

const run = async (node: Hopr, command: typeof COMMANDS[number]): Promise<[shouldExit: boolean, result: string]> => {
  if (command === 'balance') {
    const output = await getBalances(node)
    return [true, JSON.stringify(output, null, 2)]
  } else if (command === 'info') {
    const output = await getInfo(node)
    return [true, JSON.stringify(output, null, 2)]
  } else if (command === 'daemonize') {
    return [false, '']
  }
}

export default run
