// GENERATED CODE -- DO NOT EDIT!

// package: send
// file: send.proto

import * as send_pb from "./send_pb";
import * as grpc from "grpc";

interface ISendService extends grpc.ServiceDefinition<grpc.UntypedServiceImplementation> {
  send: grpc.MethodDefinition<send_pb.SendRequest, send_pb.SendResponse>;
}

export const SendService: ISendService;

export class SendClient extends grpc.Client {
  constructor(address: string, credentials: grpc.ChannelCredentials, options?: object);
  send(argument: send_pb.SendRequest, callback: grpc.requestCallback<send_pb.SendResponse>): grpc.ClientUnaryCall;
  send(argument: send_pb.SendRequest, metadataOrOptions: grpc.Metadata | grpc.CallOptions | null, callback: grpc.requestCallback<send_pb.SendResponse>): grpc.ClientUnaryCall;
  send(argument: send_pb.SendRequest, metadata: grpc.Metadata | null, options: grpc.CallOptions | null, callback: grpc.requestCallback<send_pb.SendResponse>): grpc.ClientUnaryCall;
}
