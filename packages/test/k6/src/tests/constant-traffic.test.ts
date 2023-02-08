import { Options } from 'k6/options'

import { HoprNode, Peer } from '../api/hoprd.types'
import execution from 'k6/execution'
import { ConstantTrafficNode } from './constant-traffic-node'
import { Counter } from 'k6/metrics'
import { Addresses } from '../api/hoprd.types'

const SCENARIO_NAME = 'constant-traffic'
const hoprNodes = (JSON.parse(open('./test-data.json')).nodes as HoprNode[]).filter(
  (hoprNode: HoprNode) => hoprNode.scenarios.indexOf(SCENARIO_NAME) != 1
)

// Test Options https://docs.k6.io/docs/options
export let options: Partial<Options> = {
  stages: [
    { target: 1, duration: '2s' }, // Warmup stage
    { target: hoprNodes.length, duration: '15s' }, // Test execution stage
    { target: 0, duration: '2s' } // Teardown stage
  ],
  setupTimeout: '3600000',
  // test thresholds https://docs.k6.io/docs/thresholds
  thresholds: {
    http_req_duration: ['avg<500', 'p(95)<1500'], // 95% Percentil below 1500 ms and average below 500 ms
    http_req_failed: ['rate<0.05'], // Less than 5 %
    NumberOfMessagesSuccessfullySent: ['rate<0.96'], // More than 96%
    NumberOfSentMessagesFailed: ['count<10'] // Less than 10
  }
}

let numberOfMessagesSuccessfullySent = new Counter('NumberOfMessagesSuccessfullySent')
let numberOfSentMessagesFailed = new Counter('NumberOfSentMessagesFailed')

// The Setup Function is run once before the Load Test https://docs.k6.io/docs/test-life-cycle
export function setup() {
  let peers: { [key: string]: Peer[] } = {}
  let addresses: { [key: string]: Addresses } = {}
  hoprNodes.forEach((node: HoprNode) => {
    const hoprNode = new ConstantTrafficNode(node)
    hoprNode.checkHealth()
    peers[node.alias] = hoprNode.getQualityPeers('announced')
    addresses[node.alias] = hoprNode.addresses
  })

  // anything returned here can be imported into the default function https://docs.k6.io/docs/test-life-cycle
  return { peers, addresses }
}

// This function is executed for each iteration
// default function imports the return data from the setup function https://docs.k6.io/docs/test-life-cycle
export default (dataPool: { addresses: { [key: string]: Addresses }; peers: { [key: string]: Peer[] } }) => {
  console.log(`VU Instance Id: ${execution.vu.idInInstance}`)
  const hoprNode = new ConstantTrafficNode(hoprNodes[execution.vu.idInInstance - 1], dataPool)

  const hops = Math.floor(Math.random() * 3 + 1)
  hoprNode.sendHopMessage(hops, numberOfMessagesSuccessfullySent, numberOfSentMessagesFailed)
}

export function teardown() {
  console.log('teardown will still be called even when calling exec.test.abort()')
}
