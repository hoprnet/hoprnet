import type { ContractEventEmitter, ContractEventLog } from '../tsc/web3/types'
import type { HoprChannels } from '../tsc/web3/HoprChannels'

/**
 * Typechain does not provide us with clean event types, in the lines below we infer
 * the generic type from the 'ContractEventEmitter' type.
 * TODO: start using ether.js and generate better types
 */
type ContractEventEmitters<T extends EventNames> = ReturnType<HoprChannels['events'][T]>
type extractGeneric<Type> = Type extends ContractEventEmitter<infer X> ? X : null

/**
 * HoprChannel's event names
 */
export type EventNames = Exclude<keyof HoprChannels['events'], 'allEvents'>
/**
 * HoprChannel's event interface
 */
export type Event<N extends EventNames> = ContractEventLog<extractGeneric<ContractEventEmitters<N>>>
