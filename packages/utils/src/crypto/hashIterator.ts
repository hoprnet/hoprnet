import { u8aEquals } from '../u8a'

/**
 * Iteratively generate a chain of hashes of the previous value.
 */
export async function iterateHash(
  seed: Uint8Array | undefined,
  hashFunc: (preImage: Uint8Array) => Promise<Uint8Array>,
  iterations: number,
): Promise<Uint8Array[]> {
  const result: Uint8Array[] = [seed]
  let intermediate = seed
  for (let i = 0; i < iterations; i++) {
    intermediate = await hashFunc(intermediate)
    result.push(intermediate)
  }
  return result
}

export async function recoverIteratedHash(
  hashValue: Uint8Array,
  hashFunc: (preImage: Uint8Array) => Promise<Uint8Array>,
  lookup: (index: number) => Promise<Uint8Array | undefined>,
  maxIterations: number,
  stepSize: number
): Promise<Uint8Array> {
  let intermediate: Uint8Array
  for (
    let closestIntermediate = maxIterations - (maxIterations % stepSize);
    closestIntermediate >= 0;
    closestIntermediate -= stepSize
  ) {
    intermediate = await lookup(closestIntermediate)
    try {
      return reverseHash(hashValue, hashFunc, intermediate, stepSize)
    } catch (e) {}
  }
  throw new Error('Could not find source in any block')
}

// Given a hash that is a multiple hashed result of a specified source,
// try and find it's immediate source.
export async function reverseHash(
  hashValue: Uint8Array,
  hashFunc: (source: Uint8Array) => Promise<Uint8Array>,
  startValue: Uint8Array,
  maxIterations: number
): Promise<Uint8Array> {
  let val = startValue
  for (let i = 0; i < maxIterations; i++) {
    let _tmp = await hashFunc(val)
    if (u8aEquals(_tmp, hashValue)) {
      return val
    }
    val = _tmp
  }
  throw Error(`Could not find source in givent block.`)
}
