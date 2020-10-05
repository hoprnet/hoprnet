// @TODO: support SSR
function getApiUrlFromUrl() {
  let apiUrl: string

  try {
    apiUrl = new URLSearchParams(window?.location?.search).get('apiUrl')
  } catch {}

  return apiUrl
}

/**
 * search for API_URL in this priority: URL < ENV < DEFAULT
 */
export const API_URL = getApiUrlFromUrl() ?? process.env.API_URL ?? 'http://127.0.0.1:8080'
