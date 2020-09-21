// GENERATED CODE -- DO NOT EDIT!

'use strict';
var grpc = require('grpc');
var version_pb = require('./version_pb.js');

function serialize_version_VersionRequest(arg) {
  if (!(arg instanceof version_pb.VersionRequest)) {
    throw new Error('Expected argument of type version.VersionRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_version_VersionRequest(buffer_arg) {
  return version_pb.VersionRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_version_VersionResponse(arg) {
  if (!(arg instanceof version_pb.VersionResponse)) {
    throw new Error('Expected argument of type version.VersionResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_version_VersionResponse(buffer_arg) {
  return version_pb.VersionResponse.deserializeBinary(new Uint8Array(buffer_arg));
}


var VersionService = exports.VersionService = {
  getVersion: {
    path: '/version.Version/GetVersion',
    requestStream: false,
    responseStream: false,
    requestType: version_pb.VersionRequest,
    responseType: version_pb.VersionResponse,
    requestSerialize: serialize_version_VersionRequest,
    requestDeserialize: deserialize_version_VersionRequest,
    responseSerialize: serialize_version_VersionResponse,
    responseDeserialize: deserialize_version_VersionResponse,
  },
};

exports.VersionClient = grpc.makeGenericClientConstructor(VersionService);
