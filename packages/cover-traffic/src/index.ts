import { main, State } from './ct'
import { privKeyToPeerId } from '@hoprnet/hopr-utils'

const priv = process.argv[2]
const peerId = privKeyToPeerId(priv)

function stopGracefully(signal) {
  console.log(`Process exiting with signal ${signal}`)
  process.exit()
}

if (require.main === module) {
  process.once('exit', stopGracefully)
  process.on('SIGINT', stopGracefully)
  process.on('SIGTERM', stopGracefully)
  process.on('uncaughtException', stopGracefully)

  main((_state: State) => {
    console.log('CT: State update')
  }, peerId)
}
