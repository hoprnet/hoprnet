import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import { createTestApiInstance } from '../../fixtures.js'
import { privKeyToPeerId } from '@hoprnet/hopr-utils'
import type Hopr from '@hoprnet/hopr-core'
import { ResolvedEnvironment } from '@hoprnet/hopr-core'

const node = sinon.fake() as any as Hopr
const nodePeerId = privKeyToPeerId('0x9135f358f94b59e8cdee5545eb9ecc8ff32bc3a79227a09ee2bb6b50f1ad8159')

// Use random checksummed addresses to correctly mimic outputs
const HOPR_TOKEN_ADDRESS = '0x2be12eE6D553319F01Ea85A353203feC6444928F'
const HOPR_CHANNELS_ADDRESS = '0x39344CE336712bD0280c2C374c60A42F16a84B20'
const HOPR_NEWTWORK_REGISTRY_ADDRESS = '0xBEE1F5d64b562715E749771408d06D57EE0892A7'

describe('GET /node/configuration', () => {
  let service: any
  before(async function () {
    const loaded = await createTestApiInstance(node)

    service = loaded.service

    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })

  it('should get configuration', async () => {
    node.environment = { id: 'hardhat-localhost' } as ResolvedEnvironment
    node.getPublicHoprOptions = () => ({
      environment: 'hardhat-localhost',
      network: 'a'
    })
    node.smartContractInfo = sinon.fake.returns({
      network: 'a',
      hoprTokenAddress: HOPR_TOKEN_ADDRESS,
      hoprChannelsAddress: HOPR_CHANNELS_ADDRESS,
      hoprNetworkRegistryAddress: HOPR_NEWTWORK_REGISTRY_ADDRESS,
      channelClosureSecs: 60
    })
    node.getId = sinon.fake.returns(nodePeerId)
    node.isAllowedAccessToNetwork = sinon.fake.returns(Promise.resolve(true))

    const res = await request(service).get(`/api/v2/node/configuration`)
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      environment: 'hardhat-localhost',
      network: 'a',
      hoprToken: HOPR_TOKEN_ADDRESS,
      hoprChannels: HOPR_CHANNELS_ADDRESS,
      hoprNetworkRegistry: HOPR_NEWTWORK_REGISTRY_ADDRESS,
      isEligible: true,
      channelClosurePeriod: 1
    })
  })
})
