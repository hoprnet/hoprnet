// package: balance
// file: balance.proto

import * as balance_pb from "./balance_pb";
import {grpc} from "@improbable-eng/grpc-web";

type BalanceGetNativeBalance = {
  readonly methodName: string;
  readonly service: typeof Balance;
  readonly requestStream: false;
  readonly responseStream: false;
  readonly requestType: typeof balance_pb.GetNativeBalanceRequest;
  readonly responseType: typeof balance_pb.GetNativeBalanceResponse;
};

type BalanceGetHoprBalance = {
  readonly methodName: string;
  readonly service: typeof Balance;
  readonly requestStream: false;
  readonly responseStream: false;
  readonly requestType: typeof balance_pb.GetHoprBalanceRequest;
  readonly responseType: typeof balance_pb.GetHoprBalanceResponse;
};

export class Balance {
  static readonly serviceName: string;
  static readonly GetNativeBalance: BalanceGetNativeBalance;
  static readonly GetHoprBalance: BalanceGetHoprBalance;
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

export class BalanceClient {
  readonly serviceHost: string;

  constructor(serviceHost: string, options?: grpc.RpcOptions);
  getNativeBalance(
    requestMessage: balance_pb.GetNativeBalanceRequest,
    metadata: grpc.Metadata,
    callback: (error: ServiceError|null, responseMessage: balance_pb.GetNativeBalanceResponse|null) => void
  ): UnaryResponse;
  getNativeBalance(
    requestMessage: balance_pb.GetNativeBalanceRequest,
    callback: (error: ServiceError|null, responseMessage: balance_pb.GetNativeBalanceResponse|null) => void
  ): UnaryResponse;
  getHoprBalance(
    requestMessage: balance_pb.GetHoprBalanceRequest,
    metadata: grpc.Metadata,
    callback: (error: ServiceError|null, responseMessage: balance_pb.GetHoprBalanceResponse|null) => void
  ): UnaryResponse;
  getHoprBalance(
    requestMessage: balance_pb.GetHoprBalanceRequest,
    callback: (error: ServiceError|null, responseMessage: balance_pb.GetHoprBalanceResponse|null) => void
  ): UnaryResponse;
}

