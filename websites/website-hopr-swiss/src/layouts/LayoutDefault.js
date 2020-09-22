import React from 'react'
import { layout } from '../components'

const { Header, Footer } = layout

const LayoutDefault = ({ children }) => (
  <React.Fragment>
    <Header navPosition="right" className="reveal-from-top" hideSignin hasBgColor invertColor sticky />
    <main className="site-content">{children}</main>
    <Footer />
  </React.Fragment>
)

export default LayoutDefault
