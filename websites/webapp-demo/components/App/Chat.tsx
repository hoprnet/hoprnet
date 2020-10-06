import classNames from 'classnames'
import type { IMessage } from '../../utils/store/state'
import styles from '../../styles/App/Chat.module.scss'
import { store } from '../../utils'

export default function Chat(props: { peerId?: string }) {
  const anonymousChat = !props.peerId
  const [state] = store.useTracked()
  const messagesMap: Map<string, IMessage> = state.conversations.get(props.peerId ?? '') ?? new Map()

  const messages = Array.from(messagesMap.entries())
    .filter(([, { anonymous }]) => {
      if (anonymousChat) {
        return anonymous
      } else {
        return true
      }
    })
    .sort(([, a], [, b]) => {
      return a.createdAt.valueOf() - b.createdAt.valueOf()
    })

  return (
    <div className={classNames(styles.container, 'section')}>
      {messages.map(([id, { status, message, sendByMe }]) => {
        const { from, text } = message.toJson()

        return (
          <div key={`${from}-${id}`} className={classNames(styles.message, sendByMe ? styles.send : styles.received)}>
            {text}
            <br />
            {sendByMe ? `(${status})` : null}
          </div>
        )
      })}
    </div>
  )
}
