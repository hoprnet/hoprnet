import classNames from "classnames";
import type { IMessage } from "../../utils/store/state";
import styles from "../../styles/App/Chat.module.css";
import { store } from "../../utils";

export default function Chat(props: { peerId?: string }) {
  const [state] = store.useTracked();
  const messages: Map<string, IMessage> =
    state.conversations.get(props.peerId ?? "") ?? new Map();

  const sortedByLatest = Array.from(messages.entries()).sort(([, a], [, b]) => {
    return a.createdAt.valueOf() - b.createdAt.valueOf();
  });

  return (
    <div className={classNames(styles.container, "section")}>
      {sortedByLatest.map(([id, message]) => {
        const { from, text } = message.message.toJson();
        const hasFrom = from !== "";
        const fromMe = from === state.hoprAddress;

        return (
          <div
            key={`${from}-${id}`}
            className={classNames(
              styles.message,
              hasFrom ? (fromMe ? styles.send : styles.received) : undefined
            )}
          >
            {text} {fromMe ? `(${message.status})` : null}
          </div>
        );
      })}
    </div>
  );
}
