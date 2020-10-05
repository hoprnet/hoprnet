// package: ping
// file: ping.proto

var ping_pb = require("./ping_pb");
var grpc = require("@improbable-eng/grpc-web").grpc;

var Ping = (function () {
  function Ping() {}
  Ping.serviceName = "ping.Ping";
  return Ping;
}());

Ping.GetPing = {
  methodName: "GetPing",
  service: Ping,
  requestStream: false,
  responseStream: false,
  requestType: ping_pb.PingRequest,
  responseType: ping_pb.PingResponse
};

exports.Ping = Ping;

function PingClient(serviceHost, options) {
  this.serviceHost = serviceHost;
  this.options = options || {};
}

PingClient.prototype.getPing = function getPing(requestMessage, metadata, callback) {
  if (arguments.length === 2) {
    callback = arguments[1];
  }
  var client = grpc.unary(Ping.GetPing, {
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

exports.PingClient = PingClient;

