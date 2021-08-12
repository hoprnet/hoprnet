import assert from 'assert'
import { verifySignatureFromPeerId } from './verifySignatureFromPeerId'

const myPeerId = '16Uiu2HAm3irRrBjLeHGY5GmriGZKYyNeWa7xU7z3LAj95UkMmnZM'
const message = `Hi there, this is a signed message.`
const wrongMessage = 'I’m sorry Dave, I’m afraid I can not do that'
const validSignatureFromMyPeerId =
  '0x3045022100cec20a5ad59c74ad5a27d80cc4c4cb4d87d520a4baf21e8f0cb684fb43e769c202201fffbf99e8c0a36dcedebee5870b65b1dbe38a0eef960ae1853cd115674c3752'
const validSignatureFromAnotherPeerId =
  '0x304502210097d255f4e4013a67441d9ae777abefb937be94f536aa9337920d799f301ec28402202c4eebe9b0ab73beb9a5df6e257091b3bcd4e84b85f5b5a93fb38b520697d68e'

describe(`Sign-verify: verifying signature from peerId`, function () {
  it('succeeds if the signature is valid', async function () {
    assert(verifySignatureFromPeerId(myPeerId, message, validSignatureFromMyPeerId))
  })
  it('fails if the signature is from another peer', async function () {
    assert(verifySignatureFromPeerId(myPeerId, message, validSignatureFromAnotherPeerId))
  })
  it('fails if the message does not correspond to the signature', async function () {
    assert(verifySignatureFromPeerId(myPeerId, wrongMessage, validSignatureFromAnotherPeerId))
  })
})
