// GENERATED CODE -- DO NOT EDIT!

'use strict';
var grpc = require('grpc');
var address_pb = require('./address_pb.js');

function serialize_address_GetHoprAddressRequest(arg) {
  if (!(arg instanceof address_pb.GetHoprAddressRequest)) {
    throw new Error('Expected argument of type address.GetHoprAddressRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_address_GetHoprAddressRequest(buffer_arg) {
  return address_pb.GetHoprAddressRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_address_GetHoprAddressResponse(arg) {
  if (!(arg instanceof address_pb.GetHoprAddressResponse)) {
    throw new Error('Expected argument of type address.GetHoprAddressResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_address_GetHoprAddressResponse(buffer_arg) {
  return address_pb.GetHoprAddressResponse.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_address_GetNativeAddressRequest(arg) {
  if (!(arg instanceof address_pb.GetNativeAddressRequest)) {
    throw new Error('Expected argument of type address.GetNativeAddressRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_address_GetNativeAddressRequest(buffer_arg) {
  return address_pb.GetNativeAddressRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_address_GetNativeAddressResponse(arg) {
  if (!(arg instanceof address_pb.GetNativeAddressResponse)) {
    throw new Error('Expected argument of type address.GetNativeAddressResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_address_GetNativeAddressResponse(buffer_arg) {
  return address_pb.GetNativeAddressResponse.deserializeBinary(new Uint8Array(buffer_arg));
}


var AddressService = exports.AddressService = {
  // example: ethereum address
getNativeAddress: {
    path: '/address.Address/GetNativeAddress',
    requestStream: false,
    responseStream: false,
    requestType: address_pb.GetNativeAddressRequest,
    responseType: address_pb.GetNativeAddressResponse,
    requestSerialize: serialize_address_GetNativeAddressRequest,
    requestDeserialize: deserialize_address_GetNativeAddressRequest,
    responseSerialize: serialize_address_GetNativeAddressResponse,
    responseDeserialize: deserialize_address_GetNativeAddressResponse,
  },
  getHoprAddress: {
    path: '/address.Address/GetHoprAddress',
    requestStream: false,
    responseStream: false,
    requestType: address_pb.GetHoprAddressRequest,
    responseType: address_pb.GetHoprAddressResponse,
    requestSerialize: serialize_address_GetHoprAddressRequest,
    requestDeserialize: deserialize_address_GetHoprAddressRequest,
    responseSerialize: serialize_address_GetHoprAddressResponse,
    responseDeserialize: deserialize_address_GetHoprAddressResponse,
  },
};

exports.AddressClient = grpc.makeGenericClientConstructor(AddressService);
