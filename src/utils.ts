import * as grpc from 'grpc'
import { API_URL } from './env'
import * as words from './words'

export const SetupClient = <T extends typeof grpc.Client>(Client: T): InstanceType<T> => {
  return (new Client(API_URL, grpc.credentials.createInsecure()) as unknown) as InstanceType<T>
}

export const getRandomItemFromList = <T>(items: T[]): T => {
  return items[Math.floor(Math.random() * items.length)]
}

export const generateRandomSentence = (): string => {
  const adjective = getRandomItemFromList(words.adjectives)
  const color = getRandomItemFromList(words.colors)
  const animal = getRandomItemFromList(words.animals)

  return `${adjective} ${color} ${animal}`
}
