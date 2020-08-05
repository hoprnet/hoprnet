import { ClientReadableStream } from "grpc-web";
import { ListenPromiseClient } from "@hoprnet/hopr-protos/web/listen_grpc_web_pb";
import {
  ListenRequest,
  ListenResponse,
} from "@hoprnet/hopr-protos/web/listen_pb";
import { API_URL } from "../env";

export const listenToMessages = async (): Promise<
  ClientReadableStream<ListenResponse>
> => {
  const client = new ListenPromiseClient(API_URL);

  return client.listen(new ListenRequest());
};
