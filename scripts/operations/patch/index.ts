import ganacheCore from './ganache-core'
import truffleTypings from './truffle-typings'
import typechain from './typechain'

export default () => {
  return Promise.all([ganacheCore(), truffleTypings(), typechain()])
}
