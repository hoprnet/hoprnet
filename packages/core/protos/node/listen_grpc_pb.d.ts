// GENERATED CODE -- DO NOT EDIT!

// package: listen
// file: listen.proto

import * as listen_pb from "./listen_pb";
import * as grpc from "grpc";

interface IListenService extends grpc.ServiceDefinition<grpc.UntypedServiceImplementation> {
  listen: grpc.MethodDefinition<listen_pb.ListenRequest, listen_pb.ListenResponse>;
}

export const ListenService: IListenService;

export class ListenClient extends grpc.Client {
  constructor(address: string, credentials: grpc.ChannelCredentials, options?: object);
  listen(argument: listen_pb.ListenRequest, metadataOrOptions?: grpc.Metadata | grpc.CallOptions | null): grpc.ClientReadableStream<listen_pb.ListenResponse>;
  listen(argument: listen_pb.ListenRequest, metadata?: grpc.Metadata | null, options?: grpc.CallOptions | null): grpc.ClientReadableStream<listen_pb.ListenResponse>;
}
