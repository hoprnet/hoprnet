import type PeerId from 'peer-id'
import { AbstractCommand, AutoCompleteResult, GlobalState } from '../abstractCommand'
import { styleValue, getOptions, checkPeerIdInput } from '../../utils'

const ROUTING_PATH_PREFIX = 'path='

export const options: ReadonlyArray<GlobalState['routing']> = ['manual', 'direct', 'auto']

/**
 * Convert a query to a routing path
 * @param routing
 * @returns a promise that resolves into an array for peerIds
 */
export async function queryToPeerIds(query: string): Promise<PeerId[]> {
  if (!query.startsWith(ROUTING_PATH_PREFIX)) throw Error("To enter a routing path, use 'path=<peerId>,<peerId>'.")
  const routingPath = query.slice(ROUTING_PATH_PREFIX.length, query.length)
  return routingPathToPeerIds(routingPath)
}

/**
 * Looks into `state.routing` and parses the specified peerIds
 * @param routing
 * @returns a promise that resolves into an array for peerIds
 */
export async function routingPathToPeerIds(path: string): Promise<PeerId[]> {
  return Promise.all(
    path
      .split(',')
      .filter((peerId) => peerId.length > 0)
      .map(async (peerId) => await checkPeerIdInput(peerId))
  )
}

/**
 * @param state
 * @returns routing option OR a specified routing path
 */
export function getRouting(state: GlobalState): string {
  if (state.routingPath.length > 0) return state.routingPath.map((peerId) => peerId.toB58String()).join(',')
  return state.routing
}

export class Routing extends AbstractCommand {
  public name() {
    return 'routing'
  }

  public help() {
    return 'The routing algorithm that is used to send messages'
  }

  public async execute(query: string, state: GlobalState): Promise<string | void> {
    try {
      if (!query) return styleValue(getRouting(state), 'highlight')

      // reset routingPath
      state.routingPath = []

      if (options.find((o) => o === query)) {
        state.routing = query as any
      } else {
        const peerIds = await queryToPeerIds(query)

        if (peerIds.length === 0) {
          state.routing = 'direct'
        } else {
          state.routing = 'manual'
          state.routingPath = peerIds
        }
      }

      let message = `You have set your “${styleValue(this.name(), 'highlight')}” settings to “${styleValue(
        state.routing,
        'highlight'
      )}”`

      if (state.routingPath.length > 0) {
        message = `${message} with a path of “${styleValue(getRouting(state), 'highlight')}”`
      }

      return `${message}.`
    } catch (err) {
      return styleValue(err.message, 'failure')
    }
  }

  public async autocomplete(query: string, line: string): Promise<AutoCompleteResult> {
    // nothing provided, just show all options
    if (!query) {
      return [getOptions(options.map((o) => ({ value: styleValue(o, 'highlight') }))), line]
    }

    // matches a option partly, show matches options
    const matchesPartly = options.filter((option) => {
      return option.startsWith(query)
    })

    if (matchesPartly.length > 0) {
      return [matchesPartly.map((str: string) => `settings ${this.name()} ${str}`), line]
    }

    return [[this.name()], line]
  }
}
