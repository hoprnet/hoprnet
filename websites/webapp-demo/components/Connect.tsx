import styles from '../styles/Connect.module.scss'
import { store } from '../utils'

export default function Connect(props: { onConnect?: () => any }) {
  const [state, dispatch] = store.useTracked()

  return (
    <div className={styles.container}>
      <input defaultValue={state.apiUrl} onChange={(e) => dispatch({ type: 'SET_API_URL', apiUrl: e.target.value })} />
      <button
        onClick={() => {
          store.methods.initialize(state, dispatch)
          if (props.onConnect) props.onConnect()
        }}
      >
        Connect
      </button>
    </div>
  )
}
