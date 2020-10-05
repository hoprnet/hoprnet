// GENERATED CODE -- DO NOT EDIT!

'use strict';
var grpc = require('grpc');
var channels_pb = require('./channels_pb.js');

function serialize_channels_CloseChannelRequest(arg) {
  if (!(arg instanceof channels_pb.CloseChannelRequest)) {
    throw new Error('Expected argument of type channels.CloseChannelRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_channels_CloseChannelRequest(buffer_arg) {
  return channels_pb.CloseChannelRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_channels_CloseChannelResponse(arg) {
  if (!(arg instanceof channels_pb.CloseChannelResponse)) {
    throw new Error('Expected argument of type channels.CloseChannelResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_channels_CloseChannelResponse(buffer_arg) {
  return channels_pb.CloseChannelResponse.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_channels_GetChannelDataRequest(arg) {
  if (!(arg instanceof channels_pb.GetChannelDataRequest)) {
    throw new Error('Expected argument of type channels.GetChannelDataRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_channels_GetChannelDataRequest(buffer_arg) {
  return channels_pb.GetChannelDataRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_channels_GetChannelDataResponse(arg) {
  if (!(arg instanceof channels_pb.GetChannelDataResponse)) {
    throw new Error('Expected argument of type channels.GetChannelDataResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_channels_GetChannelDataResponse(buffer_arg) {
  return channels_pb.GetChannelDataResponse.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_channels_GetChannelsRequest(arg) {
  if (!(arg instanceof channels_pb.GetChannelsRequest)) {
    throw new Error('Expected argument of type channels.GetChannelsRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_channels_GetChannelsRequest(buffer_arg) {
  return channels_pb.GetChannelsRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_channels_GetChannelsResponse(arg) {
  if (!(arg instanceof channels_pb.GetChannelsResponse)) {
    throw new Error('Expected argument of type channels.GetChannelsResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_channels_GetChannelsResponse(buffer_arg) {
  return channels_pb.GetChannelsResponse.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_channels_OpenChannelRequest(arg) {
  if (!(arg instanceof channels_pb.OpenChannelRequest)) {
    throw new Error('Expected argument of type channels.OpenChannelRequest');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_channels_OpenChannelRequest(buffer_arg) {
  return channels_pb.OpenChannelRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_channels_OpenChannelResponse(arg) {
  if (!(arg instanceof channels_pb.OpenChannelResponse)) {
    throw new Error('Expected argument of type channels.OpenChannelResponse');
  }
  return Buffer.from(arg.serializeBinary());
}

function deserialize_channels_OpenChannelResponse(buffer_arg) {
  return channels_pb.OpenChannelResponse.deserializeBinary(new Uint8Array(buffer_arg));
}


var ChannelsService = exports.ChannelsService = {
  getChannels: {
    path: '/channels.Channels/GetChannels',
    requestStream: false,
    responseStream: false,
    requestType: channels_pb.GetChannelsRequest,
    responseType: channels_pb.GetChannelsResponse,
    requestSerialize: serialize_channels_GetChannelsRequest,
    requestDeserialize: deserialize_channels_GetChannelsRequest,
    responseSerialize: serialize_channels_GetChannelsResponse,
    responseDeserialize: deserialize_channels_GetChannelsResponse,
  },
  // unable to name this 'GetChannel' because it's already used by the stub
getChannelData: {
    path: '/channels.Channels/GetChannelData',
    requestStream: false,
    responseStream: false,
    requestType: channels_pb.GetChannelDataRequest,
    responseType: channels_pb.GetChannelDataResponse,
    requestSerialize: serialize_channels_GetChannelDataRequest,
    requestDeserialize: deserialize_channels_GetChannelDataRequest,
    responseSerialize: serialize_channels_GetChannelDataResponse,
    responseDeserialize: deserialize_channels_GetChannelDataResponse,
  },
  openChannel: {
    path: '/channels.Channels/OpenChannel',
    requestStream: false,
    responseStream: false,
    requestType: channels_pb.OpenChannelRequest,
    responseType: channels_pb.OpenChannelResponse,
    requestSerialize: serialize_channels_OpenChannelRequest,
    requestDeserialize: deserialize_channels_OpenChannelRequest,
    responseSerialize: serialize_channels_OpenChannelResponse,
    responseDeserialize: deserialize_channels_OpenChannelResponse,
  },
  closeChannel: {
    path: '/channels.Channels/CloseChannel',
    requestStream: false,
    responseStream: false,
    requestType: channels_pb.CloseChannelRequest,
    responseType: channels_pb.CloseChannelResponse,
    requestSerialize: serialize_channels_CloseChannelRequest,
    requestDeserialize: deserialize_channels_CloseChannelRequest,
    responseSerialize: serialize_channels_CloseChannelResponse,
    responseDeserialize: deserialize_channels_CloseChannelResponse,
  },
};

exports.ChannelsClient = grpc.makeGenericClientConstructor(ChannelsService);
