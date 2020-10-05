// package: channels
// file: channels.proto

import * as channels_pb from "./channels_pb";
import {grpc} from "@improbable-eng/grpc-web";

type ChannelsGetChannels = {
  readonly methodName: string;
  readonly service: typeof Channels;
  readonly requestStream: false;
  readonly responseStream: false;
  readonly requestType: typeof channels_pb.GetChannelsRequest;
  readonly responseType: typeof channels_pb.GetChannelsResponse;
};

type ChannelsGetChannelData = {
  readonly methodName: string;
  readonly service: typeof Channels;
  readonly requestStream: false;
  readonly responseStream: false;
  readonly requestType: typeof channels_pb.GetChannelDataRequest;
  readonly responseType: typeof channels_pb.GetChannelDataResponse;
};

type ChannelsOpenChannel = {
  readonly methodName: string;
  readonly service: typeof Channels;
  readonly requestStream: false;
  readonly responseStream: false;
  readonly requestType: typeof channels_pb.OpenChannelRequest;
  readonly responseType: typeof channels_pb.OpenChannelResponse;
};

type ChannelsCloseChannel = {
  readonly methodName: string;
  readonly service: typeof Channels;
  readonly requestStream: false;
  readonly responseStream: false;
  readonly requestType: typeof channels_pb.CloseChannelRequest;
  readonly responseType: typeof channels_pb.CloseChannelResponse;
};

export class Channels {
  static readonly serviceName: string;
  static readonly GetChannels: ChannelsGetChannels;
  static readonly GetChannelData: ChannelsGetChannelData;
  static readonly OpenChannel: ChannelsOpenChannel;
  static readonly CloseChannel: ChannelsCloseChannel;
}

export type ServiceError = { message: string, code: number; metadata: grpc.Metadata }
export type Status = { details: string, code: number; metadata: grpc.Metadata }

interface UnaryResponse {
  cancel(): void;
}
interface ResponseStream<T> {
  cancel(): void;
  on(type: 'data', handler: (message: T) => void): ResponseStream<T>;
  on(type: 'end', handler: (status?: Status) => void): ResponseStream<T>;
  on(type: 'status', handler: (status: Status) => void): ResponseStream<T>;
}
interface RequestStream<T> {
  write(message: T): RequestStream<T>;
  end(): void;
  cancel(): void;
  on(type: 'end', handler: (status?: Status) => void): RequestStream<T>;
  on(type: 'status', handler: (status: Status) => void): RequestStream<T>;
}
interface BidirectionalStream<ReqT, ResT> {
  write(message: ReqT): BidirectionalStream<ReqT, ResT>;
  end(): void;
  cancel(): void;
  on(type: 'data', handler: (message: ResT) => void): BidirectionalStream<ReqT, ResT>;
  on(type: 'end', handler: (status?: Status) => void): BidirectionalStream<ReqT, ResT>;
  on(type: 'status', handler: (status: Status) => void): BidirectionalStream<ReqT, ResT>;
}

export class ChannelsClient {
  readonly serviceHost: string;

  constructor(serviceHost: string, options?: grpc.RpcOptions);
  getChannels(
    requestMessage: channels_pb.GetChannelsRequest,
    metadata: grpc.Metadata,
    callback: (error: ServiceError|null, responseMessage: channels_pb.GetChannelsResponse|null) => void
  ): UnaryResponse;
  getChannels(
    requestMessage: channels_pb.GetChannelsRequest,
    callback: (error: ServiceError|null, responseMessage: channels_pb.GetChannelsResponse|null) => void
  ): UnaryResponse;
  getChannelData(
    requestMessage: channels_pb.GetChannelDataRequest,
    metadata: grpc.Metadata,
    callback: (error: ServiceError|null, responseMessage: channels_pb.GetChannelDataResponse|null) => void
  ): UnaryResponse;
  getChannelData(
    requestMessage: channels_pb.GetChannelDataRequest,
    callback: (error: ServiceError|null, responseMessage: channels_pb.GetChannelDataResponse|null) => void
  ): UnaryResponse;
  openChannel(
    requestMessage: channels_pb.OpenChannelRequest,
    metadata: grpc.Metadata,
    callback: (error: ServiceError|null, responseMessage: channels_pb.OpenChannelResponse|null) => void
  ): UnaryResponse;
  openChannel(
    requestMessage: channels_pb.OpenChannelRequest,
    callback: (error: ServiceError|null, responseMessage: channels_pb.OpenChannelResponse|null) => void
  ): UnaryResponse;
  closeChannel(
    requestMessage: channels_pb.CloseChannelRequest,
    metadata: grpc.Metadata,
    callback: (error: ServiceError|null, responseMessage: channels_pb.CloseChannelResponse|null) => void
  ): UnaryResponse;
  closeChannel(
    requestMessage: channels_pb.CloseChannelRequest,
    callback: (error: ServiceError|null, responseMessage: channels_pb.CloseChannelResponse|null) => void
  ): UnaryResponse;
}

