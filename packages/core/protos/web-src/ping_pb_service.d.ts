// package: ping
// file: ping.proto

import * as ping_pb from "./ping_pb";
import {grpc} from "@improbable-eng/grpc-web";

type PingGetPing = {
  readonly methodName: string;
  readonly service: typeof Ping;
  readonly requestStream: false;
  readonly responseStream: false;
  readonly requestType: typeof ping_pb.PingRequest;
  readonly responseType: typeof ping_pb.PingResponse;
};

export class Ping {
  static readonly serviceName: string;
  static readonly GetPing: PingGetPing;
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

export class PingClient {
  readonly serviceHost: string;

  constructor(serviceHost: string, options?: grpc.RpcOptions);
  getPing(
    requestMessage: ping_pb.PingRequest,
    metadata: grpc.Metadata,
    callback: (error: ServiceError|null, responseMessage: ping_pb.PingResponse|null) => void
  ): UnaryResponse;
  getPing(
    requestMessage: ping_pb.PingRequest,
    callback: (error: ServiceError|null, responseMessage: ping_pb.PingResponse|null) => void
  ): UnaryResponse;
}

