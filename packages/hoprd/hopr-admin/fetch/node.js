import { getReq, postReq } from './client'

export const getNodeInfo = () => getReq("node/info")

export const getNodeVer = () => getReq("node/version")

export const pingNodePeer = (peerId) => postReq("node/ping", {peerId: peerId})