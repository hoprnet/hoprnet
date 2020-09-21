// package: listen
// file: listen.proto

import * as listen_pb from "./listen_pb";
import {grpc} from "@improbable-eng/grpc-web";

type ListenListen = {
  readonly methodName: string;
  readonly service: typeof Listen;
  readonly requestStream: false;
  readonly responseStream: true;
  readonly requestType: typeof listen_pb.ListenRequest;
  readonly responseType: typeof listen_pb.ListenResponse;
};

export class Listen {
  static readonly serviceName: string;
  static readonly Listen: ListenListen;
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

export class ListenClient {
  readonly serviceHost: string;

  constructor(serviceHost: string, options?: grpc.RpcOptions);
  listen(requestMessage: listen_pb.ListenRequest, metadata?: grpc.Metadata): ResponseStream<listen_pb.ListenResponse>;
}

