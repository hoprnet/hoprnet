export const createClient = (baseEndpoint) => {
  const makeRequest = (endpoint, options) => {
    return fetch(`${baseEndpoint}${endpoint}`, {
      credentials: 'same-origin',
      ...options
    }).then(async (response) => {
      if (!response.ok) {
        const errorMessage = await response.json()
        throw new Error(errorMessage || response.statusText)
      }
      return response.json()
    })
  }

  return {
    get: (endpoint) => {
      return makeRequest(endpoint, { method: 'GET' })
    }
  }
}
