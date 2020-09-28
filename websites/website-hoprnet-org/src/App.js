import React from 'react'
import { withRouter, Switch } from 'react-router-dom'
import { utils, views } from './components'

// Layouts
import LayoutDefault from './layouts/LayoutDefault'

// Views
import Home from './views/Home'
import HOPR from './views/HOPR'
import WeAre from './views/WeAre'
import ForYou from './views/ForYou'
import Node from './views/Node'
// import Sedimentum from './views/Sedimentum'
import Ecosystem from './views/Ecosystem'
import Support from './views/Support'
import NotFound from './views/NotFound'
import Partners from './views/Partners'
import Setup from './views/Setup'

const { AppRoute, ScrollReveal, ScrollToTop, insertScript } = utils
const { Disclaimer, Pdf } = views

class App extends React.Component {
  componentDidMount() {
    document.body.classList.add('is-loaded')
    this.refs.scrollReveal.init()

    // add fathom analytics
    const script = insertScript('https://panther.hoprnet.org/script.js')
    script.setAttribute('site', 'ZXTSKLDN')
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
              <AppRoute exact path="/layer0-data-privacy" component={WeAre} layout={LayoutDefault} />
              <AppRoute exact path="/do-business-with-HOPR" component={ForYou} layout={LayoutDefault} />
              <AppRoute exact path="/node" component={Node} layout={LayoutDefault} />
              {/* <AppRoute exact path="/sedimentum" component={Sedimentum} layout={LayoutDefault} /> */}
              <AppRoute exact path="/disclaimer" component={Disclaimer} layout={LayoutDefault} />
              <AppRoute exact path="/ecosystem" component={Ecosystem} layout={LayoutDefault} />
              <AppRoute exact path="/support" component={Support} layout={LayoutDefault} />
              <AppRoute exact path="/partners" component={Partners} layout={LayoutDefault} />
              <AppRoute exact path="/setup" component={Setup} layout={LayoutDefault} />
              <AppRoute
                exact
                path="/Chinese-Language-Binance-HOPR-Press-Release"
                component={Pdf('Chinese Binance HOPR Press Release.pdf')}
                layout={LayoutDefault}
              />
              <AppRoute
                exact
                path="/Korean-Language-Binance-HOPR-Press-Release"
                component={Pdf('Korean Binance HOPR Press Release.pdf')}
                layout={LayoutDefault}
              />
              <AppRoute
                exact
                path="/Japanese-Language-Binance-HOPR-Press-Release"
                component={Pdf('Japanese Binance HOPR Press Release.pdf')}
                layout={LayoutDefault}
              />
              <AppRoute component={NotFound} layout={LayoutDefault} />
            </Switch>
          )}
        />
      </>
    )
  }
}

export default withRouter(props => <App {...props} />)
