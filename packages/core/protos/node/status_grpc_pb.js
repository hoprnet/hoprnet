// GENERATED CODE -- DO NOT EDIT!

'use strict';
var grpc = require('grpc');
var status_pb = require('./status_pb.js');

function serialize_status_StatusRequest(arg) {
  if (!(arg instanceof status_pb.StatusRequest)) {
    throw new Error('Expected argument of type status.StatusRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_status_StatusRequest(buffer_arg) {
  return status_pb.StatusRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_status_StatusResponse(arg) {
  if (!(arg instanceof status_pb.StatusResponse)) {
    throw new Error('Expected argument of type status.StatusResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_status_StatusResponse(buffer_arg) {
  return status_pb.StatusResponse.deserializeBinary(new Uint8Array(buffer_arg));
}


var StatusService = exports.StatusService = {
  getStatus: {
    path: '/status.Status/GetStatus',
    requestStream: false,
    responseStream: false,
    requestType: status_pb.StatusRequest,
    responseType: status_pb.StatusResponse,
    requestSerialize: serialize_status_StatusRequest,
    requestDeserialize: deserialize_status_StatusRequest,
    responseSerialize: serialize_status_StatusResponse,
    responseDeserialize: deserialize_status_StatusResponse,
  },
};

exports.StatusClient = grpc.makeGenericClientConstructor(StatusService);
