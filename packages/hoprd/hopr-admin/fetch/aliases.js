import { getReq, postReq } from './client'

export const getAliases = () => getReq("aliases")

export const setAliases = (peerId, alias) => {
  postReq("aliases/", {"peerId": peerId, "alias": alias})
}