import { Public } from './types'

declare interface PathFinder {
  findPath(start: Public, targetLength?: number): Promise<Public[]>
}

export default PathFinder
