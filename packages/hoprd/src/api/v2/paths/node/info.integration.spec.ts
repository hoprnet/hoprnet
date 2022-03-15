import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import { createTestApiInstance } from '../../fixtures'

let node = sinon.fake() as any
const { api, service } = createTestApiInstance(node)
chai.use(chaiResponseValidator(api.apiDoc))

describe('GET /node/info', () => {
  it('should get info', async () => {
    node.environment = { id: 'hardhat-localhost' }
    node.smartContractInfo = sinon.fake.returns({
      network: 'a',
      hoprTokenAddress: 'b',
      hoprChannelsAddress: 'c',
      channelClosureSecs: 60
    })
    node.getAddressesAnnouncedToDHT = sinon.fake.returns([1, 2])
    node.getListeningAddresses = sinon.fake.returns([3, 4])

    const res = await request(service).get(`/api/v2/node/info`)
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      environment: 'hardhat-localhost',
      announcedAddress: ['1', '2'],
      listeningAddress: ['3', '4'],
      network: 'a',
      hoprToken: 'b',
      hoprChannels: 'c',
      channelClosurePeriod: 1
    })
  })
})
