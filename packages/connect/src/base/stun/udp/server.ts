import { type Socket, type RemoteInfo } from 'dgram'
import { decode, constants, createMessage } from 'stun'
import debug from 'debug'

import { create_counter } from '@hoprnet/hopr-utils'

import { IPV4_EMBEDDED_ADDRESS, isStun } from '../../../utils/index.js'
import { isStunRequest, kStunTypeMask } from '../constants.js'

const metric_udpStunRequests = create_counter('connect_counter_udp_stun_requests', 'Number of UDP STUN requests')

const verbose = debug('hopr-connect:verbose:stun')

/**
 * Handles STUN requests
 * @param socket Node.JS socket to use
 * @param data received packet
 * @param rinfo Addr+Port of the incoming connection
 * @param __fakeRInfo [testing] overwrite incoming information to intentionally send misleading STUN response
 */
export function handleUdpStunRequest(
  socket: Socket,
  data: Buffer | Uint8Array,
  rinfo: RemoteInfo,
  __fakeRInfo?: RemoteInfo
): void {
  let replyAddress = rinfo.address

  // When using 'udp6' sockets, IPv4 addresses get prefixed by ::ffff:
  if (rinfo.family === 'IPv6') {
    const match = rinfo.address.match(IPV4_EMBEDDED_ADDRESS)

    if (match) {
      rinfo.family = 'IPv4'
      rinfo.address = match[0]
    }
  }

  if (!isStun(data)) {
    return
  }

  metric_udpStunRequests.increment()

  const request = decode(Buffer.isBuffer(data) ? data : Buffer.from(data.buffer, data.byteOffset, data.byteLength))

  switch (request.type & kStunTypeMask) {
    case isStunRequest:
      const response = createMessage(constants.STUN_BINDING_RESPONSE, request.transactionId)

      verbose(`Received ${request.isLegacy() ? 'legacy ' : ''}STUN request from ${rinfo.address}:${rinfo.port}`)

      let addrInfo = rinfo
      if (__fakeRInfo) {
        if (__fakeRInfo.family === 'IPv6') {
          const match = __fakeRInfo.address.match(IPV4_EMBEDDED_ADDRESS)

          if (match) {
            addrInfo = {
              ...rinfo,
              family: 'IPv4',
              port: __fakeRInfo.port,
              address: match[0]
            }
          } else {
            addrInfo = __fakeRInfo
          }
        } else {
          addrInfo = __fakeRInfo
        }
      }

      // To be compliant with RFC 3489
      if (request.isLegacy()) {
        // Copy magic STUN cookie as specified by RFC 5389
        // @ts-ignore issue with typings for Symbol index properties
        response[Symbol.for('kCookie')] = request[Symbol.for('kCookie')]
        response.addAttribute(constants.STUN_ATTR_MAPPED_ADDRESS, addrInfo.address, addrInfo.port)
        socket.send(response.toBuffer(), rinfo.port, replyAddress)
        return
      }

      let replyPort = addrInfo.port

      // RESPONSE_PORT can be 0
      const responsePort = request.getAttribute(constants.STUN_ATTR_RESPONSE_PORT)
      if (responsePort != undefined) {
        replyPort = responsePort.value as number
      }

      // Comply with RFC 5780
      response.addAttribute(constants.STUN_ATTR_MAPPED_ADDRESS, addrInfo.address, addrInfo.port)
      response.addAttribute(constants.STUN_ATTR_XOR_MAPPED_ADDRESS, addrInfo.address, addrInfo.port)

      // Allows multiplexing STUN protocol with other protocols
      response.addFingerprint()

      socket.send(response.toBuffer(), replyPort, replyAddress)

      break
    default:
      break
  }
}
