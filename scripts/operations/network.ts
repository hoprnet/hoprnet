import { Ganache } from './utils'
const config: any = require('../../truffle-config')

export default (network: string = 'development') => {
  const networkConfig = config.networks[network]
  if (typeof network === 'undefined') {
    throw Error(`config for network '${network}' not found`)
  }

  const ganache = new Ganache({
    port: networkConfig.port
  })

  return ganache.start()
}
