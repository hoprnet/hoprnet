// GENERATED CODE -- DO NOT EDIT!

// package: status
// file: status.proto

import * as status_pb from "./status_pb";
import * as grpc from "grpc";

interface IStatusService extends grpc.ServiceDefinition<grpc.UntypedServiceImplementation> {
  getStatus: grpc.MethodDefinition<status_pb.StatusRequest, status_pb.StatusResponse>;
}

export const StatusService: IStatusService;

export class StatusClient extends grpc.Client {
  constructor(address: string, credentials: grpc.ChannelCredentials, options?: object);
  getStatus(argument: status_pb.StatusRequest, callback: grpc.requestCallback<status_pb.StatusResponse>): grpc.ClientUnaryCall;
  getStatus(argument: status_pb.StatusRequest, metadataOrOptions: grpc.Metadata | grpc.CallOptions | null, callback: grpc.requestCallback<status_pb.StatusResponse>): grpc.ClientUnaryCall;
  getStatus(argument: status_pb.StatusRequest, metadata: grpc.Metadata | null, options: grpc.CallOptions | null, callback: grpc.requestCallback<status_pb.StatusResponse>): grpc.ClientUnaryCall;
}
