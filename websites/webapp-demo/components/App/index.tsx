import { useEffect, useState } from 'react'
import styles from '../../styles/App/index.module.scss'
import Conversations from './Conversations'
import Chat from './Chat'
import Input from './Input'
import store from '../../utils/store'

export default function App() {
  const [state, dispatch] = store.useTracked()
  const [selected, setSelected] = useState('')

  useEffect(() => {
    store.methods.initialize(state, dispatch)
  }, [])

  return (
    <div className={styles.container}>
      <Conversations selected={selected} setSelected={setSelected} />
      <div className={styles.secondaryContainer}>
        <Chat peerId={selected} />
        <Input peerId={selected} />
      </div>
    </div>
  )
}
