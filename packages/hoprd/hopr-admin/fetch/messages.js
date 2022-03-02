import { postReq } from './client'

export const signAddress = (msg) => {
  postReq("messages/sign", {message: msg})
}

export const sendMessage = (body, recepient, path) => {
  postReq("messages", {body: body, recepient: recepient, path: path})
}