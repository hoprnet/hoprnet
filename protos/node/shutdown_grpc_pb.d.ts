// GENERATED CODE -- DO NOT EDIT!

// package: shutdown
// file: shutdown.proto

import * as shutdown_pb from "./shutdown_pb";
import * as grpc from "grpc";

interface IShutdownService extends grpc.ServiceDefinition<grpc.UntypedServiceImplementation> {
  shutdown: grpc.MethodDefinition<shutdown_pb.ShutdownRequest, shutdown_pb.ShutdownResponse>;
}

export const ShutdownService: IShutdownService;

export class ShutdownClient extends grpc.Client {
  constructor(address: string, credentials: grpc.ChannelCredentials, options?: object);
  shutdown(argument: shutdown_pb.ShutdownRequest, callback: grpc.requestCallback<shutdown_pb.ShutdownResponse>): grpc.ClientUnaryCall;
  shutdown(argument: shutdown_pb.ShutdownRequest, metadataOrOptions: grpc.Metadata | grpc.CallOptions | null, callback: grpc.requestCallback<shutdown_pb.ShutdownResponse>): grpc.ClientUnaryCall;
  shutdown(argument: shutdown_pb.ShutdownRequest, metadata: grpc.Metadata | null, options: grpc.CallOptions | null, callback: grpc.requestCallback<shutdown_pb.ShutdownResponse>): grpc.ClientUnaryCall;
}
