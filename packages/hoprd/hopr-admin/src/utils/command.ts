import type API from './api'
import type { Aliases } from './api'
import type { PeerId } from '@libp2p/interface-peer-id'
import { peerIdFromString } from '@libp2p/peer-id'
import { utils as ethersUtils } from 'ethers'
import { toPaddedString } from '.'

export type CacheFunctions = {
  getCachedAliases: () => Aliases
  updateAliasCache: (fn: (prevAliases: Aliases) => Aliases) => void
  getSymbols: () => { native: string; hopr: string; nativeDisplay: string; hoprDisplay: string }
}

/**
 * An abstract command.
 */
export abstract class Command {
  /**
   *
   * @param uses valid uses of this command, allows to specify various ways in which parameters can exist
   * @param hidden whether the command is shown in top level `help`
   */
  constructor(
    private uses: { [use: string]: [params: CmdParameter[], description: string] } = {},
    protected api: API,
    protected cache: CacheFunctions,
    public readonly hidden = false
  ) {}

  /**
   * @returns name of the command
   */
  abstract name(): string

  /**
   * @returns information about the command
   */
  abstract description(): string

  /**
   * Executes command
   * @param log logger that prints logs into the web console
   * @param query the query written by the user
   */
  abstract execute(log: (...args: any[]) => void, query: string): Promise<void>

  /**
   * @returns the usage of the command
   */
  public usage(): string {
    let items: [string, string][] = []

    for (const [params, desc] of Object.values(this.uses)) {
      let use: string[] = ['- usage:']

      if (params.length > 0) {
        for (const [type, givenName] of params) {
          const [defName, defDesc] = CMD_PARAMS[type]
          const name = givenName || defName // pick given name or default name
          const desc = defDesc

          use.push(`<${name} (${desc})>`)
        }
      } else {
        use.push('<none>')
      }

      items.push([use.join(' '), desc])
    }

    return toPaddedString(items)
  }

  /**
   * Generate 'no query provided' message.
   * @returns When no query is provided.
   */
  protected noQuery(): string {
    return `No query provided.\n${this.usage()}`
  }

  /**
   * Generate 'invalid query' message.
   * @returns Generic invalid query message.
   */
  protected invalidQuery(query: string): string {
    return `Invalid query, received "${query}".\n${this.usage()}`
  }

  /**
   * Generate 'invalid paramater' message.
   * @returns Specific paramater was invalid.
   */
  protected invalidParameter(param: string, type: string, error?: string): string {
    return `Invalid parameter "${param}" of type "${type}"${
      error ? ' with error "' + error + '"' : ''
    }".\n${this.usage()}`
  }

  /**
   * Generate 'failed command' message.
   * @param task what action has failed
   * @param error optional error message
   * @returns Generic error message when something has failed.
   */
  protected failedCommand(task: string, error?: string): string {
    return `Failed to ${task}${error ? ' with error "' + error + '"' : ''}.`
  }

  /**
   * Generate 'failed command' message with API
   * failure context.
   * @param response API response that failed
   * @param task what action has failed
   * @param knownErrors an object containing known errors
   * @returns Error message when something has failed.
   */
  protected async failedApiCall<T extends Response>(
    response: T,
    task: string,
    knownErrors: {
      [statusCode: number]: string | ((v: any) => string)
    }
  ): Promise<string> {
    const knownError = knownErrors[response.status]
    if (knownError) {
      return this.failedCommand(task, typeof knownError === 'string' ? knownError : knownError(await response.json()))
    } else if (response.status === 403) {
      return this.failedCommand(task, `Unauthorized. Please confirm apiToken is set`)
    } else {
      return this.failedCommand(task, `unknown error code '${response.status}'`)
    }
  }

  /**
   * Validates user's query.
   * @param query the query written by the user
   * @returns an array containing the error message (if there is one) and the query parameters
   */
  protected assertUsage(query: string): [error: string | undefined, use: string, ...parsedParams: any] {
    let result: ReturnType<typeof this.assertUsage> | undefined
    const aliases = this.cache.getCachedAliases()

    for (const [use, [params]] of Object.entries(this.uses)) {
      result = undefined

      // invalid when query is not present while parameters are expected
      if (!query && params.length > 0) {
        result = [this.noQuery(), use]
        continue
      }

      let queryParams = query.length > 0 ? query.split(' ') : []

      // if one of the params is 'everything', we treat the rest
      // past everything as one string
      const arbitraryIndex = params.findIndex((p) => p[0] === 'arbitrary')
      if (arbitraryIndex > -1) {
        const newParam = queryParams.slice(arbitraryIndex).join(' ')
        queryParams = queryParams.slice(0, arbitraryIndex)
        if (newParam.length > 0) queryParams.push(newParam)
      }

      // invalid when query params are less than expected params
      if (queryParams.length !== params.length) {
        result = [this.invalidQuery(query), use]
        continue
      }

      const parsedValues: any[] = []

      // validate each parameter
      for (let i = 0; i < params.length; i++) {
        const [paramType, customName] = params[i]
        const [, , validate] = CMD_PARAMS[paramType]
        const queryParam = queryParams[i]

        const [valid, parsedValue] = validate(queryParam, { aliases, customName })
        if (!valid) {
          result = [this.invalidParameter(queryParam, paramType), use]
        } else {
          parsedValues.push(parsedValue)
        }
      }

      // if no errors were thrown, return as validated
      if (!result) return [undefined, use, ...parsedValues]
    }

    // invalid
    return result
  }
}

/**
 * All possible command types
 */
export type CmdTypes =
  | 'hoprAddress'
  | 'nativeAddress'
  | 'hoprAddressOrAlias'
  | 'hoprOrNative'
  | 'direction'
  | 'constant'
  | 'number'
  | 'string'
  | 'arbitrary'
  | 'boolean'
export type { ChannelDirection } from './api'
export type HoprOrNative = 'hopr' | 'native'

/**
 * Used in a Command constructor to specify a command's syntax
 */
export type CmdParameter = [type: CmdTypes, customName?: string]

type CmdArg<I, O, R> = [name: string, description: string, validation: (v: I, ops: O) => [valid: boolean, value: R]]

export const CMD_PARAMS: Record<CmdTypes, CmdArg<any, any, any>> = {
  hoprAddress: [
    'HOPR address',
    "'16Ui..'",
    (v) => {
      try {
        return [true, peerIdFromString(v)]
      } catch {
        return [false, v]
      }
    }
  ],
  nativeAddress: [
    'NATIVE address',
    "'0x..",
    (v) => {
      return [ethersUtils.isAddress(v), v]
    }
  ],
  hoprAddressOrAlias: [
    'HOPR address or alias',
    "'16Ui..' or 'alice'",
    (peerIdStrOrAlias, { aliases }) => {
      // is PeerId
      let peerId: PeerId | undefined

      // try PeerId
      try {
        peerId = peerIdFromString(peerIdStrOrAlias)
      } catch {}

      // try aliases
      if (!peerId && aliases) {
        const alias = aliases[peerIdStrOrAlias]
        try {
          peerId = peerIdFromString(alias)
        } catch {}
      }

      if (peerId) return [true, peerId]
      return [false]
    }
  ] as CmdArg<string, { aliases: Record<string, string> }, PeerId>,
  hoprOrNative: [
    'currency',
    "'hopr' or 'native'",
    (input) => {
      if (typeof input !== 'string') return [false, input]
      const v = input.toLowerCase()
      return [v === 'hopr' || v === 'native', v]
    }
  ],
  direction: [
    'direction',
    "'incoming' or 'outgoing'",
    (input) => {
      if (typeof input !== 'string') return [false, input]
      const v = input.toLowerCase()
      return [v === 'incoming' || v === 'outgoing', v]
    }
  ],
  constant: [
    'constant',
    'A constant value',
    (v, { customName }) => {
      if (!customName) return [false, v]
      return [v === customName, v]
    }
  ] as CmdArg<string, { customName?: string }, any>,
  number: [
    'number',
    'Any number',
    (v) => {
      return [!isNaN(v) && isFinite(v), v]
    }
  ],
  string: [
    'string',
    'Any string with no spaces',
    (v) => {
      return [typeof v === 'string', v]
    }
  ],
  arbitrary: [
    'arbitrary',
    'An arbitrary string',
    (v) => {
      return [typeof v === 'string', v]
    }
  ],
  boolean: [
    'boolean',
    "Any boolean, 'true' or 'false'",
    (v) => {
      return [v === 'true' || v === 'false', v === 'true']
    }
  ]
}
