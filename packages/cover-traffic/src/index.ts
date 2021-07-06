import { main } from './ct'
import { privKeyToPeerId } from '@hoprnet/hopr-utils'

const priv = process.argv[2]
const peerId = privKeyToPeerId(priv)
main(() => {}, peerId)
