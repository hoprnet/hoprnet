import { Options } from 'k6/options'

import { HoprNode, Peer } from '../api/hoprd.types'
import execution from 'k6/execution'
import { ConstantTrafficNode } from './constant-traffic-node'
import { Counter } from 'k6/metrics'
import { Addresses } from '../api/hoprd.types'

const environmentName = __ENV.ENVIRONMENT_NAME
const testData = (JSON.parse(open(`./constant-traffic-${environmentName}.json`)))

const hoprNodes = (testData.nodes as HoprNode[])

// Test Options https://docs.k6.io/docs/options
export let options: Partial<Options> = {
  stages: [
    { target: 1, duration: testData.workload.warmup }, // Warmup stage
    { target: hoprNodes.length, duration: testData.workload.execution }, // Test execution stage
    { target: 0, duration: testData.workload.teardown } // Teardown stage
  ],
  setupTimeout: '3600000', // Timeout for the setup function
  // test thresholds https://docs.k6.io/docs/thresholds
  thresholds: testData.thresholds
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
  const hoprNode = new ConstantTrafficNode(hoprNodes[execution.vu.idInInstance - 1], dataPool)

  const hops = Math.floor(Math.random() * 3 + 1)
  hoprNode.sendHopMessage(hops, numberOfMessagesSuccessfullySent, numberOfSentMessagesFailed)
}

export function teardown() {
  console.log('teardown will still be called even when calling exec.test.abort()')
}
