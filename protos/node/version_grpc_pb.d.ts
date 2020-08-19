// GENERATED CODE -- DO NOT EDIT!

// package: version
// file: version.proto

import * as version_pb from "./version_pb";
import * as grpc from "grpc";

interface IVersionService extends grpc.ServiceDefinition<grpc.UntypedServiceImplementation> {
  getVersion: grpc.MethodDefinition<version_pb.VersionRequest, version_pb.VersionResponse>;
}

export const VersionService: IVersionService;

export class VersionClient extends grpc.Client {
  constructor(address: string, credentials: grpc.ChannelCredentials, options?: object);
  getVersion(argument: version_pb.VersionRequest, callback: grpc.requestCallback<version_pb.VersionResponse>): grpc.ClientUnaryCall;
  getVersion(argument: version_pb.VersionRequest, metadataOrOptions: grpc.Metadata | grpc.CallOptions | null, callback: grpc.requestCallback<version_pb.VersionResponse>): grpc.ClientUnaryCall;
  getVersion(argument: version_pb.VersionRequest, metadata: grpc.Metadata | null, options: grpc.CallOptions | null, callback: grpc.requestCallback<version_pb.VersionResponse>): grpc.ClientUnaryCall;
}
