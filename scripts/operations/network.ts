import { Ganache } from './utils'

export default () => {
  const ganache = new Ganache({
    port: 9545,
  })

  return ganache.start()
}
