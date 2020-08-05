// import {  } from "grpc-web"
import { GetHoprAddressRequest } from '@hoprnet/hopr-protos/web/address_pb'
import { AddressPromiseClient } from '@hoprnet/hopr-protos/web/address_grpc_web_pb'

// export const SetupClient = <T extends typeof grpc.Client>(Client: T): InstanceType<T> => {
//   return (new Client(API_URL, grpc.credentials.createInsecure()) as unknown) as InstanceType<T>
// }

export const getAddress = () => {
  const client = new AddressPromiseClient('http://127.0.0.1:8080')

  client
    .getHoprAddress(new GetHoprAddressRequest(), undefined)
    .then((res) => console.log(res.getAddress()))
    .catch(console.error)
}
