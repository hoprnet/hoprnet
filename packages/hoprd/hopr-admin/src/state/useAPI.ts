/*
  A react hook.
  Keeps the configuration of the API updated.
*/
import { useRef, useEffect } from 'react'
import { type Settings } from '../utils'
import API from '../utils/api'

const useAPI = (settings: Settings) => {
  const apiRef = useRef<API>(new API(settings.apiEndpoint, settings.apiToken))

  useEffect(() => {
    apiRef.current.updateConfig(settings.apiEndpoint, settings.apiToken)
  }, [settings.apiEndpoint, settings.apiToken])

  return {
    apiRef
  }
}

export default useAPI
