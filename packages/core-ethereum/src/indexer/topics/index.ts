/**
 * This folder includes the encoders / decoders required to translate
 * our SC logs to events.
 */

import type { Log } from 'web3-core'
import type { Event, EventData, Topics } from './types'
import { u8aToHex } from '@hoprnet/hopr-utils'
import { generateTopics, EventSignatures } from './utils'
import * as logs from './logs'
export * from './logs'
export * from './utils'
export * from './types'

/**
 * known event topics0 that will be used to distinguish
 * which event type we are looking at
 */
export const EventTopics0: {
  [K in keyof EventData]: Topics
} = {
  SecretHashSet: [u8aToHex(EventSignatures.SecretHashSet)],
  FundedChannel: generateTopics(EventSignatures.FundedChannel, undefined, undefined),
  OpenedChannel: generateTopics(EventSignatures.OpenedChannel, undefined, undefined),
  RedeemedTicket: generateTopics(EventSignatures.RedeemedTicket, undefined, undefined),
  InitiatedChannelClosure: generateTopics(EventSignatures.InitiatedChannelClosure, undefined, undefined),
  ClosedChannel: generateTopics(EventSignatures.ClosedChannel, undefined, undefined)
}

export const logToEvent = (log: Log): Event<any> | undefined => {
  const [topic0] = log.topics

  if (EventTopics0.SecretHashSet[0].includes(topic0)) {
    return logs.toSecretHashSetEvent(log)
  } else if (EventTopics0.FundedChannel[0].includes(topic0)) {
    return logs.toFundedChannelEvent(log)
  } else if (EventTopics0.OpenedChannel[0].includes(topic0)) {
    return logs.toOpenedChannelEvent(log)
  } else if (EventTopics0.RedeemedTicket[0].includes(topic0)) {
    return logs.toRedeemedTicketEvent(log)
  } else if (EventTopics0.InitiatedChannelClosure[0].includes(topic0)) {
    return logs.toInitiatedChannelClosureEvent(log)
  } else if (EventTopics0.ClosedChannel[0].includes(topic0)) {
    return logs.toClosedChannelEvent(log)
  }
}
