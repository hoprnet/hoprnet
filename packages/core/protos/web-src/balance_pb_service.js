// package: balance
// file: balance.proto

var balance_pb = require("./balance_pb");
var grpc = require("@improbable-eng/grpc-web").grpc;

var Balance = (function () {
  function Balance() {}
  Balance.serviceName = "balance.Balance";
  return Balance;
}());

Balance.GetNativeBalance = {
  methodName: "GetNativeBalance",
  service: Balance,
  requestStream: false,
  responseStream: false,
  requestType: balance_pb.GetNativeBalanceRequest,
  responseType: balance_pb.GetNativeBalanceResponse
};

Balance.GetHoprBalance = {
  methodName: "GetHoprBalance",
  service: Balance,
  requestStream: false,
  responseStream: false,
  requestType: balance_pb.GetHoprBalanceRequest,
  responseType: balance_pb.GetHoprBalanceResponse
};

exports.Balance = Balance;

function BalanceClient(serviceHost, options) {
  this.serviceHost = serviceHost;
  this.options = options || {};
}

BalanceClient.prototype.getNativeBalance = function getNativeBalance(requestMessage, metadata, callback) {
  if (arguments.length === 2) {
    callback = arguments[1];
  }
  var client = grpc.unary(Balance.GetNativeBalance, {
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

BalanceClient.prototype.getHoprBalance = function getHoprBalance(requestMessage, metadata, callback) {
  if (arguments.length === 2) {
    callback = arguments[1];
  }
  var client = grpc.unary(Balance.GetHoprBalance, {
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

exports.BalanceClient = BalanceClient;

