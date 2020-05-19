import ministun = require('ministun')
import { HoprOptions } from '..'

class StunServer {
  private server: ministun

  constructor(options: HoprOptions) {
    const config = {
      udp4: options.hosts.ip4 !== undefined,
      upd6: options.hosts.ip6 !== undefined,
      port: 3478,
      log: null,
      err: null,
      sw: true,
    }

    this.server = new ministun(config)
  }

  async start() {
    await this.server.start()
  }

  async stop() {
      await this.server.stop()
  }
}

export { StunServer }