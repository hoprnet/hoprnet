import { getReq, postReq } from './client'


export const accountWithdraw = (jsonBody) => {
  postReq("account/withdraw", jsonBody)
}

export const getBalances = () => {
  getReq("account/balances")
}

export const getAddresses = () => {
  getReq("account/addresses")
}