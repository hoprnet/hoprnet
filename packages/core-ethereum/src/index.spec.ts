import assert from 'assert'
import sinon from 'sinon'
import { debug, LevelDb, stringToU8a } from '@hoprnet/hopr-utils'
import HoprCoreEthereum, { Indexer, useFixtures } from './index.js'
import { sampleChainOptions } from './ethereum.mock.js'
import { ACCOUNT_A, MOCK_ADDRESS, MOCK_PUBLIC_KEY, PARTY_A } from './fixtures.js'
import { Database as Ethereum_Database, PublicKey as Ethereum_PublicKey } from '../lib/core_ethereum_db.js'

const namespace = 'hopr:test:hopr-ethereum'
const log = debug(namespace)

describe(`test HoprEthereum instance creation`, function () {
  beforeEach(async () => {
    const { indexer } = await useFixtures()
    log('ChainWrapper obtained from fixtures')
    sinon.stub(Indexer, 'prototype').callsFake(() => {
      log('indexer constructor started')
      return indexer
    })
  })
  it('should instantiate a new class w/o any issues', function () {
    log('starting new instance of HoprEthereum.')
    HoprCoreEthereum.createInstance(
      new Ethereum_Database(new LevelDb(), Ethereum_PublicKey.from_privkey(stringToU8a(ACCOUNT_A.privateKey))),
      PARTY_A(),
      stringToU8a(ACCOUNT_A.privateKey),
      sampleChainOptions
    )
    log('successfully created the HoprEthereum instance.')
  })
})

describe('test HoprEthereum', function () {
  const connector = HoprCoreEthereum.createInstance(
    new Ethereum_Database(new LevelDb(), Ethereum_PublicKey.from_privkey(stringToU8a(ACCOUNT_A.privateKey))),
    PARTY_A(),
    stringToU8a(ACCOUNT_A.privateKey),
    sampleChainOptions
  )

  it('should test isAllowedAccessToNetwork', async function () {
    // @ts-ignore
    connector.db = sinon.stub()

    const hoprNode = MOCK_PUBLIC_KEY()
    const account = MOCK_ADDRESS()

    // should be false by default
    assert((await connector.isAllowedAccessToNetwork(hoprNode)) === false, 'hoprNode is not eligible by default')

    // @ts-ignore
    connector.db.is_network_registry_enabled = () => Promise.resolve(false)
    assert(
      (await connector.isAllowedAccessToNetwork(hoprNode)) === true,
      'should become registered when register is disabled'
    )

    // @ts-ignore
    connector.db.is_network_registry_enabled = () => Promise.resolve(true)
    assert((await connector.isAllowedAccessToNetwork(hoprNode)) === false, 'should go back to being not eligible')

    // @ts-ignore
    connector.db.get_account_from_network_registry = () => Promise.resolve(account)
    // should remain false
    assert(
      (await connector.isAllowedAccessToNetwork(hoprNode)) === false,
      'eligibility should remain false when not eligible'
    )

    // @ts-ignore
    connector.db.is_eligible = () => Promise.resolve(true)
    // should be true once is eligible
    assert((await connector.isAllowedAccessToNetwork(hoprNode)) === true, 'hoprNode should be eligible')

    // @ts-ignore
    connector.db.is_eligible = () => Promise.resolve(false)
    // should be false once unset
    assert((await connector.isAllowedAccessToNetwork(hoprNode)) === false, 'hoprNode should be uneligible')

    // @ts-ignore
    connector.db.is_eligible = () => Promise.resolve(true)
    // @ts-ignore
    connector.db.get_account_from_network_registry = () => Promise.reject()
    // should be false when registry is removed
    assert((await connector.isAllowedAccessToNetwork(hoprNode)) === false, 'hoprNode should not be eligible anymore')
  })
})
