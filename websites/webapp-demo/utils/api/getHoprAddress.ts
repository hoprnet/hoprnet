import { AddressPromiseClient } from '@hoprnet/hopr-protos/web/address_grpc_web_pb'
import { GetHoprAddressRequest } from '@hoprnet/hopr-protos/web/address_pb'

export const getHoprAddress = async (apiUrl: string): Promise<string> => {
  const client = new AddressPromiseClient(apiUrl)

  return client.getHoprAddress(new GetHoprAddressRequest(), undefined).then((res) => res.getAddress())
}
