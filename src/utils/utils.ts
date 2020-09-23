import * as words from '../bots/randobot/words'
import debug from 'debug'


const log = debug('hopr-chatbot:utils')

export const getRandomItemFromList = <T>(items: T[]): T => {
  return items[Math.floor(Math.random() * items.length)]
}

export const getHOPRNodeAddressFromContent = (content: string): string => {
  return content.match(/16Uiu2HA.*?$/i)
    ? ((content) => {
        const [HOPRAddress_regexed] = content.match(/16Uiu2HA.*?$/i)
        const HOPRAddress = HOPRAddress_regexed.substr(0, 53)
        log(` - getHOPRNodeAddressFromContent - Obtained hoprAddress ${HOPRAddress} from ${content}`)
        return HOPRAddress
      })(content)
    : ''
}

export const generateRandomSentence = (): string => {
  const adjective = getRandomItemFromList(words.adjectives)
  const color = getRandomItemFromList(words.colors)
  const animal = getRandomItemFromList(words.animals)

  return `${adjective} ${color} ${animal}`
}
