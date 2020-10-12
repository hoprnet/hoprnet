import { Public } from './types'

declare interface PathSelection {
  /**
   * Samples a path with at most targetLength nodes.
   *
   * @param start node to start from
   * @param targetLength desired target length
   * @param maxIterations amount of iterations before cancelling search
   * @param filter only include nodes that pass this truthy test
   */
  findPath(
    start: Public,
    targetLength: number,
    maxIterations: number,
    filter?: (node: Public) => boolean
  ): Promise<Public[]>
}

export default PathSelection
