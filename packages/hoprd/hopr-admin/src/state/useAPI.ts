/*
  A react hook.
  Keeps the configuration of the API updated.
*/
import { useRef, useEffect } from 'react'
import { type Configuration } from '../utils'
import API from '../utils/api'

const useAPI = (config: Configuration) => {
  const apiRef = useRef<API>(new API(config.apiEndpoint, config.apiToken))

  useEffect(() => {
    apiRef.current = new API(config.apiEndpoint, config.apiToken)
  }, [config.apiEndpoint, config.apiToken])

  return {
    apiRef
  }
}

export default useAPI
