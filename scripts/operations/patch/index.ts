import typechain from './typechain'
import truffleTypings from './truffle-typings'
import ganacheCore from './ganache-core'

export default () => {
  return Promise.all([typechain(), truffleTypings(), ganacheCore()]).catch(console.error)
}
