import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
import { getAddresses } from '../chain'

const HoprToken = artifacts.require('HoprToken')

/**
 * Display unlocked accounts alongside with how much
 * ETH / HOPR they have.
 */
async function main(_params, { network, web3 }: HardhatRuntimeEnvironment, _runSuper: RunSuperFunction<any>) {
  const addresses = getAddresses()
  if (!addresses[network.name]) throw Error(`cannot find HoprToken address for network ${network.name}`)
  const hoprToken = await HoprToken.at(addresses[network.name].HoprToken)

  console.log('Running task "accounts" with config:', {
    network: network.name
  })

  const accounts = await web3.eth.getAccounts()
  const nativeBalances = await Promise.all(
    accounts.map(async (address) => {
      const amount = await web3.eth.getBalance(address)
      return web3.utils.fromWei(amount, 'ether')
    })
  )
  const hoprBalances = await Promise.all(
    accounts.map(async (address) => {
      const amount = await hoprToken.balanceOf(address)
      return web3.utils.fromWei(amount, 'ether')
    })
  )

  console.table(
    accounts.map((account, i) => {
      return {
        account,
        native: nativeBalances[i],
        hopr: hoprBalances[i]
      }
    })
  )
}

export default main
