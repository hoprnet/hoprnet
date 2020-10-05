import type { Dispatch } from 'react'
import { IActions, initialState, reducer, Provider, useTracked } from './state'
import MessageBuffer from '../message'
import * as api from '../api'
import { pseudoRandomId } from '../utils'

const methods: {
  [key: string]: (state: typeof initialState, dispatch: Dispatch<IActions>, ...args: any[]) => Promise<void>
} = {
  getHoprAddress: async (state, dispatch) => {
    const hoprAddress = await api.getHoprAddress(state.apiUrl)

    dispatch({
      type: 'SET_HOPR_ADDRESS',
      hoprAddress,
    })
  },

  initialize: async (state, dispatch) => {
    dispatch({
      type: 'RESET',
    })
    dispatch({
      type: 'SET_CONNECTION',
      connection: 'CONNECTING',
    })
    dispatch({
      type: 'SET_HOPR_ADDRESS',
      hoprAddress: undefined,
    })

    try {
      await methods.getHoprAddress(state, dispatch)
      dispatch({
        type: 'SET_CONNECTION',
        connection: 'CONNECTED',
      })
    } catch (err) {
      console.error(err)
      dispatch({
        type: 'SET_CONNECTION',
        connection: 'FAILED',
      })
    }

    const stream = await api.listenToMessages(state.apiUrl)

    stream.on('data', (res) => {
      const message = new MessageBuffer(res.getPayload_asU8())
      const { from } = message.toJson()
      const id = pseudoRandomId()

      dispatch({
        type: 'ADD_MESSAGE',
        id,
        counterParty: from,
        anonymous: from === '',
        sendByMe: false,
        message,
        createdAt: new Date(),
        status: 'SUCCESS',
      })
    })
  },

  sendMessage: async (state, dispatch, peerId: string, text: string, anonymous: boolean = false) => {
    const from = anonymous ? '' : state.hoprAddress
    const counterParty = anonymous ? '' : peerId
    const id = pseudoRandomId()

    const message = MessageBuffer.fromJson({
      from,
      text,
    })

    api
      .sendMessage(state.apiUrl, peerId, message)
      .then(() => {
        dispatch({
          type: 'UPDATE_MESSAGE_STATUS',
          id,
          counterParty,
          status: 'SUCCESS',
        })
      })
      .catch((err) => {
        console.error(err)
        dispatch({
          type: 'UPDATE_MESSAGE_STATUS',
          id,
          counterParty,
          status: 'FAILED',
        })
      })

    dispatch({
      type: 'ADD_MESSAGE',
      id,
      counterParty,
      anonymous,
      sendByMe: true,
      message,
      createdAt: new Date(),
      status: 'SENDING',
    })
  },
}

export default { initialState, reducer, methods, Provider, useTracked }
