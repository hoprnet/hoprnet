// package: ping
// file: settings.proto

import * as settings_pb from "./settings_pb";
import {grpc} from "@improbable-eng/grpc-web";

type SettingsUpdateSettings = {
  readonly methodName: string;
  readonly service: typeof Settings;
  readonly requestStream: false;
  readonly responseStream: false;
  readonly requestType: typeof settings_pb.UpdateSettingsRequest;
  readonly responseType: typeof settings_pb.UpdateSettingsResponse;
};

export class Settings {
  static readonly serviceName: string;
  static readonly UpdateSettings: SettingsUpdateSettings;
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

export class SettingsClient {
  readonly serviceHost: string;

  constructor(serviceHost: string, options?: grpc.RpcOptions);
  updateSettings(
    requestMessage: settings_pb.UpdateSettingsRequest,
    metadata: grpc.Metadata,
    callback: (error: ServiceError|null, responseMessage: settings_pb.UpdateSettingsResponse|null) => void
  ): UnaryResponse;
  updateSettings(
    requestMessage: settings_pb.UpdateSettingsRequest,
    callback: (error: ServiceError|null, responseMessage: settings_pb.UpdateSettingsResponse|null) => void
  ): UnaryResponse;
}

