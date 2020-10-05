// GENERATED CODE -- DO NOT EDIT!

'use strict';
var grpc = require('grpc');
var send_pb = require('./send_pb.js');

function serialize_send_SendRequest(arg) {
  if (!(arg instanceof send_pb.SendRequest)) {
    throw new Error('Expected argument of type send.SendRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_send_SendRequest(buffer_arg) {
  return send_pb.SendRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_send_SendResponse(arg) {
  if (!(arg instanceof send_pb.SendResponse)) {
    throw new Error('Expected argument of type send.SendResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_send_SendResponse(buffer_arg) {
  return send_pb.SendResponse.deserializeBinary(new Uint8Array(buffer_arg));
}


var SendService = exports.SendService = {
  send: {
    path: '/send.Send/Send',
    requestStream: false,
    responseStream: false,
    requestType: send_pb.SendRequest,
    responseType: send_pb.SendResponse,
    requestSerialize: serialize_send_SendRequest,
    requestDeserialize: deserialize_send_SendRequest,
    responseSerialize: serialize_send_SendResponse,
    responseDeserialize: deserialize_send_SendResponse,
  },
};

exports.SendClient = grpc.makeGenericClientConstructor(SendService);
