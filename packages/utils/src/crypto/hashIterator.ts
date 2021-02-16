import { u8aEquals } from '../u8a'

export interface Item {
  iteration: number
  source: Uint8Array
}

/**
 * Iteratively generate a chain of hashes of the previous value.
 * This is returned as a map of hashes to 'blocks'
 */
export async function iterateHash(
  seed: Uint8Array | undefined,
  hashFunc: (preImage: Uint8Array) => Promise<Uint8Array> | Uint8Array,
  iterations: number,
  stepSize: number,
  hint?: (index: number) => Uint8Array | undefined | Promise<Uint8Array | undefined>
): Promise<{
  hash: Uint8Array
  intermediates: Item[]
}> {
  const intermediates: Item[] = []
  let intermediate = seed
  let i = 0

  if (hint != undefined) {
    let closest = iterations - (iterations % stepSize)

    for (; closest >= 0; closest -= stepSize) {
      let tmp = await hint(closest)

      if (tmp != undefined) {
        intermediate = tmp
        i = closest
        break
      }
    }
  }

  if (intermediate == undefined && seed == undefined) {
    throw Error(`Cannot compute hash because no seed was given through the 'hint' function or the 'seed' argument.`)
  }

  for (; i < iterations; i++) {
    if (stepSize != undefined && i % stepSize == 0) {
      intermediates.push({
        iteration: i,
        source: intermediate
      })
    }
    intermediate = await hashFunc(intermediate)
  }

  return {
    hash: intermediate,
    intermediates
  }
}

export async function recoverIteratedHash(
  hashValue: Uint8Array,
  hashFunc: (preImage: Uint8Array) => Promise<Uint8Array>,
  lookup: (index: number) => Promise<Uint8Array | undefined>,
  maxIterations: number,
  stepSize: number,
): Promise<Uint8Array> {
  let intermediate: Uint8Array
  for (let closestIntermediate = maxIterations - (maxIterations % stepSize);
       closestIntermediate >= 0;
       closestIntermediate -= stepSize) {
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
