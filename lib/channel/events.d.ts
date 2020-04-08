/// <reference types="node" />
declare const addEvent: <N extends "SecretHashSet" | "FundedChannel" | "OpenedChannel" | "InitiatedChannelClosure" | "ClosedChannel" | "allEvents">(name: N, event: ReturnType<{
    SecretHashSet: import("../tsc/web3/types").ContractEvent<{
        account: string;
        secretHash: string;
        0: string;
        1: string;
    }>;
    FundedChannel: import("../tsc/web3/types").ContractEvent<{
        funder: string;
        recipient: string;
        counterParty: string;
        recipientAmount: string;
        counterPartyAmount: string;
        0: string;
        1: string;
        2: string;
        3: string;
        4: string;
    }>;
    OpenedChannel: import("../tsc/web3/types").ContractEvent<{
        opener: string;
        counterParty: string;
        0: string;
        1: string;
    }>;
    InitiatedChannelClosure: import("../tsc/web3/types").ContractEvent<{
        initiator: string;
        counterParty: string;
        closureTime: string;
        0: string;
        1: string;
        2: string;
    }>;
    ClosedChannel: import("../tsc/web3/types").ContractEvent<{
        closer: string;
        counterParty: string;
        partyAAmount: string;
        partyBAmount: string;
        0: string;
        1: string;
        2: string;
        3: string;
    }>;
    allEvents: (options?: import("../tsc/web3/HoprChannels").EventOptions, cb?: import("../tsc/web3/types").Callback<import("web3-core").EventLog>) => import("events").EventEmitter;
}[N]>) => ReturnType<{
    SecretHashSet: import("../tsc/web3/types").ContractEvent<{
        account: string;
        secretHash: string;
        0: string;
        1: string;
    }>;
    FundedChannel: import("../tsc/web3/types").ContractEvent<{
        funder: string;
        recipient: string;
        counterParty: string;
        recipientAmount: string;
        counterPartyAmount: string;
        0: string;
        1: string;
        2: string;
        3: string;
        4: string;
    }>;
    OpenedChannel: import("../tsc/web3/types").ContractEvent<{
        opener: string;
        counterParty: string;
        0: string;
        1: string;
    }>;
    InitiatedChannelClosure: import("../tsc/web3/types").ContractEvent<{
        initiator: string;
        counterParty: string;
        closureTime: string;
        0: string;
        1: string;
        2: string;
    }>;
    ClosedChannel: import("../tsc/web3/types").ContractEvent<{
        closer: string;
        counterParty: string;
        partyAAmount: string;
        partyBAmount: string;
        0: string;
        1: string;
        2: string;
        3: string;
    }>;
    allEvents: (options?: import("../tsc/web3/HoprChannels").EventOptions, cb?: import("../tsc/web3/types").Callback<import("web3-core").EventLog>) => import("events").EventEmitter;
}[N]>;
declare const clearEvents: (name: "SecretHashSet" | "FundedChannel" | "OpenedChannel" | "InitiatedChannelClosure" | "ClosedChannel" | "allEvents") => void;
declare const clearAllEvents: () => void;
export { addEvent, clearEvents, clearAllEvents };
