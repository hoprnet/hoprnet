import { API_URL } from './env'
import type { ClientReadableStream } from 'grpc'
import { SendClient } from '@hoprnet/hopr-protos/node/send_grpc_pb'
import { SendRequest } from '@hoprnet/hopr-protos/node/send_pb'
import { AddressClient } from '@hoprnet/hopr-protos/node/address_grpc_pb'
import { GetHoprAddressRequest } from '@hoprnet/hopr-protos/node/address_pb'
import { ListenClient } from '@hoprnet/hopr-protos/node/listen_grpc_pb'
import { ListenRequest, ListenResponse } from '@hoprnet/hopr-protos/node/listen_pb'
import { Message, IMessage } from './message'
import { SetupClient, generateRandomSentence } from './utils'

const getHoprAddress = (): Promise<string> => {
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

const sendMessage = (recepientAddress: string, message: IMessage): Promise<void> => {
  let client: SendClient

  return new Promise((resolve, reject) => {
    try {
      client = SetupClient(SendClient)

      const req = new SendRequest()
      req.setPeerId(recepientAddress)
      req.setPayload(Message.fromJson(message).toU8a())

      client.send(req, (err) => {
        if (err) return reject(err)

        console.log(`-> ${recepientAddress}: ${message.text}`)
        client.close()
        resolve()
      })
    } catch (err) {
      client.close()
      reject(err)
    }
  })
}

const getMessageStream = (): Promise<{
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

const start = async () => {
  console.log(`Connecting to ${API_URL}`)

  const hoprAddress = await getHoprAddress()
  console.log(`My HOPR address is ${hoprAddress}`)

  const { client, stream } = await getMessageStream()

  stream
    .on('data', (data) => {
      try {
        const [messageBuffer] = data.array
        const res = new ListenResponse()
        res.setPayload(messageBuffer)

        const message = new Message(res.getPayload_asU8()).toJson()
        console.log(`<- ${message.from}: ${message.text}`)

        sendMessage(message.from, {
          from: hoprAddress,
          text: `: Hello ${generateRandomSentence()}`,
        })
      } catch (err) {
        console.error(err)
      }
    })
    .on('error', (err) => {
      console.error(err)
    })
    .on('end', () => {
      client.close()
    })
}

start().catch(console.error)
