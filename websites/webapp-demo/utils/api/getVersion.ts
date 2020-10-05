import { VersionPromiseClient } from '@hoprnet/hopr-protos/web/version_grpc_web_pb'
import { VersionRequest } from '@hoprnet/hopr-protos/web/version_pb'

export const getVersion = async (apiUrl: string): Promise<string> => {
  const client = new VersionPromiseClient(apiUrl)

  return client.getVersion(new VersionRequest(), undefined).then((res) => res.getVersion())
}
