// package: address
// file: address.proto

var address_pb = require("./address_pb");
var grpc = require("@improbable-eng/grpc-web").grpc;

var Address = (function () {
  function Address() {}
  Address.serviceName = "address.Address";
  return Address;
}());

Address.GetNativeAddress = {
  methodName: "GetNativeAddress",
  service: Address,
  requestStream: false,
  responseStream: false,
  requestType: address_pb.GetNativeAddressRequest,
  responseType: address_pb.GetNativeAddressResponse
};

Address.GetHoprAddress = {
  methodName: "GetHoprAddress",
  service: Address,
  requestStream: false,
  responseStream: false,
  requestType: address_pb.GetHoprAddressRequest,
  responseType: address_pb.GetHoprAddressResponse
};

exports.Address = Address;

function AddressClient(serviceHost, options) {
  this.serviceHost = serviceHost;
  this.options = options || {};
}

AddressClient.prototype.getNativeAddress = function getNativeAddress(requestMessage, metadata, callback) {
  if (arguments.length === 2) {
    callback = arguments[1];
  }
  var client = grpc.unary(Address.GetNativeAddress, {
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

AddressClient.prototype.getHoprAddress = function getHoprAddress(requestMessage, metadata, callback) {
  if (arguments.length === 2) {
    callback = arguments[1];
  }
  var client = grpc.unary(Address.GetHoprAddress, {
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

exports.AddressClient = AddressClient;

