import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import { createTestApiInstance, invalidTestPeerId, testPeerId, testTicket } from '../../../fixtures'
import { STATUS_CODES } from '../../../utils'
// import { rewiremock } from '../../../rewiremock'
import rewiremock from 'rewiremock'
rewiremock.overrideEntryPoint(module)
// import * as formatTicketFile from '../../tickets'
// rewiremock.around(
//   () => import('./tickets'),
//   (mock) => {
//     mock(() => import('../../tickets')).with({ formatTicket: () => 'ticket' as any })
//   }
// )
// rewiremock.proxy('./tickets', (r) => ({
//   '../../tickets': r.with({ formatTicket: () => 'ticket' as any })
// }))

// rewiremock('./tickets').with({ formatTicket: () => 'ticket' as any })
// rewiremock.enable()

let node = sinon.fake() as any
node.getTickets = sinon.fake.returns([testTicket])

const { api, service } = createTestApiInstance(node)
chai.use(chaiResponseValidator(api.apiDoc))

describe('GET /channels/{peerId}/tickets', () => {
  it('should get tickets successfully', async () => {
    // const stub = sinon.stub(formatTicketFile, 'formatTicket')
    // stub.returns('ticket' as any)
    const res = await request(service).get(`/api/v2/channels/${testPeerId}/tickets`)
    // expect(res.status).to.equal(204)
    expect(res).to.satisfyApiSpec
    // expect(res.body).to.be.empty
    // expect(res.body).to.deep.equal({ muje: 'huje' })
  })

  it('should fail when no tickets to get', async () => {
    node.getTickets = sinon.fake.returns([])
    const res = await request(service).get(`/api/v2/channels/${testPeerId}/tickets`)
    expect(res.status).to.equal(404)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({ status: STATUS_CODES.TICKETS_NOT_FOUND })
  })

  it('should validate peerId', async () => {
    const res = await request(service).get(`/api/v2/channels/${invalidTestPeerId}/tickets`)
    expect(res.status).to.equal(400)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      status: STATUS_CODES.INVALID_PEERID
    })
  })

  it('should fail when node call fails', async () => {
    node.getTickets = sinon.fake.throws('')

    const res = await request(service).get(`/api/v2/channels/${testPeerId}/tickets`)
    expect(res.status).to.equal(422)
    expect(res).to.satisfyApiSpec
  })
})
