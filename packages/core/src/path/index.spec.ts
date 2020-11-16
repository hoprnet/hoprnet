import assert from 'assert'
import { randomSubset } from '@hoprnet/hopr-utils'
import PeerId from 'peer-id'
import { findPath, Channel } from '.'


async function generateGraph(nodesCount: number) {
  const nodes = []

  for (let i = 0; i < nodesCount; i++) {
    nodes.push(await PeerId.create({bits: 512}))
  }

  const edges = new Map<PeerId, PeerId[]>()

  console.log('add edges')

  // Random graph
  nodes.forEach(n => {
    edges.set(n, randomSubset(nodes, 5).filter(x => !x.equals(n)))
  })
  return { nodes, edges }
}

function validPath(path: PeerId[], edges: Map<PeerId, PeerId[]>) {
  for (let i = 0; i < path.length - 1; i++) {
    const edgeSet = edges.get(path[i])

    if (edgeSet == null || !edgeSet.includes(path[i + 1])) {
      return false
    }
  }

  return true
}

function noCircles(path: PeerId[]) {
  for (let i = 1; i < path.length; i++) {
    if (path.slice(0, i).includes(path[i]) || path.slice(i + 1).includes(path[i])) {
      return false
    }
  }

  return true
}

describe('test pathfinder', function () {
  it('should find a path', async function () {
    console.log('generateing graph')
    const { nodes, edges } = await generateGraph(10)
    console.log('generated')
    const getChannelsFromPeer = (a: PeerId): Promise<Channel[]> => Promise.resolve(edges.get(a).map((b) => [a, b, 0]))
    const path = await findPath(nodes[0], undefined, 3, undefined, getChannelsFromPeer)

    assert(
      path.length == 3 && noCircles(path) && validPath(path, edges),
      'Should find a valid acyclic path that goes through all nodes'
    )
  })
})
