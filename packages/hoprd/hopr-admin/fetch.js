/*
 * Single file for re-usability purposes of API in hopr-admin
 */

import { getReq, postReq, delReq, putReq } from './client'

/*
 * Account API
 */
export const accountWithdraw = (jsonBody) => postReq("account/withdraw", jsonBody)


export const getBalances = () => getReq("account/balances")

export const getAddresses = () => getReq("account/addresses")


/*
 * Aliases API
 */
export const getAliases = () => getReq("aliases")

export const setAliases = (peerId, alias) => postReq("aliases/", {"peerId": peerId, "alias": alias})


/*
 * Channels API
 */
export const getChannels = () => getReq("channels")

// DELETE /channels/{peerid}
export const closeChannel = (peerId) => delReq("channels/" + peerId)

// open cmd Opens a payment channel between this node and the counter party provided.
export const setChannels = (peerId, amount) => {
  postReq("channels/", {"peerId": peerId, "amount": amount})
}

/*
 * Tickets API
 */
export const redeemTickets = () => postReq(`tickets/redeem`)

export const getTickets = () =>  getReq(`tickets`)

export const getTicketStats = () => getReq(`tickets/statistics`)

/*
 * Messages API
 */
export const signMessage = (msg) => postReq("messages/sign", {message: msg})


export const sendMessage = (body, recipient, path) => {
  return postReq("messages", {body: body, recipient: recipient, path: path})
}

/*
 * Node API
 */
export const getNodeInfo = () => getReq("node/info")

export const getNodeVer = () => getReq("node/version")

export const pingNodePeer = (peerId) => postReq("node/ping", {peerId: peerId})

/*
 * Settings API
 */
export const getSettings = () => getReq("settings")

export const setSettings = (key, value) => putReq(`settings/${key}`, {key: key, value: value})