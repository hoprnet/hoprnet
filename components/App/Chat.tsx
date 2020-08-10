import styles from "../../styles/App/Chat.module.css";
import { store } from "../../utils";

export default function Chat(props: { peerId?: string }) {
  const [state] = store.useTracked();
  const messages = state.conversations.get(props.peerId ?? "") ?? [];

  return (
    <div className={`${styles.container} section`}>
      {messages.map((message, index) => {
        const { from, text } = message.toJson();
        const fromMe = from === state.hoprAddress;

        return (
          <p key={index}>
            {from ? (fromMe ? ">" : "<") : undefined} {text}
          </p>
        );
      })}
    </div>
  );
}
