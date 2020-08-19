// GENERATED CODE -- DO NOT EDIT!

// package: address
// file: address.proto

import * as address_pb from "./address_pb";
import * as grpc from "grpc";

interface IAddressService extends grpc.ServiceDefinition<grpc.UntypedServiceImplementation> {
  getNativeAddress: grpc.MethodDefinition<address_pb.GetNativeAddressRequest, address_pb.GetNativeAddressResponse>;
  getHoprAddress: grpc.MethodDefinition<address_pb.GetHoprAddressRequest, address_pb.GetHoprAddressResponse>;
}

export const AddressService: IAddressService;

export class AddressClient extends grpc.Client {
  constructor(address: string, credentials: grpc.ChannelCredentials, options?: object);
  getNativeAddress(argument: address_pb.GetNativeAddressRequest, callback: grpc.requestCallback<address_pb.GetNativeAddressResponse>): grpc.ClientUnaryCall;
  getNativeAddress(argument: address_pb.GetNativeAddressRequest, metadataOrOptions: grpc.Metadata | grpc.CallOptions | null, callback: grpc.requestCallback<address_pb.GetNativeAddressResponse>): grpc.ClientUnaryCall;
  getNativeAddress(argument: address_pb.GetNativeAddressRequest, metadata: grpc.Metadata | null, options: grpc.CallOptions | null, callback: grpc.requestCallback<address_pb.GetNativeAddressResponse>): grpc.ClientUnaryCall;
  getHoprAddress(argument: address_pb.GetHoprAddressRequest, callback: grpc.requestCallback<address_pb.GetHoprAddressResponse>): grpc.ClientUnaryCall;
  getHoprAddress(argument: address_pb.GetHoprAddressRequest, metadataOrOptions: grpc.Metadata | grpc.CallOptions | null, callback: grpc.requestCallback<address_pb.GetHoprAddressResponse>): grpc.ClientUnaryCall;
  getHoprAddress(argument: address_pb.GetHoprAddressRequest, metadata: grpc.Metadata | null, options: grpc.CallOptions | null, callback: grpc.requestCallback<address_pb.GetHoprAddressResponse>): grpc.ClientUnaryCall;
}
