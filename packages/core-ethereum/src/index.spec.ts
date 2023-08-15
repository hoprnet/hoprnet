// import assert from 'assert'
// import sinon from 'sinon'
// import { debug, LevelDb, stringToU8a } from '@hoprnet/hopr-utils'
// import HoprCoreEthereum, { Indexer, useFixtures } from './index.js'
// import { sampleChainOptions } from './ethereum.mock.js'
// import { ACCOUNT_A, MOCK_ADDRESS, MOCK_PUBLIC_KEY, PARTY_A } from './fixtures.js'
// import { Ethereum_Address, Ethereum_Database, Ethereum_PublicKey, Ethereum_Snapshot, Ethereum_U256 } from './db.js'

// const namespace = 'hopr:test:hopr-ethereum'
// const log = debug(namespace)

// describe(`test HoprEthereum instance creation`, function () {
//   beforeEach(async () => {
//     const { indexer } = await useFixtures()
//     log('ChainWrapper obtained from fixtures')
//     sinon.stub(Indexer, 'prototype').callsFake(() => {
//       log('indexer constructor started')
//       return indexer
//     })
//   })
//   it('should instantiate a new class w/o any issues', function () {
//     log('starting new instance of HoprEthereum.')
//     HoprCoreEthereum.createInstance(
//       new Ethereum_Database(
//         new LevelDb(),
//         Ethereum_PublicKey.from_privkey(stringToU8a(ACCOUNT_A.privateKey)).to_address()
//       ),
//       PARTY_A(),
//       stringToU8a(ACCOUNT_A.privateKey),
//       sampleChainOptions
//     )
//     log('successfully created the HoprEthereum instance.')
//   })
// })

// describe('test HoprEthereum', function () {
//   const connector = HoprCoreEthereum.createInstance(
//     new Ethereum_Database(
//       new LevelDb(),
//       Ethereum_PublicKey.from_privkey(stringToU8a(ACCOUNT_A.privateKey)).to_address()
//     ),
//     PARTY_A(),
//     stringToU8a(ACCOUNT_A.privateKey),
//     sampleChainOptions
//   )

//   it('should test isAllowedAccessToNetwork', async function () {
//     // @ts-ignore
//     connector.db = new Ethereum_Database(
//       new LevelDb(),
//       Ethereum_Address.deserialize(MOCK_PUBLIC_KEY().to_address().serialize())
//     )

//     const hoprNode = MOCK_PUBLIC_KEY()
//     const account = MOCK_ADDRESS()

//     // should be false by default
//     assert((await connector.isAllowedAccessToNetwork(hoprNode)) === false, 'hoprNode is not eligible by default')

//     // @ts-ignore
//     connector.db.set_network_registry(
//       false,
//       new Ethereum_Snapshot(Ethereum_U256.zero(), Ethereum_U256.zero(), Ethereum_U256.zero())
//     )
//     assert(
//       (await connector.isAllowedAccessToNetwork(hoprNode)) === true,
//       'should become registered when register is disabled'
//     )

//     // @ts-ignore
//     await connector.db.set_network_registry(
//       true,
//       new Ethereum_Snapshot(Ethereum_U256.zero(), Ethereum_U256.zero(), Ethereum_U256.zero())
//     )

//     assert((await connector.isAllowedAccessToNetwork(hoprNode)) === false, 'should go back to being not eligible')

//     // @ts-ignore
//     connector.db.get_account_from_network_registry = () => Promise.resolve(account)
//     // should remain false
//     assert(
//       (await connector.isAllowedAccessToNetwork(hoprNode)) === false,
//       'eligibility should remain false when not eligible'
//     )

//     // @ts-ignore
//     await connector.db.set_eligible(
//       Ethereum_Address.deserialize(account.serialize()),
//       true,
//       new Ethereum_Snapshot(Ethereum_U256.zero(), Ethereum_U256.zero(), Ethereum_U256.zero())
//     )
//     // @ts-ignore
//     await connector.db.add_to_network_registry(
//       Ethereum_Address.deserialize(hoprNode.to_address().serialize()),
//       Ethereum_Address.deserialize(account.serialize()),
//       new Ethereum_Snapshot(Ethereum_U256.zero(), Ethereum_U256.zero(), Ethereum_U256.zero())
//     )
//     // connector.db.is_eligible = () => Promise.resolve(true)
//     // should be true once is eligible
//     assert((await connector.isAllowedAccessToNetwork(hoprNode)) === true, 'hoprNode should be eligible')

//     // @ts-ignore
//     await connector.db.set_eligible(
//       Ethereum_Address.deserialize(account.serialize()),
//       false,
//       new Ethereum_Snapshot(Ethereum_U256.zero(), Ethereum_U256.zero(), Ethereum_U256.zero())
//     )
//     // should be false once unset
//     assert((await connector.isAllowedAccessToNetwork(hoprNode)) === false, 'hoprNode should be uneligible')

//     // @ts-ignore
//     await connector.db.set_eligible(
//       Ethereum_Address.deserialize(account.serialize()),
//       true,
//       new Ethereum_Snapshot(Ethereum_U256.zero(), Ethereum_U256.zero(), Ethereum_U256.zero())
//     )
//     // @ts-ignore
//     await connector.db.remove_from_network_registry(
//       Ethereum_Address.deserialize(hoprNode.to_address().serialize()),
//       Ethereum_Address.deserialize(account.serialize()),
//       new Ethereum_Snapshot(Ethereum_U256.zero(), Ethereum_U256.zero(), Ethereum_U256.zero())
//     )

//     // should be false when registry is removed
//     assert((await connector.isAllowedAccessToNetwork(hoprNode)) === false, 'hoprNode should not be eligible anymore')
//   })
// })
