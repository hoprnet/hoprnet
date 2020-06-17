import assert from 'assert'
import type HoprEthereum from '.'
import * as DbKeys from './dbKeys'
import * as Utils from './utils'
import * as Types from './types'
import PreImage, { GIANT_STEP_WIDTH, TOTAL_ITERATIONS } from './hashedSecret'
import { randomInteger, u8aEquals, durations, stringToU8a, u8aToHex } from '@hoprnet/hopr-utils'
import Memdown from 'memdown'
import LevelUp from 'levelup'
import { Ganache, migrate, fund } from '@hoprnet/hopr-ethereum'
import { NODE_SEEDS } from '@hoprnet/hopr-demo-seeds'
import Web3 from 'web3'
import * as testconfigs from './config.spec'
import * as configs from './config'
import HoprChannelsAbi from '@hoprnet/hopr-ethereum/build/extracted/abis/HoprChannels.json'
import Account from './account'

describe('test hashedSecret management', function () {
  this.timeout(durations.seconds(7))
  const ganache = new Ganache()
  let connector: HoprEthereum
  let preImage: PreImage

  async function generateConnector(): Promise<HoprEthereum> {
    let web3 = new Web3(configs.DEFAULT_URI)
    const chainId = await Utils.getChainId(web3)
    const network = Utils.getNetworkName(chainId)

    const connector = ({
      signTransaction: Utils.TransactionSigner(web3, stringToU8a(NODE_SEEDS[0])),
      hoprChannels: new web3.eth.Contract(HoprChannelsAbi as any, configs.CHANNELS_ADDRESSES[network]),
      web3,
      db: LevelUp(Memdown()),
      dbKeys: DbKeys,
      utils: Utils,
      types: Types,
      options: {
        debug: false,
      },
    } as unknown) as HoprEthereum

    connector.account = new Account(
      connector,
      stringToU8a(testconfigs.DEMO_ACCOUNTS[0]),
      await Utils.privKeyToPubKey(stringToU8a(testconfigs.DEMO_ACCOUNTS[0]))
    )

    connector.hashedSecret = new PreImage(connector)

    return connector
  }

  const checkIndex = async (index: number, masterSecret: Uint8Array, shouldThrow: boolean) => {
    let hash = new Types.Hash(masterSecret)
    for (let i = 0; i < index; i++) {
      hash = await connector.utils.hash(hash)
    }

    let result,
      errThrown = false
    try {
      result = await connector.hashedSecret.getPreimage(hash)
    } catch (err) {
      errThrown = true
    }

    if (shouldThrow) {
      assert(errThrown, `Must throw an error`)
    } else {
      assert(u8aEquals(await connector.utils.hash(result.preImage), hash) && index == result.index + 1)
    }
  }

  before(async function () {
    this.timeout(60e3)
    await ganache.start()
    await migrate()
    await fund(1)

    connector = await generateConnector()
  })

  after(async function () {
    await ganache.stop()
  })

  it('should publish a hashed secret', async function () {
    await connector.hashedSecret.check()

    await connector.hashedSecret.submit()

    let onChainHash = new Types.Hash(
      stringToU8a(
        (await connector.hoprChannels.methods.accounts((await connector.account.address).toHex()).call()).hashedSecret
      )
    )

    let preImage = await connector.hashedSecret.getPreimage(onChainHash)

    assert(u8aEquals(await connector.utils.hash(preImage.preImage), onChainHash))

    await connector.utils.waitForConfirmation(
      (
        await connector.signTransaction(connector.hoprChannels.methods.setHashedSecret(preImage.preImage.toHex()), {
          from: (await connector.account.address).toHex(),
          to: connector.hoprChannels.options.address,
          nonce: await connector.account.nonce,
        })
      ).send()
    )
    let updatedOnChainHash = new Types.Hash(
      stringToU8a(
        (await connector.hoprChannels.methods.accounts((await connector.account.address).toHex()).call()).hashedSecret
      )
    )

    assert(!u8aEquals(onChainHash, updatedOnChainHash), `new and old onChainSecret must not be the same`)

    let updatedPreImage = await connector.hashedSecret.getPreimage(updatedOnChainHash)

    assert(!u8aEquals(preImage.preImage, updatedPreImage.preImage), `new and old pre-image must not be the same`)

    assert(u8aEquals(await connector.utils.hash(updatedPreImage.preImage), updatedOnChainHash))
  })

  it('should generate a hashed secret and recover a pre-Image', async function () {
    await connector.hashedSecret.create()

    for (let i = 0; i < TOTAL_ITERATIONS / GIANT_STEP_WIDTH; i++) {
      assert(
        (await connector.db.get(Buffer.from(connector.dbKeys.OnChainSecretIntermediary(i * GIANT_STEP_WIDTH)))) != null
      )
    }

    const masterSecret = await connector.db.get(Buffer.from(connector.dbKeys.OnChainSecretIntermediary(0)))

    checkIndex(1, masterSecret, false)
    checkIndex(randomInteger(1, TOTAL_ITERATIONS), masterSecret, false)
    checkIndex(TOTAL_ITERATIONS, masterSecret, false)

    checkIndex(0, masterSecret, true)
    checkIndex(TOTAL_ITERATIONS + 1, masterSecret, true)
  })
})
