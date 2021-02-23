import { publicKeyConvert } from 'secp256k1'
import { Hash, AcknowledgedTicket } from './types'
import { waitForConfirmation, getSignatureParameters, pubKeyToAccountId } from './utils'
import Debug from 'debug'
import { u8aToHex } from '@hoprnet/hopr-utils'
import type Account from './account'
import type { HoprChannels } from './tsc/web3/HoprChannels'
import { stringToU8a, u8aIsEmpty } from '@hoprnet/hopr-utils'

const log = Debug('hopr-core-ethereum:chainInteractions')

const isNullAccount = (a: string) => a == null || ['0', '0x', '0x'.padEnd(66, '0')].includes(a)

export async function storeSecretOnChain(secret: Hash, account: Account, channels: HoprChannels): Promise<void> {
  log(`storing secret on chain, setting secret to ${u8aToHex(secret)}`)
  const address = (await account.address).toHex()
  if (isNullAccount((await channels.methods.accounts(address).call()).accountX)) {
    const uncompressedPubKey = publicKeyConvert(account.keys.onChain.pubKey, false).slice(1)
    log('account is also null, calling channel.init')
    try {
      await waitForConfirmation(
        (
          await account.signTransaction(
            {
              from: address,
              to: channels.options.address
            },
            channels.methods.init(
              u8aToHex(uncompressedPubKey.slice(0, 32)),
              u8aToHex(uncompressedPubKey.slice(32, 64)),
              u8aToHex(secret)
            )
          )
        ).send()
      )
    } catch (e) {
      if (e.message.match(/Account must not be set/)) {
        // There is a potential race condition due to the fact that 2 init
        // calls may be in flight at once, and therefore we may have init
        // called on an initialized account. If so, trying again should solve
        // the problem.
        log('race condition encountered in HoprChannel.init - retrying')
        return storeSecretOnChain(secret, account, channels)
      }
      throw e
    }
  } else {
    // @TODO this is potentially dangerous because it increases the account counter
    log('account is already on chain, storing secret.')
    try {
      await waitForConfirmation(
        (
          await account.signTransaction(
            {
              from: address,
              to: this.channels.options.address
            },
            this.channels.methods.setHashedSecret(u8aToHex(secret))
          )
        ).send()
      )
    } catch (e) {
      if (e.message.match(/new and old hashedSecrets are the same/)) {
        // NBD. no-op
        return
      }
      throw e
    }
  }

  log('stored on chain')
}

export async function findOnChainSecret(channels: HoprChannels, account: Account): Promise<Hash | undefined> {
  const res = await channels.methods.accounts((await account.address).toHex()).call()
  const hashedSecret = stringToU8a(res.hashedSecret)
  if (u8aIsEmpty(hashedSecret)) {
    return undefined
  }
  return new Hash(hashedSecret)
}

export async function submitTicketRedemption(ackTicket: AcknowledgedTicket, channels: HoprChannels, account: Account) {
  const ticketChallenge = ackTicket.getResponse()
  const signedTicket = ackTicket.getSignedTicket()
  const ticket = signedTicket.ticket

  log('Submitting ticket', u8aToHex(ticketChallenge))
  const { r, s, v } = getSignatureParameters(signedTicket.signature)
  const counterparty = await pubKeyToAccountId(await signedTicket.getSigner())
  const transaction = await account.signTransaction(
    {
      from: (await account.address).toHex(),
      to: channels.options.address
    },
    channels.methods.redeemTicket(
      u8aToHex(ackTicket.getPreImage()),
      u8aToHex(ackTicket.getResponse()),
      ticket.amount.toString(),
      u8aToHex(ticket.winProb),
      u8aToHex(counterparty),
      u8aToHex(r),
      u8aToHex(s),
      v + 27
    )
  )

  await transaction.send()
}
