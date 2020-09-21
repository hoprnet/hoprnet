import { Injectable } from '@nestjs/common'
import readPkg from 'read-pkg-up'
import { HOPR_CORE_DIR } from '../constants'

@Injectable()
export class SystemService {
  private readonly pkg = readPkg.sync().packageJson
  private readonly corePkg = readPkg.sync({
    cwd: HOPR_CORE_DIR,
  }).packageJson

  async getVersions(): Promise<{
    hoprServer: string
    hoprCore: string
    hoprCoreConnectorInterface: string
    hoprCoreEthereum: string
    hoprUtils: string
  }> {
    const hoprServer = this.pkg.version
    const hoprCore = this.corePkg.version
    const hoprCoreConnectorInterface = this.corePkg.dependencies['@hoprnet/hopr-core-connector-interface']
    const hoprCoreEthereum = this.corePkg.dependencies['@hoprnet/hopr-core-ethereum']
    const hoprUtils = this.corePkg.dependencies['@hoprnet/hopr-utils']

    return {
      hoprServer,
      hoprCore,
      hoprCoreConnectorInterface,
      hoprCoreEthereum,
      hoprUtils,
    }
  }
}
