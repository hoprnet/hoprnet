import { UpnpManager } from './upnp'
import assert from 'assert'
import type NatApi from 'nat-api'

class TestingUpnpManager extends UpnpManager {
  // @ts-ignore
  public client: NatApi
}
describe('test upnp', function () {
  let noUPnP = false
  it('get externalIp', async function () {
    // If the router does not support UPnP, the unit
    // awaits the timeout
    if (noUPnP) {
      this.skip()
    }
    this.timeout(3e3)
    const upnp = new TestingUpnpManager()

    const DEFAULT_TIMEOUT = upnp.client._upnpClient.timeout

    assert(DEFAULT_TIMEOUT != undefined, `nat-api implementation has changed!`)

    const start = Date.now()
    const result = await upnp.externalIp()

    if (result == undefined) {
      assert(Date.now() - start >= DEFAULT_TIMEOUT)
      noUPnP = true
    } else {
      assert(result.match(/[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}/), `result must be an IP address`)
    }

    await upnp.stop()
  })

  it('map port', async function () {
    // If the router does not support UPnP, the unit
    // awaits the timeout
    if (noUPnP) {
      this.skip()
    }
    this.timeout(3e3)
    const upnp = new TestingUpnpManager()

    // const start = Date.now()
    await assert.doesNotReject(async () => await upnp.map(50124))

    await upnp.stop()
  })
})
