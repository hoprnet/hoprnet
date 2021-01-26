/**
 * This folder includes the encoders / decoders required to translate
 * our SC logs to events.
 */

import type { Log } from 'web3-core'
import * as logs from './logs'
import type { Event, EventData, Topics } from './types'
import { generateTopics, EventSignatures } from './utils'
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
  FundedChannel: generateTopics(EventSignatures.FundedChannel, undefined, undefined),
  OpenedChannel: generateTopics(EventSignatures.OpenedChannel, undefined, undefined),
  RedeemedTicket: generateTopics(EventSignatures.RedeemedTicket, undefined, undefined),
  InitiatedChannelClosure: generateTopics(EventSignatures.InitiatedChannelClosure, undefined, undefined),
  ClosedChannel: generateTopics(EventSignatures.ClosedChannel, undefined, undefined)
}

export const logToEvent = (log: Log): Event<any> => {
  const [topic0] = log.topics

  if (EventTopics0.FundedChannel.includes(topic0)) {
    return logs.decodeFundedChannel(log)
  } else if (EventTopics0.OpenedChannel.includes(topic0)) {
    return logs.decodeOpenedChannel(log)
  } else if (EventTopics0.RedeemedTicket.includes(topic0)) {
    return logs.decodeRedeemedTicket(log)
  } else if (EventTopics0.InitiatedChannelClosure.includes(topic0)) {
    return logs.decodeInitiatedChannelClosure(log)
  } else if (EventTopics0.ClosedChannel.includes(topic0)) {
    return logs.decodeClosedChannel(log)
  } else {
    throw Error('Could not convert log to event')
  }
}
