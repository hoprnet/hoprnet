import assert from 'assert'

import TCP from '.'

describe('should create a socket and connect to it', function () {
  const upgrader = {
    upgradeOutbound: maConn => maConn,
    upgradeInbound: maConn => maConn,
  }
  const tcp = new TCP({ upgrader })
})
