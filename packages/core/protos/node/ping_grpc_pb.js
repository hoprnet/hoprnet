// GENERATED CODE -- DO NOT EDIT!

'use strict';
var grpc = require('grpc');
var ping_pb = require('./ping_pb.js');

function serialize_ping_PingRequest(arg) {
  if (!(arg instanceof ping_pb.PingRequest)) {
    throw new Error('Expected argument of type ping.PingRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_ping_PingRequest(buffer_arg) {
  return ping_pb.PingRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_ping_PingResponse(arg) {
  if (!(arg instanceof ping_pb.PingResponse)) {
    throw new Error('Expected argument of type ping.PingResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_ping_PingResponse(buffer_arg) {
  return ping_pb.PingResponse.deserializeBinary(new Uint8Array(buffer_arg));
}


var PingService = exports.PingService = {
  getPing: {
    path: '/ping.Ping/GetPing',
    requestStream: false,
    responseStream: false,
    requestType: ping_pb.PingRequest,
    responseType: ping_pb.PingResponse,
    requestSerialize: serialize_ping_PingRequest,
    requestDeserialize: deserialize_ping_PingRequest,
    responseSerialize: serialize_ping_PingResponse,
    responseDeserialize: deserialize_ping_PingResponse,
  },
};

exports.PingClient = grpc.makeGenericClientConstructor(PingService);
