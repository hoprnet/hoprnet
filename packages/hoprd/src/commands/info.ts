import type Hopr from '@hoprnet/hopr-core'
import { AbstractCommand } from './abstractCommand'

export class Info extends AbstractCommand {
  constructor(public node: Hopr) {
    super()
  }

  public name() {
    return 'info'
  }

  public help() {
    return 'Information about the HOPR Node, including any options it started with'
  }

  public async execute(): Promise<string> {
    // @TODO Add connector info etc.
    return [
      `Announcing to other nodes as: ${(await this.node.getAnnouncedAddresses()).map((ma) => ma.toString())}`,
      `Listening on: ${this.node.getListeningAddresses().map((ma) => ma.toString())}`,
      `${this.node.smartContractInfo()}`
    ].join('\n')
  }
}
