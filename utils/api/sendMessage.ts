import { SendPromiseClient } from "@hoprnet/hopr-protos/web/send_grpc_web_pb";
import { SendRequest } from "@hoprnet/hopr-protos/web/send_pb";
import { API_URL } from "../env";
import Message from "../Message";

export const sendMessage = async (peerId: string): Promise<void> => {
  const client = new SendPromiseClient(API_URL);
  const req = new SendRequest();
  req.setPeerId(peerId);
  req.setPayload(Message.fromText(peerId));

  await client.send(req);
};
