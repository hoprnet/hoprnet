import { VersionPromiseClient } from "@hoprnet/hopr-protos/web/version_grpc_web_pb";
import { VersionRequest } from "@hoprnet/hopr-protos/web/version_pb";
import { API_URL } from "../env";

export const getVersion = async (): Promise<string> => {
  const client = new VersionPromiseClient(API_URL);

  return client
    .getVersion(new VersionRequest(), undefined)
    .then((res) => res.getVersion());
};
