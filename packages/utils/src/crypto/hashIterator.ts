import { u8aEquals } from '../u8a'

export interface Intermediate {
  iteration: number
  preImage: Uint8Array
}
export async function iterateHash(
  seed: Uint8Array | undefined,
  hashFunc: (preImage: Uint8Array) => Promise<Uint8Array> | Uint8Array,
  iterations: number,
  stepSize: number,
  hint?: (index: number) => Uint8Array | undefined | Promise<Uint8Array | undefined>
): Promise<{
  hash: Uint8Array
  intermediates: Intermediate[]
}> {
  const intermediates: Intermediate[] = []

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

  console.log(i)

  for (; i < iterations; i++) {
    if (stepSize != undefined && i % stepSize == 0) {
      intermediates.push({
        iteration: i,
        preImage: intermediate
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
  hashFunc: (preImage: Uint8Array) => Promise<Uint8Array> | Uint8Array,
  hint: (index: number) => Uint8Array | undefined | Promise<Uint8Array | undefined>,
  maxIterations: number,
  stepSize?: number,
  indexHint?: number
): Promise<Intermediate | undefined> {
  let closestIntermediate: number
  if (indexHint != undefined) {
    closestIntermediate = indexHint
  } else if (stepSize != undefined && stepSize > 0) {
    closestIntermediate = maxIterations - (maxIterations % stepSize)
  } else {
    closestIntermediate = 0
  }

  let intermediate: Uint8Array
  for (; closestIntermediate >= 0; closestIntermediate -= stepSize) {
    intermediate = await hint(closestIntermediate)

    if (intermediate == undefined) {
      if (closestIntermediate == 0) {
        return
      }

      continue
    }

    for (let i = 0; i < stepSize; i++) {
      let _tmp = await hashFunc(intermediate)

      if (u8aEquals(_tmp, hashValue)) {
        return {
          preImage: intermediate,
          iteration: closestIntermediate + i
        }
      }
      intermediate = _tmp
    }
  }
}
