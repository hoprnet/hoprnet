import { getReq, postReq, delReq } from './client'

/*
 * Account API
 */
export const accountWithdraw = (jsonBody) => {
  postReq("account/withdraw", jsonBody)
}

export const getBalances = () => {
  getReq("account/balances")
}

export const getAddresses = () => {
  getReq("account/addresses")
}

/*
 * Aliases API
 */
export const getAliases = () => getReq("aliases")

export const setAliases = (peerId, alias) => {
  postReq("aliases/", {"peerId": peerId, "alias": alias})
}

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


/*
 * Messages API
 */
export const signAddress = (msg) => {
  postReq("messages/sign", {message: msg})
}

export const sendMessage = (body, recepient, path) => {
  postReq("messages", {body: body, recepient: recepient, path: path})
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