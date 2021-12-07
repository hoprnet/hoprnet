import { dbMock, debug, stringToU8a } from '@hoprnet/hopr-utils'
import sinon from 'sinon'
import HoprEthereum, { ChainWrapperSingleton, Indexer, useFixtures } from '.'
import { sampleChainOptions } from './ethereum.mock'
import { ACCOUNT_A, PARTY_A } from './fixtures'

const namespace = 'hopr:test:hopr-ethereum'
const log = debug(namespace)

describe(`HoprEthereum`, function () {
  beforeEach(async () => {
    const { chain, indexer } = await useFixtures()
    log('ChainWrapper obtained from fixtures')
    sinon.stub(ChainWrapperSingleton, 'create').callsFake(() => {
      log('chainwrapper singleton stub started')
      return Promise.resolve(chain)
    })
    sinon.stub(Indexer, 'prototype').callsFake(() => {
      log('indexer constructor started')
      return indexer;
    })
  })
  it('should instantiate a new class w/o any issues', function () {
    log('starting new instance of HoprEthereum.')
    new HoprEthereum(dbMock, PARTY_A, stringToU8a(ACCOUNT_A.privateKey), sampleChainOptions)
    log('successfully created the HoprEthereum instance.')
  })
})
