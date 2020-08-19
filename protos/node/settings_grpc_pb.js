// GENERATED CODE -- DO NOT EDIT!

'use strict';
var grpc = require('grpc');
var settings_pb = require('./settings_pb.js');

function serialize_ping_UpdateSettingsRequest(arg) {
  if (!(arg instanceof settings_pb.UpdateSettingsRequest)) {
    throw new Error('Expected argument of type ping.UpdateSettingsRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_ping_UpdateSettingsRequest(buffer_arg) {
  return settings_pb.UpdateSettingsRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_ping_UpdateSettingsResponse(arg) {
  if (!(arg instanceof settings_pb.UpdateSettingsResponse)) {
    throw new Error('Expected argument of type ping.UpdateSettingsResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_ping_UpdateSettingsResponse(buffer_arg) {
  return settings_pb.UpdateSettingsResponse.deserializeBinary(new Uint8Array(buffer_arg));
}


var SettingsService = exports.SettingsService = {
  // update setting on the fly without requiring a restart
updateSettings: {
    path: '/ping.Settings/UpdateSettings',
    requestStream: false,
    responseStream: false,
    requestType: settings_pb.UpdateSettingsRequest,
    responseType: settings_pb.UpdateSettingsResponse,
    requestSerialize: serialize_ping_UpdateSettingsRequest,
    requestDeserialize: deserialize_ping_UpdateSettingsRequest,
    responseSerialize: serialize_ping_UpdateSettingsResponse,
    responseDeserialize: deserialize_ping_UpdateSettingsResponse,
  },
};

exports.SettingsClient = grpc.makeGenericClientConstructor(SettingsService);
