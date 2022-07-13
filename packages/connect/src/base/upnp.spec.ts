import { UpnpManager } from './upnp.js'
import assert from 'assert'

describe('test upnp', function () {
  let noUPnP = false
  let upnp: UpnpManager
  beforeEach(function () {
    upnp = new UpnpManager()

    // Return quickly if router does not support UPnP
    if (noUPnP) {
      return
    }

    upnp.beforeStart()
    upnp.start()
  })

  afterEach(async function () {
    // Return quickly if router does not support UPnP
    if (noUPnP) {
      return
    }
    await upnp.stop()
  })
  it('get externalIp', async function () {
    // Return quickly if router does not support UPnP
    if (noUPnP) {
      return
    }
    this.timeout(3e3)

    const result = await upnp.externalIp()

    if (result == undefined) {
      noUPnP = true
    } else {
      assert(result.match(/[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}/), `result must be an IP address`)
    }

    await upnp.stop()
  })

  it('map port', async function () {
    // Return quickly if router does not support UPnP
    if (noUPnP) {
      return
    }
    this.timeout(3e3)

    // const start = Date.now()
    await assert.doesNotReject(async () => await upnp.map(50124))
  })
})
