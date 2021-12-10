import { main } from './index'
import { debug } from '@hoprnet/hopr-utils'

const namespace = 'hopr:test:hoprd'
const log = debug(namespace)

describe('HOPRd', () => {
  it('should close channels properly between alice and bob', async () => {
    console.log("This should be logged.")
    log('starting alice')
    await main();
    log('alice has been completed')
  })
})