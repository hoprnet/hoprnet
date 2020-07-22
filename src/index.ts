import type { ClientReadableStream } from 'grpc'
import { SendClient } from '@hoprnet/hopr-protos/node/send_grpc_pb'
import { SendRequest } from '@hoprnet/hopr-protos/node/send_pb'
import { AddressClient } from '@hoprnet/hopr-protos/node/address_grpc_pb'
import { GetHoprAddressRequest } from '@hoprnet/hopr-protos/node/address_pb'
import { ListenClient } from '@hoprnet/hopr-protos/node/listen_grpc_pb'
import { ListenRequest, ListenResponse } from '@hoprnet/hopr-protos/node/listen_pb'
import { Message, IMessage } from './message'
import { SetupClient } from './utils'
import { API_URL } from './env'

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

        client.close()
        resolve()
      })
    } catch (err) {
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
      const [messageBuffer] = data.array
      const res = new ListenResponse()
      res.setPayload(messageBuffer)

      const message = new Message(res.getPayload_asU8()).toJson()
      console.log(`- ${message.from} says: ${message.text}`)

      setTimeout(() => {
        sendMessage(message.from, {
          from: hoprAddress,
          text: 'hello',
        })
      }, 1e3)
    })
    .on('error', (err) => {
      console.error(err)
    })
    .on('end', () => {
      client.close()
    })
}

start().catch(console.error)
