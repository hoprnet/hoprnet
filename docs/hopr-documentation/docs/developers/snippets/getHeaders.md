---
id: getHeaders.js
hide_title: true
---

```js title="getHeaders.js"
export const getHeaders = (securityToken = '^^LOCAL-testing-123^^', isPost = false) => {
  const headers = new Headers()
  if (isPost) {
    headers.set('Content-Type', 'application/json')
    headers.set('Accept-Content', 'application/json')
  }
  headers.set('Authorization', 'Basic ' + btoa(securityToken))
  return headers
}
```
