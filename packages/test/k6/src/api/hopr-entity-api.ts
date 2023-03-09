import { sleep } from 'k6'
import { RefinedParams, ResponseType } from 'k6/http'
import { HoprNode } from './hoprd.types'

/**
 * Abstract hopr action entity
 */
export abstract class HoprEntityApi {
  protected node: HoprNode

  protected params: RefinedParams<ResponseType> = {}

  public constructor(node: HoprNode) {
    this.node = node
    this.params = {
      headers: {
        'x-auth-token': node.apiToken,
        'Content-Type': 'application/json'
      },
      tags: {}
    }
  }

  protected addRequestTag(name: string, value: string) {
    this.params.tags ? [name] : value
  }

  protected addRequestHeader(name: string, value: string) {
    this.params.headers ? [name] : value
  }

  protected sleep(min = 1, max = 2) {
    sleep(Math.floor(Math.random() * (max - min) + min))
  }
}
