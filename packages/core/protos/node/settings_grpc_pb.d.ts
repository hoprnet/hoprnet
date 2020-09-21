// GENERATED CODE -- DO NOT EDIT!

// package: ping
// file: settings.proto

import * as settings_pb from "./settings_pb";
import * as grpc from "grpc";

interface ISettingsService extends grpc.ServiceDefinition<grpc.UntypedServiceImplementation> {
  updateSettings: grpc.MethodDefinition<settings_pb.UpdateSettingsRequest, settings_pb.UpdateSettingsResponse>;
}

export const SettingsService: ISettingsService;

export class SettingsClient extends grpc.Client {
  constructor(address: string, credentials: grpc.ChannelCredentials, options?: object);
  updateSettings(argument: settings_pb.UpdateSettingsRequest, callback: grpc.requestCallback<settings_pb.UpdateSettingsResponse>): grpc.ClientUnaryCall;
  updateSettings(argument: settings_pb.UpdateSettingsRequest, metadataOrOptions: grpc.Metadata | grpc.CallOptions | null, callback: grpc.requestCallback<settings_pb.UpdateSettingsResponse>): grpc.ClientUnaryCall;
  updateSettings(argument: settings_pb.UpdateSettingsRequest, metadata: grpc.Metadata | null, options: grpc.CallOptions | null, callback: grpc.requestCallback<settings_pb.UpdateSettingsResponse>): grpc.ClientUnaryCall;
}
