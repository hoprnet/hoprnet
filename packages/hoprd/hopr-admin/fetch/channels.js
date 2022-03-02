import { getReq, delReq, postReq } from './client'

export const getChannels = () => getReq("channels")

// DELETE /channels/{peerid}
export const closeChannel = (peerId) => delReq("channels/" + peerId)

// open cmd Opens a payment channel between this node and the counter party provided.
export const setChannels = (peerId, amount) => {
  postReq("channels/", {"peerId": peerId, "amount": amount})
}

// TODO: not sure which to use channels or tickets route
export const redeemTickets = (peerId) => postReq(`channels/${peerId}/tickets/redeem`)

// TODO: not sure which to use channels or /tickets route
export const getTickets = (peerId) =>  getReq(`channels/${peerId}/tickets`)