const encoder = new TextEncoder();
const decoder = new TextDecoder();

export type IMessage = {
  from: string;
  text: string;
};

const isHoprAddress = (str: string): boolean => {
  if (!str.match(/16Uiu2HA.*?$/i)) return false;

  const [hoprAddress_regexed] = str.match(/16Uiu2HA.*?$/i);
  const hoprAddress = hoprAddress_regexed.substr(0, 53);
  if (hoprAddress !== str) return false;

  return true;
};

class Message extends Uint8Array {
  toU8a() {
    return new Uint8Array(this);
  }

  toJson(): IMessage {
    try {
      let from: string;
      let text: string;

      const first53 = decoder.decode(this.subarray(0, 53));

      if (isHoprAddress(first53)) {
        from = first53;
        text = decoder.decode(this.subarray(53, this.length));
      } else {
        from = "";
        text = decoder.decode(this);
      }

      return {
        from,
        text,
      };
    } catch (err) {
      console.error(err);
      throw Error("Unable to decode message");
    }
  }

  static fromJson(message: IMessage): Message {
    return new Message(encoder.encode(message.from + message.text));
  }
}

export default Message;
