import { dial } from './dialHelper'
import PeerId from 'peer-id'
import { defer } from '../async'

describe('test dialing', function () {
  it('foo', async function () {
    const neverEnding = defer<void>()

    const destination = await PeerId.create({ keyType: 'secp256k1' })
    await dial(
      {
        dialProtocol: () => {
          return neverEnding
        }
      } as any,
      destination,
      'foo',
      {
        timeout: 1e3
      }
    )
  })
})
