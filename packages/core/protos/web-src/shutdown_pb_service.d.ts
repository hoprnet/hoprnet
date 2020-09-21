// package: shutdown
// file: shutdown.proto

import * as shutdown_pb from "./shutdown_pb";
import {grpc} from "@improbable-eng/grpc-web";

type ShutdownShutdown = {
  readonly methodName: string;
  readonly service: typeof Shutdown;
  readonly requestStream: false;
  readonly responseStream: false;
  readonly requestType: typeof shutdown_pb.ShutdownRequest;
  readonly responseType: typeof shutdown_pb.ShutdownResponse;
};

export class Shutdown {
  static readonly serviceName: string;
  static readonly Shutdown: ShutdownShutdown;
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

export class ShutdownClient {
  readonly serviceHost: string;

  constructor(serviceHost: string, options?: grpc.RpcOptions);
  shutdown(
    requestMessage: shutdown_pb.ShutdownRequest,
    metadata: grpc.Metadata,
    callback: (error: ServiceError|null, responseMessage: shutdown_pb.ShutdownResponse|null) => void
  ): UnaryResponse;
  shutdown(
    requestMessage: shutdown_pb.ShutdownRequest,
    callback: (error: ServiceError|null, responseMessage: shutdown_pb.ShutdownResponse|null) => void
  ): UnaryResponse;
}

