import assert from 'assert'
import { WebsocketProvider } from 'web3-core'
import Web3 from '.'
import { wait } from '../utils'
import { DEFAULT_URI } from '../config'

const waitForEvent = (emitter: Web3['events'], name: Parameters<Web3['events']['on']>[0]) => {
  return new Promise<void>((resolve) => {
    emitter.on(name, () => {
      resolve()
    })
  })
}

describe('test custom web3', function () {
  this.timeout(1e3 * 5)

  const web3 = new Web3(DEFAULT_URI)

  it('should connect and emit event', async function () {
    await wait(1e3)
    assert(await web3.isConnected(), 'check isConnected method')
  })

  it('should disconnect and emit event', async function () {
    await Promise.all([waitForEvent(web3.events, 'disconnected'), web3.disconnect()])
    assert(!(await web3.isConnected()), 'check disconnect method')
  })

  it('should reconnect and emit event', async function () {
    await Promise.all([waitForEvent(web3.events, 'connected'), web3.connect()])

    const provider = web3.currentProvider as WebsocketProvider
    await Promise.all([
      waitForEvent(web3.events, 'connected'),
      waitForEvent(web3.events, 'reconnected'),
      provider.disconnect(0, 'client disconnected'),
    ])

    assert(await web3.isConnected(), 'check isConnected method')
  })
})
