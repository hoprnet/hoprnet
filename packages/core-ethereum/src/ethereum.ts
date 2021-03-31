import type Connector from '.'
import type Web3 from 'web3'
import type { Address } from './types'
import type { HoprToken } from './tsc/web3/HoprToken'
import BN from 'bn.js'
import { Balance, NativeBalance } from './types'
import { waitForConfirmation } from './utils'

export const getNativeBalance = async (web3: Web3, account: Address): Promise<NativeBalance> => {
  const result = await web3.eth.getBalance(account.toHex())
  return new NativeBalance(new BN(result))
}

export const getBalance = async (token: HoprToken, account: Address): Promise<Balance> => {
  const result = await token.methods.balanceOf(account.toHex()).call()
  return new Balance(new BN(result))
}

export const fundChannel = async (
  // TODO: replace with ethereum global context?
  connector: Connector,
  {
    fundAmount,
    counterparty,
    openChannel = false
  }: {
    fundAmount: Balance
    counterparty: Address
    openChannel?: boolean
  }
): Promise<void> => {
  const { web3, hoprToken: token, hoprChannels: channels, account } = connector
  const selfAddress = await connector.hexAccountAddress()

  const tx = await account.signTransaction(
    {
      from: selfAddress,
      to: token.options.address
    },
    token.methods.send(
      channels.options.address,
      fundAmount.toBN().toString(),
      web3.eth.abi.encodeParameters(['bool', 'address', 'address'], [openChannel, selfAddress, counterparty.toHex()])
    )
  )

  await waitForConfirmation(tx.send())
}
