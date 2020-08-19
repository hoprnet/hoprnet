// package: status
// file: status.proto

var status_pb = require("./status_pb");
var grpc = require("@improbable-eng/grpc-web").grpc;

var Status = (function () {
  function Status() {}
  Status.serviceName = "status.Status";
  return Status;
}());

Status.GetStatus = {
  methodName: "GetStatus",
  service: Status,
  requestStream: false,
  responseStream: false,
  requestType: status_pb.StatusRequest,
  responseType: status_pb.StatusResponse
};

exports.Status = Status;

function StatusClient(serviceHost, options) {
  this.serviceHost = serviceHost;
  this.options = options || {};
}

StatusClient.prototype.getStatus = function getStatus(requestMessage, metadata, callback) {
  if (arguments.length === 2) {
    callback = arguments[1];
  }
  var client = grpc.unary(Status.GetStatus, {
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

exports.StatusClient = StatusClient;

