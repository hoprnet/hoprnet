import React from 'react'
import { withRouter, Switch } from 'react-router-dom'
import AppRoute from './utils/AppRoute'
import ScrollReveal from './utils/ScrollReveal'

// Layouts
import LayoutDefault from './layouts/LayoutDefault'

// Views
import Home from './views/Home'
import HOPR from './views/HOPR'

class App extends React.Component {
  componentDidMount() {
    document.body.classList.add('is-loaded')
    this.refs.scrollReveal.init()

    // add silkworm analytics
    let tracker = window.document.createElement('script')
    let firstScript = window.document.getElementsByTagName('script')[0]
    tracker.defer = true
    tracker.setAttribute('site', 'DJJBUEUD')
    tracker.setAttribute('spa', 'auto')
    tracker.src = 'https://silkworm.hoprnet.io/script.js'
    firstScript.parentNode.insertBefore(tracker, firstScript)
  }

  // Route change
  componentDidUpdate(prevProps) {
    if (this.props.location.pathname !== prevProps.location.pathname) {
      this.refs.scrollReveal.init()
    }
  }

  render() {
    return (
      <ScrollReveal
        ref="scrollReveal"
        children={() => (
          <Switch>
            <AppRoute exact path="/" component={Home} layout={LayoutDefault} />
            <AppRoute exact path="/HOPR" component={HOPR} layout={LayoutDefault} />
          </Switch>
        )}
      />
    )
  }
}

export default withRouter(props => <App {...props} />)
