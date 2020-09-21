// package: listen
// file: listen.proto

var listen_pb = require("./listen_pb");
var grpc = require("@improbable-eng/grpc-web").grpc;

var Listen = (function () {
  function Listen() {}
  Listen.serviceName = "listen.Listen";
  return Listen;
}());

Listen.Listen = {
  methodName: "Listen",
  service: Listen,
  requestStream: false,
  responseStream: true,
  requestType: listen_pb.ListenRequest,
  responseType: listen_pb.ListenResponse
};

exports.Listen = Listen;

function ListenClient(serviceHost, options) {
  this.serviceHost = serviceHost;
  this.options = options || {};
}

ListenClient.prototype.listen = function listen(requestMessage, metadata) {
  var listeners = {
    data: [],
    end: [],
    status: []
  };
  var client = grpc.invoke(Listen.Listen, {
    request: requestMessage,
    host: this.serviceHost,
    metadata: metadata,
    transport: this.options.transport,
    debug: this.options.debug,
    onMessage: function (responseMessage) {
      listeners.data.forEach(function (handler) {
        handler(responseMessage);
      });
    },
    onEnd: function (status, statusMessage, trailers) {
      listeners.status.forEach(function (handler) {
        handler({ code: status, details: statusMessage, metadata: trailers });
      });
      listeners.end.forEach(function (handler) {
        handler({ code: status, details: statusMessage, metadata: trailers });
      });
      listeners = null;
    }
  });
  return {
    on: function (type, handler) {
      listeners[type].push(handler);
      return this;
    },
    cancel: function () {
      listeners = null;
      client.close();
    }
  };
};

exports.ListenClient = ListenClient;

