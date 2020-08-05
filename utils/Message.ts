const encoder = new TextEncoder();
const decoder = new TextDecoder();

class Message extends Uint8Array {
  toU8a() {
    return new Uint8Array(this);
  }

  toText(): string {
    return decoder.decode(this);
  }

  static fromText(message: string): Message {
    return new Message(encoder.encode(message));
  }
}

export default Message;
