import React from 'react'
import { withRouter, Switch } from 'react-router-dom'
import { utils, views } from './components'

// Layouts
import LayoutDefault from './layouts/LayoutDefault'

// Views
import Home from './views/Home'
import HOPR from './views/HOPR'

const { AppRoute, ScrollReveal, ScrollToTop, insertScript } = utils
const { Disclaimer } = views

class App extends React.Component {
  componentDidMount() {
    document.body.classList.add('is-loaded')
    this.refs.scrollReveal.init()

    // add fathom analytics
    const script = insertScript('https://coyote.hopr.swiss/script.js')
    script.setAttribute('site', 'OXJHRYAB')
    script.setAttribute('spa', 'auto')
  }

  // Route change
  componentDidUpdate(prevProps) {
    if (this.props.location.pathname !== prevProps.location.pathname) {
      this.refs.scrollReveal.init()
    }
  }

  render() {
    return (
      <>
        <ScrollToTop />
        <ScrollReveal
          ref="scrollReveal"
          children={() => (
            <Switch>
              <AppRoute exact path="/" component={Home} layout={LayoutDefault} />
              <AppRoute exact path="/who-is-HOPR" component={HOPR} layout={LayoutDefault} />
              <AppRoute exact path="/disclaimer" component={Disclaimer} layout={LayoutDefault} />
            </Switch>
          )}
        />
      </>
    )
  }
}

export default withRouter(props => <App {...props} />)
