import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
import { convertPubKeyFromB58String, PublicKey } from '@hoprnet/hopr-utils'
import { HoprToken__factory, HoprToken } from '../types'
import { getContractData, Networks } from '..'
import { readdir, readFile } from 'fs/promises'
import Wallet from 'ethereumjs-wallet'

const send = (signer, txparams) =>
  signer.sendTransaction(txparams, (error, transactionHash: string) => {
    if (error) {
      console.log(`Error: ${error}`)
    }
    console.log(`transactionHash: ${transactionHash}`)
  })

const nativeAddress = (hoprAddress: string) => convertPubKeyFromB58String(hoprAddress).toAddress().toHex()

async function getIdentities(pw: string) {
  const tmpFiles = await readdir('/tmp')

  const identityFilenames = tmpFiles.filter((fileName: string) => fileName.match(/[a-zA-Z]+-identity/))

  return Promise.all(
    identityFilenames.map((identityFilnename: string) =>
      readFile(`/tmp/${identityFilnename}`).then(async (data: Buffer) => {
        const w = await Wallet.fromV3(JSON.parse(new TextDecoder().decode(data)), pw)

        return PublicKey.fromPrivKey(w.getPrivateKey()).toAddress().toHex()
      })
    )
  )
}

async function fundNode(
  address: string,
  hoprToken: HoprToken,
  amount: string = '1.0',
  { ethers, network }: HardhatRuntimeEnvironment
) {
  const etherAmount = '1.0'
  const signer = ethers.provider.getSigner()
  const tx = {
    to: address,
    value: ethers.utils.parseEther(etherAmount)
  }
  console.log(`💧💰 Sending ${etherAmount} ETH to ${address} on network ${network.name}`)
  await send(signer, tx)
  const finalAmount = ethers.utils.parseEther(amount).toString()
  console.log(`💧🟡 Sending ${ethers.utils.formatEther(finalAmount)} HOPR to ${address} on network ${network.name}`)
  await hoprToken.mint(address, finalAmount, ethers.constants.HashZero, ethers.constants.HashZero, {
    gasLimit: 200e3
  })
}

type ArgsType =
  | {
      address: string
      amount: string
      ishopraddress?: boolean
      uselocalidentities: false
    }
  | {
      uselocalidentities: true
      amount?: string
      address?: undefined
      ishopraddress?: boolean
    }

/**
 * Faucets HOPR and ETH tokens to a local account with HOPR
 */
async function main(args: ArgsType, hre: HardhatRuntimeEnvironment, _runSuper: RunSuperFunction<any>) {
  if (hre.network.tags.development) {
    console.error('🌵 Faucet is only valid in a development network')
    return
  }

  let hoprTokenAddress: string
  try {
    console.log(hre.network.name)

    const contract = getContractData(hre.network.name as Networks, 'HoprToken')
    hoprTokenAddress = contract.address
  } catch (err) {
    console.error('⛓  You need to ensure the network deployed the contracts')
    return
  }

  const hoprToken = HoprToken__factory.connect(hoprTokenAddress, hre.ethers.provider).connect(
    hre.ethers.provider.getSigner()
  )

  if (args.uselocalidentities) {
    const addresses = await getIdentities('local')

    await Promise.all(addresses.map((addr) => fundNode(addr, hoprToken, args.amount, hre)))
  } else {
    await fundNode(args.ishopraddress ? nativeAddress(args.address) : args.address, hoprToken, args.amount, hre)
  }
}

export default main
