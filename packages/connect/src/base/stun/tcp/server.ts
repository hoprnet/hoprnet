import { type AddressInfo, type Socket, createConnection } from 'net'
import debug from 'debug'
import { decode, constants, createMessage } from 'stun'

import { isStun } from '../../../utils/index.js'
import { isStunRequest, kStunTypeMask } from '../constants.js'

const verbose = debug('hopr-connect:verbose:stun:tcp')

/**
 * Handles TCP STUN requests, mostly used by other nodes
 * to determine whether TCP socket is exposed to public.
 *
 * Called once TCP multiplexer detects STUN traffic
 *
 * @param socket TCP socket of incoming STUN traffic
 * @param __fakeRInfo [testing] overwrite incoming information to intentionally send misleading STUN response
 */
export async function handleTcpStunRequest(
  socket: Socket,
  stream: AsyncIterable<Uint8Array>,
  __fakeRInfo?: AddressInfo
): Promise<void> {
  let replyAddress = socket.remoteAddress as string

  const rinfo = {
    address: socket.remoteAddress as string,
    port: socket.remotePort as number,
    family: socket.remoteFamily
  }

  if (socket.remoteFamily === 'IPv6') {
    const match = rinfo.address.match(/(?<=::ffff:)[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}/)

    if (match) {
      rinfo.family = 'IPv4'
      rinfo.address = match[0]
    }
  }

  for await (const data of stream) {
    if (!isStun(data)) {
      return
    }

    const request = decode(Buffer.from(data.buffer, data.byteOffset, data.byteLength))

    switch (request.type & kStunTypeMask) {
      case isStunRequest:
        const response = createMessage(constants.STUN_BINDING_RESPONSE, request.transactionId)

        verbose(
          `Received ${request.isLegacy() ? 'legacy ' : ''}STUN request from ${socket.remoteAddress}:${
            socket.remotePort
          }`
        )

        let addrInfo = rinfo
        if (__fakeRInfo) {
          if (__fakeRInfo.family === 'IPv6') {
            const match = __fakeRInfo.address.match(/(?<=::ffff:)[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}/)

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
          socket.write(response.toBuffer())
          socket.end()
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

        if (responsePort != undefined) {
          const secondarySocket = createConnection(replyPort, replyAddress, () => {
            secondarySocket.write(response.toBuffer(), () => {
              secondarySocket.end()
            })
          })
          socket.end()
        } else {
          socket.write(response.toBuffer(), () => {
            socket.end()
          })
        }

        break
      default:
        break
    }
  }
}
