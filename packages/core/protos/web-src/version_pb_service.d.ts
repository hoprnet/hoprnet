// package: version
// file: version.proto

import * as version_pb from "./version_pb";
import {grpc} from "@improbable-eng/grpc-web";

type VersionGetVersion = {
  readonly methodName: string;
  readonly service: typeof Version;
  readonly requestStream: false;
  readonly responseStream: false;
  readonly requestType: typeof version_pb.VersionRequest;
  readonly responseType: typeof version_pb.VersionResponse;
};

export class Version {
  static readonly serviceName: string;
  static readonly GetVersion: VersionGetVersion;
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

export class VersionClient {
  readonly serviceHost: string;

  constructor(serviceHost: string, options?: grpc.RpcOptions);
  getVersion(
    requestMessage: version_pb.VersionRequest,
    metadata: grpc.Metadata,
    callback: (error: ServiceError|null, responseMessage: version_pb.VersionResponse|null) => void
  ): UnaryResponse;
  getVersion(
    requestMessage: version_pb.VersionRequest,
    callback: (error: ServiceError|null, responseMessage: version_pb.VersionResponse|null) => void
  ): UnaryResponse;
}

