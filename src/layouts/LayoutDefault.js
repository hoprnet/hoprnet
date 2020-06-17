import React from 'react'
import Header from '../components/layout/Header'
import Footer from '../components/layout/Footer'

const LayoutDefault = ({ children }) => (
  <React.Fragment>
    <Header navPosition="right" className="reveal-from-top" hideSignin hasBgColor invertColor sticky />
    <main className="site-content">{children}</main>
    <Footer />
  </React.Fragment>
)

export default LayoutDefault
