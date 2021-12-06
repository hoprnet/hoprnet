import { dbMock, debug, privKeyToPeerId, PublicKey } from '@hoprnet/hopr-utils'
import HoprEthereum from '.'
import { sampleChainOptions } from './ethereum.mock'

const namespace = 'hopr:test:hopr-ethereum'
const log = debug(namespace)

const privateKey = '0xcb1e5d91d46eb54a477a7eefec9c87a1575e3e5384d38f990f19c09aa8ddd332'
const mockPeerId = privKeyToPeerId(privateKey)

describe(`HoprEthereum`, function () {
  it('should instantiate a new class w/o any issues', function () {
    log('starting new instance of HoprEthereum.')
    new HoprEthereum(dbMock, PublicKey.fromPeerId(mockPeerId), mockPeerId.privKey.marshal(), sampleChainOptions)
    log('successfully created the HoprEthereum instance.')
  })
})
