import type API from './api'
import type { Aliases } from './api'
import type { PeerId } from '@libp2p/interface-peer-id'
import { peerIdFromString } from '@libp2p/peer-id'
import { utils as ethersUtils } from 'ethers'
import { toPaddedString } from '.'

export type CacheFunctions = {
  getCachedAliases: () => Aliases
  updateAliasCache: (fn: (prevAliases: Aliases) => Aliases) => void
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
        for (const [type, name, optional] of params) {
          // const [paramDesc] = CMD_PARAMS[type]
          // use.push(`<${name} [${optional ? '?' : ''}${type} (${paramDesc})]>`)
          use.push(`<${name} [${optional ? '?' : ''}${type}]>`)
        }
      } else {
        use.push('<none>')
      }

      items.push([use.join(' '), desc])
    }

    return toPaddedString(items)
  }

  /**
   * @returns Generic invalid query message.
   */
  protected invalidUsage(query: string): string {
    return `Invalid arguments, received "${query}".\n${this.usage()}`
  }

  /**
   * @param task what has failed
   * @returns Generic error message when request has failed.
   */
  protected invalidResponse(task: string): string {
    return `Failed to ${task}.`
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
        result = [`No query provided.\n${this.usage()}`, use]
        continue
      }

      const queryParams = query.length > 0 ? query.split(' ') : ''

      // invalid when query params and expected params are not the same length
      if (queryParams.length !== params.length) {
        result = [this.invalidUsage(query), use]
        continue
      }

      const parsedValues: any[] = []

      // validate each parameter
      for (let i = 0; i < params.length; i++) {
        const [paramType] = params[i]
        const [, validate] = CMD_PARAMS[paramType]
        const queryParam = queryParams[i]

        const [valid, parsedValue] = validate(queryParam, { aliases })
        if (!valid) {
          result = [`Incorrect parameter "${queryParam}" of type "${paramType}".\n${this.usage()}`, use]
          continue
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

export type CmdParameter = [type: CmdTypes, name: string, optional?: boolean]
type CmdTypes =
  | 'hoprAddress'
  | 'nativeAddress'
  | 'hoprAddressOrAlias'
  | 'hoprOrNative'
  | 'number'
  | 'string'
  | 'boolean'
  | 'constant'
type CmdArg<I, O, R> = [description: string, validation: (v: I, ops: O) => [valid: boolean, value: R]]

export const CMD_PARAMS: Record<CmdTypes, CmdArg<any, any, any>> = {
  hoprAddress: [
    'A HOPR address (PeerId)',
    (v) => {
      try {
        return [true, peerIdFromString(v)]
      } catch {
        return [false, v]
      }
    }
  ],
  nativeAddress: [
    'A native address',
    (v) => {
      return [ethersUtils.isAddress(v), v]
    }
  ],
  hoprAddressOrAlias: [
    'A HOPR address (PeerId) or an alias',
    (peerIdStrOrAlias, { aliases }) => {
      // is PeerId
      let peerId: PeerId | undefined

      // try PeerId
      try {
        peerId = peerIdFromString(peerIdStrOrAlias)
      } catch {
        console.log(`Could not create peer id from '${peerIdStrOrAlias}'`)
      }

      // try aliases
      if (!peerId && aliases) {
        const alias = aliases[peerIdStrOrAlias]
        try {
          peerId = peerIdFromString(alias)
        } catch {
          console.log(`Could not create peer id from alias '${alias}' for '${peerIdStrOrAlias}'`)
        }
      }

      if (peerId) return [true, peerId]
      return [false]
    }
  ] as CmdArg<string, { aliases: Record<string, string> }, PeerId>,
  hoprOrNative: [
    "'HOPR' or 'NATIVE'",
    (v) => {
      return [v === 'HOPR' || v === 'NATIVE', v]
    }
  ],
  number: [
    'A number',
    (v) => {
      return [!isNaN(v), v]
    }
  ],
  string: [
    'A string',
    (v) => {
      return [typeof v === 'string', v]
    }
  ],
  boolean: [
    'A boolean',
    (v) => {
      return [v === 'true' || v === 'false', Boolean(v)]
    }
  ],
  constant: [
    'A constant value',
    (v) => {
      return [true, v]
    }
  ]
}
