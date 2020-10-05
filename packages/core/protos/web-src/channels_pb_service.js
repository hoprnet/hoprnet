// package: channels
// file: channels.proto

var channels_pb = require("./channels_pb");
var grpc = require("@improbable-eng/grpc-web").grpc;

var Channels = (function () {
  function Channels() {}
  Channels.serviceName = "channels.Channels";
  return Channels;
}());

Channels.GetChannels = {
  methodName: "GetChannels",
  service: Channels,
  requestStream: false,
  responseStream: false,
  requestType: channels_pb.GetChannelsRequest,
  responseType: channels_pb.GetChannelsResponse
};

Channels.GetChannelData = {
  methodName: "GetChannelData",
  service: Channels,
  requestStream: false,
  responseStream: false,
  requestType: channels_pb.GetChannelDataRequest,
  responseType: channels_pb.GetChannelDataResponse
};

Channels.OpenChannel = {
  methodName: "OpenChannel",
  service: Channels,
  requestStream: false,
  responseStream: false,
  requestType: channels_pb.OpenChannelRequest,
  responseType: channels_pb.OpenChannelResponse
};

Channels.CloseChannel = {
  methodName: "CloseChannel",
  service: Channels,
  requestStream: false,
  responseStream: false,
  requestType: channels_pb.CloseChannelRequest,
  responseType: channels_pb.CloseChannelResponse
};

exports.Channels = Channels;

function ChannelsClient(serviceHost, options) {
  this.serviceHost = serviceHost;
  this.options = options || {};
}

ChannelsClient.prototype.getChannels = function getChannels(requestMessage, metadata, callback) {
  if (arguments.length === 2) {
    callback = arguments[1];
  }
  var client = grpc.unary(Channels.GetChannels, {
    request: requestMessage,
    host: this.serviceHost,
    metadata: metadata,
    transport: this.options.transport,
    debug: this.options.debug,
    onEnd: function (response) {
      if (callback) {
        if (response.status !== grpc.Code.OK) {
          var err = new Error(response.statusMessage);
          err.code = response.status;
          err.metadata = response.trailers;
          callback(err, null);
        } else {
          callback(null, response.message);
        }
      }
    }
  });
  return {
    cancel: function () {
      callback = null;
      client.close();
    }
  };
};

ChannelsClient.prototype.getChannelData = function getChannelData(requestMessage, metadata, callback) {
  if (arguments.length === 2) {
    callback = arguments[1];
  }
  var client = grpc.unary(Channels.GetChannelData, {
    request: requestMessage,
    host: this.serviceHost,
    metadata: metadata,
    transport: this.options.transport,
    debug: this.options.debug,
    onEnd: function (response) {
      if (callback) {
        if (response.status !== grpc.Code.OK) {
          var err = new Error(response.statusMessage);
          err.code = response.status;
          err.metadata = response.trailers;
          callback(err, null);
        } else {
          callback(null, response.message);
        }
      }
    }
  });
  return {
    cancel: function () {
      callback = null;
      client.close();
    }
  };
};

ChannelsClient.prototype.openChannel = function openChannel(requestMessage, metadata, callback) {
  if (arguments.length === 2) {
    callback = arguments[1];
  }
  var client = grpc.unary(Channels.OpenChannel, {
    request: requestMessage,
    host: this.serviceHost,
    metadata: metadata,
    transport: this.options.transport,
    debug: this.options.debug,
    onEnd: function (response) {
      if (callback) {
        if (response.status !== grpc.Code.OK) {
          var err = new Error(response.statusMessage);
          err.code = response.status;
          err.metadata = response.trailers;
          callback(err, null);
        } else {
          callback(null, response.message);
        }
      }
    }
  });
  return {
    cancel: function () {
      callback = null;
      client.close();
    }
  };
};

ChannelsClient.prototype.closeChannel = function closeChannel(requestMessage, metadata, callback) {
  if (arguments.length === 2) {
    callback = arguments[1];
  }
  var client = grpc.unary(Channels.CloseChannel, {
    request: requestMessage,
    host: this.serviceHost,
    metadata: metadata,
    transport: this.options.transport,
    debug: this.options.debug,
    onEnd: function (response) {
      if (callback) {
        if (response.status !== grpc.Code.OK) {
          var err = new Error(response.statusMessage);
          err.code = response.status;
          err.metadata = response.trailers;
          callback(err, null);
        } else {
          callback(null, response.message);
        }
      }
    }
  });
  return {
    cancel: function () {
      callback = null;
      client.close();
    }
  };
};

exports.ChannelsClient = ChannelsClient;

