import { Ganache } from '@hoprnet/hopr-testing'
import networks from '../../truffle-networks'

export default () => {
  const ganache = new Ganache({
    port: networks.development.port,
  })

  return ganache.start()
}
