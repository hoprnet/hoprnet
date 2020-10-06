import { SendPromiseClient } from '@hoprnet/hopr-protos/web/send_grpc_web_pb'
import { SendRequest } from '@hoprnet/hopr-protos/web/send_pb'
import Message from '../message'

export const sendMessage = async (apiUrl: string, peerId: string, message: Message): Promise<void> => {
  const client = new SendPromiseClient(apiUrl)
  const req = new SendRequest()

  req.setPeerId(peerId)
  req.setPayload(message.toU8a())

  await client.send(req)
}
