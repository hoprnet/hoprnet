const insertScript = src => {
  let tracker = window.document.createElement('script')
  let firstScript = window.document.getElementsByTagName('script')[0]
  tracker.defer = true
  tracker.src = src

  if (typeof firstScript !== 'undefined') {
    firstScript.parentNode.insertBefore(tracker, firstScript)
  } else {
    // handle testing environment
    window.document.getElementsByTagName('body')[0].insertBefore(tracker, firstScript)
  }

  return tracker
}

export default insertScript
