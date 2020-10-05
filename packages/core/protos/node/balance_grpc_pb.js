// GENERATED CODE -- DO NOT EDIT!

'use strict';
var grpc = require('grpc');
var balance_pb = require('./balance_pb.js');

function serialize_balance_GetHoprBalanceRequest(arg) {
  if (!(arg instanceof balance_pb.GetHoprBalanceRequest)) {
    throw new Error('Expected argument of type balance.GetHoprBalanceRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_balance_GetHoprBalanceRequest(buffer_arg) {
  return balance_pb.GetHoprBalanceRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_balance_GetHoprBalanceResponse(arg) {
  if (!(arg instanceof balance_pb.GetHoprBalanceResponse)) {
    throw new Error('Expected argument of type balance.GetHoprBalanceResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_balance_GetHoprBalanceResponse(buffer_arg) {
  return balance_pb.GetHoprBalanceResponse.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_balance_GetNativeBalanceRequest(arg) {
  if (!(arg instanceof balance_pb.GetNativeBalanceRequest)) {
    throw new Error('Expected argument of type balance.GetNativeBalanceRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_balance_GetNativeBalanceRequest(buffer_arg) {
  return balance_pb.GetNativeBalanceRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_balance_GetNativeBalanceResponse(arg) {
  if (!(arg instanceof balance_pb.GetNativeBalanceResponse)) {
    throw new Error('Expected argument of type balance.GetNativeBalanceResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_balance_GetNativeBalanceResponse(buffer_arg) {
  return balance_pb.GetNativeBalanceResponse.deserializeBinary(new Uint8Array(buffer_arg));
}


var BalanceService = exports.BalanceService = {
  // example: ETHER
getNativeBalance: {
    path: '/balance.Balance/GetNativeBalance',
    requestStream: false,
    responseStream: false,
    requestType: balance_pb.GetNativeBalanceRequest,
    responseType: balance_pb.GetNativeBalanceResponse,
    requestSerialize: serialize_balance_GetNativeBalanceRequest,
    requestDeserialize: deserialize_balance_GetNativeBalanceRequest,
    responseSerialize: serialize_balance_GetNativeBalanceResponse,
    responseDeserialize: deserialize_balance_GetNativeBalanceResponse,
  },
  getHoprBalance: {
    path: '/balance.Balance/GetHoprBalance',
    requestStream: false,
    responseStream: false,
    requestType: balance_pb.GetHoprBalanceRequest,
    responseType: balance_pb.GetHoprBalanceResponse,
    requestSerialize: serialize_balance_GetHoprBalanceRequest,
    requestDeserialize: deserialize_balance_GetHoprBalanceRequest,
    responseSerialize: serialize_balance_GetHoprBalanceResponse,
    responseDeserialize: deserialize_balance_GetHoprBalanceResponse,
  },
};

exports.BalanceClient = grpc.makeGenericClientConstructor(BalanceService);
