import truffleTypings from './truffle-typings'
import typechain from './typechain'

export default () => {
  return Promise.all([truffleTypings(), typechain()])
}
