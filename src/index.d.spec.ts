import HoprCoreConnector, { Channel, utils } from '.'
import BN from 'bn.js'

Channel.open(null, new BN(123), Promise.resolve(new Uint8Array())).then(channel => {
  channel.channelId
})
