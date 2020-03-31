import { HoprChannels as IHoprChannels } from '../tsc/web3/HoprChannels'

type EventNames = keyof IHoprChannels['events']
type EventEmitter<N extends EventNames> = ReturnType<IHoprChannels['events'][N]>
type EventEmitters = EventEmitter<EventNames>

const store = new Map<EventNames, EventEmitters[]>()

const addEvent = <N extends EventNames>(name: N, event: EventEmitter<N>): EventEmitter<N> => {
  const events = store.get(name) || []

  events.push(event)
  store.set(name, events)

  return event
}

const clearEvents = (name: EventNames): void => {
  const events = store.get(name) || []

  // @TODO: needs testing
  for (const event of events) {
    event.removeAllListeners()
  }
  store.set(name, [])
}

const clearAllEvents = (): void => {
  for (const events of store.keys()) {
    clearEvents(events)
  }
}

export { addEvent, clearEvents, clearAllEvents }
