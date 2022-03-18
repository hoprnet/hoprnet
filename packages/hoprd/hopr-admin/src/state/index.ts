/*
  A react hook.
  Manages the dApps state.
*/
import { useState, useMemo } from 'react'
import cookies from 'js-cookie'
import useAPI from './useAPI'
import useWS from './useWS'
import { type Settings, isSSR, getUrlParams, API_TOKEN_COOKIE } from '../utils'

const useAppState = () => {
  // search for parameters from url
  const urlParams = !isSSR ? getUrlParams(location) : {}
  // search for apiToken in cookies
  const apiTokenFromCookies = cookies.get(API_TOKEN_COOKIE)

  const [state, setState] = useState<{
    settings: Settings
    aliases: Record<string, string>
  }>({
    settings: {
      apiEndpoint: urlParams.apiEndpoint || 'http://localhost:3001/',
      apiToken: urlParams.apiToken || apiTokenFromCookies || undefined
    },
    aliases: {}
  })

  // initialize API
  const api = useAPI(state.settings)

  // initialize websocket connections
  const streamWS = useWS(state.settings, '/api/v2/node/stream/websocket')
  const messagesWS = useWS(state.settings, '/api/v2/messages/websocket')

  /**
   * Updates the app's settings.
   * Changes are propagated to all hooks.
   */
  const updateSettings = (newSettings: Partial<Settings>) => {
    setState((state) => {
      for (const [k, v] of Object.entries(newSettings)) {
        state.settings[k] = v
      }
      return state
    })
  }

  /**
   * Updates the app's aliases.
   */
  const updateAliases = (newAliases: Record<string, string>) => {
    setState((state) => {
      for (const [k, v] of Object.entries(newAliases)) {
        state.aliases[k] = v
      }
      return state
    })
  }

  /**
   * Connection status of the app.
   * Takes into account all WS connections.
   */
  const status = useMemo<'DISCONNECTED' | 'CONNECTED'>(() => {
    console.log('USE MEMO', streamWS.state.status, messagesWS.state.status)
    if (streamWS.state.status === 'CONNECTED' && messagesWS.state.status === 'CONNECTED') return 'CONNECTED'
    return 'DISCONNECTED'
  }, [streamWS.state.status, messagesWS.state.status])

  return {
    state,
    api,
    streamWS,
    messagesWS,
    updateSettings,
    updateAliases,
    status
  }
}

export default useAppState
