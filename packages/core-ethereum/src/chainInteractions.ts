import { Public, ChannelEntry } from './types'
import BN from 'bn.js'
import { getId, getParties, isGanache, pubKeyToAccountId } from './utils'
import { getWeb3 } from './web3'
import { u8aToHex } from '@hoprnet/hopr-utils'

export async function getChannel(self: Public, counterparty: Public): Promise<ChannelEntry> {
  //const self = new Public(this.coreConnector.account.keys.onChain.pubKey)
  const selfAccountId = await self.toAccountId()
  const counterpartyAccountId = await counterparty.toAccountId()
  const [partyAAccountId] = getParties(selfAccountId, counterpartyAccountId)

  // HACK: when running our unit/intergration tests using ganache, the indexer doesn't have enough
  // time to pick up the events and reduce the data - here we are doing 2 things wrong:
  // 1. all our unit tests are actually intergration tests, nothing is mocked
  // 2. our actual intergration tests do not have any block mining time
  // this will be tackled in the upcoming refactor
  if (isGanache(this.coreConnector.network)) {
    const channelId = await getId(selfAccountId, counterpartyAccountId)
    const response = await this.coreConnector.hoprChannels.methods.channels(channelId.toHex()).call()

    return new ChannelEntry(undefined, {
      blockNumber: new BN(0),
      transactionIndex: new BN(0),
      logIndex: new BN(0),
      deposit: new BN(response.deposit),
      partyABalance: new BN(response.partyABalance),
      closureTime: new BN(response.closureTime),
      stateCounter: new BN(response.stateCounter),
      closureByPartyA: response.closureByPartyA
    })
  } else {
    let channelEntry = await this.coreConnector.indexer.getChannelEntry(
      partyAAccountId.eq(selfAccountId) ? self : counterparty,
      partyAAccountId.eq(selfAccountId) ? counterparty : self
    )
    if (channelEntry) return channelEntry

    // when channelEntry is not found, the onchain data is all 0
    return new ChannelEntry(undefined, {
      blockNumber: new BN(0),
      transactionIndex: new BN(0),
      logIndex: new BN(0),
      deposit: new BN(0),
      partyABalance: new BN(0),
      closureTime: new BN(0),
      stateCounter: new BN(0),
      closureByPartyA: false
    })
  }
}

export async function initiateChannelSettlement(): Promise<string> {
  const { hoprChannels } = getWeb3()
  let receipt: string
  try {
    if (status === 'OPEN') {
      const tx = await account.signTransaction(
        {
          from: (await account.address).toHex(),
          to: hoprChannels.options.address
        },
        hoprChannels.methods.initiateChannelClosure(u8aToHex(await pubKeyToAccountId(this.counterparty)))
      )

      receipt = tx.transactionHash
      tx.send()
    } else if (status === 'PENDING') {
      const tx = await account.signTransaction(
        {
          from: (await account.address).toHex(),
          to: hoprChannels.options.address
        },
        hoprChannels.methods.claimChannelClosure(u8aToHex(await pubKeyToAccountId(this.counterparty)))
      )

      receipt = tx.transactionHash
      tx.send()
    }

    return receipt
  } catch (error) {
    throw error
  }
}
