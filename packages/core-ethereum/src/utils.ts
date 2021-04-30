import { Networks, networks } from '@hoprnet/hopr-ethereum'
import { Hash, Signature } from './types'

/**
 * Get current network's name.
 *
 * @param chainId a chain id
 * @returns the network's name
 */
export function getNetworkName(chainId: number): Networks {
  const entry = Object.entries(networks).find(([_, options]) => options.chainId === chainId)

  if (entry) return entry[0] as Networks
  return 'localhost'
}

/**
 * Get current network's name.
 *
 * @param network
 * @returns the network's name
 */
export function getNetworkGasPrice(network: Networks): number | undefined {
  const entry = Object.entries(networks).find((entry) => entry[0] === network)

  if (entry && entry[1].gas) return entry[1].gas
  return undefined
}

/**
 * Get r,s,v values of a signature
 */
export function getSignatureParameters(
  signature: Signature
): {
  r: Hash
  s: Hash
  v: number
} {
  return {
    r: new Hash(signature.signature.slice(0, 32)),
    s: new Hash(signature.signature.slice(32, 64)),
    v: signature.recovery
  }
}
