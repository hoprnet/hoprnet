import { useReducer } from 'react'
import { createContainer } from 'react-tracked'
import { API_URL } from '../env'
import IMessageBuffer from '../message'

export type IMessage = {
  id: string
  counterParty: string
  anonymous: boolean
  sendByMe: boolean
  message: IMessageBuffer
  createdAt: Date
  status: 'SENDING' | 'SUCCESS' | 'FAILED'
}

export const initialState: {
  apiUrl: string
  connection: 'CONNECTING' | 'CONNECTED' | 'FAILED' | 'DISCONNECTED'
  hoprAddress?: string
  conversations: Map<string, Map<string, IMessage>> // [counterParty, [messageId, message]]
} = {
  apiUrl: API_URL,
  connection: 'DISCONNECTED',
  hoprAddress: undefined,
  conversations: new Map([['', new Map()]]), // add anonymous conversation
}

export type IActions =
  | {
      type: 'RESET'
    }
  | {
      type: 'SET_API_URL'
      apiUrl: typeof initialState['apiUrl']
    }
  | {
      type: 'SET_CONNECTION'
      connection: typeof initialState['connection']
    }
  | {
      type: 'SET_HOPR_ADDRESS'
      hoprAddress: typeof initialState['hoprAddress']
    }
  | ({
      type: 'ADD_MESSAGE'
      id: string
    } & IMessage)
  | {
      type: 'UPDATE_MESSAGE_STATUS'
      id: string
      counterParty: string
      status: IMessage['status']
    }

export const reducer = (state: typeof initialState, action: IActions): typeof initialState => {
  switch (action.type) {
    case 'RESET':
      return { ...state, conversations: new Map(initialState.conversations) }
    case 'SET_API_URL':
      return { ...state, apiUrl: action.apiUrl }
    case 'SET_CONNECTION':
      return { ...state, connection: action.connection }
    case 'SET_HOPR_ADDRESS':
      return { ...state, hoprAddress: action.hoprAddress }
    case 'ADD_MESSAGE': {
      const conversations = new Map(state.conversations)
      const conversation: Map<string, IMessage> = conversations.has(action.counterParty)
        ? new Map(conversations.get(action.counterParty))
        : new Map()

      // update conversation
      conversation.set(action.id, {
        id: action.id,
        counterParty: action.counterParty,
        anonymous: action.anonymous,
        sendByMe: action.sendByMe,
        message: action.message,
        createdAt: action.createdAt,
        status: action.status,
      })

      // update conversations
      conversations.set(action.counterParty, conversation)

      return {
        ...state,
        conversations,
      }
    }
    case 'UPDATE_MESSAGE_STATUS': {
      const conversations = new Map(state.conversations)
      const conversation: Map<string, IMessage> = conversations.has(action.counterParty)
        ? new Map(conversations.get(action.counterParty))
        : undefined
      if (!conversation) return state

      const message: IMessage = conversation.has(action.id)
        ? {
            ...conversation.get(action.id),
          }
        : undefined
      if (!message) return state

      message.status = action.status
      conversation.set(action.id, message)
      conversations.set(action.counterParty, conversation)

      return {
        ...state,
        conversations,
      }
    }
    default:
      throw new Error(`unknown action type: ${action!.type}`)
  }
}

const useValue = (ops: { reducer: typeof reducer; initialState: typeof initialState }) => {
  return useReducer(ops.reducer, ops.initialState)
}

const { Provider, useTracked } = createContainer(useValue)

export { Provider, useTracked }
