// package: shutdown
// file: shutdown.proto

var shutdown_pb = require("./shutdown_pb");
var grpc = require("@improbable-eng/grpc-web").grpc;

var Shutdown = (function () {
  function Shutdown() {}
  Shutdown.serviceName = "shutdown.Shutdown";
  return Shutdown;
}());

Shutdown.Shutdown = {
  methodName: "Shutdown",
  service: Shutdown,
  requestStream: false,
  responseStream: false,
  requestType: shutdown_pb.ShutdownRequest,
  responseType: shutdown_pb.ShutdownResponse
};

exports.Shutdown = Shutdown;

function ShutdownClient(serviceHost, options) {
  this.serviceHost = serviceHost;
  this.options = options || {};
}

ShutdownClient.prototype.shutdown = function shutdown(requestMessage, metadata, callback) {
  if (arguments.length === 2) {
    callback = arguments[1];
  }
  var client = grpc.unary(Shutdown.Shutdown, {
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

exports.ShutdownClient = ShutdownClient;

