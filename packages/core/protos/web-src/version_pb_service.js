// package: version
// file: version.proto

var version_pb = require("./version_pb");
var grpc = require("@improbable-eng/grpc-web").grpc;

var Version = (function () {
  function Version() {}
  Version.serviceName = "version.Version";
  return Version;
}());

Version.GetVersion = {
  methodName: "GetVersion",
  service: Version,
  requestStream: false,
  responseStream: false,
  requestType: version_pb.VersionRequest,
  responseType: version_pb.VersionResponse
};

exports.Version = Version;

function VersionClient(serviceHost, options) {
  this.serviceHost = serviceHost;
  this.options = options || {};
}

VersionClient.prototype.getVersion = function getVersion(requestMessage, metadata, callback) {
  if (arguments.length === 2) {
    callback = arguments[1];
  }
  var client = grpc.unary(Version.GetVersion, {
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

exports.VersionClient = VersionClient;

