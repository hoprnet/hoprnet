import sinon from 'sinon'
import assert from 'assert'
import { ping } from './ping'
import { isError, _createTestState } from '.'

const peerId = '16Uiu2HAmRFjDov6sbcZeppbnNFFTdx5hFoBzr8csBgevtKUex8y9'
const invalidPeerId = 'definetly not a valid peerId'

let node = sinon.fake() as any

describe("ping", () => {
    it("should ping successfuly", async () => {
        const state = _createTestState()
        node.ping = sinon.fake.returns({ latency: 10 })
        const res = await ping({ node, state, peerId })
        if (isError(res)) throw new Error()
        assert.equal(res.latency, 10)
    })
    it("should return error on invalid peerId", async () => {
        const state = _createTestState()
        node.ping = sinon.fake.returns({ latency: 10 })
        const err = await ping({ node, state, peerId: invalidPeerId })
        if (!isError(err)) throw new Error()
        assert.equal(err.message, "invalidPeerId")
    })
    it("should return propper error on ping fail", async () => {
        const state = _createTestState()
        node.ping = sinon.fake.throws("")
        const err = await ping({ node, state, peerId })
        if (!isError(err)) throw new Error()
        assert.equal(err.message, "failure")
    })
})