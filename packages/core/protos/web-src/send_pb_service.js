// package: send
// file: send.proto

var send_pb = require("./send_pb");
var grpc = require("@improbable-eng/grpc-web").grpc;

var Send = (function () {
  function Send() {}
  Send.serviceName = "send.Send";
  return Send;
}());

Send.Send = {
  methodName: "Send",
  service: Send,
  requestStream: false,
  responseStream: false,
  requestType: send_pb.SendRequest,
  responseType: send_pb.SendResponse
};

exports.Send = Send;

function SendClient(serviceHost, options) {
  this.serviceHost = serviceHost;
  this.options = options || {};
}

SendClient.prototype.send = function send(requestMessage, metadata, callback) {
  if (arguments.length === 2) {
    callback = arguments[1];
  }
  var client = grpc.unary(Send.Send, {
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

exports.SendClient = SendClient;

