import * as grpc from 'grpc'
import { GetHoprBalanceRequest } from '@hoprnet/hopr-protos/node/balance_pb'
import { BalanceClient } from '@hoprnet/hopr-protos/node/balance_grpc_pb'
import { StatusRequest } from '@hoprnet/hopr-protos/node/status_pb'
import { WithdrawClient } from '@hoprnet/hopr-protos/node/withdraw_grpc_pb'
import { WithdrawHoprRequest } from '@hoprnet/hopr-protos/node/withdraw_pb'
import { StatusClient } from '@hoprnet/hopr-protos/node/status_grpc_pb'
import { GetHoprAddressRequest } from '@hoprnet/hopr-protos/node/address_pb'
import { ListenClient } from '@hoprnet/hopr-protos/node/listen_grpc_pb'
import { AddressClient } from '@hoprnet/hopr-protos/node/address_grpc_pb'
import { SendRequest } from '@hoprnet/hopr-protos/node/send_pb'
import { SendClient } from '@hoprnet/hopr-protos/node/send_grpc_pb'
import type { ClientReadableStream } from 'grpc'
import { ListenRequest, ListenResponse } from '@hoprnet/hopr-protos/node/listen_pb'
import { Message, IMessage } from './message'
import { API_URL } from './env'
import * as words from './words'


export const SetupClient = <T extends typeof grpc.Client>(Client: T): InstanceType<T> => {
  return (new Client(API_URL, grpc.credentials.createInsecure()) as unknown) as InstanceType<T>
}

export const getRandomItemFromList = <T>(items: T[]): T => {
  return items[Math.floor(Math.random() * items.length)]
}

export const getHOPRNodeAddressFromContent = (content: string): string => {
  return content.match(/16Uiu2HA.*?$/i) ?
      (content => {
          const [HOPRAddress_regexed] = content.match(/16Uiu2HA.*?$/i)
          const HOPRAddress = HOPRAddress_regexed.substr(0, 53)
          console.log('HoprAddress', HOPRAddress)
          return HOPRAddress;
      })(content)
      : ''
}

export const generateRandomSentence = (): string => {
  const adjective = getRandomItemFromList(words.adjectives)
  const color = getRandomItemFromList(words.colors)
  const animal = getRandomItemFromList(words.animals)

  return `${adjective} ${color} ${animal}`
}

export const getMessageStream = (): Promise<{
  client: ListenClient
  stream: ClientReadableStream<ListenResponse>
}> => {
  let client: ListenClient

  return new Promise((resolve, reject) => {
    try {
      client = SetupClient(ListenClient)
      const stream = client.listen(new ListenRequest())

      resolve({
        client,
        stream,
      })
    } catch (err) {
      reject(err)
    }
  })
}

export const sendMessage = (recepientAddress: string, message: IMessage, annonymous?: boolean, intermediatePeers?: Array<string>): Promise<void> => {
  let client: SendClient

  return new Promise((resolve, reject) => {
    try {
      client = SetupClient(SendClient)
      if (annonymous) {
        message.from = ''
      }
      const req = new SendRequest()
      req.setPeerId(recepientAddress)
      req.setPayload(Message.fromJson(message).toU8a())
      if (intermediatePeers) {
        req.setIntermediatePeerIdsList(intermediatePeers)
      }

      client.send(req, (err) => {
        if (err) return reject(err)

        console.log(`-> ${recepientAddress}:${message.text}`)
        client.close()
        resolve()
      })
    } catch (err) {
      client.close()
      reject(err)
    }
  })
}

export const sendXHOPR = (recipient: string, amount: number) => {
  let client: WithdrawClient

  return new Promise((resolve, reject) => {
    try {
      client = SetupClient(WithdrawClient)
      const req = new WithdrawHoprRequest()
      req.setRecipient(recipient)
      req.setAmount(`${amount}`)
      client.withdrawHopr(req, (err, res) => {
        if (err) return reject(err)
        client.close()
        resolve()
      })
    } catch (err) {
      client.close()
      reject(err)
    }
  })
}

export const getStatus = (): Promise<number> => {
  let client: StatusClient

  return new Promise((resolve, reject) => {
    try {
      client = SetupClient(StatusClient)

      client.getStatus(new StatusRequest(), (err, res) => {
        if (err) return reject(err)

        client.close()
        resolve(res.getConnectedNodes())
      })
    } catch (err) {
      client.close()
      reject(err)
    }
  })
}

export const getHoprBalance = (): Promise<string> => {
  let client: BalanceClient

  return new Promise((resolve, reject) => {
    try {
      client = SetupClient(BalanceClient)

      client.getHoprBalance(new GetHoprBalanceRequest(), (err, res) => {
        if (err) return reject(err)

        client.close()
        resolve(res.getAmount())
      })
    } catch (err) {
      client.close()
      reject(err)
    }
  })
}

export const getHoprAddress = (): Promise<string> => {
  let client: AddressClient

  return new Promise((resolve, reject) => {
    try {
      client = SetupClient(AddressClient)

      client.getHoprAddress(new GetHoprAddressRequest(), (err, res) => {
        if (err) return reject(err)

        client.close()
        resolve(res.getAddress())
      })
    } catch (err) {
      client.close()
      reject(err)
    }
  })
}

