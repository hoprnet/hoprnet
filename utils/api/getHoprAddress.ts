import { AddressPromiseClient } from "@hoprnet/hopr-protos/web/address_grpc_web_pb";
import { GetHoprAddressRequest } from "@hoprnet/hopr-protos/web/address_pb";
import { API_URL } from "../env";

export const getHoprAddress = async (): Promise<string> => {
  const client = new AddressPromiseClient(API_URL);

  return client
    .getHoprAddress(new GetHoprAddressRequest(), undefined)
    .then((res) => res.getAddress());
};
