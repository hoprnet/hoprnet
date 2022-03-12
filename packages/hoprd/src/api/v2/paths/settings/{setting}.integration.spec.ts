import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import { createTestApiInstance } from '../../fixtures'
import { STATUS_CODES } from '../../utils'
import { SettingKey } from '../../../../types'

let node = sinon.fake() as any
const { api, service } = createTestApiInstance(node)
chai.use(chaiResponseValidator(api.apiDoc))

describe('PUT /settings/{setting}', () => {
  it('should set setting successfuly', async () => {
    const res = await request(service)
      .put(`/api/v2/settings/${SettingKey.INCLUDE_RECIPIENT}`)
      .send({ settingValue: true })
    expect(res.status).to.equal(204)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.be.empty
  })

  it('should return error when invalid setting key is provided', async () => {
    const res = await request(service).put(`/api/v2/settings/invalidKey`).send({ settingValue: true })
    expect(res.status).to.equal(400)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({ status: STATUS_CODES.INVALID_SETTING })
  })

  it('should throw error when invalid value provided', async () => {
    const res = await request(service)
      .put(`/api/v2/settings/${SettingKey.INCLUDE_RECIPIENT}`)
      .send({ settingValue: 'true' })
    expect(res.status).to.equal(400)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({ status: STATUS_CODES.INVALID_SETTING_VALUE })

    const res2 = await request(service).put(`/api/v2/settings/${SettingKey.STRATEGY}`).send({ settingValue: 'abcd' })
    expect(res2.status).to.equal(400)
    expect(res2).to.satisfyApiSpec
    expect(res2.body).to.deep.equal({ status: STATUS_CODES.INVALID_SETTING_VALUE })
  })
})
