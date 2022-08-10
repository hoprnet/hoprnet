/*
  A react hook.
  Manages the dApps state.
*/
import type { Aliases } from '../utils/api'
import { useState, useMemo } from 'react'
import cookies from 'js-cookie'
import useAPI from './useAPI'
import useWS from './useWS'
import { type Configuration, isSSR, getUrlParams, API_TOKEN_COOKIE } from '../utils'

const useAppState = () => {
  // search for parameters from url
  const urlParams = !isSSR ? getUrlParams(location) : {}
  // search for apiToken in cookies
  const apiTokenFromCookies = cookies.get(API_TOKEN_COOKIE)

  const [state, setState] = useState<{
    config: Configuration
    aliases: Aliases
  }>({
    config: {
      apiEndpoint: urlParams.apiEndpoint || 'http://localhost:3001/',
      apiToken: urlParams.apiToken || apiTokenFromCookies || undefined
    },
    aliases: {}
  })

  // initialize API
  const api = useAPI(state.config)

  // initialize websocket connections
  const streamWS = useWS(state.config, '/api/v2/node/stream/websocket')
  const messagesWS = useWS(state.config, '/api/v2/messages/websocket')

  /**
   * Updates the app's connectivity config.
   * Changes are propagated to all hooks.
   */
  const updateConfig = (update: (prevConfig: Configuration) => Configuration) => {
    setState((state) => {
      state.config = update(state.config)
      return state
    })
  }

  /**
   * Updates the app's aliases.
   */
  const updateAliases = (update: (prevAliases: Aliases) => Aliases) => {
    setState((state) => {
      state.aliases = update(state.aliases)
      return state
    })
  }

  /**
   * Connection status of the app.
   * Takes into account all WS connections.
   */
  const status = useMemo<'DISCONNECTED' | 'CONNECTED'>(() => {
    if (streamWS.state.status === 'CONNECTED' && messagesWS.state.status === 'CONNECTED') return 'CONNECTED'
    return 'DISCONNECTED'
  }, [streamWS.state.status, messagesWS.state.status])

  return {
    state,
    api,
    streamWS,
    messagesWS,
    updateConfig,
    updateAliases,
    status
  }
}

export default useAppState
