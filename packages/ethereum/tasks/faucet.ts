import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
import { NODE_SEEDS } from '@hoprnet/hopr-demo-seeds'
import { HoprToken__factory } from '../types'
import { promisify } from 'util'
import { stat, readFile } from 'fs'

const statAsync = promisify(stat)
const readFileAsync = promisify(readFile)
const send = (signer, txparams) =>
  signer.sendTransaction(txparams, (error, transactionHash) => {
    if (error) {
      console.log(`Error: ${error}`)
    }
    console.log(`transactionHash: ${transactionHash}`)
  })

const getHoprTokenAddress = async (addressesFile) => {
  try {
    const deployedContracts = await readFileAsync(addressesFile, 'utf8')
    return JSON.parse(deployedContracts).contracts['HoprToken'].address
  } catch (err) {
    console.log('⛔️ Error when obtaining local address', err)
    return
  }
}

/**
 * Faucets HOPR and ETH tokens to a local account with HOPR
 */
async function main(
  { address, amount }: { address: string; amount: string },
  { ethers, network }: HardhatRuntimeEnvironment,
  _runSuper: RunSuperFunction<any>
) {
  if (network.name !== 'localhost') {
    console.error('🌵 Faucet is only valid in localhost network')
    return
  }
  const addressesFile = __dirname + '/../hardhat/cache/deployed_contracts.json'
  if (!(await statAsync(addressesFile))) {
    console.error('⛓  You need to ensure the network deployed the contracts')
    return
  }

  console.log('🚰 Starting local faucet task')
  const etherAmount = '1.0'
  const signer = ethers.provider.getSigner()
  const minterWallet = new ethers.Wallet(NODE_SEEDS[0], ethers.provider)
  const tx = {
    to: address,
    value: ethers.utils.parseEther(etherAmount)
  }
  const hoprTokenAddress = await getHoprTokenAddress(addressesFile)
  const hoprToken = HoprToken__factory.connect(hoprTokenAddress, ethers.provider).connect(minterWallet)

  console.log(`💧💰 Sending ${etherAmount} ETH to ${address} on network ${network.name}`)
  await send(signer, tx)

  console.log(`💧🟡 Sending ${ethers.utils.formatEther(amount)} HOPR to ${address} on network ${network.name}`)
  await hoprToken.mint(address, amount, ethers.constants.HashZero, ethers.constants.HashZero, {
    from: minterWallet.getAddress(),
    gasLimit: 200e3
  })
}

export default main
