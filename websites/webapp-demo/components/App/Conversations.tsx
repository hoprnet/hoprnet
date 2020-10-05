import styles from '../../styles/App/Conversations.module.scss'
import { store } from '../../utils'

export default function Conversations(props: { selected: string; setSelected: (peerId: string) => void }) {
  const [state] = store.useTracked()
  const peerIds = Array.from(state.conversations.keys())

  return (
    <div className={styles.container}>
      <div className={styles.list}>
        {peerIds.map((peerId) => {
          const display = peerId === '' ? 'Anonymous' : `..${peerId.substr(-7)}`
          const selected = peerId === props.selected

          return (
            <div
              key={peerId}
              className={`clickable ${selected ? styles.clicked : ''} ${styles.conversation}`}
              onClick={() => props.setSelected(peerId)}
            >
              {display}
            </div>
          )
        })}
      </div>
    </div>
  )
}
