// GENERATED CODE -- DO NOT EDIT!

// package: channels
// file: channels.proto

import * as channels_pb from "./channels_pb";
import * as grpc from "grpc";

interface IChannelsService extends grpc.ServiceDefinition<grpc.UntypedServiceImplementation> {
  getChannels: grpc.MethodDefinition<channels_pb.GetChannelsRequest, channels_pb.GetChannelsResponse>;
  getChannelData: grpc.MethodDefinition<channels_pb.GetChannelDataRequest, channels_pb.GetChannelDataResponse>;
  openChannel: grpc.MethodDefinition<channels_pb.OpenChannelRequest, channels_pb.OpenChannelResponse>;
  closeChannel: grpc.MethodDefinition<channels_pb.CloseChannelRequest, channels_pb.CloseChannelResponse>;
}

export const ChannelsService: IChannelsService;

export class ChannelsClient extends grpc.Client {
  constructor(address: string, credentials: grpc.ChannelCredentials, options?: object);
  getChannels(argument: channels_pb.GetChannelsRequest, callback: grpc.requestCallback<channels_pb.GetChannelsResponse>): grpc.ClientUnaryCall;
  getChannels(argument: channels_pb.GetChannelsRequest, metadataOrOptions: grpc.Metadata | grpc.CallOptions | null, callback: grpc.requestCallback<channels_pb.GetChannelsResponse>): grpc.ClientUnaryCall;
  getChannels(argument: channels_pb.GetChannelsRequest, metadata: grpc.Metadata | null, options: grpc.CallOptions | null, callback: grpc.requestCallback<channels_pb.GetChannelsResponse>): grpc.ClientUnaryCall;
  getChannelData(argument: channels_pb.GetChannelDataRequest, callback: grpc.requestCallback<channels_pb.GetChannelDataResponse>): grpc.ClientUnaryCall;
  getChannelData(argument: channels_pb.GetChannelDataRequest, metadataOrOptions: grpc.Metadata | grpc.CallOptions | null, callback: grpc.requestCallback<channels_pb.GetChannelDataResponse>): grpc.ClientUnaryCall;
  getChannelData(argument: channels_pb.GetChannelDataRequest, metadata: grpc.Metadata | null, options: grpc.CallOptions | null, callback: grpc.requestCallback<channels_pb.GetChannelDataResponse>): grpc.ClientUnaryCall;
  openChannel(argument: channels_pb.OpenChannelRequest, callback: grpc.requestCallback<channels_pb.OpenChannelResponse>): grpc.ClientUnaryCall;
  openChannel(argument: channels_pb.OpenChannelRequest, metadataOrOptions: grpc.Metadata | grpc.CallOptions | null, callback: grpc.requestCallback<channels_pb.OpenChannelResponse>): grpc.ClientUnaryCall;
  openChannel(argument: channels_pb.OpenChannelRequest, metadata: grpc.Metadata | null, options: grpc.CallOptions | null, callback: grpc.requestCallback<channels_pb.OpenChannelResponse>): grpc.ClientUnaryCall;
  closeChannel(argument: channels_pb.CloseChannelRequest, callback: grpc.requestCallback<channels_pb.CloseChannelResponse>): grpc.ClientUnaryCall;
  closeChannel(argument: channels_pb.CloseChannelRequest, metadataOrOptions: grpc.Metadata | grpc.CallOptions | null, callback: grpc.requestCallback<channels_pb.CloseChannelResponse>): grpc.ClientUnaryCall;
  closeChannel(argument: channels_pb.CloseChannelRequest, metadata: grpc.Metadata | null, options: grpc.CallOptions | null, callback: grpc.requestCallback<channels_pb.CloseChannelResponse>): grpc.ClientUnaryCall;
}
