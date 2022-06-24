import { Multiaddr } from '@multiformats/multiaddr'
import assert from 'assert'

import { maToClass, AddressClass, compareAddressesPublicMode, compareAddressesLocalMode } from './addressSorters.js'

const PUBLIC_ADDRESS = new Multiaddr(
  `/ip4/84.148.73.225/tcp/62492/p2p/16Uiu2HAm85aCSXNVxwQPBsfHm2hZEvNRmYxvfBhHSQgNgKyKBnWG`
)
const CIRCUIT_ADDRESS_1 = new Multiaddr(
  `/p2p/16Uiu2HAkxr5N4BJRXeL4zY7kLSfkQTQ4dcTvGZ4pZKqZ6frRdtAq/p2p-circuit/p2p/16Uiu2HAm85aCSXNVxwQPBsfHm2hZEvNRmYxvfBhHSQgNgKyKBnWG`
)
const CIRCUIT_ADDRESS_2 = new Multiaddr(
  `/p2p/16Uiu2HAkzVnLLd8HzqhqHY1j7P4g3n6kX6FSb23YeB7xeiyEqdaa/p2p-circuit/p2p/16Uiu2HAm85aCSXNVxwQPBsfHm2hZEvNRmYxvfBhHSQgNgKyKBnWG`
)
const LOCAL_B_ADDRESS = new Multiaddr(
  `/ip4/172.17.0.3/tcp/12033/p2p/16Uiu2HAm85aCSXNVxwQPBsfHm2hZEvNRmYxvfBhHSQgNgKyKBnWG`
)
const LOOPBACK_ADDRESS = new Multiaddr(
  `/ip4/127.0.0.1/tcp/12033/p2p/16Uiu2HAm85aCSXNVxwQPBsfHm2hZEvNRmYxvfBhHSQgNgKyKBnWG`
)

describe('test address sorting', function () {
  it('classification', function () {
    assert(maToClass(PUBLIC_ADDRESS) == AddressClass.Public)
    assert(maToClass(CIRCUIT_ADDRESS_1) == AddressClass.Circuit)
    assert(maToClass(CIRCUIT_ADDRESS_2) == AddressClass.Circuit)
    assert(maToClass(LOCAL_B_ADDRESS) == AddressClass.PrivateB)
    assert(maToClass(LOOPBACK_ADDRESS) == AddressClass.Loopback)
  })

  it('sort', function () {
    const addrs = [PUBLIC_ADDRESS, CIRCUIT_ADDRESS_1, LOCAL_B_ADDRESS, LOOPBACK_ADDRESS]

    const addrsPublicOrder = [PUBLIC_ADDRESS, CIRCUIT_ADDRESS_1, LOCAL_B_ADDRESS, LOOPBACK_ADDRESS]

    const addrsLocalOrder = [LOOPBACK_ADDRESS, LOCAL_B_ADDRESS, CIRCUIT_ADDRESS_1, PUBLIC_ADDRESS]

    assert(
      addrs
        .sort(compareAddressesPublicMode)
        .every((addr: Multiaddr, index: number) => addr.equals(addrsPublicOrder[index]))
    )

    assert(
      addrs
        .sort(compareAddressesLocalMode)
        .every((addr: Multiaddr, index: number) => addr.equals(addrsLocalOrder[index]))
    )
  })
})
