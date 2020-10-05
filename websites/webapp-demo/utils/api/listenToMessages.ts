import { ClientReadableStream } from 'grpc-web'
import { ListenPromiseClient } from '@hoprnet/hopr-protos/web/listen_grpc_web_pb'
import { ListenRequest, ListenResponse } from '@hoprnet/hopr-protos/web/listen_pb'

/**
 *
 * @returns a readable stream
 */
export const listenToMessages = async (apiUrl: string): Promise<ClientReadableStream<ListenResponse>> => {
  const client = new ListenPromiseClient(apiUrl)

  return client.listen(new ListenRequest())
}
