// package: ping
// file: settings.proto

var settings_pb = require("./settings_pb");
var grpc = require("@improbable-eng/grpc-web").grpc;

var Settings = (function () {
  function Settings() {}
  Settings.serviceName = "ping.Settings";
  return Settings;
}());

Settings.UpdateSettings = {
  methodName: "UpdateSettings",
  service: Settings,
  requestStream: false,
  responseStream: false,
  requestType: settings_pb.UpdateSettingsRequest,
  responseType: settings_pb.UpdateSettingsResponse
};

exports.Settings = Settings;

function SettingsClient(serviceHost, options) {
  this.serviceHost = serviceHost;
  this.options = options || {};
}

SettingsClient.prototype.updateSettings = function updateSettings(requestMessage, metadata, callback) {
  if (arguments.length === 2) {
    callback = arguments[1];
  }
  var client = grpc.unary(Settings.UpdateSettings, {
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

exports.SettingsClient = SettingsClient;

