import { ReactNode } from 'react'
import styles from '../styles/Layout.module.scss'
import Header from './Header'
import Footer from './Footer'
import { store } from '../utils'

export default function Layout(props: { children?: ReactNode }) {
  return (
    <store.Provider initialState={store.initialState} reducer={store.reducer}>
      <div className={styles.container}>
        <Header />
        <main>{props.children}</main>
        <Footer />
      </div>
    </store.Provider>
  )
}
