import type { HoprChannels, TypedEventFilter, TypedEvent, HoprToken } from '@hoprnet/hopr-ethereum'

/**
 * Typechain does not provide us with clean event types, in the lines below we infer
 * the generic type from the 'HoprChannels.filters'.
 * This allows us to retrieve HoprChannel's events.
 */
type extractEventArgs<Type> = Type extends TypedEventFilter<infer A, infer D> ? A & D : null

export type EventNames = keyof HoprChannels['filters']
export type Event<T extends EventNames> = TypedEvent<extractEventArgs<ReturnType<HoprChannels['filters'][T]>>>
export type TokenEventNames = keyof HoprToken['filters']
export type TokenEvent<T extends TokenEventNames> = TypedEvent<extractEventArgs<ReturnType<HoprToken['filters'][T]>>>

// export type IndexerEventNames = EventNames | TokenEventNames
// export type IndexerEvent = Event<EventNames>  | TokenEvent<TokenEventNames>

export type IndexerEvents = 'annouce' | 'withdraw-hopr' | 'withdraw-native'