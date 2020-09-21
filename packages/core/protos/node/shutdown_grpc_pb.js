// GENERATED CODE -- DO NOT EDIT!

'use strict';
var grpc = require('grpc');
var shutdown_pb = require('./shutdown_pb.js');

function serialize_shutdown_ShutdownRequest(arg) {
  if (!(arg instanceof shutdown_pb.ShutdownRequest)) {
    throw new Error('Expected argument of type shutdown.ShutdownRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_shutdown_ShutdownRequest(buffer_arg) {
  return shutdown_pb.ShutdownRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_shutdown_ShutdownResponse(arg) {
  if (!(arg instanceof shutdown_pb.ShutdownResponse)) {
    throw new Error('Expected argument of type shutdown.ShutdownResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_shutdown_ShutdownResponse(buffer_arg) {
  return shutdown_pb.ShutdownResponse.deserializeBinary(new Uint8Array(buffer_arg));
}


var ShutdownService = exports.ShutdownService = {
  shutdown: {
    path: '/shutdown.Shutdown/Shutdown',
    requestStream: false,
    responseStream: false,
    requestType: shutdown_pb.ShutdownRequest,
    responseType: shutdown_pb.ShutdownResponse,
    requestSerialize: serialize_shutdown_ShutdownRequest,
    requestDeserialize: deserialize_shutdown_ShutdownRequest,
    responseSerialize: serialize_shutdown_ShutdownResponse,
    responseDeserialize: deserialize_shutdown_ShutdownResponse,
  },
};

exports.ShutdownClient = grpc.makeGenericClientConstructor(ShutdownService);
