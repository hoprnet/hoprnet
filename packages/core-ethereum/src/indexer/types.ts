import type { HoprChannels } from '../contracts'
import type { TypedListener, TypedEventFilter, TypedEvent } from '../contracts/commons'

/**
 * Typechain does not provide us with clean event types, in the lines below we infer
 * the generic type from the 'ContractEventEmitter' type.
 * TODO: start using ether.js and generate better types
 */
// type ContractEventEmitters<T extends EventNames> = ReturnType<HoprChannels['prototype']['filters'][T]>

type extractGeneric<Type> = Type extends TypedEventFilter<infer A, infer D> ? { args: A; data: D } : null
type A = extractGeneric<ReturnType<HoprChannels['filters']['AccountInitialized']>>
type B = TypedListener<A['args'], A['data']>
type C = Parameters<B>

/**
 * HoprChannel's event names
 */
//  export type EventNames = Exclude<HoprChannels['prototype']['filters'], 'allEvents'>
export type EventNames = keyof HoprChannels['filters']

/**
 * HoprChannel's event interface
 */
// export type Event<N extends EventNames> = ContractEventLog<extractGeneric<ContractEventEmitters<N>>>
export type Event<T extends EventNames> = TypedEvent<extractGeneric<HoprChannels['filters'][T]>>

type X = TypedEventFilter<[string, string, string], { account: string; uncompressedPubKey: string; secret: string }>
