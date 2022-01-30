import { listOpenChannels } from '../../../../../lib/api/v2/logic/channel'
import sinon from 'sinon'

let node = sinon.fake() as any

describe('listOpenChannels', () => {
  it('should work', () => {
    // NOTE: how does one go about mocking it?
    listOpenChannels({ node })
  })
  it('should fail', () => {
    listOpenChannels({ node })
  })
})
