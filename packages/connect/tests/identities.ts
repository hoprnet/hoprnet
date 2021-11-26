import PeerId from 'peer-id'

const Alice = Uint8Array.from([
  8, 2, 18, 32, 143, 114, 18, 156, 186, 207, 242, 255, 116, 75, 164, 53, 121, 130, 42, 201, 169, 1, 1, 105, 210, 158,
  183, 69, 162, 182, 149, 57, 195, 5, 32, 197
])

const Bob = Uint8Array.from([
  8, 2, 18, 32, 68, 193, 158, 115, 89, 1, 52, 120, 4, 122, 229, 155, 214, 30, 201, 125, 65, 187, 5, 88, 9, 59, 131, 74,
  0, 180, 254, 167, 89, 157, 140, 44
])

const Charly = Uint8Array.from([
  8, 2, 18, 32, 143, 106, 26, 136, 4, 70, 164, 190, 44, 120, 236, 165, 128, 119, 161, 205, 183, 108, 223, 91, 210, 102,
  32, 142, 113, 218, 116, 30, 138, 100, 193, 3
])

const Dave = Uint8Array.from([
  8, 2, 18, 32, 156, 120, 128, 26, 4, 9, 140, 40, 69, 123, 82, 255, 52, 202, 217, 138, 163, 52, 46, 98, 78, 0, 195, 180,
  47, 6, 144, 95, 9, 142, 116, 248
])

const Ed = Uint8Array.from([
  8, 2, 18, 32, 193, 20, 41, 106, 106, 73, 0, 238, 26, 177, 138, 16, 87, 18, 19, 226, 10, 156, 23, 61, 250, 125, 15,
  157, 51, 169, 214, 5, 252, 155, 38, 132
])

export function getIdentity(name: string): Uint8Array {
  switch (name) {
    case 'alice':
      return Alice
    case 'bob':
      return Bob
    case 'charly':
      return Charly
    case 'dave':
      return Dave
    case 'ed':
      return Ed
    default:
      throw new Error(`unknown identity ${name}`)
  }
}

export { Alice, Bob, Charly, Dave, Ed }

export async function peerIdForIdentity(identityName: string): Promise<PeerId> {
  return PeerId.createFromPrivKey(getIdentity(identityName))
}

export async function identityFromPeerId(peerIdToCheck: PeerId): Promise<string> {
  console.log(peerIdToCheck)
  for (const identityName of ['alice', 'bob', 'charly', 'dave', 'ed']) {
    const peerId = await peerIdForIdentity(identityName)
    if (peerId.toB58String() === peerIdToCheck.toB58String()) {
      return identityName
    }
  }

  console.log(`can't find identity for peerId ${peerIdToCheck}`)
  return 'unknown'
}
