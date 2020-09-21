// GENERATED CODE -- DO NOT EDIT!

// package: ping
// file: ping.proto

import * as ping_pb from "./ping_pb";
import * as grpc from "grpc";

interface IPingService extends grpc.ServiceDefinition<grpc.UntypedServiceImplementation> {
  getPing: grpc.MethodDefinition<ping_pb.PingRequest, ping_pb.PingResponse>;
}

export const PingService: IPingService;

export class PingClient extends grpc.Client {
  constructor(address: string, credentials: grpc.ChannelCredentials, options?: object);
  getPing(argument: ping_pb.PingRequest, callback: grpc.requestCallback<ping_pb.PingResponse>): grpc.ClientUnaryCall;
  getPing(argument: ping_pb.PingRequest, metadataOrOptions: grpc.Metadata | grpc.CallOptions | null, callback: grpc.requestCallback<ping_pb.PingResponse>): grpc.ClientUnaryCall;
  getPing(argument: ping_pb.PingRequest, metadata: grpc.Metadata | null, options: grpc.CallOptions | null, callback: grpc.requestCallback<ping_pb.PingResponse>): grpc.ClientUnaryCall;
}
