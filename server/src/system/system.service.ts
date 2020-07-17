import { Injectable } from '@nestjs/common'
import readPkg from 'read-pkg-up'
import { HOPR_PROTOS_DIR } from '../constants'

@Injectable()
export class SystemService {
  private readonly pkg = readPkg.sync().packageJson
  private readonly nodePkg = readPkg.sync({
    cwd: HOPR_PROTOS_DIR,
  }).packageJson

  async getVersions(): Promise<{
    hoprServer: string
    hoprCore: string
    hoprCoreConnectorInterface: string
    hoprCoreEthereum: string
    hoprUtils: string
  }> {
    const hoprServer = this.pkg.version
    const hoprCore = this.nodePkg.version
    const hoprCoreConnectorInterface = this.nodePkg.dependencies['@hoprnet/hopr-core-connector-interface']
    const hoprCoreEthereum = this.nodePkg.dependencies['@hoprnet/hopr-core-ethereum']
    const hoprUtils = this.nodePkg.dependencies['@hoprnet/hopr-utils']

    return {
      hoprServer,
      hoprCore,
      hoprCoreConnectorInterface,
      hoprCoreEthereum,
      hoprUtils,
    }
  }
}
