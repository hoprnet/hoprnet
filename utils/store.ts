import { useReducer, Dispatch } from "react";
import { createContainer } from "react-tracked";
import { API_URL } from "./env";
import Message from "./message";
import * as api from "./api";

const initialState: {
  apiUrl: string;
  connection: "CONNECTED" | "DISCONNECTED" | "DISCONNECTED";
  hoprAddress?: string;
  conversations: Map<string, Message[]>;
} = {
  apiUrl: API_URL,
  connection: "DISCONNECTED",
  hoprAddress: undefined,
  conversations: new Map(),
};

type IActions =
  | {
      type: "SET_API_URL";
      apiUrl: typeof initialState["apiUrl"];
    }
  | {
      type: "SET_CONNECTION";
      connection: typeof initialState["connection"];
    }
  | {
      type: "SET_HOPR_ADDRESS";
      hoprAddress: typeof initialState["hoprAddress"];
    }
  | {
      type: "UPDATE_CONVERSATION";
      counterParty: string;
      message: Message;
    };

const reducer = (
  state: typeof initialState,
  action: IActions
): typeof initialState => {
  switch (action.type) {
    case "SET_API_URL":
      return { ...state, apiUrl: action.apiUrl };
    case "SET_CONNECTION":
      return { ...state, connection: action.connection };
    case "SET_HOPR_ADDRESS":
      return { ...state, hoprAddress: action.hoprAddress };
    case "UPDATE_CONVERSATION": {
      const conversation = [
        ...(state.conversations.get(action.counterParty) ?? []),
      ];
      conversation.push(action.message);
      const conversations = new Map(Array.from(state.conversations.entries()));
      conversations.set(action.counterParty, conversation);

      return {
        ...state,
        conversations,
      };
    }
    default:
      throw new Error(`unknown action type: ${action!.type}`);
  }
};

const methods: {
  [key: string]: (
    state: typeof initialState,
    dispatch: Dispatch<IActions>,
    ...args: any[]
  ) => Promise<void>;
} = {
  getHoprAddress: async (state, dispatch) => {
    const hoprAddress = await api.getHoprAddress(state.apiUrl);

    dispatch({
      type: "SET_HOPR_ADDRESS",
      hoprAddress,
    });
  },

  initialize: async (state, dispatch) => {
    dispatch({
      type: "SET_CONNECTION",
      connection: "DISCONNECTED",
    });
    dispatch({
      type: "SET_HOPR_ADDRESS",
      hoprAddress: undefined,
    });

    await methods.getHoprAddress(state, dispatch);

    dispatch({
      type: "SET_CONNECTION",
      connection: "CONNECTED",
    });

    const stream = await api.listenToMessages(state.apiUrl);

    stream.on("data", (res) => {
      const message = new Message(res.getPayload_asU8());
      const { from: counterParty } = message.toJson();

      dispatch({
        type: "UPDATE_CONVERSATION",
        counterParty,
        message,
      });
    });
  },

  sendMessage: async (
    state,
    dispatch,
    peerId: string,
    text: string,
    anonymous: boolean = false
  ) => {
    const counterParty = anonymous ? "" : state.hoprAddress;

    const message = Message.fromJson({
      from: counterParty,
      text,
    });

    await api.sendMessage(state.apiUrl, peerId, message);

    dispatch({
      type: "UPDATE_CONVERSATION",
      counterParty,
      message,
    });
  },
};

const useValue = (ops: {
  reducer: typeof reducer;
  initialState: typeof initialState;
}) => {
  return useReducer(ops.reducer, ops.initialState);
};

const { Provider, useTracked } = createContainer(useValue);

export default { initialState, reducer, methods, Provider, useTracked };
