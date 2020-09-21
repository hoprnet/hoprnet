// GENERATED CODE -- DO NOT EDIT!

'use strict';
var grpc = require('grpc');
var listen_pb = require('./listen_pb.js');

function serialize_listen_ListenRequest(arg) {
  if (!(arg instanceof listen_pb.ListenRequest)) {
    throw new Error('Expected argument of type listen.ListenRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_listen_ListenRequest(buffer_arg) {
  return listen_pb.ListenRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_listen_ListenResponse(arg) {
  if (!(arg instanceof listen_pb.ListenResponse)) {
    throw new Error('Expected argument of type listen.ListenResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_listen_ListenResponse(buffer_arg) {
  return listen_pb.ListenResponse.deserializeBinary(new Uint8Array(buffer_arg));
}


var ListenService = exports.ListenService = {
  listen: {
    path: '/listen.Listen/Listen',
    requestStream: false,
    responseStream: true,
    requestType: listen_pb.ListenRequest,
    responseType: listen_pb.ListenResponse,
    requestSerialize: serialize_listen_ListenRequest,
    requestDeserialize: deserialize_listen_ListenRequest,
    responseSerialize: serialize_listen_ListenResponse,
    responseDeserialize: deserialize_listen_ListenResponse,
  },
};

exports.ListenClient = grpc.makeGenericClientConstructor(ListenService);
