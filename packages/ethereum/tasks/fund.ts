import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'

async function main(
  { address, amount, accounts: providedAccounts }: { address: string; amount: string; accounts: string[] },
  { web3, network }: HardhatRuntimeEnvironment,
  _runSuper: RunSuperFunction<any>
) {
  const HoprToken = artifacts.require('HoprToken')
  const hoprToken = await HoprToken.at(address)

  const accounts = providedAccounts.length > 0 ? providedAccounts : await web3.eth.getAccounts()
  const owner = accounts[0]

  console.log('Running task "fund" with config:', {
    network: network.name,
    address,
    amount,
    accounts
  })

  for (const account of accounts) {
    await hoprToken.mint(account, amount, '0x00', '0x00', {
      from: owner
    })

    console.log(`Funded: ${account}`)
  }
}

export default main
