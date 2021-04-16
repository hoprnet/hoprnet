/**TODO This is extracted from example documentation
/* Here we need to actually implement the IPFS appender logic
/* And add types to this
*/

// This is the function that generates an appender function
function ipfsAppender(layout, timezoneOffset) {
  // This is the appender function itself
  return (loggingEvent) => {
    // Testing if appender works by adding custom value at every log
    console.log('HOPR IS AWESOME!!!!!!!!!!!!!!!!!!!')
    process.stdout.write(`${layout(loggingEvent, timezoneOffset)}\n`)
  }
}

// stdout configure doesn't need to use findAppender, or levels
function configure(config, layouts) {
  // the default layout for the appender
  let layout = layouts.colouredLayout
  // check if there is another layout specified
  if (config.layout) {
    // load the layout
    layout = layouts.layout(config.layout.type, config.layout)
  }
  //create a new appender instance
  return ipfsAppender(layout, config.timezoneOffset)
}

const configureModule = {
  configure: configure
}

export { configureModule as ipfsAppender }
