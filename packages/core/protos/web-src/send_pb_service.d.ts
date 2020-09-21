// package: send
// file: send.proto

import * as send_pb from "./send_pb";
import {grpc} from "@improbable-eng/grpc-web";

type SendSend = {
  readonly methodName: string;
  readonly service: typeof Send;
  readonly requestStream: false;
  readonly responseStream: false;
  readonly requestType: typeof send_pb.SendRequest;
  readonly responseType: typeof send_pb.SendResponse;
};

export class Send {
  static readonly serviceName: string;
  static readonly Send: SendSend;
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

export class SendClient {
  readonly serviceHost: string;

  constructor(serviceHost: string, options?: grpc.RpcOptions);
  send(
    requestMessage: send_pb.SendRequest,
    metadata: grpc.Metadata,
    callback: (error: ServiceError|null, responseMessage: send_pb.SendResponse|null) => void
  ): UnaryResponse;
  send(
    requestMessage: send_pb.SendRequest,
    callback: (error: ServiceError|null, responseMessage: send_pb.SendResponse|null) => void
  ): UnaryResponse;
}

