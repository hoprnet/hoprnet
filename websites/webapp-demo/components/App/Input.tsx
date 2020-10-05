import { useState, useEffect } from 'react'
import classNames from 'classnames'
import styles from '../../styles/App/Input.module.scss'
import store from '../../utils/store'

export default function Input(props: { peerId?: string }) {
  const [state, dispatch] = store.useTracked()
  const [peerId, setPeerId] = useState('')
  const [message, setMessage] = useState('')
  const [anonymous, setAnonymous] = useState(true)
  const anonymousRoom = !props.peerId

  let hasPeerId = !!props.peerId
  useEffect(() => {
    hasPeerId = !!props.peerId
    setAnonymous(!hasPeerId)
  }, [props.peerId])

  const sendMessage = () => {
    if (props.peerId && !message) return
    else if (!props.peerId && (!peerId || !message)) return

    store.methods.sendMessage(state, dispatch, peerId === '' ? props.peerId : peerId, message, anonymous)

    setPeerId('')
    setMessage('')
  }

  const onKeyDown = (e: React.KeyboardEvent) => {
    if (e.key !== 'Enter') return
    sendMessage()
  }

  return (
    <div className={classNames(styles.container, 'section')} onKeyDown={onKeyDown}>
      {anonymousRoom ? (
        <input
          placeholder="peer id"
          className={styles.input}
          value={peerId}
          onChange={(e) => setPeerId(e.target.value)}
        />
      ) : undefined}
      <input
        placeholder="message"
        className={styles.input}
        value={message}
        onChange={(e) => setMessage(e.target.value)}
      />
      {!hasPeerId && (
        <div className={styles.checkbox}>
          <span>anonymous</span>
          <input type="checkbox" checked={anonymous} onChange={(e) => setAnonymous(e.target.checked)} />
        </div>
      )}
      <button className={classNames(styles.button, 'clickable')} onClick={sendMessage}>
        Send
      </button>
    </div>
  )
}
