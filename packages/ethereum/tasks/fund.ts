import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'

const HoprToken = artifacts.require('HoprToken')

/**
 * Funds all unlocked accounts with HOPR
 */
async function main(
  { address, amount, accountsToFund }: { address: string; amount: string; accountsToFund: number },
  { web3, network }: HardhatRuntimeEnvironment,
  _runSuper: RunSuperFunction<any>
) {
  console.log({
    address,
    network: network.name
  })
  const hoprToken = await HoprToken.at(address)
  const accounts = (await web3.eth.getAccounts()).slice(0, accountsToFund)
  const owner = accounts[0]

  console.log('Running task "fund" with config:', {
    network: network.name,
    address,
    amount,
    accounts
  })

  for (const account of accounts) {
    await hoprToken.mint(account, amount, '0x00', '0x00', {
      from: owner,
      gas: 200e3
    })

    console.log(`Funded: ${account}`)
  }
}

export default main
