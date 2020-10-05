import { useState } from 'react'
import styles from '../styles/Header.module.scss'
import Connect from './Connect'
import store from '../utils/store'

export default function Header() {
  const [state] = store.useTracked()
  const [popupOpened, togglePopup] = useState<boolean>(false)

  return (
    <header className={`${styles.container} section`}>
      <div className={styles.logo}>
        <h1>HOPR Chat Demo</h1>
      </div>
      <div className={styles.settings}>
        {popupOpened ? (
          <Connect onConnect={() => togglePopup(false)} />
        ) : (
          <div className={styles.statusContainer}>
            <span className={styles.address}>{state.hoprAddress}</span>{' '}
            <span className={styles.status}>{state.connection}</span>
          </div>
        )}
        <span className={`${styles.cogwheel} clickable`} onClick={() => togglePopup(!popupOpened)}>
          ⚙️
        </span>
      </div>
    </header>
  )
}
