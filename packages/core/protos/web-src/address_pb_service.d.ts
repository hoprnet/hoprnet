// package: address
// file: address.proto

import * as address_pb from "./address_pb";
import {grpc} from "@improbable-eng/grpc-web";

type AddressGetNativeAddress = {
  readonly methodName: string;
  readonly service: typeof Address;
  readonly requestStream: false;
  readonly responseStream: false;
  readonly requestType: typeof address_pb.GetNativeAddressRequest;
  readonly responseType: typeof address_pb.GetNativeAddressResponse;
};

type AddressGetHoprAddress = {
  readonly methodName: string;
  readonly service: typeof Address;
  readonly requestStream: false;
  readonly responseStream: false;
  readonly requestType: typeof address_pb.GetHoprAddressRequest;
  readonly responseType: typeof address_pb.GetHoprAddressResponse;
};

export class Address {
  static readonly serviceName: string;
  static readonly GetNativeAddress: AddressGetNativeAddress;
  static readonly GetHoprAddress: AddressGetHoprAddress;
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

export class AddressClient {
  readonly serviceHost: string;

  constructor(serviceHost: string, options?: grpc.RpcOptions);
  getNativeAddress(
    requestMessage: address_pb.GetNativeAddressRequest,
    metadata: grpc.Metadata,
    callback: (error: ServiceError|null, responseMessage: address_pb.GetNativeAddressResponse|null) => void
  ): UnaryResponse;
  getNativeAddress(
    requestMessage: address_pb.GetNativeAddressRequest,
    callback: (error: ServiceError|null, responseMessage: address_pb.GetNativeAddressResponse|null) => void
  ): UnaryResponse;
  getHoprAddress(
    requestMessage: address_pb.GetHoprAddressRequest,
    metadata: grpc.Metadata,
    callback: (error: ServiceError|null, responseMessage: address_pb.GetHoprAddressResponse|null) => void
  ): UnaryResponse;
  getHoprAddress(
    requestMessage: address_pb.GetHoprAddressRequest,
    callback: (error: ServiceError|null, responseMessage: address_pb.GetHoprAddressResponse|null) => void
  ): UnaryResponse;
}

