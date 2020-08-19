// GENERATED CODE -- DO NOT EDIT!

// package: balance
// file: balance.proto

import * as balance_pb from "./balance_pb";
import * as grpc from "grpc";

interface IBalanceService extends grpc.ServiceDefinition<grpc.UntypedServiceImplementation> {
  getNativeBalance: grpc.MethodDefinition<balance_pb.GetNativeBalanceRequest, balance_pb.GetNativeBalanceResponse>;
  getHoprBalance: grpc.MethodDefinition<balance_pb.GetHoprBalanceRequest, balance_pb.GetHoprBalanceResponse>;
}

export const BalanceService: IBalanceService;

export class BalanceClient extends grpc.Client {
  constructor(address: string, credentials: grpc.ChannelCredentials, options?: object);
  getNativeBalance(argument: balance_pb.GetNativeBalanceRequest, callback: grpc.requestCallback<balance_pb.GetNativeBalanceResponse>): grpc.ClientUnaryCall;
  getNativeBalance(argument: balance_pb.GetNativeBalanceRequest, metadataOrOptions: grpc.Metadata | grpc.CallOptions | null, callback: grpc.requestCallback<balance_pb.GetNativeBalanceResponse>): grpc.ClientUnaryCall;
  getNativeBalance(argument: balance_pb.GetNativeBalanceRequest, metadata: grpc.Metadata | null, options: grpc.CallOptions | null, callback: grpc.requestCallback<balance_pb.GetNativeBalanceResponse>): grpc.ClientUnaryCall;
  getHoprBalance(argument: balance_pb.GetHoprBalanceRequest, callback: grpc.requestCallback<balance_pb.GetHoprBalanceResponse>): grpc.ClientUnaryCall;
  getHoprBalance(argument: balance_pb.GetHoprBalanceRequest, metadataOrOptions: grpc.Metadata | grpc.CallOptions | null, callback: grpc.requestCallback<balance_pb.GetHoprBalanceResponse>): grpc.ClientUnaryCall;
  getHoprBalance(argument: balance_pb.GetHoprBalanceRequest, metadata: grpc.Metadata | null, options: grpc.CallOptions | null, callback: grpc.requestCallback<balance_pb.GetHoprBalanceResponse>): grpc.ClientUnaryCall;
}
