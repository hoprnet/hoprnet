'use strict'

const registerOpeningRequest = require('./open')
const registerSettleRequest = require('./settle')

module.exports = (node) => {
    registerOpeningRequest(node)
    registerSettleRequest(node)
}