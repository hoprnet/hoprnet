import type { ContractData, NetworkTag } from '../../chain'
import { join } from 'path'
import { promises } from 'fs'

const { mkdir, readFile, writeFile } = promises
const OUTPUT_DIR = join(__dirname, '..', '..', 'chain')
const OUTPUT_FILE = join(OUTPUT_DIR, 'contracts.json')

export const storeContract = async (
  network: string,
  tags: Record<NetworkTag, boolean>,
  name: string,
  address: string,
  deployedAt: number
): Promise<void> => {
  // do not store testing addresses
  if (tags.testing) return

  let contracts: {
    [network: string]: {
      [contract: string]: ContractData
    }
  }

  try {
    contracts = JSON.parse(await readFile(OUTPUT_FILE, { encoding: 'utf-8' }))
  } catch {
    contracts = {}
  }

  if (!contracts[network]) contracts[network] = {}
  contracts[network][name] = {
    address,
    deployedAt
  }

  await mkdir(OUTPUT_DIR, { recursive: true })
  await writeFile(OUTPUT_FILE, JSON.stringify(contracts, null, 2))
}

export const getContract = async (network: string, name: string): Promise<ContractData> => {
  // HACK: hardhat keeps using localhost internally - fix coming in https://github.com/hoprnet/hoprnet/pull/1676
  if (network === 'hardhat') network = 'localhost'

  try {
    return require(OUTPUT_FILE)?.[network]?.[name]
  } catch {
    throw Error('You need to ensure the network deployed the contracts')
  }
}
