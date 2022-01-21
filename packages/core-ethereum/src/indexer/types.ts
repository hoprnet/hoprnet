import type { HoprToken, HoprChannels, HoprNetworkRegistry, TypedEventFilter } from '@hoprnet/hopr-ethereum'

/**
 * Typechain does not provide us with clean event types, in the lines below we infer
 * the generic type from the 'HoprChannels.filters'.
 * This allows us to retrieve HoprChannel's events.
 */
type extractEventArgs<Type> = Type extends TypedEventFilter<infer A> ? A : null

export type EventNames = keyof HoprChannels['filters']
export type Event<T extends EventNames> = extractEventArgs<ReturnType<Pick<HoprChannels['filters'], T>[T]>>
export type TokenEventNames = keyof HoprToken['filters']
export type TokenEvent<T extends TokenEventNames> = extractEventArgs<ReturnType<Pick<HoprToken['filters'], T>[T]>>
export type RegistryEventNames = keyof HoprNetworkRegistry['filters']
export type RegistryEvent<T extends RegistryEventNames> = extractEventArgs<ReturnType<Pick<HoprNetworkRegistry['filters'], T>[T]>>
export type IndexerEvents = 'announce' | 'withdraw-hopr' | 'withdraw-native' | 'channel-updated'
