///Module containing a contract's types and functions.
/**

```solidity
library Enum {
    type Operation is uint8;
}
```*/
#[allow(
    non_camel_case_types,
    non_snake_case,
    clippy::pub_underscore_fields,
    clippy::style,
    clippy::empty_structs_with_brackets
)]
pub mod Enum {
    use super::*;
    use alloy::sol_types as alloy_sol_types;
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct Operation(u8);
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[automatically_derived]
        impl alloy_sol_types::private::SolTypeValue<Operation> for u8 {
            #[inline]
            fn stv_to_tokens(
                &self,
            ) -> <alloy::sol_types::sol_data::Uint<
                8,
            > as alloy_sol_types::SolType>::Token<'_> {
                alloy_sol_types::private::SolTypeValue::<
                    alloy::sol_types::sol_data::Uint<8>,
                >::stv_to_tokens(self)
            }
            #[inline]
            fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
                <alloy::sol_types::sol_data::Uint<
                    8,
                > as alloy_sol_types::SolType>::tokenize(self)
                    .0
            }
            #[inline]
            fn stv_abi_encode_packed_to(
                &self,
                out: &mut alloy_sol_types::private::Vec<u8>,
            ) {
                <alloy::sol_types::sol_data::Uint<
                    8,
                > as alloy_sol_types::SolType>::abi_encode_packed_to(self, out)
            }
            #[inline]
            fn stv_abi_packed_encoded_size(&self) -> usize {
                <alloy::sol_types::sol_data::Uint<
                    8,
                > as alloy_sol_types::SolType>::abi_encoded_size(self)
            }
        }
        #[automatically_derived]
        impl Operation {
            /// The Solidity type name.
            pub const NAME: &'static str = stringify!(@ name);
            /// Convert from the underlying value type.
            #[inline]
            pub const fn from_underlying(value: u8) -> Self {
                Self(value)
            }
            /// Return the underlying value.
            #[inline]
            pub const fn into_underlying(self) -> u8 {
                self.0
            }
            /// Return the single encoding of this value, delegating to the
            /// underlying type.
            #[inline]
            pub fn abi_encode(&self) -> alloy_sol_types::private::Vec<u8> {
                <Self as alloy_sol_types::SolType>::abi_encode(&self.0)
            }
            /// Return the packed encoding of this value, delegating to the
            /// underlying type.
            #[inline]
            pub fn abi_encode_packed(&self) -> alloy_sol_types::private::Vec<u8> {
                <Self as alloy_sol_types::SolType>::abi_encode_packed(&self.0)
            }
        }
        #[automatically_derived]
        impl From<u8> for Operation {
            fn from(value: u8) -> Self {
                Self::from_underlying(value)
            }
        }
        #[automatically_derived]
        impl From<Operation> for u8 {
            fn from(value: Operation) -> Self {
                value.into_underlying()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolType for Operation {
            type RustType = u8;
            type Token<'a> = <alloy::sol_types::sol_data::Uint<
                8,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SOL_NAME: &'static str = Self::NAME;
            const ENCODED_SIZE: Option<usize> = <alloy::sol_types::sol_data::Uint<
                8,
            > as alloy_sol_types::SolType>::ENCODED_SIZE;
            const PACKED_ENCODED_SIZE: Option<usize> = <alloy::sol_types::sol_data::Uint<
                8,
            > as alloy_sol_types::SolType>::PACKED_ENCODED_SIZE;
            #[inline]
            fn valid_token(token: &Self::Token<'_>) -> bool {
                Self::type_check(token).is_ok()
            }
            #[inline]
            fn type_check(token: &Self::Token<'_>) -> alloy_sol_types::Result<()> {
                <alloy::sol_types::sol_data::Uint<
                    8,
                > as alloy_sol_types::SolType>::type_check(token)
            }
            #[inline]
            fn detokenize(token: Self::Token<'_>) -> Self::RustType {
                <alloy::sol_types::sol_data::Uint<
                    8,
                > as alloy_sol_types::SolType>::detokenize(token)
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::EventTopic for Operation {
            #[inline]
            fn topic_preimage_length(rust: &Self::RustType) -> usize {
                <alloy::sol_types::sol_data::Uint<
                    8,
                > as alloy_sol_types::EventTopic>::topic_preimage_length(rust)
            }
            #[inline]
            fn encode_topic_preimage(
                rust: &Self::RustType,
                out: &mut alloy_sol_types::private::Vec<u8>,
            ) {
                <alloy::sol_types::sol_data::Uint<
                    8,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(rust, out)
            }
            #[inline]
            fn encode_topic(
                rust: &Self::RustType,
            ) -> alloy_sol_types::abi::token::WordToken {
                <alloy::sol_types::sol_data::Uint<
                    8,
                > as alloy_sol_types::EventTopic>::encode_topic(rust)
            }
        }
    };
    use alloy::contract as alloy_contract;
    /**Creates a new wrapper around an on-chain [`Enum`](self) contract instance.

See the [wrapper's documentation](`EnumInstance`) for more details.*/
    #[inline]
    pub const fn new<
        P: alloy_contract::private::Provider<N>,
        N: alloy_contract::private::Network,
    >(address: alloy_sol_types::private::Address, provider: P) -> EnumInstance<P, N> {
        EnumInstance::<P, N>::new(address, provider)
    }
    /**A [`Enum`](self) instance.

Contains type-safe methods for interacting with an on-chain instance of the
[`Enum`](self) contract located at a given `address`, using a given
provider `P`.

If the contract bytecode is available (see the [`sol!`](alloy_sol_types::sol!)
documentation on how to provide it), the `deploy` and `deploy_builder` methods can
be used to deploy a new instance of the contract.

See the [module-level documentation](self) for all the available methods.*/
    #[derive(Clone)]
    pub struct EnumInstance<P, N = alloy_contract::private::Ethereum> {
        address: alloy_sol_types::private::Address,
        provider: P,
        _network: ::core::marker::PhantomData<N>,
    }
    #[automatically_derived]
    impl<P, N> ::core::fmt::Debug for EnumInstance<P, N> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple("EnumInstance").field(&self.address).finish()
        }
    }
    /// Instantiation and getters/setters.
    #[automatically_derived]
    impl<
        P: alloy_contract::private::Provider<N>,
        N: alloy_contract::private::Network,
    > EnumInstance<P, N> {
        /**Creates a new wrapper around an on-chain [`Enum`](self) contract instance.

See the [wrapper's documentation](`EnumInstance`) for more details.*/
        #[inline]
        pub const fn new(
            address: alloy_sol_types::private::Address,
            provider: P,
        ) -> Self {
            Self {
                address,
                provider,
                _network: ::core::marker::PhantomData,
            }
        }
        /// Returns a reference to the address.
        #[inline]
        pub const fn address(&self) -> &alloy_sol_types::private::Address {
            &self.address
        }
        /// Sets the address.
        #[inline]
        pub fn set_address(&mut self, address: alloy_sol_types::private::Address) {
            self.address = address;
        }
        /// Sets the address and returns `self`.
        pub fn at(mut self, address: alloy_sol_types::private::Address) -> Self {
            self.set_address(address);
            self
        }
        /// Returns a reference to the provider.
        #[inline]
        pub const fn provider(&self) -> &P {
            &self.provider
        }
    }
    impl<P: ::core::clone::Clone, N> EnumInstance<&P, N> {
        /// Clones the provider and returns a new instance with the cloned provider.
        #[inline]
        pub fn with_cloned_provider(self) -> EnumInstance<P, N> {
            EnumInstance {
                address: self.address,
                provider: ::core::clone::Clone::clone(&self.provider),
                _network: ::core::marker::PhantomData,
            }
        }
    }
    /// Function calls.
    #[automatically_derived]
    impl<
        P: alloy_contract::private::Provider<N>,
        N: alloy_contract::private::Network,
    > EnumInstance<P, N> {
        /// Creates a new call builder using this contract instance's provider and address.
        ///
        /// Note that the call can be any function call, not just those defined in this
        /// contract. Prefer using the other methods for building type-safe contract calls.
        pub fn call_builder<C: alloy_sol_types::SolCall>(
            &self,
            call: &C,
        ) -> alloy_contract::SolCallBuilder<&P, C, N> {
            alloy_contract::SolCallBuilder::new_sol(&self.provider, &self.address, call)
        }
    }
    /// Event filters.
    #[automatically_derived]
    impl<
        P: alloy_contract::private::Provider<N>,
        N: alloy_contract::private::Network,
    > EnumInstance<P, N> {
        /// Creates a new event filter using this contract instance's provider and address.
        ///
        /// Note that the type can be any event, not just those defined in this contract.
        /// Prefer using the other methods for building type-safe event filters.
        pub fn event_filter<E: alloy_sol_types::SolEvent>(
            &self,
        ) -> alloy_contract::Event<&P, E, N> {
            alloy_contract::Event::new_sol(&self.provider, &self.address)
        }
    }
}
/**

Generated by the following Solidity interface...
```solidity
library Enum {
    type Operation is uint8;
}

interface HoprNodeManagementModule {
    type GranularPermission is uint8;
    type Target is uint256;

    error AddressIsZero();
    error AlreadyInitialized();
    error ArrayTooLong();
    error ArraysDifferentLength();
    error CalldataOutOfBounds();
    error CannotChangeOwner();
    error DefaultPermissionRejected();
    error DelegateCallNotAllowed();
    error FunctionSignatureTooShort();
    error GranularPermissionRejected();
    error NoMembership();
    error NodePermissionRejected();
    error NonExistentKey();
    error ParameterNotAllowed();
    error PermissionNotConfigured();
    error PermissionNotFound();
    error SafeMultisendSameAddress();
    error SendNotAllowed();
    error TargetAddressNotAllowed();
    error TargetIsNotScoped();
    error TargetIsScoped();
    error TooManyCapabilities();
    error UnacceptableMultiSendOffset();
    error WithMembership();

    event AdminChanged(address previousAdmin, address newAdmin);
    event BeaconUpgraded(address indexed beacon);
    event ExecutionFailure();
    event ExecutionSuccess();
    event Initialized(uint8 version);
    event NodeAdded(address indexed node);
    event NodeRemoved(address indexed node);
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
    event SetMultisendAddress(address indexed multisendAddress);
    event Upgraded(address indexed implementation);

    constructor();

    function addChannelsAndTokenTarget(Target defaultTarget) external;
    function addNode(address nodeAddress) external;
    function decodeFunctionSigsAndPermissions(bytes32 encoded, uint256 length) external pure returns (bytes4[] memory functionSigs, GranularPermission[] memory permissions);
    function encodeFunctionSigsAndPermissions(bytes4[] memory functionSigs, GranularPermission[] memory permissions) external pure returns (bytes32 encoded, uint256 length);
    function execTransactionFromModule(address to, uint256 value, bytes memory data, Enum.Operation operation) external returns (bool success);
    function execTransactionFromModuleReturnData(address to, uint256 value, bytes memory data, Enum.Operation operation) external returns (bool, bytes memory);
    function getGranularPermissions(bytes32 capabilityKey, bytes32 pairId) external view returns (GranularPermission);
    function getTargets() external view returns (Target[] memory);
    function includeNode(Target nodeDefaultTarget) external;
    function initialize(bytes memory initParams) external;
    function isHoprNodeManagementModule() external view returns (bool);
    function isNode(address nodeAddress) external view returns (bool);
    function multisend() external view returns (address);
    function owner() external view returns (address);
    function proxiableUUID() external view returns (bytes32);
    function removeNode(address nodeAddress) external;
    function renounceOwnership() external;
    function revokeTarget(address targetAddress) external;
    function scopeChannelsCapabilities(address targetAddress, bytes32 channelId, bytes32 encodedSigsPermissions) external;
    function scopeSendCapability(address nodeAddress, address beneficiary, GranularPermission permission) external;
    function scopeTargetChannels(Target defaultTarget) external;
    function scopeTargetSend(Target defaultTarget) external;
    function scopeTargetToken(Target defaultTarget) external;
    function scopeTokenCapabilities(address nodeAddress, address targetAddress, address beneficiary, bytes32 encodedSigsPermissions) external;
    function setMultisend(address _multisend) external;
    function transferOwnership(address newOwner) external;
    function tryGetTarget(address targetAddress) external view returns (bool, Target);
    function upgradeTo(address newImplementation) external;
    function upgradeToAndCall(address newImplementation, bytes memory data) external payable;
}
```

...which was generated by the following JSON ABI:
```json
[
  {
    "type": "constructor",
    "inputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "addChannelsAndTokenTarget",
    "inputs": [
      {
        "name": "defaultTarget",
        "type": "uint256",
        "internalType": "Target"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "addNode",
    "inputs": [
      {
        "name": "nodeAddress",
        "type": "address",
        "internalType": "address"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "decodeFunctionSigsAndPermissions",
    "inputs": [
      {
        "name": "encoded",
        "type": "bytes32",
        "internalType": "bytes32"
      },
      {
        "name": "length",
        "type": "uint256",
        "internalType": "uint256"
      }
    ],
    "outputs": [
      {
        "name": "functionSigs",
        "type": "bytes4[]",
        "internalType": "bytes4[]"
      },
      {
        "name": "permissions",
        "type": "uint8[]",
        "internalType": "enum GranularPermission[]"
      }
    ],
    "stateMutability": "pure"
  },
  {
    "type": "function",
    "name": "encodeFunctionSigsAndPermissions",
    "inputs": [
      {
        "name": "functionSigs",
        "type": "bytes4[]",
        "internalType": "bytes4[]"
      },
      {
        "name": "permissions",
        "type": "uint8[]",
        "internalType": "enum GranularPermission[]"
      }
    ],
    "outputs": [
      {
        "name": "encoded",
        "type": "bytes32",
        "internalType": "bytes32"
      },
      {
        "name": "length",
        "type": "uint256",
        "internalType": "uint256"
      }
    ],
    "stateMutability": "pure"
  },
  {
    "type": "function",
    "name": "execTransactionFromModule",
    "inputs": [
      {
        "name": "to",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "value",
        "type": "uint256",
        "internalType": "uint256"
      },
      {
        "name": "data",
        "type": "bytes",
        "internalType": "bytes"
      },
      {
        "name": "operation",
        "type": "uint8",
        "internalType": "enum Enum.Operation"
      }
    ],
    "outputs": [
      {
        "name": "success",
        "type": "bool",
        "internalType": "bool"
      }
    ],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "execTransactionFromModuleReturnData",
    "inputs": [
      {
        "name": "to",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "value",
        "type": "uint256",
        "internalType": "uint256"
      },
      {
        "name": "data",
        "type": "bytes",
        "internalType": "bytes"
      },
      {
        "name": "operation",
        "type": "uint8",
        "internalType": "enum Enum.Operation"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bool",
        "internalType": "bool"
      },
      {
        "name": "",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "getGranularPermissions",
    "inputs": [
      {
        "name": "capabilityKey",
        "type": "bytes32",
        "internalType": "bytes32"
      },
      {
        "name": "pairId",
        "type": "bytes32",
        "internalType": "bytes32"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "uint8",
        "internalType": "enum GranularPermission"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "getTargets",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "uint256[]",
        "internalType": "Target[]"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "includeNode",
    "inputs": [
      {
        "name": "nodeDefaultTarget",
        "type": "uint256",
        "internalType": "Target"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "initialize",
    "inputs": [
      {
        "name": "initParams",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "isHoprNodeManagementModule",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "bool",
        "internalType": "bool"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "isNode",
    "inputs": [
      {
        "name": "nodeAddress",
        "type": "address",
        "internalType": "address"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bool",
        "internalType": "bool"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "multisend",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "address",
        "internalType": "address"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "owner",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "address",
        "internalType": "address"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "proxiableUUID",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "bytes32",
        "internalType": "bytes32"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "removeNode",
    "inputs": [
      {
        "name": "nodeAddress",
        "type": "address",
        "internalType": "address"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "renounceOwnership",
    "inputs": [],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "revokeTarget",
    "inputs": [
      {
        "name": "targetAddress",
        "type": "address",
        "internalType": "address"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "scopeChannelsCapabilities",
    "inputs": [
      {
        "name": "targetAddress",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "channelId",
        "type": "bytes32",
        "internalType": "bytes32"
      },
      {
        "name": "encodedSigsPermissions",
        "type": "bytes32",
        "internalType": "bytes32"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "scopeSendCapability",
    "inputs": [
      {
        "name": "nodeAddress",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "beneficiary",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "permission",
        "type": "uint8",
        "internalType": "enum GranularPermission"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "scopeTargetChannels",
    "inputs": [
      {
        "name": "defaultTarget",
        "type": "uint256",
        "internalType": "Target"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "scopeTargetSend",
    "inputs": [
      {
        "name": "defaultTarget",
        "type": "uint256",
        "internalType": "Target"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "scopeTargetToken",
    "inputs": [
      {
        "name": "defaultTarget",
        "type": "uint256",
        "internalType": "Target"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "scopeTokenCapabilities",
    "inputs": [
      {
        "name": "nodeAddress",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "targetAddress",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "beneficiary",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "encodedSigsPermissions",
        "type": "bytes32",
        "internalType": "bytes32"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "setMultisend",
    "inputs": [
      {
        "name": "_multisend",
        "type": "address",
        "internalType": "address"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "transferOwnership",
    "inputs": [
      {
        "name": "newOwner",
        "type": "address",
        "internalType": "address"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "tryGetTarget",
    "inputs": [
      {
        "name": "targetAddress",
        "type": "address",
        "internalType": "address"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bool",
        "internalType": "bool"
      },
      {
        "name": "",
        "type": "uint256",
        "internalType": "Target"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "upgradeTo",
    "inputs": [
      {
        "name": "newImplementation",
        "type": "address",
        "internalType": "address"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "upgradeToAndCall",
    "inputs": [
      {
        "name": "newImplementation",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "data",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "outputs": [],
    "stateMutability": "payable"
  },
  {
    "type": "event",
    "name": "AdminChanged",
    "inputs": [
      {
        "name": "previousAdmin",
        "type": "address",
        "indexed": false,
        "internalType": "address"
      },
      {
        "name": "newAdmin",
        "type": "address",
        "indexed": false,
        "internalType": "address"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "BeaconUpgraded",
    "inputs": [
      {
        "name": "beacon",
        "type": "address",
        "indexed": true,
        "internalType": "address"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "ExecutionFailure",
    "inputs": [],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "ExecutionSuccess",
    "inputs": [],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "Initialized",
    "inputs": [
      {
        "name": "version",
        "type": "uint8",
        "indexed": false,
        "internalType": "uint8"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "NodeAdded",
    "inputs": [
      {
        "name": "node",
        "type": "address",
        "indexed": true,
        "internalType": "address"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "NodeRemoved",
    "inputs": [
      {
        "name": "node",
        "type": "address",
        "indexed": true,
        "internalType": "address"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "OwnershipTransferred",
    "inputs": [
      {
        "name": "previousOwner",
        "type": "address",
        "indexed": true,
        "internalType": "address"
      },
      {
        "name": "newOwner",
        "type": "address",
        "indexed": true,
        "internalType": "address"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "SetMultisendAddress",
    "inputs": [
      {
        "name": "multisendAddress",
        "type": "address",
        "indexed": true,
        "internalType": "address"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "Upgraded",
    "inputs": [
      {
        "name": "implementation",
        "type": "address",
        "indexed": true,
        "internalType": "address"
      }
    ],
    "anonymous": false
  },
  {
    "type": "error",
    "name": "AddressIsZero",
    "inputs": []
  },
  {
    "type": "error",
    "name": "AlreadyInitialized",
    "inputs": []
  },
  {
    "type": "error",
    "name": "ArrayTooLong",
    "inputs": []
  },
  {
    "type": "error",
    "name": "ArraysDifferentLength",
    "inputs": []
  },
  {
    "type": "error",
    "name": "CalldataOutOfBounds",
    "inputs": []
  },
  {
    "type": "error",
    "name": "CannotChangeOwner",
    "inputs": []
  },
  {
    "type": "error",
    "name": "DefaultPermissionRejected",
    "inputs": []
  },
  {
    "type": "error",
    "name": "DelegateCallNotAllowed",
    "inputs": []
  },
  {
    "type": "error",
    "name": "FunctionSignatureTooShort",
    "inputs": []
  },
  {
    "type": "error",
    "name": "GranularPermissionRejected",
    "inputs": []
  },
  {
    "type": "error",
    "name": "NoMembership",
    "inputs": []
  },
  {
    "type": "error",
    "name": "NodePermissionRejected",
    "inputs": []
  },
  {
    "type": "error",
    "name": "NonExistentKey",
    "inputs": []
  },
  {
    "type": "error",
    "name": "ParameterNotAllowed",
    "inputs": []
  },
  {
    "type": "error",
    "name": "PermissionNotConfigured",
    "inputs": []
  },
  {
    "type": "error",
    "name": "PermissionNotFound",
    "inputs": []
  },
  {
    "type": "error",
    "name": "SafeMultisendSameAddress",
    "inputs": []
  },
  {
    "type": "error",
    "name": "SendNotAllowed",
    "inputs": []
  },
  {
    "type": "error",
    "name": "TargetAddressNotAllowed",
    "inputs": []
  },
  {
    "type": "error",
    "name": "TargetIsNotScoped",
    "inputs": []
  },
  {
    "type": "error",
    "name": "TargetIsScoped",
    "inputs": []
  },
  {
    "type": "error",
    "name": "TooManyCapabilities",
    "inputs": []
  },
  {
    "type": "error",
    "name": "UnacceptableMultiSendOffset",
    "inputs": []
  },
  {
    "type": "error",
    "name": "WithMembership",
    "inputs": []
  }
]
```*/
#[allow(
    non_camel_case_types,
    non_snake_case,
    clippy::pub_underscore_fields,
    clippy::style,
    clippy::empty_structs_with_brackets
)]
pub mod HoprNodeManagementModule {
    use super::*;
    use alloy::sol_types as alloy_sol_types;
    /// The creation / init bytecode of the contract.
    ///
    /// ```text
    ///0x60a0604052306080523480156200001557600080fd5b506200002062000026565b620000e7565b600054610100900460ff1615620000935760405162461bcd60e51b815260206004820152602760248201527f496e697469616c697a61626c653a20636f6e747261637420697320696e697469604482015266616c697a696e6760c81b606482015260840160405180910390fd5b60005460ff90811614620000e5576000805460ff191660ff9081179091556040519081527f7f26b83ff96e1f2b6a682f133852f6798a09c465da95921460cefb38474024989060200160405180910390a15b565b6080516136806200011f600039600081816106040152818161064d015281816109cf01528181610a0f0152610b3f01526136806000f3fe6080604052600436106101c25760003560e01c8063739c4b08116100f7578063b573696211610095578063dc446a4a11610064578063dc446a4a14610540578063df4e6f8a1461056d578063f2fde38b146105a4578063fa19501d146105c457600080fd5b8063b5736962146104c0578063c68605c8146104e0578063c68c3a8314610500578063dc06109d1461052057600080fd5b80639d95f1cc116100d15780639d95f1cc14610440578063a2450f8914610460578063a76c9a2f14610480578063b2b99ec9146104a057600080fd5b8063739c4b08146103e25780638b95eccd146104025780638da5cb5b1461042257600080fd5b80634f1ef2861161016457806356f551171161013e57806356f551171461034857806360976c4b1461037d57806363fe3b56146103ab578063715018a6146103cd57600080fd5b80634f1ef286146102e45780635229073f146102f757806352d1902d1461032557600080fd5b80633659cfe6116101a05780633659cfe61461026f578063439fab911461028f578063468721a7146102af5780634a1ba408146102cf57600080fd5b806301750152146101c7578063294402cc146102155780633401cde81461024d575b600080fd5b3480156101d357600080fd5b506102006101e2366004612ca4565b6001600160a01b0316600090815260cc602052604090205460ff1690565b60405190151581526020015b60405180910390f35b34801561022157600080fd5b5060c954610235906001600160a01b031681565b6040516001600160a01b03909116815260200161020c565b34801561025957600080fd5b5061026d610268366004612ca4565b6105e4565b005b34801561027b57600080fd5b5061026d61028a366004612ca4565b6105fa565b34801561029b57600080fd5b5061026d6102aa366004612d86565b6106df565b3480156102bb57600080fd5b506102006102ca366004612dbb565b61092b565b3480156102db57600080fd5b50610200600181565b61026d6102f2366004612e63565b6109c5565b34801561030357600080fd5b50610317610312366004612dbb565b610a91565b60405161020c929190612f03565b34801561033157600080fd5b5061033a610b32565b60405190815260200161020c565b34801561035457600080fd5b50610368610363366004612fc3565b610be5565b6040805192835260208301919091520161020c565b34801561038957600080fd5b5061039d610398366004613089565b610bfe565b60405161020c9291906130d5565b3480156103b757600080fd5b506103c0610c0b565b60405161020c9190613161565b3480156103d957600080fd5b5061026d610c1c565b3480156103ee57600080fd5b5061026d6103fd3660046131a5565b610c30565b34801561040e57600080fd5b5061026d61041d366004612ca4565b610c43565b34801561042e57600080fd5b506097546001600160a01b0316610235565b34801561044c57600080fd5b5061026d61045b366004612ca4565b610c95565b34801561046c57600080fd5b5061026d61047b3660046131a5565b610ca6565b34801561048c57600080fd5b5061026d61049b3660046131a5565b610cb7565b3480156104ac57600080fd5b5061026d6104bb366004612ca4565b610cca565b3480156104cc57600080fd5b5061026d6104db3660046131a5565b610d54565b3480156104ec57600080fd5b5061026d6104fb3660046131be565b610d8c565b34801561050c57600080fd5b5061026d61051b36600461320f565b610da8565b34801561052c57600080fd5b5061026d61053b3660046131a5565b610dc2565b34801561054c57600080fd5b5061056061055b366004613089565b610e1d565b60405161020c9190613256565b34801561057957600080fd5b5061058d610588366004612ca4565b610e41565b60408051921515835260208301919091520161020c565b3480156105b057600080fd5b5061026d6105bf366004612ca4565b610e58565b3480156105d057600080fd5b5061026d6105df366004613264565b610ece565b6105ec610ee3565b6105f760ca82610f3d565b50565b6001600160a01b037f000000000000000000000000000000000000000000000000000000000000000016300361064b5760405162461bcd60e51b815260040161064290613299565b60405180910390fd5b7f00000000000000000000000000000000000000000000000000000000000000006001600160a01b0316610694600080516020613604833981519152546001600160a01b031690565b6001600160a01b0316146106ba5760405162461bcd60e51b8152600401610642906132e5565b6106c381610fa3565b604080516000808252602082019092526105f791839190610fab565b600054610100900460ff16158080156106ff5750600054600160ff909116105b806107195750303b158015610719575060005460ff166001145b61077c5760405162461bcd60e51b815260206004820152602e60248201527f496e697469616c697a61626c653a20636f6e747261637420697320616c72656160448201526d191e481a5b9a5d1a585b1a5e995960921b6064820152608401610642565b6000805460ff19166001179055801561079f576000805461ff0019166101001790555b6000806000848060200190518101906107b89190613331565b919450925090506001600160a01b03831615806107dc57506001600160a01b038216155b156107fa5760405163867915ab60e01b815260040160405180910390fd5b816001600160a01b0316836001600160a01b03160361082c5760405163598a0e2160e01b815260040160405180910390fd5b60006108406097546001600160a01b031690565b6001600160a01b0316141580610860575060c9546001600160a01b031615155b1561087d5760405162dc149f60e41b815260040160405180910390fd5b60c980546001600160a01b0319166001600160a01b0384161790556108a181611116565b6108aa836111dc565b6040516001600160a01b038316907f5fe6aabf4e790843df43ae0e22b58620066fb389295bedc06a92df6c3b28777d90600090a25050508015610927576000805461ff0019169055604051600181527f7f26b83ff96e1f2b6a682f133852f6798a09c465da95921460cefb38474024989060200160405180910390a15b5050565b33600090815260cc602052604081205460ff1661095b57604051631fb1d3e560e31b815260040160405180910390fd5b60c9546109789060ca906001600160a01b0316888888888861122e565b6109bb868686868080601f0160208091040260200160405190810160405280939291908181526020018383808284376000920191909152508892506112d9915050565b9695505050505050565b6001600160a01b037f0000000000000000000000000000000000000000000000000000000000000000163003610a0d5760405162461bcd60e51b815260040161064290613299565b7f00000000000000000000000000000000000000000000000000000000000000006001600160a01b0316610a56600080516020613604833981519152546001600160a01b031690565b6001600160a01b031614610a7c5760405162461bcd60e51b8152600401610642906132e5565b610a8582610fa3565b61092782826001610fab565b33600090815260cc602052604081205460609060ff16610ac457604051631fb1d3e560e31b815260040160405180910390fd5b60c954610ae19060ca906001600160a01b0316898989898961122e565b610b24878787878080601f0160208091040260200160405190810160405280939291908181526020018383808284376000920191909152508992506113c9915050565b915091509550959350505050565b6000306001600160a01b037f00000000000000000000000000000000000000000000000000000000000000001614610bd25760405162461bcd60e51b815260206004820152603860248201527f555550535570677261646561626c653a206d757374206e6f742062652063616c60448201527f6c6564207468726f7567682064656c656761746563616c6c00000000000000006064820152608401610642565b5060008051602061360483398151915290565b600080610bf284846114c3565b915091505b9250929050565b606080610bf284846115d7565b6060610c1760ca611786565b905090565b610c24610ee3565b610c2e60006111dc565b565b610c38610ee3565b6105f760ca826117e2565b610c4b610ee3565b60c980546001600160a01b0319166001600160a01b0383169081179091556040517f5fe6aabf4e790843df43ae0e22b58620066fb389295bedc06a92df6c3b28777d90600090a250565b610c9d610ee3565b6105f7816118a2565b610cae610ee3565b6105f781611116565b610cbf610ee3565b6105f760ca82611928565b610cd2610ee3565b6001600160a01b038116600090815260cc602052604090205460ff16610d0b57604051631fb1d3e560e31b815260040160405180910390fd5b6001600160a01b038116600081815260cc6020526040808220805460ff19169055517fcfc24166db4bb677e857cacabd1541fb2b30645021b27c5130419589b84db52b9190a250565b610d5c610ee3565b6000610d688260601c90565b9050610d73816118a2565b610d7e60ca836119d9565b61092760ca82836001611a8b565b610d94610ee3565b610da260ca85858585611b27565b50505050565b610db0610ee3565b610dbd60ca848484611a8b565b505050565b610dca610ee3565b6000610dd68260601c90565b6001600160a01b038116600090815260cc602052604090205490915060ff16610e1257604051631fb1d3e560e31b815260040160405180910390fd5b61092760ca836119d9565b600082815260cd6020908152604080832084845290915290205460ff165b92915050565b600080610e4f60ca84611cab565b91509150915091565b610e60610ee3565b6001600160a01b038116610ec55760405162461bcd60e51b815260206004820152602660248201527f4f776e61626c653a206e6577206f776e657220697320746865207a65726f206160448201526564647265737360d01b6064820152608401610642565b6105f7816111dc565b610ed6610ee3565b610dbd60ca848484611d0e565b6097546001600160a01b03163314610c2e5760405162461bcd60e51b815260206004820181905260248201527f4f776e61626c653a2063616c6c6572206973206e6f7420746865206f776e65726044820152606401610642565b6000610f498383611e55565b90508015610f8a576040516001600160a01b038316907f0dfce1ea4ba1eeba891ffb2a066790fbc293a9e517fe61d49d156a30165f93f390600090a2505050565b604051634a89032160e01b815260040160405180910390fd5b6105f7610ee3565b7f4910fdfa16fed3260ed0e7147f7cc6da11a60208b5b9406d12a635614ffd91435460ff1615610fde57610dbd83611f7d565b826001600160a01b03166352d1902d6040518163ffffffff1660e01b8152600401602060405180830381865afa925050508015611038575060408051601f3d908101601f1916820190925261103591810190613374565b60015b61109b5760405162461bcd60e51b815260206004820152602e60248201527f45524331393637557067726164653a206e657720696d706c656d656e7461746960448201526d6f6e206973206e6f74205555505360901b6064820152608401610642565b600080516020613604833981519152811461110a5760405162461bcd60e51b815260206004820152602960248201527f45524331393637557067726164653a20756e737570706f727465642070726f786044820152681a58589b195555525160ba1b6064820152608401610642565b50610dbd838383612019565b60006111228260601c90565b90506000816001600160a01b031663fc0c546a6040518163ffffffff1660e01b8152600401602060405180830381865afa158015611164573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190611188919061338d565b90506111b360ca6bffffffffffffffffffffffff8516606085901b6001600160601b031916176117e2565b610dbd60ca6bffffffffffffffffffffffff8516606084901b6001600160601b03191617611928565b609780546001600160a01b038381166001600160a01b0319831681179093556040519116919082907f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e090600090a35050565b846001600160a01b0316866001600160a01b03160361128c576112878784848080601f01602080910402602001604051908101604052809392919081815260200183838082843760009201919091525061203e92505050565b6112d0565b6112d087868686868080601f0160208091040260200160405190810160405280939291908181526020018383808284376000920191909152508892506120df915050565b50505050505050565b60006112ed6097546001600160a01b031690565b6001600160a01b031663468721a7868686866040518563ffffffff1660e01b815260040161131e94939291906133aa565b6020604051808303816000875af115801561133d573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906113619190613402565b90508015611397576040517f4e2e86d21375ebcbf6e93df5ebdd5a915bf830245904c3b54f48adf0170aae4b90600090a16113c1565b6040517fc24d93608a03d263ff191d7677141f5e94c496e593108f3aae0cb5b70494c4d390600090a15b949350505050565b600060606113df6097546001600160a01b031690565b6001600160a01b0316635229073f878787876040518563ffffffff1660e01b815260040161141094939291906133aa565b6000604051808303816000875af115801561142f573d6000803e3d6000fd5b505050506040513d6000823e601f3d908101601f19168201604052611457919081019061341d565b90925090508115611490576040517f4e2e86d21375ebcbf6e93df5ebdd5a915bf830245904c3b54f48adf0170aae4b90600090a16114ba565b6040517fc24d93608a03d263ff191d7677141f5e94c496e593108f3aae0cb5b70494c4d390600090a15b94509492505050565b8151600090819060078111156114ec576040516317a4d98760e31b815260040160405180910390fd5b835185511461150e576040516374f4d53760e01b815260040160405180910390fd5b6000805b82811015611571576115258160206134bd565b6115309060e06134d4565b60e0888381518110611544576115446134e7565b60200260200101516001600160e01b031916901c901b821791508080611569906134fd565b915050611512565b5060005b828110156115cc576115888160026134bd565b86828151811061159a5761159a6134e7565b602002602001015160028111156115b3576115b36130ab565b901b9190911790806115c4816134fd565b915050611575565b509590945092505050565b60608060078311156115fc576040516317a4d98760e31b815260040160405180910390fd5b8267ffffffffffffffff81111561161557611615612cc1565b60405190808252806020026020018201604052801561163e578160200160208202803683370190505b5091508267ffffffffffffffff81111561165a5761165a612cc1565b604051908082528060200260200182016040528015611683578160200160208202803683370190505b50905060005b838110156116eb5760e061169e8260206134bd565b6116a99060e06134d4565b86901c901b8382815181106116c0576116c06134e7565b6001600160e01b031990921660209283029190910190910152806116e3816134fd565b915050611689565b5060005b8381101561177e5760fe6117048260026134bd565b61170f9060fe6134d4565b8660001c901b901c60ff16600281111561172b5761172b6130ab565b82828151811061173d5761173d6134e7565b60200260200101906002811115611756576117566130ab565b90816002811115611769576117696130ab565b90525080611776816134fd565b9150506116ef565b509250929050565b6060816000018054806020026020016040519081016040528092919081815260200182805480156117d657602002820191906000526020600020905b8154815260200190600101908083116117c2575b50505050509050919050565b60006117ee8260601c90565b90506001600160a01b0381166118175760405163867915ab60e01b815260040160405180910390fd5b611821838261231f565b1561183f576040516374603e9560e11b815260040160405180910390fd5b600061184c836001612340565b905061185884826123d2565b50816001600160a01b03167f5ffb06b0b0e8ad6a8f3c5831d499dfa612d9c9d4dc107bbd66f18f61a6492e718260405161189491815260200190565b60405180910390a250505050565b6001600160a01b038116600090815260cc602052604090205460ff16156118dc576040516338e816a560e21b815260040160405180910390fd5b6001600160a01b038116600081815260cc6020526040808220805460ff19166001179055517fb25d03aaf308d7291709be1ea28b800463cf3a9a4c4a5555d7333a964c1dfebd9190a250565b60006119348260601c90565b90506001600160a01b03811661195d5760405163867915ab60e01b815260040160405180910390fd5b611967838261231f565b15611985576040516374603e9560e11b815260040160405180910390fd5b60006119918382612340565b905061199d84826123d2565b50816001600160a01b03167faaf26bb12aa89ee96bbe19667a6a055727b75d3f6ed7b8b611ef6519180209d68260405161189491815260200190565b60006119e58260601c90565b90506001600160a01b038116611a0e5760405163867915ab60e01b815260040160405180910390fd5b611a18838261231f565b15611a36576040516374603e9560e11b815260040160405180910390fd5b6000611a43836002612340565b9050611a4f84826123d2565b50816001600160a01b03167f1ee2791f2caf0e92a9dc32a37a9ea53ab6ac7a6fb8f2d090e53a067d3a43f6ac8260405161189491815260200190565b600080805260038501602052604081208291611aa7868661243e565b81526020810191909152604001600020805460ff19166001836002811115611ad157611ad16130ab565b0217905550816001600160a01b0316836001600160a01b03167f7487530ddff120799505e52b1b19b6933f85a9eeae9220c80a7ad7c429b612ae83604051611b199190613256565b60405180910390a350505050565b600080611b358360026115d7565b9150915060005b6002811015611ca1578251600090849083908110611b5c57611b5c6134e7565b60200260200101516001600160e01b03191614611c8f576000611b9887858481518110611b8b57611b8b6134e7565b6020026020010151612483565b9050828281518110611bac57611bac6134e7565b60200260200101518960030160008381526020019081526020016000206000611bd58b8b61243e565b81526020810191909152604001600020805460ff19166001836002811115611bff57611bff6130ab565b0217905550856001600160a01b0316876001600160a01b0316896001600160a01b03167fa3df710420b01cc30ff300309abbc7fadd4630d4ab385b0f5a126fb4babe762b878681518110611c5557611c556134e7565b6020026020010151878781518110611c6f57611c6f6134e7565b6020026020010151604051611c85929190613516565b60405180910390a4505b80611c99816134fd565b915050611b3c565b5050505050505050565b6001600160a01b03811660009081526001830160205260408120548190808203611cdc576000809250925050610bf7565b600185611ce982846134d4565b81548110611cf957611cf96134e7565b90600052602060002001549250925050610bf7565b600080611d1c8360076115d7565b9150915060005b60078110156112d0578251600090849083908110611d4357611d436134e7565b60200260200101516001600160e01b03191614611e43576000611d7287858481518110611b8b57611b8b6134e7565b9050828281518110611d8657611d866134e7565b602090810291909101810151600083815260038b01835260408082208a835290935291909120805460ff19166001836002811115611dc657611dc66130ab565b021790555085876001600160a01b03167ff2ffd4f09d58d06824188033d3318d06eb957bfb1a8ffed9af78e1f19168b904868581518110611e0957611e096134e7565b6020026020010151868681518110611e2357611e236134e7565b6020026020010151604051611e39929190613516565b60405180910390a3505b80611e4d816134fd565b915050611d23565b6001600160a01b03811660009081526001830160205260408120548015611f73576000611e836001836134d4565b8554909150600090611e97906001906134d4565b9050818114611f15576000866000018281548110611eb757611eb76134e7565b9060005260206000200154905080876000018481548110611eda57611eda6134e7565b906000526020600020018190555083876001016000611ef98460601c90565b6001600160a01b03168152602081019190915260400160002055505b8554869080611f2657611f26613534565b60019003818190600052602060002001600090559055856001016000866001600160a01b03166001600160a01b031681526020019081526020016000206000905560019350505050610e3b565b6000915050610e3b565b6001600160a01b0381163b611fea5760405162461bcd60e51b815260206004820152602d60248201527f455243313936373a206e657720696d706c656d656e746174696f6e206973206e60448201526c1bdd08184818dbdb9d1c9858dd609a1b6064820152608401610642565b60008051602061360483398151915280546001600160a01b0319166001600160a01b0392909216919091179055565b612022836124cf565b60008251118061202f5750805b15610dbd57610da2838361250f565b60008060006060600080602487015190508060201461207057604051637ed1113760e01b815260040160405180910390fd5b60645b87518110156120d4578088015160f81c96506001810188015160601c955060158101880151945060358101880151925060358101880193506120b8898787878b6120df565b6120c383605561354a565b6120cd908261354a565b9050612073565b505050505050505050565b8151158015906120f0575060048251105b1561210e57604051632342609160e11b815260040160405180910390fd5b600061211a8686612534565b905061212784838361259f565b60006121328461355d565b9050600061214285518484612641565b90506000816003811115612158576121586130ab565b0361217657604051635872303760e01b815260040160405180910390fd5b600381600381111561218a5761218a6130ab565b0361219757505050612318565b6000806121a3856127e9565b60028111156121b4576121b46130ab565b036121d5576121ce896121c78a86612483565b8589612804565b9050612239565b60016121e0856127e9565b60028111156121f1576121f16130ab565b0361220b576121ce896122048a86612483565b858961289b565b6002612216856127e9565b6002811115612227576122276130ab565b036122395761223689896129ca565b90505b600281600281111561224d5761224d6130ab565b148061228657506000816002811115612268576122686130ab565b14801561228657506001826003811115612284576122846130ab565b145b156122a45760405163864dd1e760e01b815260040160405180910390fd5b60018160028111156122b8576122b86130ab565b14806122f1575060008160028111156122d3576122d36130ab565b1480156122f1575060028260038111156122ef576122ef6130ab565b145b156122ff5750505050612318565b6040516308d5a8b160e31b815260040160405180910390fd5b5050505050565b6001600160a01b031660009081526001919091016020526040902054151590565b600080806001846002811115612358576123586130ab565b0361237057506aff0000000000000000ffff196123ab565b6000846002811115612384576123846130ab565b0361239c57506aff00ffffffffffffff0000196123ab565b506aff00ffffffffffffffffff195b808516915060508460028111156123c4576123c46130ab565b901b91909117949350505050565b60006123e7836123e28460601c90565b61231f565b61243657825460018181018555600085815260208120909201849055845491908501906124148560601c90565b6001600160a01b03168152602081019190915260400160002055506001610e3b565b506000610e3b565b6040516001600160601b0319606084811b8216602084015283901b16603482015260009060480160405160208183030381529060405280519060200120905092915050565b6040516001600160601b0319606084901b1660208201526001600160e01b0319821660348201526000906038016040516020818303038152906040526124c890613594565b9392505050565b6124d881611f7d565b6040516001600160a01b038216907fbc7cd75a20ee27fd9adebab32041f755214dbc6bffa90cc0225b39da2e5c2d3b90600090a250565b60606124c88383604051806060016040528060278152602001613624602791396129fe565b6001600160a01b038116600090815260018301602052604081205480820361256f57604051632d0519ad60e01b815260040160405180910390fd5b8361257b6001836134d4565b8154811061258b5761258b6134e7565b906000526020600020015491505092915050565b60016125aa82612a6c565b60018111156125bb576125bb6130ab565b146125d957604051633bcd102b60e21b815260040160405180910390fd5b60018260018111156125ed576125ed6130ab565b0361260b576040516306c4a1c760e11b815260040160405180910390fd5b6000831180156126235750612621816002612a87565b155b15610dbd576040516309e9cd4960e01b815260040160405180910390fd5b60008061264d84612abd565b905084158061266457506001600160e01b03198316155b806126805750600381600381111561267e5761267e6130ab565b145b8061269c5750600081600381111561269a5761269a6130ab565b145b156126a85790506124c8565b6000637993b94760e11b6001600160e01b03198516016126d4576126cd856000612ad8565b90506127bf565b63ab5d120b60e01b6001600160e01b03198516016126f7576126cd856002612ad8565b634259a0bb60e01b6001600160e01b031985160161271a576126cd856003612ad8565b639aeaeb4160e01b6001600160e01b031985160161273d576126cd856004612ad8565b63f5413a7160e01b6001600160e01b0319851601612760576126cd856005612ad8565b63f6a1584d60e01b6001600160e01b0319851601612783576126cd856007612ad8565b633213221d60e11b6001600160e01b03198516016127a6576126cd856008612ad8565b6040516318f4c12360e11b815260040160405180910390fd5b60008160048111156127d3576127d36130ab565b036127e0575090506124c8565b6109bb81612b2c565b600060ff605083901c166002811115610e3b57610e3b6130ab565b60006001600160e01b0319831663095ea7b360e01b1480159061283857506001600160e01b03198316634decdde360e11b14155b15612856576040516318f4c12360e11b815260040160405180910390fd5b6000612863600084612b86565b90506000612871338361243e565b60008781526003890160209081526040808320938352929052205460ff1692505050949350505050565b6000806128a9600084612b86565b90506001600160a01b03811633146128d457604051636eb0315f60e01b815260040160405180910390fd5b6000637993b94760e11b6001600160e01b0319861601612900576128f9600185612b86565b90506129a1565b63ab5d120b60e01b6001600160e01b0319861601612939576000612925600186612b86565b9050612931818461243e565b9150506129a1565b6001600160e01b0319851663bda65f4560e01b148061296857506001600160e01b0319851663651514bf60e01b145b8061298357506001600160e01b03198516630abec58f60e01b145b156127a6576000612995600186612b86565b9050612931838261243e565b60008681526003880160209081526040808320938352929052205460ff16915050949350505050565b6000806129d7338461243e565b60008080526003860160209081526040808320938352929052205460ff1691505092915050565b6060600080856001600160a01b031685604051612a1b91906135bb565b600060405180830381855af49150503d8060008114612a56576040519150601f19603f3d011682016040523d82523d6000602084013e612a5b565b606091505b50915091506109bb86838387612bf1565b600060ff605883901c166001811115610e3b57610e3b6130ab565b6000816002811115612a9b57612a9b6130ab565b612aa4846127e9565b6002811115612ab557612ab56130ab565b149392505050565b600060ff604883901c166003811115610e3b57610e3b6130ab565b600060098210612afb5760405163b44af9af60e01b815260040160405180910390fd5b6000612b088360086134bd565b612b139060b861354a565b905083811b60f81c60048111156113c1576113c16130ab565b600080826004811115612b4157612b416130ab565b90508060ff16600003612b675760405163d8455a1360e01b815260040160405180910390fd5b612b726001826135d7565b60ff1660038111156124c8576124c86130ab565b6000612b938360206134bd565b612b9e90600461354a565b612ba990602061354a565b82511015612bca57604051631d098e2d60e21b815260040160405180910390fd5b6000612bd78460206134bd565b612be290600461354a565b92909201602001519392505050565b60608315612c60578251600003612c59576001600160a01b0385163b612c595760405162461bcd60e51b815260206004820152601d60248201527f416464726573733a2063616c6c20746f206e6f6e2d636f6e74726163740000006044820152606401610642565b50816113c1565b6113c18383815115612c755781518083602001fd5b8060405162461bcd60e51b815260040161064291906135f0565b6001600160a01b03811681146105f757600080fd5b600060208284031215612cb657600080fd5b81356124c881612c8f565b634e487b7160e01b600052604160045260246000fd5b604051601f8201601f1916810167ffffffffffffffff81118282101715612d0057612d00612cc1565b604052919050565b600067ffffffffffffffff821115612d2257612d22612cc1565b50601f01601f191660200190565b600082601f830112612d4157600080fd5b8135612d54612d4f82612d08565b612cd7565b818152846020838601011115612d6957600080fd5b816020850160208301376000918101602001919091529392505050565b600060208284031215612d9857600080fd5b813567ffffffffffffffff811115612daf57600080fd5b6113c184828501612d30565b600080600080600060808688031215612dd357600080fd5b8535612dde81612c8f565b945060208601359350604086013567ffffffffffffffff80821115612e0257600080fd5b818801915088601f830112612e1657600080fd5b813581811115612e2557600080fd5b896020828501011115612e3757600080fd5b602083019550809450505050606086013560028110612e5557600080fd5b809150509295509295909350565b60008060408385031215612e7657600080fd5b8235612e8181612c8f565b9150602083013567ffffffffffffffff811115612e9d57600080fd5b612ea985828601612d30565b9150509250929050565b60005b83811015612ece578181015183820152602001612eb6565b50506000910152565b60008151808452612eef816020860160208601612eb3565b601f01601f19169290920160200192915050565b82151581526040602082015260006113c16040830184612ed7565b600067ffffffffffffffff821115612f3857612f38612cc1565b5060051b60200190565b803560038110612f5157600080fd5b919050565b600082601f830112612f6757600080fd5b81356020612f77612d4f83612f1e565b82815260059290921b84018101918181019086841115612f9657600080fd5b8286015b84811015612fb857612fab81612f42565b8352918301918301612f9a565b509695505050505050565b60008060408385031215612fd657600080fd5b823567ffffffffffffffff80821115612fee57600080fd5b818501915085601f83011261300257600080fd5b81356020613012612d4f83612f1e565b82815260059290921b8401810191818101908984111561303157600080fd5b948201945b838610156130665785356001600160e01b0319811681146130575760008081fd5b82529482019490820190613036565b9650508601359250508082111561307c57600080fd5b50612ea985828601612f56565b6000806040838503121561309c57600080fd5b50508035926020909101359150565b634e487b7160e01b600052602160045260246000fd5b600381106130d1576130d16130ab565b9052565b604080825283519082018190526000906020906060840190828701845b828110156131185781516001600160e01b031916845292840192908401906001016130f2565b5050508381038285015284518082528583019183019060005b81811015613154576131448385516130c1565b9284019291840191600101613131565b5090979650505050505050565b6020808252825182820181905260009190848201906040850190845b818110156131995783518352928401929184019160010161317d565b50909695505050505050565b6000602082840312156131b757600080fd5b5035919050565b600080600080608085870312156131d457600080fd5b84356131df81612c8f565b935060208501356131ef81612c8f565b925060408501356131ff81612c8f565b9396929550929360600135925050565b60008060006060848603121561322457600080fd5b833561322f81612c8f565b9250602084013561323f81612c8f565b915061324d60408501612f42565b90509250925092565b60208101610e3b82846130c1565b60008060006060848603121561327957600080fd5b833561328481612c8f565b95602085013595506040909401359392505050565b6020808252602c908201527f46756e6374696f6e206d7573742062652063616c6c6564207468726f7567682060408201526b19195b1959d85d1958d85b1b60a21b606082015260800190565b6020808252602c908201527f46756e6374696f6e206d7573742062652063616c6c6564207468726f7567682060408201526b6163746976652070726f787960a01b606082015260800190565b60008060006060848603121561334657600080fd5b835161335181612c8f565b602085015190935061336281612c8f565b80925050604084015190509250925092565b60006020828403121561338657600080fd5b5051919050565b60006020828403121561339f57600080fd5b81516124c881612c8f565b60018060a01b03851681528360208201526080604082015260006133d16080830185612ed7565b9050600283106133e3576133e36130ab565b82606083015295945050505050565b80518015158114612f5157600080fd5b60006020828403121561341457600080fd5b6124c8826133f2565b6000806040838503121561343057600080fd5b613439836133f2565b9150602083015167ffffffffffffffff81111561345557600080fd5b8301601f8101851361346657600080fd5b8051613474612d4f82612d08565b81815286602083850101111561348957600080fd5b61349a826020830160208601612eb3565b8093505050509250929050565b634e487b7160e01b600052601160045260246000fd5b8082028115828204841417610e3b57610e3b6134a7565b81810381811115610e3b57610e3b6134a7565b634e487b7160e01b600052603260045260246000fd5b60006001820161350f5761350f6134a7565b5060010190565b6001600160e01b031983168152604081016124c860208301846130c1565b634e487b7160e01b600052603160045260246000fd5b80820180821115610e3b57610e3b6134a7565b805160208201516001600160e01b0319808216929190600483101561358c5780818460040360031b1b83161693505b505050919050565b805160208083015191908110156135b5576000198160200360031b1b821691505b50919050565b600082516135cd818460208701612eb3565b9190910192915050565b60ff8281168282160390811115610e3b57610e3b6134a7565b6020815260006124c86020830184612ed756fe360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc416464726573733a206c6f772d6c6576656c2064656c65676174652063616c6c206661696c6564a2646970667358221220ec1b42309ca1be4e1dea19d21acd6d364096f019a09d088fde775ab08446545564736f6c63430008130033
    /// ```
    #[rustfmt::skip]
    #[allow(clippy::all)]
    pub static BYTECODE: alloy_sol_types::private::Bytes = alloy_sol_types::private::Bytes::from_static(
        b"`\xA0`@R0`\x80R4\x80\x15b\0\0\x15W`\0\x80\xFD[Pb\0\0 b\0\0&V[b\0\0\xE7V[`\0Ta\x01\0\x90\x04`\xFF\x16\x15b\0\0\x93W`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`'`$\x82\x01R\x7FInitializable: contract is initi`D\x82\x01Rfalizing`\xC8\x1B`d\x82\x01R`\x84\x01`@Q\x80\x91\x03\x90\xFD[`\0T`\xFF\x90\x81\x16\x14b\0\0\xE5W`\0\x80T`\xFF\x19\x16`\xFF\x90\x81\x17\x90\x91U`@Q\x90\x81R\x7F\x7F&\xB8?\xF9n\x1F+jh/\x138R\xF6y\x8A\t\xC4e\xDA\x95\x92\x14`\xCE\xFB8G@$\x98\x90` \x01`@Q\x80\x91\x03\x90\xA1[V[`\x80Qa6\x80b\0\x01\x1F`\09`\0\x81\x81a\x06\x04\x01R\x81\x81a\x06M\x01R\x81\x81a\t\xCF\x01R\x81\x81a\n\x0F\x01Ra\x0B?\x01Ra6\x80`\0\xF3\xFE`\x80`@R`\x046\x10a\x01\xC2W`\x005`\xE0\x1C\x80cs\x9CK\x08\x11a\0\xF7W\x80c\xB5sib\x11a\0\x95W\x80c\xDCDjJ\x11a\0dW\x80c\xDCDjJ\x14a\x05@W\x80c\xDFNo\x8A\x14a\x05mW\x80c\xF2\xFD\xE3\x8B\x14a\x05\xA4W\x80c\xFA\x19P\x1D\x14a\x05\xC4W`\0\x80\xFD[\x80c\xB5sib\x14a\x04\xC0W\x80c\xC6\x86\x05\xC8\x14a\x04\xE0W\x80c\xC6\x8C:\x83\x14a\x05\0W\x80c\xDC\x06\x10\x9D\x14a\x05 W`\0\x80\xFD[\x80c\x9D\x95\xF1\xCC\x11a\0\xD1W\x80c\x9D\x95\xF1\xCC\x14a\x04@W\x80c\xA2E\x0F\x89\x14a\x04`W\x80c\xA7l\x9A/\x14a\x04\x80W\x80c\xB2\xB9\x9E\xC9\x14a\x04\xA0W`\0\x80\xFD[\x80cs\x9CK\x08\x14a\x03\xE2W\x80c\x8B\x95\xEC\xCD\x14a\x04\x02W\x80c\x8D\xA5\xCB[\x14a\x04\"W`\0\x80\xFD[\x80cO\x1E\xF2\x86\x11a\x01dW\x80cV\xF5Q\x17\x11a\x01>W\x80cV\xF5Q\x17\x14a\x03HW\x80c`\x97lK\x14a\x03}W\x80cc\xFE;V\x14a\x03\xABW\x80cqP\x18\xA6\x14a\x03\xCDW`\0\x80\xFD[\x80cO\x1E\xF2\x86\x14a\x02\xE4W\x80cR)\x07?\x14a\x02\xF7W\x80cR\xD1\x90-\x14a\x03%W`\0\x80\xFD[\x80c6Y\xCF\xE6\x11a\x01\xA0W\x80c6Y\xCF\xE6\x14a\x02oW\x80cC\x9F\xAB\x91\x14a\x02\x8FW\x80cF\x87!\xA7\x14a\x02\xAFW\x80cJ\x1B\xA4\x08\x14a\x02\xCFW`\0\x80\xFD[\x80c\x01u\x01R\x14a\x01\xC7W\x80c)D\x02\xCC\x14a\x02\x15W\x80c4\x01\xCD\xE8\x14a\x02MW[`\0\x80\xFD[4\x80\x15a\x01\xD3W`\0\x80\xFD[Pa\x02\0a\x01\xE26`\x04a,\xA4V[`\x01`\x01`\xA0\x1B\x03\x16`\0\x90\x81R`\xCC` R`@\x90 T`\xFF\x16\x90V[`@Q\x90\x15\x15\x81R` \x01[`@Q\x80\x91\x03\x90\xF3[4\x80\x15a\x02!W`\0\x80\xFD[P`\xC9Ta\x025\x90`\x01`\x01`\xA0\x1B\x03\x16\x81V[`@Q`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x81R` \x01a\x02\x0CV[4\x80\x15a\x02YW`\0\x80\xFD[Pa\x02ma\x02h6`\x04a,\xA4V[a\x05\xE4V[\0[4\x80\x15a\x02{W`\0\x80\xFD[Pa\x02ma\x02\x8A6`\x04a,\xA4V[a\x05\xFAV[4\x80\x15a\x02\x9BW`\0\x80\xFD[Pa\x02ma\x02\xAA6`\x04a-\x86V[a\x06\xDFV[4\x80\x15a\x02\xBBW`\0\x80\xFD[Pa\x02\0a\x02\xCA6`\x04a-\xBBV[a\t+V[4\x80\x15a\x02\xDBW`\0\x80\xFD[Pa\x02\0`\x01\x81V[a\x02ma\x02\xF26`\x04a.cV[a\t\xC5V[4\x80\x15a\x03\x03W`\0\x80\xFD[Pa\x03\x17a\x03\x126`\x04a-\xBBV[a\n\x91V[`@Qa\x02\x0C\x92\x91\x90a/\x03V[4\x80\x15a\x031W`\0\x80\xFD[Pa\x03:a\x0B2V[`@Q\x90\x81R` \x01a\x02\x0CV[4\x80\x15a\x03TW`\0\x80\xFD[Pa\x03ha\x03c6`\x04a/\xC3V[a\x0B\xE5V[`@\x80Q\x92\x83R` \x83\x01\x91\x90\x91R\x01a\x02\x0CV[4\x80\x15a\x03\x89W`\0\x80\xFD[Pa\x03\x9Da\x03\x986`\x04a0\x89V[a\x0B\xFEV[`@Qa\x02\x0C\x92\x91\x90a0\xD5V[4\x80\x15a\x03\xB7W`\0\x80\xFD[Pa\x03\xC0a\x0C\x0BV[`@Qa\x02\x0C\x91\x90a1aV[4\x80\x15a\x03\xD9W`\0\x80\xFD[Pa\x02ma\x0C\x1CV[4\x80\x15a\x03\xEEW`\0\x80\xFD[Pa\x02ma\x03\xFD6`\x04a1\xA5V[a\x0C0V[4\x80\x15a\x04\x0EW`\0\x80\xFD[Pa\x02ma\x04\x1D6`\x04a,\xA4V[a\x0CCV[4\x80\x15a\x04.W`\0\x80\xFD[P`\x97T`\x01`\x01`\xA0\x1B\x03\x16a\x025V[4\x80\x15a\x04LW`\0\x80\xFD[Pa\x02ma\x04[6`\x04a,\xA4V[a\x0C\x95V[4\x80\x15a\x04lW`\0\x80\xFD[Pa\x02ma\x04{6`\x04a1\xA5V[a\x0C\xA6V[4\x80\x15a\x04\x8CW`\0\x80\xFD[Pa\x02ma\x04\x9B6`\x04a1\xA5V[a\x0C\xB7V[4\x80\x15a\x04\xACW`\0\x80\xFD[Pa\x02ma\x04\xBB6`\x04a,\xA4V[a\x0C\xCAV[4\x80\x15a\x04\xCCW`\0\x80\xFD[Pa\x02ma\x04\xDB6`\x04a1\xA5V[a\rTV[4\x80\x15a\x04\xECW`\0\x80\xFD[Pa\x02ma\x04\xFB6`\x04a1\xBEV[a\r\x8CV[4\x80\x15a\x05\x0CW`\0\x80\xFD[Pa\x02ma\x05\x1B6`\x04a2\x0FV[a\r\xA8V[4\x80\x15a\x05,W`\0\x80\xFD[Pa\x02ma\x05;6`\x04a1\xA5V[a\r\xC2V[4\x80\x15a\x05LW`\0\x80\xFD[Pa\x05`a\x05[6`\x04a0\x89V[a\x0E\x1DV[`@Qa\x02\x0C\x91\x90a2VV[4\x80\x15a\x05yW`\0\x80\xFD[Pa\x05\x8Da\x05\x886`\x04a,\xA4V[a\x0EAV[`@\x80Q\x92\x15\x15\x83R` \x83\x01\x91\x90\x91R\x01a\x02\x0CV[4\x80\x15a\x05\xB0W`\0\x80\xFD[Pa\x02ma\x05\xBF6`\x04a,\xA4V[a\x0EXV[4\x80\x15a\x05\xD0W`\0\x80\xFD[Pa\x02ma\x05\xDF6`\x04a2dV[a\x0E\xCEV[a\x05\xECa\x0E\xE3V[a\x05\xF7`\xCA\x82a\x0F=V[PV[`\x01`\x01`\xA0\x1B\x03\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x160\x03a\x06KW`@QbF\x1B\xCD`\xE5\x1B\x81R`\x04\x01a\x06B\x90a2\x99V[`@Q\x80\x91\x03\x90\xFD[\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\x01`\x01`\xA0\x1B\x03\x16a\x06\x94`\0\x80Q` a6\x04\x839\x81Q\x91RT`\x01`\x01`\xA0\x1B\x03\x16\x90V[`\x01`\x01`\xA0\x1B\x03\x16\x14a\x06\xBAW`@QbF\x1B\xCD`\xE5\x1B\x81R`\x04\x01a\x06B\x90a2\xE5V[a\x06\xC3\x81a\x0F\xA3V[`@\x80Q`\0\x80\x82R` \x82\x01\x90\x92Ra\x05\xF7\x91\x83\x91\x90a\x0F\xABV[`\0Ta\x01\0\x90\x04`\xFF\x16\x15\x80\x80\x15a\x06\xFFWP`\0T`\x01`\xFF\x90\x91\x16\x10[\x80a\x07\x19WP0;\x15\x80\x15a\x07\x19WP`\0T`\xFF\x16`\x01\x14[a\x07|W`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`.`$\x82\x01R\x7FInitializable: contract is alrea`D\x82\x01Rm\x19\x1EH\x1A[\x9A]\x1AX[\x1A^\x99Y`\x92\x1B`d\x82\x01R`\x84\x01a\x06BV[`\0\x80T`\xFF\x19\x16`\x01\x17\x90U\x80\x15a\x07\x9FW`\0\x80Ta\xFF\0\x19\x16a\x01\0\x17\x90U[`\0\x80`\0\x84\x80` \x01\x90Q\x81\x01\x90a\x07\xB8\x91\x90a31V[\x91\x94P\x92P\x90P`\x01`\x01`\xA0\x1B\x03\x83\x16\x15\x80a\x07\xDCWP`\x01`\x01`\xA0\x1B\x03\x82\x16\x15[\x15a\x07\xFAW`@Qc\x86y\x15\xAB`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x81`\x01`\x01`\xA0\x1B\x03\x16\x83`\x01`\x01`\xA0\x1B\x03\x16\x03a\x08,W`@QcY\x8A\x0E!`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0a\x08@`\x97T`\x01`\x01`\xA0\x1B\x03\x16\x90V[`\x01`\x01`\xA0\x1B\x03\x16\x14\x15\x80a\x08`WP`\xC9T`\x01`\x01`\xA0\x1B\x03\x16\x15\x15[\x15a\x08}W`@Qb\xDC\x14\x9F`\xE4\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\xC9\x80T`\x01`\x01`\xA0\x1B\x03\x19\x16`\x01`\x01`\xA0\x1B\x03\x84\x16\x17\x90Ua\x08\xA1\x81a\x11\x16V[a\x08\xAA\x83a\x11\xDCV[`@Q`\x01`\x01`\xA0\x1B\x03\x83\x16\x90\x7F_\xE6\xAA\xBFNy\x08C\xDFC\xAE\x0E\"\xB5\x86 \x06o\xB3\x89)[\xED\xC0j\x92\xDFl;(w}\x90`\0\x90\xA2PPP\x80\x15a\t'W`\0\x80Ta\xFF\0\x19\x16\x90U`@Q`\x01\x81R\x7F\x7F&\xB8?\xF9n\x1F+jh/\x138R\xF6y\x8A\t\xC4e\xDA\x95\x92\x14`\xCE\xFB8G@$\x98\x90` \x01`@Q\x80\x91\x03\x90\xA1[PPV[3`\0\x90\x81R`\xCC` R`@\x81 T`\xFF\x16a\t[W`@Qc\x1F\xB1\xD3\xE5`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\xC9Ta\tx\x90`\xCA\x90`\x01`\x01`\xA0\x1B\x03\x16\x88\x88\x88\x88\x88a\x12.V[a\t\xBB\x86\x86\x86\x86\x80\x80`\x1F\x01` \x80\x91\x04\x02` \x01`@Q\x90\x81\x01`@R\x80\x93\x92\x91\x90\x81\x81R` \x01\x83\x83\x80\x82\x847`\0\x92\x01\x91\x90\x91RP\x88\x92Pa\x12\xD9\x91PPV[\x96\x95PPPPPPV[`\x01`\x01`\xA0\x1B\x03\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x160\x03a\n\rW`@QbF\x1B\xCD`\xE5\x1B\x81R`\x04\x01a\x06B\x90a2\x99V[\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\x01`\x01`\xA0\x1B\x03\x16a\nV`\0\x80Q` a6\x04\x839\x81Q\x91RT`\x01`\x01`\xA0\x1B\x03\x16\x90V[`\x01`\x01`\xA0\x1B\x03\x16\x14a\n|W`@QbF\x1B\xCD`\xE5\x1B\x81R`\x04\x01a\x06B\x90a2\xE5V[a\n\x85\x82a\x0F\xA3V[a\t'\x82\x82`\x01a\x0F\xABV[3`\0\x90\x81R`\xCC` R`@\x81 T``\x90`\xFF\x16a\n\xC4W`@Qc\x1F\xB1\xD3\xE5`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\xC9Ta\n\xE1\x90`\xCA\x90`\x01`\x01`\xA0\x1B\x03\x16\x89\x89\x89\x89\x89a\x12.V[a\x0B$\x87\x87\x87\x87\x80\x80`\x1F\x01` \x80\x91\x04\x02` \x01`@Q\x90\x81\x01`@R\x80\x93\x92\x91\x90\x81\x81R` \x01\x83\x83\x80\x82\x847`\0\x92\x01\x91\x90\x91RP\x89\x92Pa\x13\xC9\x91PPV[\x91P\x91P\x95P\x95\x93PPPPV[`\x000`\x01`\x01`\xA0\x1B\x03\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x16\x14a\x0B\xD2W`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`8`$\x82\x01R\x7FUUPSUpgradeable: must not be cal`D\x82\x01R\x7Fled through delegatecall\0\0\0\0\0\0\0\0`d\x82\x01R`\x84\x01a\x06BV[P`\0\x80Q` a6\x04\x839\x81Q\x91R\x90V[`\0\x80a\x0B\xF2\x84\x84a\x14\xC3V[\x91P\x91P[\x92P\x92\x90PV[``\x80a\x0B\xF2\x84\x84a\x15\xD7V[``a\x0C\x17`\xCAa\x17\x86V[\x90P\x90V[a\x0C$a\x0E\xE3V[a\x0C.`\0a\x11\xDCV[V[a\x0C8a\x0E\xE3V[a\x05\xF7`\xCA\x82a\x17\xE2V[a\x0CKa\x0E\xE3V[`\xC9\x80T`\x01`\x01`\xA0\x1B\x03\x19\x16`\x01`\x01`\xA0\x1B\x03\x83\x16\x90\x81\x17\x90\x91U`@Q\x7F_\xE6\xAA\xBFNy\x08C\xDFC\xAE\x0E\"\xB5\x86 \x06o\xB3\x89)[\xED\xC0j\x92\xDFl;(w}\x90`\0\x90\xA2PV[a\x0C\x9Da\x0E\xE3V[a\x05\xF7\x81a\x18\xA2V[a\x0C\xAEa\x0E\xE3V[a\x05\xF7\x81a\x11\x16V[a\x0C\xBFa\x0E\xE3V[a\x05\xF7`\xCA\x82a\x19(V[a\x0C\xD2a\x0E\xE3V[`\x01`\x01`\xA0\x1B\x03\x81\x16`\0\x90\x81R`\xCC` R`@\x90 T`\xFF\x16a\r\x0BW`@Qc\x1F\xB1\xD3\xE5`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01`\x01`\xA0\x1B\x03\x81\x16`\0\x81\x81R`\xCC` R`@\x80\x82 \x80T`\xFF\x19\x16\x90UQ\x7F\xCF\xC2Af\xDBK\xB6w\xE8W\xCA\xCA\xBD\x15A\xFB+0dP!\xB2|Q0A\x95\x89\xB8M\xB5+\x91\x90\xA2PV[a\r\\a\x0E\xE3V[`\0a\rh\x82``\x1C\x90V[\x90Pa\rs\x81a\x18\xA2V[a\r~`\xCA\x83a\x19\xD9V[a\t'`\xCA\x82\x83`\x01a\x1A\x8BV[a\r\x94a\x0E\xE3V[a\r\xA2`\xCA\x85\x85\x85\x85a\x1B'V[PPPPV[a\r\xB0a\x0E\xE3V[a\r\xBD`\xCA\x84\x84\x84a\x1A\x8BV[PPPV[a\r\xCAa\x0E\xE3V[`\0a\r\xD6\x82``\x1C\x90V[`\x01`\x01`\xA0\x1B\x03\x81\x16`\0\x90\x81R`\xCC` R`@\x90 T\x90\x91P`\xFF\x16a\x0E\x12W`@Qc\x1F\xB1\xD3\xE5`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\t'`\xCA\x83a\x19\xD9V[`\0\x82\x81R`\xCD` \x90\x81R`@\x80\x83 \x84\x84R\x90\x91R\x90 T`\xFF\x16[\x92\x91PPV[`\0\x80a\x0EO`\xCA\x84a\x1C\xABV[\x91P\x91P\x91P\x91V[a\x0E`a\x0E\xE3V[`\x01`\x01`\xA0\x1B\x03\x81\x16a\x0E\xC5W`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`&`$\x82\x01R\x7FOwnable: new owner is the zero a`D\x82\x01Reddress`\xD0\x1B`d\x82\x01R`\x84\x01a\x06BV[a\x05\xF7\x81a\x11\xDCV[a\x0E\xD6a\x0E\xE3V[a\r\xBD`\xCA\x84\x84\x84a\x1D\x0EV[`\x97T`\x01`\x01`\xA0\x1B\x03\x163\x14a\x0C.W`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01\x81\x90R`$\x82\x01R\x7FOwnable: caller is not the owner`D\x82\x01R`d\x01a\x06BV[`\0a\x0FI\x83\x83a\x1EUV[\x90P\x80\x15a\x0F\x8AW`@Q`\x01`\x01`\xA0\x1B\x03\x83\x16\x90\x7F\r\xFC\xE1\xEAK\xA1\xEE\xBA\x89\x1F\xFB*\x06g\x90\xFB\xC2\x93\xA9\xE5\x17\xFEa\xD4\x9D\x15j0\x16_\x93\xF3\x90`\0\x90\xA2PPPV[`@QcJ\x89\x03!`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x05\xF7a\x0E\xE3V[\x7FI\x10\xFD\xFA\x16\xFE\xD3&\x0E\xD0\xE7\x14\x7F|\xC6\xDA\x11\xA6\x02\x08\xB5\xB9@m\x12\xA65aO\xFD\x91CT`\xFF\x16\x15a\x0F\xDEWa\r\xBD\x83a\x1F}V[\x82`\x01`\x01`\xA0\x1B\x03\x16cR\xD1\x90-`@Q\x81c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x92PPP\x80\x15a\x108WP`@\x80Q`\x1F=\x90\x81\x01`\x1F\x19\x16\x82\x01\x90\x92Ra\x105\x91\x81\x01\x90a3tV[`\x01[a\x10\x9BW`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`.`$\x82\x01R\x7FERC1967Upgrade: new implementati`D\x82\x01Rmon is not UUPS`\x90\x1B`d\x82\x01R`\x84\x01a\x06BV[`\0\x80Q` a6\x04\x839\x81Q\x91R\x81\x14a\x11\nW`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`)`$\x82\x01R\x7FERC1967Upgrade: unsupported prox`D\x82\x01Rh\x1AXX\x9B\x19UURQ`\xBA\x1B`d\x82\x01R`\x84\x01a\x06BV[Pa\r\xBD\x83\x83\x83a \x19V[`\0a\x11\"\x82``\x1C\x90V[\x90P`\0\x81`\x01`\x01`\xA0\x1B\x03\x16c\xFC\x0CTj`@Q\x81c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x11dW=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x11\x88\x91\x90a3\x8DV[\x90Pa\x11\xB3`\xCAk\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x85\x16``\x85\x90\x1B`\x01`\x01``\x1B\x03\x19\x16\x17a\x17\xE2V[a\r\xBD`\xCAk\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x85\x16``\x84\x90\x1B`\x01`\x01``\x1B\x03\x19\x16\x17a\x19(V[`\x97\x80T`\x01`\x01`\xA0\x1B\x03\x83\x81\x16`\x01`\x01`\xA0\x1B\x03\x19\x83\x16\x81\x17\x90\x93U`@Q\x91\x16\x91\x90\x82\x90\x7F\x8B\xE0\x07\x9CS\x16Y\x14\x13D\xCD\x1F\xD0\xA4\xF2\x84\x19I\x7F\x97\"\xA3\xDA\xAF\xE3\xB4\x18okdW\xE0\x90`\0\x90\xA3PPV[\x84`\x01`\x01`\xA0\x1B\x03\x16\x86`\x01`\x01`\xA0\x1B\x03\x16\x03a\x12\x8CWa\x12\x87\x87\x84\x84\x80\x80`\x1F\x01` \x80\x91\x04\x02` \x01`@Q\x90\x81\x01`@R\x80\x93\x92\x91\x90\x81\x81R` \x01\x83\x83\x80\x82\x847`\0\x92\x01\x91\x90\x91RPa >\x92PPPV[a\x12\xD0V[a\x12\xD0\x87\x86\x86\x86\x86\x80\x80`\x1F\x01` \x80\x91\x04\x02` \x01`@Q\x90\x81\x01`@R\x80\x93\x92\x91\x90\x81\x81R` \x01\x83\x83\x80\x82\x847`\0\x92\x01\x91\x90\x91RP\x88\x92Pa \xDF\x91PPV[PPPPPPPV[`\0a\x12\xED`\x97T`\x01`\x01`\xA0\x1B\x03\x16\x90V[`\x01`\x01`\xA0\x1B\x03\x16cF\x87!\xA7\x86\x86\x86\x86`@Q\x85c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01a\x13\x1E\x94\x93\x92\x91\x90a3\xAAV[` `@Q\x80\x83\x03\x81`\0\x87Z\xF1\x15\x80\x15a\x13=W=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x13a\x91\x90a4\x02V[\x90P\x80\x15a\x13\x97W`@Q\x7FN.\x86\xD2\x13u\xEB\xCB\xF6\xE9=\xF5\xEB\xDDZ\x91[\xF80$Y\x04\xC3\xB5OH\xAD\xF0\x17\n\xAEK\x90`\0\x90\xA1a\x13\xC1V[`@Q\x7F\xC2M\x93`\x8A\x03\xD2c\xFF\x19\x1Dvw\x14\x1F^\x94\xC4\x96\xE5\x93\x10\x8F:\xAE\x0C\xB5\xB7\x04\x94\xC4\xD3\x90`\0\x90\xA1[\x94\x93PPPPV[`\0``a\x13\xDF`\x97T`\x01`\x01`\xA0\x1B\x03\x16\x90V[`\x01`\x01`\xA0\x1B\x03\x16cR)\x07?\x87\x87\x87\x87`@Q\x85c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01a\x14\x10\x94\x93\x92\x91\x90a3\xAAV[`\0`@Q\x80\x83\x03\x81`\0\x87Z\xF1\x15\x80\x15a\x14/W=`\0\x80>=`\0\xFD[PPPP`@Q=`\0\x82>`\x1F=\x90\x81\x01`\x1F\x19\x16\x82\x01`@Ra\x14W\x91\x90\x81\x01\x90a4\x1DV[\x90\x92P\x90P\x81\x15a\x14\x90W`@Q\x7FN.\x86\xD2\x13u\xEB\xCB\xF6\xE9=\xF5\xEB\xDDZ\x91[\xF80$Y\x04\xC3\xB5OH\xAD\xF0\x17\n\xAEK\x90`\0\x90\xA1a\x14\xBAV[`@Q\x7F\xC2M\x93`\x8A\x03\xD2c\xFF\x19\x1Dvw\x14\x1F^\x94\xC4\x96\xE5\x93\x10\x8F:\xAE\x0C\xB5\xB7\x04\x94\xC4\xD3\x90`\0\x90\xA1[\x94P\x94\x92PPPV[\x81Q`\0\x90\x81\x90`\x07\x81\x11\x15a\x14\xECW`@Qc\x17\xA4\xD9\x87`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x83Q\x85Q\x14a\x15\x0EW`@Qct\xF4\xD57`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0\x80[\x82\x81\x10\x15a\x15qWa\x15%\x81` a4\xBDV[a\x150\x90`\xE0a4\xD4V[`\xE0\x88\x83\x81Q\x81\x10a\x15DWa\x15Da4\xE7V[` \x02` \x01\x01Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x90\x1C\x90\x1B\x82\x17\x91P\x80\x80a\x15i\x90a4\xFDV[\x91PPa\x15\x12V[P`\0[\x82\x81\x10\x15a\x15\xCCWa\x15\x88\x81`\x02a4\xBDV[\x86\x82\x81Q\x81\x10a\x15\x9AWa\x15\x9Aa4\xE7V[` \x02` \x01\x01Q`\x02\x81\x11\x15a\x15\xB3Wa\x15\xB3a0\xABV[\x90\x1B\x91\x90\x91\x17\x90\x80a\x15\xC4\x81a4\xFDV[\x91PPa\x15uV[P\x95\x90\x94P\x92PPPV[``\x80`\x07\x83\x11\x15a\x15\xFCW`@Qc\x17\xA4\xD9\x87`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x82g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x16\x15Wa\x16\x15a,\xC1V[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x16>W\x81` \x01` \x82\x02\x806\x837\x01\x90P[P\x91P\x82g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x16ZWa\x16Za,\xC1V[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x16\x83W\x81` \x01` \x82\x02\x806\x837\x01\x90P[P\x90P`\0[\x83\x81\x10\x15a\x16\xEBW`\xE0a\x16\x9E\x82` a4\xBDV[a\x16\xA9\x90`\xE0a4\xD4V[\x86\x90\x1C\x90\x1B\x83\x82\x81Q\x81\x10a\x16\xC0Wa\x16\xC0a4\xE7V[`\x01`\x01`\xE0\x1B\x03\x19\x90\x92\x16` \x92\x83\x02\x91\x90\x91\x01\x90\x91\x01R\x80a\x16\xE3\x81a4\xFDV[\x91PPa\x16\x89V[P`\0[\x83\x81\x10\x15a\x17~W`\xFEa\x17\x04\x82`\x02a4\xBDV[a\x17\x0F\x90`\xFEa4\xD4V[\x86`\0\x1C\x90\x1B\x90\x1C`\xFF\x16`\x02\x81\x11\x15a\x17+Wa\x17+a0\xABV[\x82\x82\x81Q\x81\x10a\x17=Wa\x17=a4\xE7V[` \x02` \x01\x01\x90`\x02\x81\x11\x15a\x17VWa\x17Va0\xABV[\x90\x81`\x02\x81\x11\x15a\x17iWa\x17ia0\xABV[\x90RP\x80a\x17v\x81a4\xFDV[\x91PPa\x16\xEFV[P\x92P\x92\x90PV[``\x81`\0\x01\x80T\x80` \x02` \x01`@Q\x90\x81\x01`@R\x80\x92\x91\x90\x81\x81R` \x01\x82\x80T\x80\x15a\x17\xD6W` \x02\x82\x01\x91\x90`\0R` `\0 \x90[\x81T\x81R` \x01\x90`\x01\x01\x90\x80\x83\x11a\x17\xC2W[PPPPP\x90P\x91\x90PV[`\0a\x17\xEE\x82``\x1C\x90V[\x90P`\x01`\x01`\xA0\x1B\x03\x81\x16a\x18\x17W`@Qc\x86y\x15\xAB`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x18!\x83\x82a#\x1FV[\x15a\x18?W`@Qct`>\x95`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0a\x18L\x83`\x01a#@V[\x90Pa\x18X\x84\x82a#\xD2V[P\x81`\x01`\x01`\xA0\x1B\x03\x16\x7F_\xFB\x06\xB0\xB0\xE8\xADj\x8F<X1\xD4\x99\xDF\xA6\x12\xD9\xC9\xD4\xDC\x10{\xBDf\xF1\x8Fa\xA6I.q\x82`@Qa\x18\x94\x91\x81R` \x01\x90V[`@Q\x80\x91\x03\x90\xA2PPPPV[`\x01`\x01`\xA0\x1B\x03\x81\x16`\0\x90\x81R`\xCC` R`@\x90 T`\xFF\x16\x15a\x18\xDCW`@Qc8\xE8\x16\xA5`\xE2\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01`\x01`\xA0\x1B\x03\x81\x16`\0\x81\x81R`\xCC` R`@\x80\x82 \x80T`\xFF\x19\x16`\x01\x17\x90UQ\x7F\xB2]\x03\xAA\xF3\x08\xD7)\x17\t\xBE\x1E\xA2\x8B\x80\x04c\xCF:\x9ALJUU\xD73:\x96L\x1D\xFE\xBD\x91\x90\xA2PV[`\0a\x194\x82``\x1C\x90V[\x90P`\x01`\x01`\xA0\x1B\x03\x81\x16a\x19]W`@Qc\x86y\x15\xAB`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x19g\x83\x82a#\x1FV[\x15a\x19\x85W`@Qct`>\x95`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0a\x19\x91\x83\x82a#@V[\x90Pa\x19\x9D\x84\x82a#\xD2V[P\x81`\x01`\x01`\xA0\x1B\x03\x16\x7F\xAA\xF2k\xB1*\xA8\x9E\xE9k\xBE\x19fzj\x05W'\xB7]?n\xD7\xB8\xB6\x11\xEFe\x19\x18\x02\t\xD6\x82`@Qa\x18\x94\x91\x81R` \x01\x90V[`\0a\x19\xE5\x82``\x1C\x90V[\x90P`\x01`\x01`\xA0\x1B\x03\x81\x16a\x1A\x0EW`@Qc\x86y\x15\xAB`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x1A\x18\x83\x82a#\x1FV[\x15a\x1A6W`@Qct`>\x95`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0a\x1AC\x83`\x02a#@V[\x90Pa\x1AO\x84\x82a#\xD2V[P\x81`\x01`\x01`\xA0\x1B\x03\x16\x7F\x1E\xE2y\x1F,\xAF\x0E\x92\xA9\xDC2\xA3z\x9E\xA5:\xB6\xACzo\xB8\xF2\xD0\x90\xE5:\x06}:C\xF6\xAC\x82`@Qa\x18\x94\x91\x81R` \x01\x90V[`\0\x80\x80R`\x03\x85\x01` R`@\x81 \x82\x91a\x1A\xA7\x86\x86a$>V[\x81R` \x81\x01\x91\x90\x91R`@\x01`\0 \x80T`\xFF\x19\x16`\x01\x83`\x02\x81\x11\x15a\x1A\xD1Wa\x1A\xD1a0\xABV[\x02\x17\x90UP\x81`\x01`\x01`\xA0\x1B\x03\x16\x83`\x01`\x01`\xA0\x1B\x03\x16\x7Ft\x87S\r\xDF\xF1 y\x95\x05\xE5+\x1B\x19\xB6\x93?\x85\xA9\xEE\xAE\x92 \xC8\nz\xD7\xC4)\xB6\x12\xAE\x83`@Qa\x1B\x19\x91\x90a2VV[`@Q\x80\x91\x03\x90\xA3PPPPV[`\0\x80a\x1B5\x83`\x02a\x15\xD7V[\x91P\x91P`\0[`\x02\x81\x10\x15a\x1C\xA1W\x82Q`\0\x90\x84\x90\x83\x90\x81\x10a\x1B\\Wa\x1B\\a4\xE7V[` \x02` \x01\x01Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x14a\x1C\x8FW`\0a\x1B\x98\x87\x85\x84\x81Q\x81\x10a\x1B\x8BWa\x1B\x8Ba4\xE7V[` \x02` \x01\x01Qa$\x83V[\x90P\x82\x82\x81Q\x81\x10a\x1B\xACWa\x1B\xACa4\xE7V[` \x02` \x01\x01Q\x89`\x03\x01`\0\x83\x81R` \x01\x90\x81R` \x01`\0 `\0a\x1B\xD5\x8B\x8Ba$>V[\x81R` \x81\x01\x91\x90\x91R`@\x01`\0 \x80T`\xFF\x19\x16`\x01\x83`\x02\x81\x11\x15a\x1B\xFFWa\x1B\xFFa0\xABV[\x02\x17\x90UP\x85`\x01`\x01`\xA0\x1B\x03\x16\x87`\x01`\x01`\xA0\x1B\x03\x16\x89`\x01`\x01`\xA0\x1B\x03\x16\x7F\xA3\xDFq\x04 \xB0\x1C\xC3\x0F\xF3\x000\x9A\xBB\xC7\xFA\xDDF0\xD4\xAB8[\x0FZ\x12o\xB4\xBA\xBEv+\x87\x86\x81Q\x81\x10a\x1CUWa\x1CUa4\xE7V[` \x02` \x01\x01Q\x87\x87\x81Q\x81\x10a\x1CoWa\x1Coa4\xE7V[` \x02` \x01\x01Q`@Qa\x1C\x85\x92\x91\x90a5\x16V[`@Q\x80\x91\x03\x90\xA4P[\x80a\x1C\x99\x81a4\xFDV[\x91PPa\x1B<V[PPPPPPPPV[`\x01`\x01`\xA0\x1B\x03\x81\x16`\0\x90\x81R`\x01\x83\x01` R`@\x81 T\x81\x90\x80\x82\x03a\x1C\xDCW`\0\x80\x92P\x92PPa\x0B\xF7V[`\x01\x85a\x1C\xE9\x82\x84a4\xD4V[\x81T\x81\x10a\x1C\xF9Wa\x1C\xF9a4\xE7V[\x90`\0R` `\0 \x01T\x92P\x92PPa\x0B\xF7V[`\0\x80a\x1D\x1C\x83`\x07a\x15\xD7V[\x91P\x91P`\0[`\x07\x81\x10\x15a\x12\xD0W\x82Q`\0\x90\x84\x90\x83\x90\x81\x10a\x1DCWa\x1DCa4\xE7V[` \x02` \x01\x01Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x14a\x1ECW`\0a\x1Dr\x87\x85\x84\x81Q\x81\x10a\x1B\x8BWa\x1B\x8Ba4\xE7V[\x90P\x82\x82\x81Q\x81\x10a\x1D\x86Wa\x1D\x86a4\xE7V[` \x90\x81\x02\x91\x90\x91\x01\x81\x01Q`\0\x83\x81R`\x03\x8B\x01\x83R`@\x80\x82 \x8A\x83R\x90\x93R\x91\x90\x91 \x80T`\xFF\x19\x16`\x01\x83`\x02\x81\x11\x15a\x1D\xC6Wa\x1D\xC6a0\xABV[\x02\x17\x90UP\x85\x87`\x01`\x01`\xA0\x1B\x03\x16\x7F\xF2\xFF\xD4\xF0\x9DX\xD0h$\x18\x803\xD31\x8D\x06\xEB\x95{\xFB\x1A\x8F\xFE\xD9\xAFx\xE1\xF1\x91h\xB9\x04\x86\x85\x81Q\x81\x10a\x1E\tWa\x1E\ta4\xE7V[` \x02` \x01\x01Q\x86\x86\x81Q\x81\x10a\x1E#Wa\x1E#a4\xE7V[` \x02` \x01\x01Q`@Qa\x1E9\x92\x91\x90a5\x16V[`@Q\x80\x91\x03\x90\xA3P[\x80a\x1EM\x81a4\xFDV[\x91PPa\x1D#V[`\x01`\x01`\xA0\x1B\x03\x81\x16`\0\x90\x81R`\x01\x83\x01` R`@\x81 T\x80\x15a\x1FsW`\0a\x1E\x83`\x01\x83a4\xD4V[\x85T\x90\x91P`\0\x90a\x1E\x97\x90`\x01\x90a4\xD4V[\x90P\x81\x81\x14a\x1F\x15W`\0\x86`\0\x01\x82\x81T\x81\x10a\x1E\xB7Wa\x1E\xB7a4\xE7V[\x90`\0R` `\0 \x01T\x90P\x80\x87`\0\x01\x84\x81T\x81\x10a\x1E\xDAWa\x1E\xDAa4\xE7V[\x90`\0R` `\0 \x01\x81\x90UP\x83\x87`\x01\x01`\0a\x1E\xF9\x84``\x1C\x90V[`\x01`\x01`\xA0\x1B\x03\x16\x81R` \x81\x01\x91\x90\x91R`@\x01`\0 UP[\x85T\x86\x90\x80a\x1F&Wa\x1F&a54V[`\x01\x90\x03\x81\x81\x90`\0R` `\0 \x01`\0\x90U\x90U\x85`\x01\x01`\0\x86`\x01`\x01`\xA0\x1B\x03\x16`\x01`\x01`\xA0\x1B\x03\x16\x81R` \x01\x90\x81R` \x01`\0 `\0\x90U`\x01\x93PPPPa\x0E;V[`\0\x91PPa\x0E;V[`\x01`\x01`\xA0\x1B\x03\x81\x16;a\x1F\xEAW`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`-`$\x82\x01R\x7FERC1967: new implementation is n`D\x82\x01Rl\x1B\xDD\x08\x18H\x18\xDB\xDB\x9D\x1C\x98X\xDD`\x9A\x1B`d\x82\x01R`\x84\x01a\x06BV[`\0\x80Q` a6\x04\x839\x81Q\x91R\x80T`\x01`\x01`\xA0\x1B\x03\x19\x16`\x01`\x01`\xA0\x1B\x03\x92\x90\x92\x16\x91\x90\x91\x17\x90UV[a \"\x83a$\xCFV[`\0\x82Q\x11\x80a /WP\x80[\x15a\r\xBDWa\r\xA2\x83\x83a%\x0FV[`\0\x80`\0```\0\x80`$\x87\x01Q\x90P\x80` \x14a pW`@Qc~\xD1\x117`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`d[\x87Q\x81\x10\x15a \xD4W\x80\x88\x01Q`\xF8\x1C\x96P`\x01\x81\x01\x88\x01Q``\x1C\x95P`\x15\x81\x01\x88\x01Q\x94P`5\x81\x01\x88\x01Q\x92P`5\x81\x01\x88\x01\x93Pa \xB8\x89\x87\x87\x87\x8Ba \xDFV[a \xC3\x83`Ua5JV[a \xCD\x90\x82a5JV[\x90Pa sV[PPPPPPPPPV[\x81Q\x15\x80\x15\x90a \xF0WP`\x04\x82Q\x10[\x15a!\x0EW`@Qc#B`\x91`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0a!\x1A\x86\x86a%4V[\x90Pa!'\x84\x83\x83a%\x9FV[`\0a!2\x84a5]V[\x90P`\0a!B\x85Q\x84\x84a&AV[\x90P`\0\x81`\x03\x81\x11\x15a!XWa!Xa0\xABV[\x03a!vW`@QcXr07`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x03\x81`\x03\x81\x11\x15a!\x8AWa!\x8Aa0\xABV[\x03a!\x97WPPPa#\x18V[`\0\x80a!\xA3\x85a'\xE9V[`\x02\x81\x11\x15a!\xB4Wa!\xB4a0\xABV[\x03a!\xD5Wa!\xCE\x89a!\xC7\x8A\x86a$\x83V[\x85\x89a(\x04V[\x90Pa\"9V[`\x01a!\xE0\x85a'\xE9V[`\x02\x81\x11\x15a!\xF1Wa!\xF1a0\xABV[\x03a\"\x0BWa!\xCE\x89a\"\x04\x8A\x86a$\x83V[\x85\x89a(\x9BV[`\x02a\"\x16\x85a'\xE9V[`\x02\x81\x11\x15a\"'Wa\"'a0\xABV[\x03a\"9Wa\"6\x89\x89a)\xCAV[\x90P[`\x02\x81`\x02\x81\x11\x15a\"MWa\"Ma0\xABV[\x14\x80a\"\x86WP`\0\x81`\x02\x81\x11\x15a\"hWa\"ha0\xABV[\x14\x80\x15a\"\x86WP`\x01\x82`\x03\x81\x11\x15a\"\x84Wa\"\x84a0\xABV[\x14[\x15a\"\xA4W`@Qc\x86M\xD1\xE7`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01\x81`\x02\x81\x11\x15a\"\xB8Wa\"\xB8a0\xABV[\x14\x80a\"\xF1WP`\0\x81`\x02\x81\x11\x15a\"\xD3Wa\"\xD3a0\xABV[\x14\x80\x15a\"\xF1WP`\x02\x82`\x03\x81\x11\x15a\"\xEFWa\"\xEFa0\xABV[\x14[\x15a\"\xFFWPPPPa#\x18V[`@Qc\x08\xD5\xA8\xB1`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[PPPPPV[`\x01`\x01`\xA0\x1B\x03\x16`\0\x90\x81R`\x01\x91\x90\x91\x01` R`@\x90 T\x15\x15\x90V[`\0\x80\x80`\x01\x84`\x02\x81\x11\x15a#XWa#Xa0\xABV[\x03a#pWPj\xFF\0\0\0\0\0\0\0\0\xFF\xFF\x19a#\xABV[`\0\x84`\x02\x81\x11\x15a#\x84Wa#\x84a0\xABV[\x03a#\x9CWPj\xFF\0\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\0\x19a#\xABV[Pj\xFF\0\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x19[\x80\x85\x16\x91P`P\x84`\x02\x81\x11\x15a#\xC4Wa#\xC4a0\xABV[\x90\x1B\x91\x90\x91\x17\x94\x93PPPPV[`\0a#\xE7\x83a#\xE2\x84``\x1C\x90V[a#\x1FV[a$6W\x82T`\x01\x81\x81\x01\x85U`\0\x85\x81R` \x81 \x90\x92\x01\x84\x90U\x84T\x91\x90\x85\x01\x90a$\x14\x85``\x1C\x90V[`\x01`\x01`\xA0\x1B\x03\x16\x81R` \x81\x01\x91\x90\x91R`@\x01`\0 UP`\x01a\x0E;V[P`\0a\x0E;V[`@Q`\x01`\x01``\x1B\x03\x19``\x84\x81\x1B\x82\x16` \x84\x01R\x83\x90\x1B\x16`4\x82\x01R`\0\x90`H\x01`@Q` \x81\x83\x03\x03\x81R\x90`@R\x80Q\x90` \x01 \x90P\x92\x91PPV[`@Q`\x01`\x01``\x1B\x03\x19``\x84\x90\x1B\x16` \x82\x01R`\x01`\x01`\xE0\x1B\x03\x19\x82\x16`4\x82\x01R`\0\x90`8\x01`@Q` \x81\x83\x03\x03\x81R\x90`@Ra$\xC8\x90a5\x94V[\x93\x92PPPV[a$\xD8\x81a\x1F}V[`@Q`\x01`\x01`\xA0\x1B\x03\x82\x16\x90\x7F\xBC|\xD7Z \xEE'\xFD\x9A\xDE\xBA\xB3 A\xF7U!M\xBCk\xFF\xA9\x0C\xC0\"[9\xDA.\\-;\x90`\0\x90\xA2PV[``a$\xC8\x83\x83`@Q\x80``\x01`@R\x80`'\x81R` \x01a6$`'\x919a)\xFEV[`\x01`\x01`\xA0\x1B\x03\x81\x16`\0\x90\x81R`\x01\x83\x01` R`@\x81 T\x80\x82\x03a%oW`@Qc-\x05\x19\xAD`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x83a%{`\x01\x83a4\xD4V[\x81T\x81\x10a%\x8BWa%\x8Ba4\xE7V[\x90`\0R` `\0 \x01T\x91PP\x92\x91PPV[`\x01a%\xAA\x82a*lV[`\x01\x81\x11\x15a%\xBBWa%\xBBa0\xABV[\x14a%\xD9W`@Qc;\xCD\x10+`\xE2\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01\x82`\x01\x81\x11\x15a%\xEDWa%\xEDa0\xABV[\x03a&\x0BW`@Qc\x06\xC4\xA1\xC7`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0\x83\x11\x80\x15a&#WPa&!\x81`\x02a*\x87V[\x15[\x15a\r\xBDW`@Qc\t\xE9\xCDI`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0\x80a&M\x84a*\xBDV[\x90P\x84\x15\x80a&dWP`\x01`\x01`\xE0\x1B\x03\x19\x83\x16\x15[\x80a&\x80WP`\x03\x81`\x03\x81\x11\x15a&~Wa&~a0\xABV[\x14[\x80a&\x9CWP`\0\x81`\x03\x81\x11\x15a&\x9AWa&\x9Aa0\xABV[\x14[\x15a&\xA8W\x90Pa$\xC8V[`\0cy\x93\xB9G`\xE1\x1B`\x01`\x01`\xE0\x1B\x03\x19\x85\x16\x01a&\xD4Wa&\xCD\x85`\0a*\xD8V[\x90Pa'\xBFV[c\xAB]\x12\x0B`\xE0\x1B`\x01`\x01`\xE0\x1B\x03\x19\x85\x16\x01a&\xF7Wa&\xCD\x85`\x02a*\xD8V[cBY\xA0\xBB`\xE0\x1B`\x01`\x01`\xE0\x1B\x03\x19\x85\x16\x01a'\x1AWa&\xCD\x85`\x03a*\xD8V[c\x9A\xEA\xEBA`\xE0\x1B`\x01`\x01`\xE0\x1B\x03\x19\x85\x16\x01a'=Wa&\xCD\x85`\x04a*\xD8V[c\xF5A:q`\xE0\x1B`\x01`\x01`\xE0\x1B\x03\x19\x85\x16\x01a'`Wa&\xCD\x85`\x05a*\xD8V[c\xF6\xA1XM`\xE0\x1B`\x01`\x01`\xE0\x1B\x03\x19\x85\x16\x01a'\x83Wa&\xCD\x85`\x07a*\xD8V[c2\x13\"\x1D`\xE1\x1B`\x01`\x01`\xE0\x1B\x03\x19\x85\x16\x01a'\xA6Wa&\xCD\x85`\x08a*\xD8V[`@Qc\x18\xF4\xC1#`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0\x81`\x04\x81\x11\x15a'\xD3Wa'\xD3a0\xABV[\x03a'\xE0WP\x90Pa$\xC8V[a\t\xBB\x81a+,V[`\0`\xFF`P\x83\x90\x1C\x16`\x02\x81\x11\x15a\x0E;Wa\x0E;a0\xABV[`\0`\x01`\x01`\xE0\x1B\x03\x19\x83\x16c\t^\xA7\xB3`\xE0\x1B\x14\x80\x15\x90a(8WP`\x01`\x01`\xE0\x1B\x03\x19\x83\x16cM\xEC\xDD\xE3`\xE1\x1B\x14\x15[\x15a(VW`@Qc\x18\xF4\xC1#`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0a(c`\0\x84a+\x86V[\x90P`\0a(q3\x83a$>V[`\0\x87\x81R`\x03\x89\x01` \x90\x81R`@\x80\x83 \x93\x83R\x92\x90R T`\xFF\x16\x92PPP\x94\x93PPPPV[`\0\x80a(\xA9`\0\x84a+\x86V[\x90P`\x01`\x01`\xA0\x1B\x03\x81\x163\x14a(\xD4W`@Qcn\xB01_`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0cy\x93\xB9G`\xE1\x1B`\x01`\x01`\xE0\x1B\x03\x19\x86\x16\x01a)\0Wa(\xF9`\x01\x85a+\x86V[\x90Pa)\xA1V[c\xAB]\x12\x0B`\xE0\x1B`\x01`\x01`\xE0\x1B\x03\x19\x86\x16\x01a)9W`\0a)%`\x01\x86a+\x86V[\x90Pa)1\x81\x84a$>V[\x91PPa)\xA1V[`\x01`\x01`\xE0\x1B\x03\x19\x85\x16c\xBD\xA6_E`\xE0\x1B\x14\x80a)hWP`\x01`\x01`\xE0\x1B\x03\x19\x85\x16ce\x15\x14\xBF`\xE0\x1B\x14[\x80a)\x83WP`\x01`\x01`\xE0\x1B\x03\x19\x85\x16c\n\xBE\xC5\x8F`\xE0\x1B\x14[\x15a'\xA6W`\0a)\x95`\x01\x86a+\x86V[\x90Pa)1\x83\x82a$>V[`\0\x86\x81R`\x03\x88\x01` \x90\x81R`@\x80\x83 \x93\x83R\x92\x90R T`\xFF\x16\x91PP\x94\x93PPPPV[`\0\x80a)\xD73\x84a$>V[`\0\x80\x80R`\x03\x86\x01` \x90\x81R`@\x80\x83 \x93\x83R\x92\x90R T`\xFF\x16\x91PP\x92\x91PPV[```\0\x80\x85`\x01`\x01`\xA0\x1B\x03\x16\x85`@Qa*\x1B\x91\x90a5\xBBV[`\0`@Q\x80\x83\x03\x81\x85Z\xF4\x91PP=\x80`\0\x81\x14a*VW`@Q\x91P`\x1F\x19`?=\x01\x16\x82\x01`@R=\x82R=`\0` \x84\x01>a*[V[``\x91P[P\x91P\x91Pa\t\xBB\x86\x83\x83\x87a+\xF1V[`\0`\xFF`X\x83\x90\x1C\x16`\x01\x81\x11\x15a\x0E;Wa\x0E;a0\xABV[`\0\x81`\x02\x81\x11\x15a*\x9BWa*\x9Ba0\xABV[a*\xA4\x84a'\xE9V[`\x02\x81\x11\x15a*\xB5Wa*\xB5a0\xABV[\x14\x93\x92PPPV[`\0`\xFF`H\x83\x90\x1C\x16`\x03\x81\x11\x15a\x0E;Wa\x0E;a0\xABV[`\0`\t\x82\x10a*\xFBW`@Qc\xB4J\xF9\xAF`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0a+\x08\x83`\x08a4\xBDV[a+\x13\x90`\xB8a5JV[\x90P\x83\x81\x1B`\xF8\x1C`\x04\x81\x11\x15a\x13\xC1Wa\x13\xC1a0\xABV[`\0\x80\x82`\x04\x81\x11\x15a+AWa+Aa0\xABV[\x90P\x80`\xFF\x16`\0\x03a+gW`@Qc\xD8EZ\x13`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a+r`\x01\x82a5\xD7V[`\xFF\x16`\x03\x81\x11\x15a$\xC8Wa$\xC8a0\xABV[`\0a+\x93\x83` a4\xBDV[a+\x9E\x90`\x04a5JV[a+\xA9\x90` a5JV[\x82Q\x10\x15a+\xCAW`@Qc\x1D\t\x8E-`\xE2\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0a+\xD7\x84` a4\xBDV[a+\xE2\x90`\x04a5JV[\x92\x90\x92\x01` \x01Q\x93\x92PPPV[``\x83\x15a,`W\x82Q`\0\x03a,YW`\x01`\x01`\xA0\x1B\x03\x85\x16;a,YW`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`\x1D`$\x82\x01R\x7FAddress: call to non-contract\0\0\0`D\x82\x01R`d\x01a\x06BV[P\x81a\x13\xC1V[a\x13\xC1\x83\x83\x81Q\x15a,uW\x81Q\x80\x83` \x01\xFD[\x80`@QbF\x1B\xCD`\xE5\x1B\x81R`\x04\x01a\x06B\x91\x90a5\xF0V[`\x01`\x01`\xA0\x1B\x03\x81\x16\x81\x14a\x05\xF7W`\0\x80\xFD[`\0` \x82\x84\x03\x12\x15a,\xB6W`\0\x80\xFD[\x815a$\xC8\x81a,\x8FV[cNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[`@Q`\x1F\x82\x01`\x1F\x19\x16\x81\x01g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x82\x82\x10\x17\x15a-\0Wa-\0a,\xC1V[`@R\x91\x90PV[`\0g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x15a-\"Wa-\"a,\xC1V[P`\x1F\x01`\x1F\x19\x16` \x01\x90V[`\0\x82`\x1F\x83\x01\x12a-AW`\0\x80\xFD[\x815a-Ta-O\x82a-\x08V[a,\xD7V[\x81\x81R\x84` \x83\x86\x01\x01\x11\x15a-iW`\0\x80\xFD[\x81` \x85\x01` \x83\x017`\0\x91\x81\x01` \x01\x91\x90\x91R\x93\x92PPPV[`\0` \x82\x84\x03\x12\x15a-\x98W`\0\x80\xFD[\x815g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a-\xAFW`\0\x80\xFD[a\x13\xC1\x84\x82\x85\x01a-0V[`\0\x80`\0\x80`\0`\x80\x86\x88\x03\x12\x15a-\xD3W`\0\x80\xFD[\x855a-\xDE\x81a,\x8FV[\x94P` \x86\x015\x93P`@\x86\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x80\x82\x11\x15a.\x02W`\0\x80\xFD[\x81\x88\x01\x91P\x88`\x1F\x83\x01\x12a.\x16W`\0\x80\xFD[\x815\x81\x81\x11\x15a.%W`\0\x80\xFD[\x89` \x82\x85\x01\x01\x11\x15a.7W`\0\x80\xFD[` \x83\x01\x95P\x80\x94PPPP``\x86\x015`\x02\x81\x10a.UW`\0\x80\xFD[\x80\x91PP\x92\x95P\x92\x95\x90\x93PV[`\0\x80`@\x83\x85\x03\x12\x15a.vW`\0\x80\xFD[\x825a.\x81\x81a,\x8FV[\x91P` \x83\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a.\x9DW`\0\x80\xFD[a.\xA9\x85\x82\x86\x01a-0V[\x91PP\x92P\x92\x90PV[`\0[\x83\x81\x10\x15a.\xCEW\x81\x81\x01Q\x83\x82\x01R` \x01a.\xB6V[PP`\0\x91\x01RV[`\0\x81Q\x80\x84Ra.\xEF\x81` \x86\x01` \x86\x01a.\xB3V[`\x1F\x01`\x1F\x19\x16\x92\x90\x92\x01` \x01\x92\x91PPV[\x82\x15\x15\x81R`@` \x82\x01R`\0a\x13\xC1`@\x83\x01\x84a.\xD7V[`\0g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x15a/8Wa/8a,\xC1V[P`\x05\x1B` \x01\x90V[\x805`\x03\x81\x10a/QW`\0\x80\xFD[\x91\x90PV[`\0\x82`\x1F\x83\x01\x12a/gW`\0\x80\xFD[\x815` a/wa-O\x83a/\x1EV[\x82\x81R`\x05\x92\x90\x92\x1B\x84\x01\x81\x01\x91\x81\x81\x01\x90\x86\x84\x11\x15a/\x96W`\0\x80\xFD[\x82\x86\x01[\x84\x81\x10\x15a/\xB8Wa/\xAB\x81a/BV[\x83R\x91\x83\x01\x91\x83\x01a/\x9AV[P\x96\x95PPPPPPV[`\0\x80`@\x83\x85\x03\x12\x15a/\xD6W`\0\x80\xFD[\x825g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x80\x82\x11\x15a/\xEEW`\0\x80\xFD[\x81\x85\x01\x91P\x85`\x1F\x83\x01\x12a0\x02W`\0\x80\xFD[\x815` a0\x12a-O\x83a/\x1EV[\x82\x81R`\x05\x92\x90\x92\x1B\x84\x01\x81\x01\x91\x81\x81\x01\x90\x89\x84\x11\x15a01W`\0\x80\xFD[\x94\x82\x01\x94[\x83\x86\x10\x15a0fW\x855`\x01`\x01`\xE0\x1B\x03\x19\x81\x16\x81\x14a0WW`\0\x80\x81\xFD[\x82R\x94\x82\x01\x94\x90\x82\x01\x90a06V[\x96PP\x86\x015\x92PP\x80\x82\x11\x15a0|W`\0\x80\xFD[Pa.\xA9\x85\x82\x86\x01a/VV[`\0\x80`@\x83\x85\x03\x12\x15a0\x9CW`\0\x80\xFD[PP\x805\x92` \x90\x91\x015\x91PV[cNH{q`\xE0\x1B`\0R`!`\x04R`$`\0\xFD[`\x03\x81\x10a0\xD1Wa0\xD1a0\xABV[\x90RV[`@\x80\x82R\x83Q\x90\x82\x01\x81\x90R`\0\x90` \x90``\x84\x01\x90\x82\x87\x01\x84[\x82\x81\x10\x15a1\x18W\x81Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x84R\x92\x84\x01\x92\x90\x84\x01\x90`\x01\x01a0\xF2V[PPP\x83\x81\x03\x82\x85\x01R\x84Q\x80\x82R\x85\x83\x01\x91\x83\x01\x90`\0[\x81\x81\x10\x15a1TWa1D\x83\x85Qa0\xC1V[\x92\x84\x01\x92\x91\x84\x01\x91`\x01\x01a11V[P\x90\x97\x96PPPPPPPV[` \x80\x82R\x82Q\x82\x82\x01\x81\x90R`\0\x91\x90\x84\x82\x01\x90`@\x85\x01\x90\x84[\x81\x81\x10\x15a1\x99W\x83Q\x83R\x92\x84\x01\x92\x91\x84\x01\x91`\x01\x01a1}V[P\x90\x96\x95PPPPPPV[`\0` \x82\x84\x03\x12\x15a1\xB7W`\0\x80\xFD[P5\x91\x90PV[`\0\x80`\0\x80`\x80\x85\x87\x03\x12\x15a1\xD4W`\0\x80\xFD[\x845a1\xDF\x81a,\x8FV[\x93P` \x85\x015a1\xEF\x81a,\x8FV[\x92P`@\x85\x015a1\xFF\x81a,\x8FV[\x93\x96\x92\x95P\x92\x93``\x015\x92PPV[`\0\x80`\0``\x84\x86\x03\x12\x15a2$W`\0\x80\xFD[\x835a2/\x81a,\x8FV[\x92P` \x84\x015a2?\x81a,\x8FV[\x91Pa2M`@\x85\x01a/BV[\x90P\x92P\x92P\x92V[` \x81\x01a\x0E;\x82\x84a0\xC1V[`\0\x80`\0``\x84\x86\x03\x12\x15a2yW`\0\x80\xFD[\x835a2\x84\x81a,\x8FV[\x95` \x85\x015\x95P`@\x90\x94\x015\x93\x92PPPV[` \x80\x82R`,\x90\x82\x01R\x7FFunction must be called through `@\x82\x01Rk\x19\x19[\x19Y\xD8]\x19X\xD8[\x1B`\xA2\x1B``\x82\x01R`\x80\x01\x90V[` \x80\x82R`,\x90\x82\x01R\x7FFunction must be called through `@\x82\x01Rkactive proxy`\xA0\x1B``\x82\x01R`\x80\x01\x90V[`\0\x80`\0``\x84\x86\x03\x12\x15a3FW`\0\x80\xFD[\x83Qa3Q\x81a,\x8FV[` \x85\x01Q\x90\x93Pa3b\x81a,\x8FV[\x80\x92PP`@\x84\x01Q\x90P\x92P\x92P\x92V[`\0` \x82\x84\x03\x12\x15a3\x86W`\0\x80\xFD[PQ\x91\x90PV[`\0` \x82\x84\x03\x12\x15a3\x9FW`\0\x80\xFD[\x81Qa$\xC8\x81a,\x8FV[`\x01\x80`\xA0\x1B\x03\x85\x16\x81R\x83` \x82\x01R`\x80`@\x82\x01R`\0a3\xD1`\x80\x83\x01\x85a.\xD7V[\x90P`\x02\x83\x10a3\xE3Wa3\xE3a0\xABV[\x82``\x83\x01R\x95\x94PPPPPV[\x80Q\x80\x15\x15\x81\x14a/QW`\0\x80\xFD[`\0` \x82\x84\x03\x12\x15a4\x14W`\0\x80\xFD[a$\xC8\x82a3\xF2V[`\0\x80`@\x83\x85\x03\x12\x15a40W`\0\x80\xFD[a49\x83a3\xF2V[\x91P` \x83\x01Qg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a4UW`\0\x80\xFD[\x83\x01`\x1F\x81\x01\x85\x13a4fW`\0\x80\xFD[\x80Qa4ta-O\x82a-\x08V[\x81\x81R\x86` \x83\x85\x01\x01\x11\x15a4\x89W`\0\x80\xFD[a4\x9A\x82` \x83\x01` \x86\x01a.\xB3V[\x80\x93PPPP\x92P\x92\x90PV[cNH{q`\xE0\x1B`\0R`\x11`\x04R`$`\0\xFD[\x80\x82\x02\x81\x15\x82\x82\x04\x84\x14\x17a\x0E;Wa\x0E;a4\xA7V[\x81\x81\x03\x81\x81\x11\x15a\x0E;Wa\x0E;a4\xA7V[cNH{q`\xE0\x1B`\0R`2`\x04R`$`\0\xFD[`\0`\x01\x82\x01a5\x0FWa5\x0Fa4\xA7V[P`\x01\x01\x90V[`\x01`\x01`\xE0\x1B\x03\x19\x83\x16\x81R`@\x81\x01a$\xC8` \x83\x01\x84a0\xC1V[cNH{q`\xE0\x1B`\0R`1`\x04R`$`\0\xFD[\x80\x82\x01\x80\x82\x11\x15a\x0E;Wa\x0E;a4\xA7V[\x80Q` \x82\x01Q`\x01`\x01`\xE0\x1B\x03\x19\x80\x82\x16\x92\x91\x90`\x04\x83\x10\x15a5\x8CW\x80\x81\x84`\x04\x03`\x03\x1B\x1B\x83\x16\x16\x93P[PPP\x91\x90PV[\x80Q` \x80\x83\x01Q\x91\x90\x81\x10\x15a5\xB5W`\0\x19\x81` \x03`\x03\x1B\x1B\x82\x16\x91P[P\x91\x90PV[`\0\x82Qa5\xCD\x81\x84` \x87\x01a.\xB3V[\x91\x90\x91\x01\x92\x91PPV[`\xFF\x82\x81\x16\x82\x82\x16\x03\x90\x81\x11\x15a\x0E;Wa\x0E;a4\xA7V[` \x81R`\0a$\xC8` \x83\x01\x84a.\xD7V\xFE6\x08\x94\xA1;\xA1\xA3!\x06g\xC8(I-\xB9\x8D\xCA> v\xCC75\xA9 \xA3\xCAP]8+\xBCAddress: low-level delegate call failed\xA2dipfsX\"\x12 \xEC\x1BB0\x9C\xA1\xBEN\x1D\xEA\x19\xD2\x1A\xCDm6@\x96\xF0\x19\xA0\x9D\x08\x8F\xDEwZ\xB0\x84FTUdsolcC\0\x08\x13\x003",
    );
    /// The runtime bytecode of the contract, as deployed on the network.
    ///
    /// ```text
    ///0x6080604052600436106101c25760003560e01c8063739c4b08116100f7578063b573696211610095578063dc446a4a11610064578063dc446a4a14610540578063df4e6f8a1461056d578063f2fde38b146105a4578063fa19501d146105c457600080fd5b8063b5736962146104c0578063c68605c8146104e0578063c68c3a8314610500578063dc06109d1461052057600080fd5b80639d95f1cc116100d15780639d95f1cc14610440578063a2450f8914610460578063a76c9a2f14610480578063b2b99ec9146104a057600080fd5b8063739c4b08146103e25780638b95eccd146104025780638da5cb5b1461042257600080fd5b80634f1ef2861161016457806356f551171161013e57806356f551171461034857806360976c4b1461037d57806363fe3b56146103ab578063715018a6146103cd57600080fd5b80634f1ef286146102e45780635229073f146102f757806352d1902d1461032557600080fd5b80633659cfe6116101a05780633659cfe61461026f578063439fab911461028f578063468721a7146102af5780634a1ba408146102cf57600080fd5b806301750152146101c7578063294402cc146102155780633401cde81461024d575b600080fd5b3480156101d357600080fd5b506102006101e2366004612ca4565b6001600160a01b0316600090815260cc602052604090205460ff1690565b60405190151581526020015b60405180910390f35b34801561022157600080fd5b5060c954610235906001600160a01b031681565b6040516001600160a01b03909116815260200161020c565b34801561025957600080fd5b5061026d610268366004612ca4565b6105e4565b005b34801561027b57600080fd5b5061026d61028a366004612ca4565b6105fa565b34801561029b57600080fd5b5061026d6102aa366004612d86565b6106df565b3480156102bb57600080fd5b506102006102ca366004612dbb565b61092b565b3480156102db57600080fd5b50610200600181565b61026d6102f2366004612e63565b6109c5565b34801561030357600080fd5b50610317610312366004612dbb565b610a91565b60405161020c929190612f03565b34801561033157600080fd5b5061033a610b32565b60405190815260200161020c565b34801561035457600080fd5b50610368610363366004612fc3565b610be5565b6040805192835260208301919091520161020c565b34801561038957600080fd5b5061039d610398366004613089565b610bfe565b60405161020c9291906130d5565b3480156103b757600080fd5b506103c0610c0b565b60405161020c9190613161565b3480156103d957600080fd5b5061026d610c1c565b3480156103ee57600080fd5b5061026d6103fd3660046131a5565b610c30565b34801561040e57600080fd5b5061026d61041d366004612ca4565b610c43565b34801561042e57600080fd5b506097546001600160a01b0316610235565b34801561044c57600080fd5b5061026d61045b366004612ca4565b610c95565b34801561046c57600080fd5b5061026d61047b3660046131a5565b610ca6565b34801561048c57600080fd5b5061026d61049b3660046131a5565b610cb7565b3480156104ac57600080fd5b5061026d6104bb366004612ca4565b610cca565b3480156104cc57600080fd5b5061026d6104db3660046131a5565b610d54565b3480156104ec57600080fd5b5061026d6104fb3660046131be565b610d8c565b34801561050c57600080fd5b5061026d61051b36600461320f565b610da8565b34801561052c57600080fd5b5061026d61053b3660046131a5565b610dc2565b34801561054c57600080fd5b5061056061055b366004613089565b610e1d565b60405161020c9190613256565b34801561057957600080fd5b5061058d610588366004612ca4565b610e41565b60408051921515835260208301919091520161020c565b3480156105b057600080fd5b5061026d6105bf366004612ca4565b610e58565b3480156105d057600080fd5b5061026d6105df366004613264565b610ece565b6105ec610ee3565b6105f760ca82610f3d565b50565b6001600160a01b037f000000000000000000000000000000000000000000000000000000000000000016300361064b5760405162461bcd60e51b815260040161064290613299565b60405180910390fd5b7f00000000000000000000000000000000000000000000000000000000000000006001600160a01b0316610694600080516020613604833981519152546001600160a01b031690565b6001600160a01b0316146106ba5760405162461bcd60e51b8152600401610642906132e5565b6106c381610fa3565b604080516000808252602082019092526105f791839190610fab565b600054610100900460ff16158080156106ff5750600054600160ff909116105b806107195750303b158015610719575060005460ff166001145b61077c5760405162461bcd60e51b815260206004820152602e60248201527f496e697469616c697a61626c653a20636f6e747261637420697320616c72656160448201526d191e481a5b9a5d1a585b1a5e995960921b6064820152608401610642565b6000805460ff19166001179055801561079f576000805461ff0019166101001790555b6000806000848060200190518101906107b89190613331565b919450925090506001600160a01b03831615806107dc57506001600160a01b038216155b156107fa5760405163867915ab60e01b815260040160405180910390fd5b816001600160a01b0316836001600160a01b03160361082c5760405163598a0e2160e01b815260040160405180910390fd5b60006108406097546001600160a01b031690565b6001600160a01b0316141580610860575060c9546001600160a01b031615155b1561087d5760405162dc149f60e41b815260040160405180910390fd5b60c980546001600160a01b0319166001600160a01b0384161790556108a181611116565b6108aa836111dc565b6040516001600160a01b038316907f5fe6aabf4e790843df43ae0e22b58620066fb389295bedc06a92df6c3b28777d90600090a25050508015610927576000805461ff0019169055604051600181527f7f26b83ff96e1f2b6a682f133852f6798a09c465da95921460cefb38474024989060200160405180910390a15b5050565b33600090815260cc602052604081205460ff1661095b57604051631fb1d3e560e31b815260040160405180910390fd5b60c9546109789060ca906001600160a01b0316888888888861122e565b6109bb868686868080601f0160208091040260200160405190810160405280939291908181526020018383808284376000920191909152508892506112d9915050565b9695505050505050565b6001600160a01b037f0000000000000000000000000000000000000000000000000000000000000000163003610a0d5760405162461bcd60e51b815260040161064290613299565b7f00000000000000000000000000000000000000000000000000000000000000006001600160a01b0316610a56600080516020613604833981519152546001600160a01b031690565b6001600160a01b031614610a7c5760405162461bcd60e51b8152600401610642906132e5565b610a8582610fa3565b61092782826001610fab565b33600090815260cc602052604081205460609060ff16610ac457604051631fb1d3e560e31b815260040160405180910390fd5b60c954610ae19060ca906001600160a01b0316898989898961122e565b610b24878787878080601f0160208091040260200160405190810160405280939291908181526020018383808284376000920191909152508992506113c9915050565b915091509550959350505050565b6000306001600160a01b037f00000000000000000000000000000000000000000000000000000000000000001614610bd25760405162461bcd60e51b815260206004820152603860248201527f555550535570677261646561626c653a206d757374206e6f742062652063616c60448201527f6c6564207468726f7567682064656c656761746563616c6c00000000000000006064820152608401610642565b5060008051602061360483398151915290565b600080610bf284846114c3565b915091505b9250929050565b606080610bf284846115d7565b6060610c1760ca611786565b905090565b610c24610ee3565b610c2e60006111dc565b565b610c38610ee3565b6105f760ca826117e2565b610c4b610ee3565b60c980546001600160a01b0319166001600160a01b0383169081179091556040517f5fe6aabf4e790843df43ae0e22b58620066fb389295bedc06a92df6c3b28777d90600090a250565b610c9d610ee3565b6105f7816118a2565b610cae610ee3565b6105f781611116565b610cbf610ee3565b6105f760ca82611928565b610cd2610ee3565b6001600160a01b038116600090815260cc602052604090205460ff16610d0b57604051631fb1d3e560e31b815260040160405180910390fd5b6001600160a01b038116600081815260cc6020526040808220805460ff19169055517fcfc24166db4bb677e857cacabd1541fb2b30645021b27c5130419589b84db52b9190a250565b610d5c610ee3565b6000610d688260601c90565b9050610d73816118a2565b610d7e60ca836119d9565b61092760ca82836001611a8b565b610d94610ee3565b610da260ca85858585611b27565b50505050565b610db0610ee3565b610dbd60ca848484611a8b565b505050565b610dca610ee3565b6000610dd68260601c90565b6001600160a01b038116600090815260cc602052604090205490915060ff16610e1257604051631fb1d3e560e31b815260040160405180910390fd5b61092760ca836119d9565b600082815260cd6020908152604080832084845290915290205460ff165b92915050565b600080610e4f60ca84611cab565b91509150915091565b610e60610ee3565b6001600160a01b038116610ec55760405162461bcd60e51b815260206004820152602660248201527f4f776e61626c653a206e6577206f776e657220697320746865207a65726f206160448201526564647265737360d01b6064820152608401610642565b6105f7816111dc565b610ed6610ee3565b610dbd60ca848484611d0e565b6097546001600160a01b03163314610c2e5760405162461bcd60e51b815260206004820181905260248201527f4f776e61626c653a2063616c6c6572206973206e6f7420746865206f776e65726044820152606401610642565b6000610f498383611e55565b90508015610f8a576040516001600160a01b038316907f0dfce1ea4ba1eeba891ffb2a066790fbc293a9e517fe61d49d156a30165f93f390600090a2505050565b604051634a89032160e01b815260040160405180910390fd5b6105f7610ee3565b7f4910fdfa16fed3260ed0e7147f7cc6da11a60208b5b9406d12a635614ffd91435460ff1615610fde57610dbd83611f7d565b826001600160a01b03166352d1902d6040518163ffffffff1660e01b8152600401602060405180830381865afa925050508015611038575060408051601f3d908101601f1916820190925261103591810190613374565b60015b61109b5760405162461bcd60e51b815260206004820152602e60248201527f45524331393637557067726164653a206e657720696d706c656d656e7461746960448201526d6f6e206973206e6f74205555505360901b6064820152608401610642565b600080516020613604833981519152811461110a5760405162461bcd60e51b815260206004820152602960248201527f45524331393637557067726164653a20756e737570706f727465642070726f786044820152681a58589b195555525160ba1b6064820152608401610642565b50610dbd838383612019565b60006111228260601c90565b90506000816001600160a01b031663fc0c546a6040518163ffffffff1660e01b8152600401602060405180830381865afa158015611164573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190611188919061338d565b90506111b360ca6bffffffffffffffffffffffff8516606085901b6001600160601b031916176117e2565b610dbd60ca6bffffffffffffffffffffffff8516606084901b6001600160601b03191617611928565b609780546001600160a01b038381166001600160a01b0319831681179093556040519116919082907f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e090600090a35050565b846001600160a01b0316866001600160a01b03160361128c576112878784848080601f01602080910402602001604051908101604052809392919081815260200183838082843760009201919091525061203e92505050565b6112d0565b6112d087868686868080601f0160208091040260200160405190810160405280939291908181526020018383808284376000920191909152508892506120df915050565b50505050505050565b60006112ed6097546001600160a01b031690565b6001600160a01b031663468721a7868686866040518563ffffffff1660e01b815260040161131e94939291906133aa565b6020604051808303816000875af115801561133d573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906113619190613402565b90508015611397576040517f4e2e86d21375ebcbf6e93df5ebdd5a915bf830245904c3b54f48adf0170aae4b90600090a16113c1565b6040517fc24d93608a03d263ff191d7677141f5e94c496e593108f3aae0cb5b70494c4d390600090a15b949350505050565b600060606113df6097546001600160a01b031690565b6001600160a01b0316635229073f878787876040518563ffffffff1660e01b815260040161141094939291906133aa565b6000604051808303816000875af115801561142f573d6000803e3d6000fd5b505050506040513d6000823e601f3d908101601f19168201604052611457919081019061341d565b90925090508115611490576040517f4e2e86d21375ebcbf6e93df5ebdd5a915bf830245904c3b54f48adf0170aae4b90600090a16114ba565b6040517fc24d93608a03d263ff191d7677141f5e94c496e593108f3aae0cb5b70494c4d390600090a15b94509492505050565b8151600090819060078111156114ec576040516317a4d98760e31b815260040160405180910390fd5b835185511461150e576040516374f4d53760e01b815260040160405180910390fd5b6000805b82811015611571576115258160206134bd565b6115309060e06134d4565b60e0888381518110611544576115446134e7565b60200260200101516001600160e01b031916901c901b821791508080611569906134fd565b915050611512565b5060005b828110156115cc576115888160026134bd565b86828151811061159a5761159a6134e7565b602002602001015160028111156115b3576115b36130ab565b901b9190911790806115c4816134fd565b915050611575565b509590945092505050565b60608060078311156115fc576040516317a4d98760e31b815260040160405180910390fd5b8267ffffffffffffffff81111561161557611615612cc1565b60405190808252806020026020018201604052801561163e578160200160208202803683370190505b5091508267ffffffffffffffff81111561165a5761165a612cc1565b604051908082528060200260200182016040528015611683578160200160208202803683370190505b50905060005b838110156116eb5760e061169e8260206134bd565b6116a99060e06134d4565b86901c901b8382815181106116c0576116c06134e7565b6001600160e01b031990921660209283029190910190910152806116e3816134fd565b915050611689565b5060005b8381101561177e5760fe6117048260026134bd565b61170f9060fe6134d4565b8660001c901b901c60ff16600281111561172b5761172b6130ab565b82828151811061173d5761173d6134e7565b60200260200101906002811115611756576117566130ab565b90816002811115611769576117696130ab565b90525080611776816134fd565b9150506116ef565b509250929050565b6060816000018054806020026020016040519081016040528092919081815260200182805480156117d657602002820191906000526020600020905b8154815260200190600101908083116117c2575b50505050509050919050565b60006117ee8260601c90565b90506001600160a01b0381166118175760405163867915ab60e01b815260040160405180910390fd5b611821838261231f565b1561183f576040516374603e9560e11b815260040160405180910390fd5b600061184c836001612340565b905061185884826123d2565b50816001600160a01b03167f5ffb06b0b0e8ad6a8f3c5831d499dfa612d9c9d4dc107bbd66f18f61a6492e718260405161189491815260200190565b60405180910390a250505050565b6001600160a01b038116600090815260cc602052604090205460ff16156118dc576040516338e816a560e21b815260040160405180910390fd5b6001600160a01b038116600081815260cc6020526040808220805460ff19166001179055517fb25d03aaf308d7291709be1ea28b800463cf3a9a4c4a5555d7333a964c1dfebd9190a250565b60006119348260601c90565b90506001600160a01b03811661195d5760405163867915ab60e01b815260040160405180910390fd5b611967838261231f565b15611985576040516374603e9560e11b815260040160405180910390fd5b60006119918382612340565b905061199d84826123d2565b50816001600160a01b03167faaf26bb12aa89ee96bbe19667a6a055727b75d3f6ed7b8b611ef6519180209d68260405161189491815260200190565b60006119e58260601c90565b90506001600160a01b038116611a0e5760405163867915ab60e01b815260040160405180910390fd5b611a18838261231f565b15611a36576040516374603e9560e11b815260040160405180910390fd5b6000611a43836002612340565b9050611a4f84826123d2565b50816001600160a01b03167f1ee2791f2caf0e92a9dc32a37a9ea53ab6ac7a6fb8f2d090e53a067d3a43f6ac8260405161189491815260200190565b600080805260038501602052604081208291611aa7868661243e565b81526020810191909152604001600020805460ff19166001836002811115611ad157611ad16130ab565b0217905550816001600160a01b0316836001600160a01b03167f7487530ddff120799505e52b1b19b6933f85a9eeae9220c80a7ad7c429b612ae83604051611b199190613256565b60405180910390a350505050565b600080611b358360026115d7565b9150915060005b6002811015611ca1578251600090849083908110611b5c57611b5c6134e7565b60200260200101516001600160e01b03191614611c8f576000611b9887858481518110611b8b57611b8b6134e7565b6020026020010151612483565b9050828281518110611bac57611bac6134e7565b60200260200101518960030160008381526020019081526020016000206000611bd58b8b61243e565b81526020810191909152604001600020805460ff19166001836002811115611bff57611bff6130ab565b0217905550856001600160a01b0316876001600160a01b0316896001600160a01b03167fa3df710420b01cc30ff300309abbc7fadd4630d4ab385b0f5a126fb4babe762b878681518110611c5557611c556134e7565b6020026020010151878781518110611c6f57611c6f6134e7565b6020026020010151604051611c85929190613516565b60405180910390a4505b80611c99816134fd565b915050611b3c565b5050505050505050565b6001600160a01b03811660009081526001830160205260408120548190808203611cdc576000809250925050610bf7565b600185611ce982846134d4565b81548110611cf957611cf96134e7565b90600052602060002001549250925050610bf7565b600080611d1c8360076115d7565b9150915060005b60078110156112d0578251600090849083908110611d4357611d436134e7565b60200260200101516001600160e01b03191614611e43576000611d7287858481518110611b8b57611b8b6134e7565b9050828281518110611d8657611d866134e7565b602090810291909101810151600083815260038b01835260408082208a835290935291909120805460ff19166001836002811115611dc657611dc66130ab565b021790555085876001600160a01b03167ff2ffd4f09d58d06824188033d3318d06eb957bfb1a8ffed9af78e1f19168b904868581518110611e0957611e096134e7565b6020026020010151868681518110611e2357611e236134e7565b6020026020010151604051611e39929190613516565b60405180910390a3505b80611e4d816134fd565b915050611d23565b6001600160a01b03811660009081526001830160205260408120548015611f73576000611e836001836134d4565b8554909150600090611e97906001906134d4565b9050818114611f15576000866000018281548110611eb757611eb76134e7565b9060005260206000200154905080876000018481548110611eda57611eda6134e7565b906000526020600020018190555083876001016000611ef98460601c90565b6001600160a01b03168152602081019190915260400160002055505b8554869080611f2657611f26613534565b60019003818190600052602060002001600090559055856001016000866001600160a01b03166001600160a01b031681526020019081526020016000206000905560019350505050610e3b565b6000915050610e3b565b6001600160a01b0381163b611fea5760405162461bcd60e51b815260206004820152602d60248201527f455243313936373a206e657720696d706c656d656e746174696f6e206973206e60448201526c1bdd08184818dbdb9d1c9858dd609a1b6064820152608401610642565b60008051602061360483398151915280546001600160a01b0319166001600160a01b0392909216919091179055565b612022836124cf565b60008251118061202f5750805b15610dbd57610da2838361250f565b60008060006060600080602487015190508060201461207057604051637ed1113760e01b815260040160405180910390fd5b60645b87518110156120d4578088015160f81c96506001810188015160601c955060158101880151945060358101880151925060358101880193506120b8898787878b6120df565b6120c383605561354a565b6120cd908261354a565b9050612073565b505050505050505050565b8151158015906120f0575060048251105b1561210e57604051632342609160e11b815260040160405180910390fd5b600061211a8686612534565b905061212784838361259f565b60006121328461355d565b9050600061214285518484612641565b90506000816003811115612158576121586130ab565b0361217657604051635872303760e01b815260040160405180910390fd5b600381600381111561218a5761218a6130ab565b0361219757505050612318565b6000806121a3856127e9565b60028111156121b4576121b46130ab565b036121d5576121ce896121c78a86612483565b8589612804565b9050612239565b60016121e0856127e9565b60028111156121f1576121f16130ab565b0361220b576121ce896122048a86612483565b858961289b565b6002612216856127e9565b6002811115612227576122276130ab565b036122395761223689896129ca565b90505b600281600281111561224d5761224d6130ab565b148061228657506000816002811115612268576122686130ab565b14801561228657506001826003811115612284576122846130ab565b145b156122a45760405163864dd1e760e01b815260040160405180910390fd5b60018160028111156122b8576122b86130ab565b14806122f1575060008160028111156122d3576122d36130ab565b1480156122f1575060028260038111156122ef576122ef6130ab565b145b156122ff5750505050612318565b6040516308d5a8b160e31b815260040160405180910390fd5b5050505050565b6001600160a01b031660009081526001919091016020526040902054151590565b600080806001846002811115612358576123586130ab565b0361237057506aff0000000000000000ffff196123ab565b6000846002811115612384576123846130ab565b0361239c57506aff00ffffffffffffff0000196123ab565b506aff00ffffffffffffffffff195b808516915060508460028111156123c4576123c46130ab565b901b91909117949350505050565b60006123e7836123e28460601c90565b61231f565b61243657825460018181018555600085815260208120909201849055845491908501906124148560601c90565b6001600160a01b03168152602081019190915260400160002055506001610e3b565b506000610e3b565b6040516001600160601b0319606084811b8216602084015283901b16603482015260009060480160405160208183030381529060405280519060200120905092915050565b6040516001600160601b0319606084901b1660208201526001600160e01b0319821660348201526000906038016040516020818303038152906040526124c890613594565b9392505050565b6124d881611f7d565b6040516001600160a01b038216907fbc7cd75a20ee27fd9adebab32041f755214dbc6bffa90cc0225b39da2e5c2d3b90600090a250565b60606124c88383604051806060016040528060278152602001613624602791396129fe565b6001600160a01b038116600090815260018301602052604081205480820361256f57604051632d0519ad60e01b815260040160405180910390fd5b8361257b6001836134d4565b8154811061258b5761258b6134e7565b906000526020600020015491505092915050565b60016125aa82612a6c565b60018111156125bb576125bb6130ab565b146125d957604051633bcd102b60e21b815260040160405180910390fd5b60018260018111156125ed576125ed6130ab565b0361260b576040516306c4a1c760e11b815260040160405180910390fd5b6000831180156126235750612621816002612a87565b155b15610dbd576040516309e9cd4960e01b815260040160405180910390fd5b60008061264d84612abd565b905084158061266457506001600160e01b03198316155b806126805750600381600381111561267e5761267e6130ab565b145b8061269c5750600081600381111561269a5761269a6130ab565b145b156126a85790506124c8565b6000637993b94760e11b6001600160e01b03198516016126d4576126cd856000612ad8565b90506127bf565b63ab5d120b60e01b6001600160e01b03198516016126f7576126cd856002612ad8565b634259a0bb60e01b6001600160e01b031985160161271a576126cd856003612ad8565b639aeaeb4160e01b6001600160e01b031985160161273d576126cd856004612ad8565b63f5413a7160e01b6001600160e01b0319851601612760576126cd856005612ad8565b63f6a1584d60e01b6001600160e01b0319851601612783576126cd856007612ad8565b633213221d60e11b6001600160e01b03198516016127a6576126cd856008612ad8565b6040516318f4c12360e11b815260040160405180910390fd5b60008160048111156127d3576127d36130ab565b036127e0575090506124c8565b6109bb81612b2c565b600060ff605083901c166002811115610e3b57610e3b6130ab565b60006001600160e01b0319831663095ea7b360e01b1480159061283857506001600160e01b03198316634decdde360e11b14155b15612856576040516318f4c12360e11b815260040160405180910390fd5b6000612863600084612b86565b90506000612871338361243e565b60008781526003890160209081526040808320938352929052205460ff1692505050949350505050565b6000806128a9600084612b86565b90506001600160a01b03811633146128d457604051636eb0315f60e01b815260040160405180910390fd5b6000637993b94760e11b6001600160e01b0319861601612900576128f9600185612b86565b90506129a1565b63ab5d120b60e01b6001600160e01b0319861601612939576000612925600186612b86565b9050612931818461243e565b9150506129a1565b6001600160e01b0319851663bda65f4560e01b148061296857506001600160e01b0319851663651514bf60e01b145b8061298357506001600160e01b03198516630abec58f60e01b145b156127a6576000612995600186612b86565b9050612931838261243e565b60008681526003880160209081526040808320938352929052205460ff16915050949350505050565b6000806129d7338461243e565b60008080526003860160209081526040808320938352929052205460ff1691505092915050565b6060600080856001600160a01b031685604051612a1b91906135bb565b600060405180830381855af49150503d8060008114612a56576040519150601f19603f3d011682016040523d82523d6000602084013e612a5b565b606091505b50915091506109bb86838387612bf1565b600060ff605883901c166001811115610e3b57610e3b6130ab565b6000816002811115612a9b57612a9b6130ab565b612aa4846127e9565b6002811115612ab557612ab56130ab565b149392505050565b600060ff604883901c166003811115610e3b57610e3b6130ab565b600060098210612afb5760405163b44af9af60e01b815260040160405180910390fd5b6000612b088360086134bd565b612b139060b861354a565b905083811b60f81c60048111156113c1576113c16130ab565b600080826004811115612b4157612b416130ab565b90508060ff16600003612b675760405163d8455a1360e01b815260040160405180910390fd5b612b726001826135d7565b60ff1660038111156124c8576124c86130ab565b6000612b938360206134bd565b612b9e90600461354a565b612ba990602061354a565b82511015612bca57604051631d098e2d60e21b815260040160405180910390fd5b6000612bd78460206134bd565b612be290600461354a565b92909201602001519392505050565b60608315612c60578251600003612c59576001600160a01b0385163b612c595760405162461bcd60e51b815260206004820152601d60248201527f416464726573733a2063616c6c20746f206e6f6e2d636f6e74726163740000006044820152606401610642565b50816113c1565b6113c18383815115612c755781518083602001fd5b8060405162461bcd60e51b815260040161064291906135f0565b6001600160a01b03811681146105f757600080fd5b600060208284031215612cb657600080fd5b81356124c881612c8f565b634e487b7160e01b600052604160045260246000fd5b604051601f8201601f1916810167ffffffffffffffff81118282101715612d0057612d00612cc1565b604052919050565b600067ffffffffffffffff821115612d2257612d22612cc1565b50601f01601f191660200190565b600082601f830112612d4157600080fd5b8135612d54612d4f82612d08565b612cd7565b818152846020838601011115612d6957600080fd5b816020850160208301376000918101602001919091529392505050565b600060208284031215612d9857600080fd5b813567ffffffffffffffff811115612daf57600080fd5b6113c184828501612d30565b600080600080600060808688031215612dd357600080fd5b8535612dde81612c8f565b945060208601359350604086013567ffffffffffffffff80821115612e0257600080fd5b818801915088601f830112612e1657600080fd5b813581811115612e2557600080fd5b896020828501011115612e3757600080fd5b602083019550809450505050606086013560028110612e5557600080fd5b809150509295509295909350565b60008060408385031215612e7657600080fd5b8235612e8181612c8f565b9150602083013567ffffffffffffffff811115612e9d57600080fd5b612ea985828601612d30565b9150509250929050565b60005b83811015612ece578181015183820152602001612eb6565b50506000910152565b60008151808452612eef816020860160208601612eb3565b601f01601f19169290920160200192915050565b82151581526040602082015260006113c16040830184612ed7565b600067ffffffffffffffff821115612f3857612f38612cc1565b5060051b60200190565b803560038110612f5157600080fd5b919050565b600082601f830112612f6757600080fd5b81356020612f77612d4f83612f1e565b82815260059290921b84018101918181019086841115612f9657600080fd5b8286015b84811015612fb857612fab81612f42565b8352918301918301612f9a565b509695505050505050565b60008060408385031215612fd657600080fd5b823567ffffffffffffffff80821115612fee57600080fd5b818501915085601f83011261300257600080fd5b81356020613012612d4f83612f1e565b82815260059290921b8401810191818101908984111561303157600080fd5b948201945b838610156130665785356001600160e01b0319811681146130575760008081fd5b82529482019490820190613036565b9650508601359250508082111561307c57600080fd5b50612ea985828601612f56565b6000806040838503121561309c57600080fd5b50508035926020909101359150565b634e487b7160e01b600052602160045260246000fd5b600381106130d1576130d16130ab565b9052565b604080825283519082018190526000906020906060840190828701845b828110156131185781516001600160e01b031916845292840192908401906001016130f2565b5050508381038285015284518082528583019183019060005b81811015613154576131448385516130c1565b9284019291840191600101613131565b5090979650505050505050565b6020808252825182820181905260009190848201906040850190845b818110156131995783518352928401929184019160010161317d565b50909695505050505050565b6000602082840312156131b757600080fd5b5035919050565b600080600080608085870312156131d457600080fd5b84356131df81612c8f565b935060208501356131ef81612c8f565b925060408501356131ff81612c8f565b9396929550929360600135925050565b60008060006060848603121561322457600080fd5b833561322f81612c8f565b9250602084013561323f81612c8f565b915061324d60408501612f42565b90509250925092565b60208101610e3b82846130c1565b60008060006060848603121561327957600080fd5b833561328481612c8f565b95602085013595506040909401359392505050565b6020808252602c908201527f46756e6374696f6e206d7573742062652063616c6c6564207468726f7567682060408201526b19195b1959d85d1958d85b1b60a21b606082015260800190565b6020808252602c908201527f46756e6374696f6e206d7573742062652063616c6c6564207468726f7567682060408201526b6163746976652070726f787960a01b606082015260800190565b60008060006060848603121561334657600080fd5b835161335181612c8f565b602085015190935061336281612c8f565b80925050604084015190509250925092565b60006020828403121561338657600080fd5b5051919050565b60006020828403121561339f57600080fd5b81516124c881612c8f565b60018060a01b03851681528360208201526080604082015260006133d16080830185612ed7565b9050600283106133e3576133e36130ab565b82606083015295945050505050565b80518015158114612f5157600080fd5b60006020828403121561341457600080fd5b6124c8826133f2565b6000806040838503121561343057600080fd5b613439836133f2565b9150602083015167ffffffffffffffff81111561345557600080fd5b8301601f8101851361346657600080fd5b8051613474612d4f82612d08565b81815286602083850101111561348957600080fd5b61349a826020830160208601612eb3565b8093505050509250929050565b634e487b7160e01b600052601160045260246000fd5b8082028115828204841417610e3b57610e3b6134a7565b81810381811115610e3b57610e3b6134a7565b634e487b7160e01b600052603260045260246000fd5b60006001820161350f5761350f6134a7565b5060010190565b6001600160e01b031983168152604081016124c860208301846130c1565b634e487b7160e01b600052603160045260246000fd5b80820180821115610e3b57610e3b6134a7565b805160208201516001600160e01b0319808216929190600483101561358c5780818460040360031b1b83161693505b505050919050565b805160208083015191908110156135b5576000198160200360031b1b821691505b50919050565b600082516135cd818460208701612eb3565b9190910192915050565b60ff8281168282160390811115610e3b57610e3b6134a7565b6020815260006124c86020830184612ed756fe360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc416464726573733a206c6f772d6c6576656c2064656c65676174652063616c6c206661696c6564a2646970667358221220ec1b42309ca1be4e1dea19d21acd6d364096f019a09d088fde775ab08446545564736f6c63430008130033
    /// ```
    #[rustfmt::skip]
    #[allow(clippy::all)]
    pub static DEPLOYED_BYTECODE: alloy_sol_types::private::Bytes = alloy_sol_types::private::Bytes::from_static(
        b"`\x80`@R`\x046\x10a\x01\xC2W`\x005`\xE0\x1C\x80cs\x9CK\x08\x11a\0\xF7W\x80c\xB5sib\x11a\0\x95W\x80c\xDCDjJ\x11a\0dW\x80c\xDCDjJ\x14a\x05@W\x80c\xDFNo\x8A\x14a\x05mW\x80c\xF2\xFD\xE3\x8B\x14a\x05\xA4W\x80c\xFA\x19P\x1D\x14a\x05\xC4W`\0\x80\xFD[\x80c\xB5sib\x14a\x04\xC0W\x80c\xC6\x86\x05\xC8\x14a\x04\xE0W\x80c\xC6\x8C:\x83\x14a\x05\0W\x80c\xDC\x06\x10\x9D\x14a\x05 W`\0\x80\xFD[\x80c\x9D\x95\xF1\xCC\x11a\0\xD1W\x80c\x9D\x95\xF1\xCC\x14a\x04@W\x80c\xA2E\x0F\x89\x14a\x04`W\x80c\xA7l\x9A/\x14a\x04\x80W\x80c\xB2\xB9\x9E\xC9\x14a\x04\xA0W`\0\x80\xFD[\x80cs\x9CK\x08\x14a\x03\xE2W\x80c\x8B\x95\xEC\xCD\x14a\x04\x02W\x80c\x8D\xA5\xCB[\x14a\x04\"W`\0\x80\xFD[\x80cO\x1E\xF2\x86\x11a\x01dW\x80cV\xF5Q\x17\x11a\x01>W\x80cV\xF5Q\x17\x14a\x03HW\x80c`\x97lK\x14a\x03}W\x80cc\xFE;V\x14a\x03\xABW\x80cqP\x18\xA6\x14a\x03\xCDW`\0\x80\xFD[\x80cO\x1E\xF2\x86\x14a\x02\xE4W\x80cR)\x07?\x14a\x02\xF7W\x80cR\xD1\x90-\x14a\x03%W`\0\x80\xFD[\x80c6Y\xCF\xE6\x11a\x01\xA0W\x80c6Y\xCF\xE6\x14a\x02oW\x80cC\x9F\xAB\x91\x14a\x02\x8FW\x80cF\x87!\xA7\x14a\x02\xAFW\x80cJ\x1B\xA4\x08\x14a\x02\xCFW`\0\x80\xFD[\x80c\x01u\x01R\x14a\x01\xC7W\x80c)D\x02\xCC\x14a\x02\x15W\x80c4\x01\xCD\xE8\x14a\x02MW[`\0\x80\xFD[4\x80\x15a\x01\xD3W`\0\x80\xFD[Pa\x02\0a\x01\xE26`\x04a,\xA4V[`\x01`\x01`\xA0\x1B\x03\x16`\0\x90\x81R`\xCC` R`@\x90 T`\xFF\x16\x90V[`@Q\x90\x15\x15\x81R` \x01[`@Q\x80\x91\x03\x90\xF3[4\x80\x15a\x02!W`\0\x80\xFD[P`\xC9Ta\x025\x90`\x01`\x01`\xA0\x1B\x03\x16\x81V[`@Q`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x81R` \x01a\x02\x0CV[4\x80\x15a\x02YW`\0\x80\xFD[Pa\x02ma\x02h6`\x04a,\xA4V[a\x05\xE4V[\0[4\x80\x15a\x02{W`\0\x80\xFD[Pa\x02ma\x02\x8A6`\x04a,\xA4V[a\x05\xFAV[4\x80\x15a\x02\x9BW`\0\x80\xFD[Pa\x02ma\x02\xAA6`\x04a-\x86V[a\x06\xDFV[4\x80\x15a\x02\xBBW`\0\x80\xFD[Pa\x02\0a\x02\xCA6`\x04a-\xBBV[a\t+V[4\x80\x15a\x02\xDBW`\0\x80\xFD[Pa\x02\0`\x01\x81V[a\x02ma\x02\xF26`\x04a.cV[a\t\xC5V[4\x80\x15a\x03\x03W`\0\x80\xFD[Pa\x03\x17a\x03\x126`\x04a-\xBBV[a\n\x91V[`@Qa\x02\x0C\x92\x91\x90a/\x03V[4\x80\x15a\x031W`\0\x80\xFD[Pa\x03:a\x0B2V[`@Q\x90\x81R` \x01a\x02\x0CV[4\x80\x15a\x03TW`\0\x80\xFD[Pa\x03ha\x03c6`\x04a/\xC3V[a\x0B\xE5V[`@\x80Q\x92\x83R` \x83\x01\x91\x90\x91R\x01a\x02\x0CV[4\x80\x15a\x03\x89W`\0\x80\xFD[Pa\x03\x9Da\x03\x986`\x04a0\x89V[a\x0B\xFEV[`@Qa\x02\x0C\x92\x91\x90a0\xD5V[4\x80\x15a\x03\xB7W`\0\x80\xFD[Pa\x03\xC0a\x0C\x0BV[`@Qa\x02\x0C\x91\x90a1aV[4\x80\x15a\x03\xD9W`\0\x80\xFD[Pa\x02ma\x0C\x1CV[4\x80\x15a\x03\xEEW`\0\x80\xFD[Pa\x02ma\x03\xFD6`\x04a1\xA5V[a\x0C0V[4\x80\x15a\x04\x0EW`\0\x80\xFD[Pa\x02ma\x04\x1D6`\x04a,\xA4V[a\x0CCV[4\x80\x15a\x04.W`\0\x80\xFD[P`\x97T`\x01`\x01`\xA0\x1B\x03\x16a\x025V[4\x80\x15a\x04LW`\0\x80\xFD[Pa\x02ma\x04[6`\x04a,\xA4V[a\x0C\x95V[4\x80\x15a\x04lW`\0\x80\xFD[Pa\x02ma\x04{6`\x04a1\xA5V[a\x0C\xA6V[4\x80\x15a\x04\x8CW`\0\x80\xFD[Pa\x02ma\x04\x9B6`\x04a1\xA5V[a\x0C\xB7V[4\x80\x15a\x04\xACW`\0\x80\xFD[Pa\x02ma\x04\xBB6`\x04a,\xA4V[a\x0C\xCAV[4\x80\x15a\x04\xCCW`\0\x80\xFD[Pa\x02ma\x04\xDB6`\x04a1\xA5V[a\rTV[4\x80\x15a\x04\xECW`\0\x80\xFD[Pa\x02ma\x04\xFB6`\x04a1\xBEV[a\r\x8CV[4\x80\x15a\x05\x0CW`\0\x80\xFD[Pa\x02ma\x05\x1B6`\x04a2\x0FV[a\r\xA8V[4\x80\x15a\x05,W`\0\x80\xFD[Pa\x02ma\x05;6`\x04a1\xA5V[a\r\xC2V[4\x80\x15a\x05LW`\0\x80\xFD[Pa\x05`a\x05[6`\x04a0\x89V[a\x0E\x1DV[`@Qa\x02\x0C\x91\x90a2VV[4\x80\x15a\x05yW`\0\x80\xFD[Pa\x05\x8Da\x05\x886`\x04a,\xA4V[a\x0EAV[`@\x80Q\x92\x15\x15\x83R` \x83\x01\x91\x90\x91R\x01a\x02\x0CV[4\x80\x15a\x05\xB0W`\0\x80\xFD[Pa\x02ma\x05\xBF6`\x04a,\xA4V[a\x0EXV[4\x80\x15a\x05\xD0W`\0\x80\xFD[Pa\x02ma\x05\xDF6`\x04a2dV[a\x0E\xCEV[a\x05\xECa\x0E\xE3V[a\x05\xF7`\xCA\x82a\x0F=V[PV[`\x01`\x01`\xA0\x1B\x03\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x160\x03a\x06KW`@QbF\x1B\xCD`\xE5\x1B\x81R`\x04\x01a\x06B\x90a2\x99V[`@Q\x80\x91\x03\x90\xFD[\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\x01`\x01`\xA0\x1B\x03\x16a\x06\x94`\0\x80Q` a6\x04\x839\x81Q\x91RT`\x01`\x01`\xA0\x1B\x03\x16\x90V[`\x01`\x01`\xA0\x1B\x03\x16\x14a\x06\xBAW`@QbF\x1B\xCD`\xE5\x1B\x81R`\x04\x01a\x06B\x90a2\xE5V[a\x06\xC3\x81a\x0F\xA3V[`@\x80Q`\0\x80\x82R` \x82\x01\x90\x92Ra\x05\xF7\x91\x83\x91\x90a\x0F\xABV[`\0Ta\x01\0\x90\x04`\xFF\x16\x15\x80\x80\x15a\x06\xFFWP`\0T`\x01`\xFF\x90\x91\x16\x10[\x80a\x07\x19WP0;\x15\x80\x15a\x07\x19WP`\0T`\xFF\x16`\x01\x14[a\x07|W`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`.`$\x82\x01R\x7FInitializable: contract is alrea`D\x82\x01Rm\x19\x1EH\x1A[\x9A]\x1AX[\x1A^\x99Y`\x92\x1B`d\x82\x01R`\x84\x01a\x06BV[`\0\x80T`\xFF\x19\x16`\x01\x17\x90U\x80\x15a\x07\x9FW`\0\x80Ta\xFF\0\x19\x16a\x01\0\x17\x90U[`\0\x80`\0\x84\x80` \x01\x90Q\x81\x01\x90a\x07\xB8\x91\x90a31V[\x91\x94P\x92P\x90P`\x01`\x01`\xA0\x1B\x03\x83\x16\x15\x80a\x07\xDCWP`\x01`\x01`\xA0\x1B\x03\x82\x16\x15[\x15a\x07\xFAW`@Qc\x86y\x15\xAB`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x81`\x01`\x01`\xA0\x1B\x03\x16\x83`\x01`\x01`\xA0\x1B\x03\x16\x03a\x08,W`@QcY\x8A\x0E!`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0a\x08@`\x97T`\x01`\x01`\xA0\x1B\x03\x16\x90V[`\x01`\x01`\xA0\x1B\x03\x16\x14\x15\x80a\x08`WP`\xC9T`\x01`\x01`\xA0\x1B\x03\x16\x15\x15[\x15a\x08}W`@Qb\xDC\x14\x9F`\xE4\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\xC9\x80T`\x01`\x01`\xA0\x1B\x03\x19\x16`\x01`\x01`\xA0\x1B\x03\x84\x16\x17\x90Ua\x08\xA1\x81a\x11\x16V[a\x08\xAA\x83a\x11\xDCV[`@Q`\x01`\x01`\xA0\x1B\x03\x83\x16\x90\x7F_\xE6\xAA\xBFNy\x08C\xDFC\xAE\x0E\"\xB5\x86 \x06o\xB3\x89)[\xED\xC0j\x92\xDFl;(w}\x90`\0\x90\xA2PPP\x80\x15a\t'W`\0\x80Ta\xFF\0\x19\x16\x90U`@Q`\x01\x81R\x7F\x7F&\xB8?\xF9n\x1F+jh/\x138R\xF6y\x8A\t\xC4e\xDA\x95\x92\x14`\xCE\xFB8G@$\x98\x90` \x01`@Q\x80\x91\x03\x90\xA1[PPV[3`\0\x90\x81R`\xCC` R`@\x81 T`\xFF\x16a\t[W`@Qc\x1F\xB1\xD3\xE5`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\xC9Ta\tx\x90`\xCA\x90`\x01`\x01`\xA0\x1B\x03\x16\x88\x88\x88\x88\x88a\x12.V[a\t\xBB\x86\x86\x86\x86\x80\x80`\x1F\x01` \x80\x91\x04\x02` \x01`@Q\x90\x81\x01`@R\x80\x93\x92\x91\x90\x81\x81R` \x01\x83\x83\x80\x82\x847`\0\x92\x01\x91\x90\x91RP\x88\x92Pa\x12\xD9\x91PPV[\x96\x95PPPPPPV[`\x01`\x01`\xA0\x1B\x03\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x160\x03a\n\rW`@QbF\x1B\xCD`\xE5\x1B\x81R`\x04\x01a\x06B\x90a2\x99V[\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\x01`\x01`\xA0\x1B\x03\x16a\nV`\0\x80Q` a6\x04\x839\x81Q\x91RT`\x01`\x01`\xA0\x1B\x03\x16\x90V[`\x01`\x01`\xA0\x1B\x03\x16\x14a\n|W`@QbF\x1B\xCD`\xE5\x1B\x81R`\x04\x01a\x06B\x90a2\xE5V[a\n\x85\x82a\x0F\xA3V[a\t'\x82\x82`\x01a\x0F\xABV[3`\0\x90\x81R`\xCC` R`@\x81 T``\x90`\xFF\x16a\n\xC4W`@Qc\x1F\xB1\xD3\xE5`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\xC9Ta\n\xE1\x90`\xCA\x90`\x01`\x01`\xA0\x1B\x03\x16\x89\x89\x89\x89\x89a\x12.V[a\x0B$\x87\x87\x87\x87\x80\x80`\x1F\x01` \x80\x91\x04\x02` \x01`@Q\x90\x81\x01`@R\x80\x93\x92\x91\x90\x81\x81R` \x01\x83\x83\x80\x82\x847`\0\x92\x01\x91\x90\x91RP\x89\x92Pa\x13\xC9\x91PPV[\x91P\x91P\x95P\x95\x93PPPPV[`\x000`\x01`\x01`\xA0\x1B\x03\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x16\x14a\x0B\xD2W`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`8`$\x82\x01R\x7FUUPSUpgradeable: must not be cal`D\x82\x01R\x7Fled through delegatecall\0\0\0\0\0\0\0\0`d\x82\x01R`\x84\x01a\x06BV[P`\0\x80Q` a6\x04\x839\x81Q\x91R\x90V[`\0\x80a\x0B\xF2\x84\x84a\x14\xC3V[\x91P\x91P[\x92P\x92\x90PV[``\x80a\x0B\xF2\x84\x84a\x15\xD7V[``a\x0C\x17`\xCAa\x17\x86V[\x90P\x90V[a\x0C$a\x0E\xE3V[a\x0C.`\0a\x11\xDCV[V[a\x0C8a\x0E\xE3V[a\x05\xF7`\xCA\x82a\x17\xE2V[a\x0CKa\x0E\xE3V[`\xC9\x80T`\x01`\x01`\xA0\x1B\x03\x19\x16`\x01`\x01`\xA0\x1B\x03\x83\x16\x90\x81\x17\x90\x91U`@Q\x7F_\xE6\xAA\xBFNy\x08C\xDFC\xAE\x0E\"\xB5\x86 \x06o\xB3\x89)[\xED\xC0j\x92\xDFl;(w}\x90`\0\x90\xA2PV[a\x0C\x9Da\x0E\xE3V[a\x05\xF7\x81a\x18\xA2V[a\x0C\xAEa\x0E\xE3V[a\x05\xF7\x81a\x11\x16V[a\x0C\xBFa\x0E\xE3V[a\x05\xF7`\xCA\x82a\x19(V[a\x0C\xD2a\x0E\xE3V[`\x01`\x01`\xA0\x1B\x03\x81\x16`\0\x90\x81R`\xCC` R`@\x90 T`\xFF\x16a\r\x0BW`@Qc\x1F\xB1\xD3\xE5`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01`\x01`\xA0\x1B\x03\x81\x16`\0\x81\x81R`\xCC` R`@\x80\x82 \x80T`\xFF\x19\x16\x90UQ\x7F\xCF\xC2Af\xDBK\xB6w\xE8W\xCA\xCA\xBD\x15A\xFB+0dP!\xB2|Q0A\x95\x89\xB8M\xB5+\x91\x90\xA2PV[a\r\\a\x0E\xE3V[`\0a\rh\x82``\x1C\x90V[\x90Pa\rs\x81a\x18\xA2V[a\r~`\xCA\x83a\x19\xD9V[a\t'`\xCA\x82\x83`\x01a\x1A\x8BV[a\r\x94a\x0E\xE3V[a\r\xA2`\xCA\x85\x85\x85\x85a\x1B'V[PPPPV[a\r\xB0a\x0E\xE3V[a\r\xBD`\xCA\x84\x84\x84a\x1A\x8BV[PPPV[a\r\xCAa\x0E\xE3V[`\0a\r\xD6\x82``\x1C\x90V[`\x01`\x01`\xA0\x1B\x03\x81\x16`\0\x90\x81R`\xCC` R`@\x90 T\x90\x91P`\xFF\x16a\x0E\x12W`@Qc\x1F\xB1\xD3\xE5`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\t'`\xCA\x83a\x19\xD9V[`\0\x82\x81R`\xCD` \x90\x81R`@\x80\x83 \x84\x84R\x90\x91R\x90 T`\xFF\x16[\x92\x91PPV[`\0\x80a\x0EO`\xCA\x84a\x1C\xABV[\x91P\x91P\x91P\x91V[a\x0E`a\x0E\xE3V[`\x01`\x01`\xA0\x1B\x03\x81\x16a\x0E\xC5W`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`&`$\x82\x01R\x7FOwnable: new owner is the zero a`D\x82\x01Reddress`\xD0\x1B`d\x82\x01R`\x84\x01a\x06BV[a\x05\xF7\x81a\x11\xDCV[a\x0E\xD6a\x0E\xE3V[a\r\xBD`\xCA\x84\x84\x84a\x1D\x0EV[`\x97T`\x01`\x01`\xA0\x1B\x03\x163\x14a\x0C.W`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01\x81\x90R`$\x82\x01R\x7FOwnable: caller is not the owner`D\x82\x01R`d\x01a\x06BV[`\0a\x0FI\x83\x83a\x1EUV[\x90P\x80\x15a\x0F\x8AW`@Q`\x01`\x01`\xA0\x1B\x03\x83\x16\x90\x7F\r\xFC\xE1\xEAK\xA1\xEE\xBA\x89\x1F\xFB*\x06g\x90\xFB\xC2\x93\xA9\xE5\x17\xFEa\xD4\x9D\x15j0\x16_\x93\xF3\x90`\0\x90\xA2PPPV[`@QcJ\x89\x03!`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x05\xF7a\x0E\xE3V[\x7FI\x10\xFD\xFA\x16\xFE\xD3&\x0E\xD0\xE7\x14\x7F|\xC6\xDA\x11\xA6\x02\x08\xB5\xB9@m\x12\xA65aO\xFD\x91CT`\xFF\x16\x15a\x0F\xDEWa\r\xBD\x83a\x1F}V[\x82`\x01`\x01`\xA0\x1B\x03\x16cR\xD1\x90-`@Q\x81c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x92PPP\x80\x15a\x108WP`@\x80Q`\x1F=\x90\x81\x01`\x1F\x19\x16\x82\x01\x90\x92Ra\x105\x91\x81\x01\x90a3tV[`\x01[a\x10\x9BW`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`.`$\x82\x01R\x7FERC1967Upgrade: new implementati`D\x82\x01Rmon is not UUPS`\x90\x1B`d\x82\x01R`\x84\x01a\x06BV[`\0\x80Q` a6\x04\x839\x81Q\x91R\x81\x14a\x11\nW`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`)`$\x82\x01R\x7FERC1967Upgrade: unsupported prox`D\x82\x01Rh\x1AXX\x9B\x19UURQ`\xBA\x1B`d\x82\x01R`\x84\x01a\x06BV[Pa\r\xBD\x83\x83\x83a \x19V[`\0a\x11\"\x82``\x1C\x90V[\x90P`\0\x81`\x01`\x01`\xA0\x1B\x03\x16c\xFC\x0CTj`@Q\x81c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x11dW=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x11\x88\x91\x90a3\x8DV[\x90Pa\x11\xB3`\xCAk\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x85\x16``\x85\x90\x1B`\x01`\x01``\x1B\x03\x19\x16\x17a\x17\xE2V[a\r\xBD`\xCAk\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x85\x16``\x84\x90\x1B`\x01`\x01``\x1B\x03\x19\x16\x17a\x19(V[`\x97\x80T`\x01`\x01`\xA0\x1B\x03\x83\x81\x16`\x01`\x01`\xA0\x1B\x03\x19\x83\x16\x81\x17\x90\x93U`@Q\x91\x16\x91\x90\x82\x90\x7F\x8B\xE0\x07\x9CS\x16Y\x14\x13D\xCD\x1F\xD0\xA4\xF2\x84\x19I\x7F\x97\"\xA3\xDA\xAF\xE3\xB4\x18okdW\xE0\x90`\0\x90\xA3PPV[\x84`\x01`\x01`\xA0\x1B\x03\x16\x86`\x01`\x01`\xA0\x1B\x03\x16\x03a\x12\x8CWa\x12\x87\x87\x84\x84\x80\x80`\x1F\x01` \x80\x91\x04\x02` \x01`@Q\x90\x81\x01`@R\x80\x93\x92\x91\x90\x81\x81R` \x01\x83\x83\x80\x82\x847`\0\x92\x01\x91\x90\x91RPa >\x92PPPV[a\x12\xD0V[a\x12\xD0\x87\x86\x86\x86\x86\x80\x80`\x1F\x01` \x80\x91\x04\x02` \x01`@Q\x90\x81\x01`@R\x80\x93\x92\x91\x90\x81\x81R` \x01\x83\x83\x80\x82\x847`\0\x92\x01\x91\x90\x91RP\x88\x92Pa \xDF\x91PPV[PPPPPPPV[`\0a\x12\xED`\x97T`\x01`\x01`\xA0\x1B\x03\x16\x90V[`\x01`\x01`\xA0\x1B\x03\x16cF\x87!\xA7\x86\x86\x86\x86`@Q\x85c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01a\x13\x1E\x94\x93\x92\x91\x90a3\xAAV[` `@Q\x80\x83\x03\x81`\0\x87Z\xF1\x15\x80\x15a\x13=W=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x13a\x91\x90a4\x02V[\x90P\x80\x15a\x13\x97W`@Q\x7FN.\x86\xD2\x13u\xEB\xCB\xF6\xE9=\xF5\xEB\xDDZ\x91[\xF80$Y\x04\xC3\xB5OH\xAD\xF0\x17\n\xAEK\x90`\0\x90\xA1a\x13\xC1V[`@Q\x7F\xC2M\x93`\x8A\x03\xD2c\xFF\x19\x1Dvw\x14\x1F^\x94\xC4\x96\xE5\x93\x10\x8F:\xAE\x0C\xB5\xB7\x04\x94\xC4\xD3\x90`\0\x90\xA1[\x94\x93PPPPV[`\0``a\x13\xDF`\x97T`\x01`\x01`\xA0\x1B\x03\x16\x90V[`\x01`\x01`\xA0\x1B\x03\x16cR)\x07?\x87\x87\x87\x87`@Q\x85c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01a\x14\x10\x94\x93\x92\x91\x90a3\xAAV[`\0`@Q\x80\x83\x03\x81`\0\x87Z\xF1\x15\x80\x15a\x14/W=`\0\x80>=`\0\xFD[PPPP`@Q=`\0\x82>`\x1F=\x90\x81\x01`\x1F\x19\x16\x82\x01`@Ra\x14W\x91\x90\x81\x01\x90a4\x1DV[\x90\x92P\x90P\x81\x15a\x14\x90W`@Q\x7FN.\x86\xD2\x13u\xEB\xCB\xF6\xE9=\xF5\xEB\xDDZ\x91[\xF80$Y\x04\xC3\xB5OH\xAD\xF0\x17\n\xAEK\x90`\0\x90\xA1a\x14\xBAV[`@Q\x7F\xC2M\x93`\x8A\x03\xD2c\xFF\x19\x1Dvw\x14\x1F^\x94\xC4\x96\xE5\x93\x10\x8F:\xAE\x0C\xB5\xB7\x04\x94\xC4\xD3\x90`\0\x90\xA1[\x94P\x94\x92PPPV[\x81Q`\0\x90\x81\x90`\x07\x81\x11\x15a\x14\xECW`@Qc\x17\xA4\xD9\x87`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x83Q\x85Q\x14a\x15\x0EW`@Qct\xF4\xD57`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0\x80[\x82\x81\x10\x15a\x15qWa\x15%\x81` a4\xBDV[a\x150\x90`\xE0a4\xD4V[`\xE0\x88\x83\x81Q\x81\x10a\x15DWa\x15Da4\xE7V[` \x02` \x01\x01Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x90\x1C\x90\x1B\x82\x17\x91P\x80\x80a\x15i\x90a4\xFDV[\x91PPa\x15\x12V[P`\0[\x82\x81\x10\x15a\x15\xCCWa\x15\x88\x81`\x02a4\xBDV[\x86\x82\x81Q\x81\x10a\x15\x9AWa\x15\x9Aa4\xE7V[` \x02` \x01\x01Q`\x02\x81\x11\x15a\x15\xB3Wa\x15\xB3a0\xABV[\x90\x1B\x91\x90\x91\x17\x90\x80a\x15\xC4\x81a4\xFDV[\x91PPa\x15uV[P\x95\x90\x94P\x92PPPV[``\x80`\x07\x83\x11\x15a\x15\xFCW`@Qc\x17\xA4\xD9\x87`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x82g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x16\x15Wa\x16\x15a,\xC1V[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x16>W\x81` \x01` \x82\x02\x806\x837\x01\x90P[P\x91P\x82g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x16ZWa\x16Za,\xC1V[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x16\x83W\x81` \x01` \x82\x02\x806\x837\x01\x90P[P\x90P`\0[\x83\x81\x10\x15a\x16\xEBW`\xE0a\x16\x9E\x82` a4\xBDV[a\x16\xA9\x90`\xE0a4\xD4V[\x86\x90\x1C\x90\x1B\x83\x82\x81Q\x81\x10a\x16\xC0Wa\x16\xC0a4\xE7V[`\x01`\x01`\xE0\x1B\x03\x19\x90\x92\x16` \x92\x83\x02\x91\x90\x91\x01\x90\x91\x01R\x80a\x16\xE3\x81a4\xFDV[\x91PPa\x16\x89V[P`\0[\x83\x81\x10\x15a\x17~W`\xFEa\x17\x04\x82`\x02a4\xBDV[a\x17\x0F\x90`\xFEa4\xD4V[\x86`\0\x1C\x90\x1B\x90\x1C`\xFF\x16`\x02\x81\x11\x15a\x17+Wa\x17+a0\xABV[\x82\x82\x81Q\x81\x10a\x17=Wa\x17=a4\xE7V[` \x02` \x01\x01\x90`\x02\x81\x11\x15a\x17VWa\x17Va0\xABV[\x90\x81`\x02\x81\x11\x15a\x17iWa\x17ia0\xABV[\x90RP\x80a\x17v\x81a4\xFDV[\x91PPa\x16\xEFV[P\x92P\x92\x90PV[``\x81`\0\x01\x80T\x80` \x02` \x01`@Q\x90\x81\x01`@R\x80\x92\x91\x90\x81\x81R` \x01\x82\x80T\x80\x15a\x17\xD6W` \x02\x82\x01\x91\x90`\0R` `\0 \x90[\x81T\x81R` \x01\x90`\x01\x01\x90\x80\x83\x11a\x17\xC2W[PPPPP\x90P\x91\x90PV[`\0a\x17\xEE\x82``\x1C\x90V[\x90P`\x01`\x01`\xA0\x1B\x03\x81\x16a\x18\x17W`@Qc\x86y\x15\xAB`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x18!\x83\x82a#\x1FV[\x15a\x18?W`@Qct`>\x95`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0a\x18L\x83`\x01a#@V[\x90Pa\x18X\x84\x82a#\xD2V[P\x81`\x01`\x01`\xA0\x1B\x03\x16\x7F_\xFB\x06\xB0\xB0\xE8\xADj\x8F<X1\xD4\x99\xDF\xA6\x12\xD9\xC9\xD4\xDC\x10{\xBDf\xF1\x8Fa\xA6I.q\x82`@Qa\x18\x94\x91\x81R` \x01\x90V[`@Q\x80\x91\x03\x90\xA2PPPPV[`\x01`\x01`\xA0\x1B\x03\x81\x16`\0\x90\x81R`\xCC` R`@\x90 T`\xFF\x16\x15a\x18\xDCW`@Qc8\xE8\x16\xA5`\xE2\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01`\x01`\xA0\x1B\x03\x81\x16`\0\x81\x81R`\xCC` R`@\x80\x82 \x80T`\xFF\x19\x16`\x01\x17\x90UQ\x7F\xB2]\x03\xAA\xF3\x08\xD7)\x17\t\xBE\x1E\xA2\x8B\x80\x04c\xCF:\x9ALJUU\xD73:\x96L\x1D\xFE\xBD\x91\x90\xA2PV[`\0a\x194\x82``\x1C\x90V[\x90P`\x01`\x01`\xA0\x1B\x03\x81\x16a\x19]W`@Qc\x86y\x15\xAB`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x19g\x83\x82a#\x1FV[\x15a\x19\x85W`@Qct`>\x95`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0a\x19\x91\x83\x82a#@V[\x90Pa\x19\x9D\x84\x82a#\xD2V[P\x81`\x01`\x01`\xA0\x1B\x03\x16\x7F\xAA\xF2k\xB1*\xA8\x9E\xE9k\xBE\x19fzj\x05W'\xB7]?n\xD7\xB8\xB6\x11\xEFe\x19\x18\x02\t\xD6\x82`@Qa\x18\x94\x91\x81R` \x01\x90V[`\0a\x19\xE5\x82``\x1C\x90V[\x90P`\x01`\x01`\xA0\x1B\x03\x81\x16a\x1A\x0EW`@Qc\x86y\x15\xAB`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x1A\x18\x83\x82a#\x1FV[\x15a\x1A6W`@Qct`>\x95`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0a\x1AC\x83`\x02a#@V[\x90Pa\x1AO\x84\x82a#\xD2V[P\x81`\x01`\x01`\xA0\x1B\x03\x16\x7F\x1E\xE2y\x1F,\xAF\x0E\x92\xA9\xDC2\xA3z\x9E\xA5:\xB6\xACzo\xB8\xF2\xD0\x90\xE5:\x06}:C\xF6\xAC\x82`@Qa\x18\x94\x91\x81R` \x01\x90V[`\0\x80\x80R`\x03\x85\x01` R`@\x81 \x82\x91a\x1A\xA7\x86\x86a$>V[\x81R` \x81\x01\x91\x90\x91R`@\x01`\0 \x80T`\xFF\x19\x16`\x01\x83`\x02\x81\x11\x15a\x1A\xD1Wa\x1A\xD1a0\xABV[\x02\x17\x90UP\x81`\x01`\x01`\xA0\x1B\x03\x16\x83`\x01`\x01`\xA0\x1B\x03\x16\x7Ft\x87S\r\xDF\xF1 y\x95\x05\xE5+\x1B\x19\xB6\x93?\x85\xA9\xEE\xAE\x92 \xC8\nz\xD7\xC4)\xB6\x12\xAE\x83`@Qa\x1B\x19\x91\x90a2VV[`@Q\x80\x91\x03\x90\xA3PPPPV[`\0\x80a\x1B5\x83`\x02a\x15\xD7V[\x91P\x91P`\0[`\x02\x81\x10\x15a\x1C\xA1W\x82Q`\0\x90\x84\x90\x83\x90\x81\x10a\x1B\\Wa\x1B\\a4\xE7V[` \x02` \x01\x01Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x14a\x1C\x8FW`\0a\x1B\x98\x87\x85\x84\x81Q\x81\x10a\x1B\x8BWa\x1B\x8Ba4\xE7V[` \x02` \x01\x01Qa$\x83V[\x90P\x82\x82\x81Q\x81\x10a\x1B\xACWa\x1B\xACa4\xE7V[` \x02` \x01\x01Q\x89`\x03\x01`\0\x83\x81R` \x01\x90\x81R` \x01`\0 `\0a\x1B\xD5\x8B\x8Ba$>V[\x81R` \x81\x01\x91\x90\x91R`@\x01`\0 \x80T`\xFF\x19\x16`\x01\x83`\x02\x81\x11\x15a\x1B\xFFWa\x1B\xFFa0\xABV[\x02\x17\x90UP\x85`\x01`\x01`\xA0\x1B\x03\x16\x87`\x01`\x01`\xA0\x1B\x03\x16\x89`\x01`\x01`\xA0\x1B\x03\x16\x7F\xA3\xDFq\x04 \xB0\x1C\xC3\x0F\xF3\x000\x9A\xBB\xC7\xFA\xDDF0\xD4\xAB8[\x0FZ\x12o\xB4\xBA\xBEv+\x87\x86\x81Q\x81\x10a\x1CUWa\x1CUa4\xE7V[` \x02` \x01\x01Q\x87\x87\x81Q\x81\x10a\x1CoWa\x1Coa4\xE7V[` \x02` \x01\x01Q`@Qa\x1C\x85\x92\x91\x90a5\x16V[`@Q\x80\x91\x03\x90\xA4P[\x80a\x1C\x99\x81a4\xFDV[\x91PPa\x1B<V[PPPPPPPPV[`\x01`\x01`\xA0\x1B\x03\x81\x16`\0\x90\x81R`\x01\x83\x01` R`@\x81 T\x81\x90\x80\x82\x03a\x1C\xDCW`\0\x80\x92P\x92PPa\x0B\xF7V[`\x01\x85a\x1C\xE9\x82\x84a4\xD4V[\x81T\x81\x10a\x1C\xF9Wa\x1C\xF9a4\xE7V[\x90`\0R` `\0 \x01T\x92P\x92PPa\x0B\xF7V[`\0\x80a\x1D\x1C\x83`\x07a\x15\xD7V[\x91P\x91P`\0[`\x07\x81\x10\x15a\x12\xD0W\x82Q`\0\x90\x84\x90\x83\x90\x81\x10a\x1DCWa\x1DCa4\xE7V[` \x02` \x01\x01Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x14a\x1ECW`\0a\x1Dr\x87\x85\x84\x81Q\x81\x10a\x1B\x8BWa\x1B\x8Ba4\xE7V[\x90P\x82\x82\x81Q\x81\x10a\x1D\x86Wa\x1D\x86a4\xE7V[` \x90\x81\x02\x91\x90\x91\x01\x81\x01Q`\0\x83\x81R`\x03\x8B\x01\x83R`@\x80\x82 \x8A\x83R\x90\x93R\x91\x90\x91 \x80T`\xFF\x19\x16`\x01\x83`\x02\x81\x11\x15a\x1D\xC6Wa\x1D\xC6a0\xABV[\x02\x17\x90UP\x85\x87`\x01`\x01`\xA0\x1B\x03\x16\x7F\xF2\xFF\xD4\xF0\x9DX\xD0h$\x18\x803\xD31\x8D\x06\xEB\x95{\xFB\x1A\x8F\xFE\xD9\xAFx\xE1\xF1\x91h\xB9\x04\x86\x85\x81Q\x81\x10a\x1E\tWa\x1E\ta4\xE7V[` \x02` \x01\x01Q\x86\x86\x81Q\x81\x10a\x1E#Wa\x1E#a4\xE7V[` \x02` \x01\x01Q`@Qa\x1E9\x92\x91\x90a5\x16V[`@Q\x80\x91\x03\x90\xA3P[\x80a\x1EM\x81a4\xFDV[\x91PPa\x1D#V[`\x01`\x01`\xA0\x1B\x03\x81\x16`\0\x90\x81R`\x01\x83\x01` R`@\x81 T\x80\x15a\x1FsW`\0a\x1E\x83`\x01\x83a4\xD4V[\x85T\x90\x91P`\0\x90a\x1E\x97\x90`\x01\x90a4\xD4V[\x90P\x81\x81\x14a\x1F\x15W`\0\x86`\0\x01\x82\x81T\x81\x10a\x1E\xB7Wa\x1E\xB7a4\xE7V[\x90`\0R` `\0 \x01T\x90P\x80\x87`\0\x01\x84\x81T\x81\x10a\x1E\xDAWa\x1E\xDAa4\xE7V[\x90`\0R` `\0 \x01\x81\x90UP\x83\x87`\x01\x01`\0a\x1E\xF9\x84``\x1C\x90V[`\x01`\x01`\xA0\x1B\x03\x16\x81R` \x81\x01\x91\x90\x91R`@\x01`\0 UP[\x85T\x86\x90\x80a\x1F&Wa\x1F&a54V[`\x01\x90\x03\x81\x81\x90`\0R` `\0 \x01`\0\x90U\x90U\x85`\x01\x01`\0\x86`\x01`\x01`\xA0\x1B\x03\x16`\x01`\x01`\xA0\x1B\x03\x16\x81R` \x01\x90\x81R` \x01`\0 `\0\x90U`\x01\x93PPPPa\x0E;V[`\0\x91PPa\x0E;V[`\x01`\x01`\xA0\x1B\x03\x81\x16;a\x1F\xEAW`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`-`$\x82\x01R\x7FERC1967: new implementation is n`D\x82\x01Rl\x1B\xDD\x08\x18H\x18\xDB\xDB\x9D\x1C\x98X\xDD`\x9A\x1B`d\x82\x01R`\x84\x01a\x06BV[`\0\x80Q` a6\x04\x839\x81Q\x91R\x80T`\x01`\x01`\xA0\x1B\x03\x19\x16`\x01`\x01`\xA0\x1B\x03\x92\x90\x92\x16\x91\x90\x91\x17\x90UV[a \"\x83a$\xCFV[`\0\x82Q\x11\x80a /WP\x80[\x15a\r\xBDWa\r\xA2\x83\x83a%\x0FV[`\0\x80`\0```\0\x80`$\x87\x01Q\x90P\x80` \x14a pW`@Qc~\xD1\x117`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`d[\x87Q\x81\x10\x15a \xD4W\x80\x88\x01Q`\xF8\x1C\x96P`\x01\x81\x01\x88\x01Q``\x1C\x95P`\x15\x81\x01\x88\x01Q\x94P`5\x81\x01\x88\x01Q\x92P`5\x81\x01\x88\x01\x93Pa \xB8\x89\x87\x87\x87\x8Ba \xDFV[a \xC3\x83`Ua5JV[a \xCD\x90\x82a5JV[\x90Pa sV[PPPPPPPPPV[\x81Q\x15\x80\x15\x90a \xF0WP`\x04\x82Q\x10[\x15a!\x0EW`@Qc#B`\x91`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0a!\x1A\x86\x86a%4V[\x90Pa!'\x84\x83\x83a%\x9FV[`\0a!2\x84a5]V[\x90P`\0a!B\x85Q\x84\x84a&AV[\x90P`\0\x81`\x03\x81\x11\x15a!XWa!Xa0\xABV[\x03a!vW`@QcXr07`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x03\x81`\x03\x81\x11\x15a!\x8AWa!\x8Aa0\xABV[\x03a!\x97WPPPa#\x18V[`\0\x80a!\xA3\x85a'\xE9V[`\x02\x81\x11\x15a!\xB4Wa!\xB4a0\xABV[\x03a!\xD5Wa!\xCE\x89a!\xC7\x8A\x86a$\x83V[\x85\x89a(\x04V[\x90Pa\"9V[`\x01a!\xE0\x85a'\xE9V[`\x02\x81\x11\x15a!\xF1Wa!\xF1a0\xABV[\x03a\"\x0BWa!\xCE\x89a\"\x04\x8A\x86a$\x83V[\x85\x89a(\x9BV[`\x02a\"\x16\x85a'\xE9V[`\x02\x81\x11\x15a\"'Wa\"'a0\xABV[\x03a\"9Wa\"6\x89\x89a)\xCAV[\x90P[`\x02\x81`\x02\x81\x11\x15a\"MWa\"Ma0\xABV[\x14\x80a\"\x86WP`\0\x81`\x02\x81\x11\x15a\"hWa\"ha0\xABV[\x14\x80\x15a\"\x86WP`\x01\x82`\x03\x81\x11\x15a\"\x84Wa\"\x84a0\xABV[\x14[\x15a\"\xA4W`@Qc\x86M\xD1\xE7`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01\x81`\x02\x81\x11\x15a\"\xB8Wa\"\xB8a0\xABV[\x14\x80a\"\xF1WP`\0\x81`\x02\x81\x11\x15a\"\xD3Wa\"\xD3a0\xABV[\x14\x80\x15a\"\xF1WP`\x02\x82`\x03\x81\x11\x15a\"\xEFWa\"\xEFa0\xABV[\x14[\x15a\"\xFFWPPPPa#\x18V[`@Qc\x08\xD5\xA8\xB1`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[PPPPPV[`\x01`\x01`\xA0\x1B\x03\x16`\0\x90\x81R`\x01\x91\x90\x91\x01` R`@\x90 T\x15\x15\x90V[`\0\x80\x80`\x01\x84`\x02\x81\x11\x15a#XWa#Xa0\xABV[\x03a#pWPj\xFF\0\0\0\0\0\0\0\0\xFF\xFF\x19a#\xABV[`\0\x84`\x02\x81\x11\x15a#\x84Wa#\x84a0\xABV[\x03a#\x9CWPj\xFF\0\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\0\x19a#\xABV[Pj\xFF\0\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x19[\x80\x85\x16\x91P`P\x84`\x02\x81\x11\x15a#\xC4Wa#\xC4a0\xABV[\x90\x1B\x91\x90\x91\x17\x94\x93PPPPV[`\0a#\xE7\x83a#\xE2\x84``\x1C\x90V[a#\x1FV[a$6W\x82T`\x01\x81\x81\x01\x85U`\0\x85\x81R` \x81 \x90\x92\x01\x84\x90U\x84T\x91\x90\x85\x01\x90a$\x14\x85``\x1C\x90V[`\x01`\x01`\xA0\x1B\x03\x16\x81R` \x81\x01\x91\x90\x91R`@\x01`\0 UP`\x01a\x0E;V[P`\0a\x0E;V[`@Q`\x01`\x01``\x1B\x03\x19``\x84\x81\x1B\x82\x16` \x84\x01R\x83\x90\x1B\x16`4\x82\x01R`\0\x90`H\x01`@Q` \x81\x83\x03\x03\x81R\x90`@R\x80Q\x90` \x01 \x90P\x92\x91PPV[`@Q`\x01`\x01``\x1B\x03\x19``\x84\x90\x1B\x16` \x82\x01R`\x01`\x01`\xE0\x1B\x03\x19\x82\x16`4\x82\x01R`\0\x90`8\x01`@Q` \x81\x83\x03\x03\x81R\x90`@Ra$\xC8\x90a5\x94V[\x93\x92PPPV[a$\xD8\x81a\x1F}V[`@Q`\x01`\x01`\xA0\x1B\x03\x82\x16\x90\x7F\xBC|\xD7Z \xEE'\xFD\x9A\xDE\xBA\xB3 A\xF7U!M\xBCk\xFF\xA9\x0C\xC0\"[9\xDA.\\-;\x90`\0\x90\xA2PV[``a$\xC8\x83\x83`@Q\x80``\x01`@R\x80`'\x81R` \x01a6$`'\x919a)\xFEV[`\x01`\x01`\xA0\x1B\x03\x81\x16`\0\x90\x81R`\x01\x83\x01` R`@\x81 T\x80\x82\x03a%oW`@Qc-\x05\x19\xAD`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x83a%{`\x01\x83a4\xD4V[\x81T\x81\x10a%\x8BWa%\x8Ba4\xE7V[\x90`\0R` `\0 \x01T\x91PP\x92\x91PPV[`\x01a%\xAA\x82a*lV[`\x01\x81\x11\x15a%\xBBWa%\xBBa0\xABV[\x14a%\xD9W`@Qc;\xCD\x10+`\xE2\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01\x82`\x01\x81\x11\x15a%\xEDWa%\xEDa0\xABV[\x03a&\x0BW`@Qc\x06\xC4\xA1\xC7`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0\x83\x11\x80\x15a&#WPa&!\x81`\x02a*\x87V[\x15[\x15a\r\xBDW`@Qc\t\xE9\xCDI`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0\x80a&M\x84a*\xBDV[\x90P\x84\x15\x80a&dWP`\x01`\x01`\xE0\x1B\x03\x19\x83\x16\x15[\x80a&\x80WP`\x03\x81`\x03\x81\x11\x15a&~Wa&~a0\xABV[\x14[\x80a&\x9CWP`\0\x81`\x03\x81\x11\x15a&\x9AWa&\x9Aa0\xABV[\x14[\x15a&\xA8W\x90Pa$\xC8V[`\0cy\x93\xB9G`\xE1\x1B`\x01`\x01`\xE0\x1B\x03\x19\x85\x16\x01a&\xD4Wa&\xCD\x85`\0a*\xD8V[\x90Pa'\xBFV[c\xAB]\x12\x0B`\xE0\x1B`\x01`\x01`\xE0\x1B\x03\x19\x85\x16\x01a&\xF7Wa&\xCD\x85`\x02a*\xD8V[cBY\xA0\xBB`\xE0\x1B`\x01`\x01`\xE0\x1B\x03\x19\x85\x16\x01a'\x1AWa&\xCD\x85`\x03a*\xD8V[c\x9A\xEA\xEBA`\xE0\x1B`\x01`\x01`\xE0\x1B\x03\x19\x85\x16\x01a'=Wa&\xCD\x85`\x04a*\xD8V[c\xF5A:q`\xE0\x1B`\x01`\x01`\xE0\x1B\x03\x19\x85\x16\x01a'`Wa&\xCD\x85`\x05a*\xD8V[c\xF6\xA1XM`\xE0\x1B`\x01`\x01`\xE0\x1B\x03\x19\x85\x16\x01a'\x83Wa&\xCD\x85`\x07a*\xD8V[c2\x13\"\x1D`\xE1\x1B`\x01`\x01`\xE0\x1B\x03\x19\x85\x16\x01a'\xA6Wa&\xCD\x85`\x08a*\xD8V[`@Qc\x18\xF4\xC1#`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0\x81`\x04\x81\x11\x15a'\xD3Wa'\xD3a0\xABV[\x03a'\xE0WP\x90Pa$\xC8V[a\t\xBB\x81a+,V[`\0`\xFF`P\x83\x90\x1C\x16`\x02\x81\x11\x15a\x0E;Wa\x0E;a0\xABV[`\0`\x01`\x01`\xE0\x1B\x03\x19\x83\x16c\t^\xA7\xB3`\xE0\x1B\x14\x80\x15\x90a(8WP`\x01`\x01`\xE0\x1B\x03\x19\x83\x16cM\xEC\xDD\xE3`\xE1\x1B\x14\x15[\x15a(VW`@Qc\x18\xF4\xC1#`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0a(c`\0\x84a+\x86V[\x90P`\0a(q3\x83a$>V[`\0\x87\x81R`\x03\x89\x01` \x90\x81R`@\x80\x83 \x93\x83R\x92\x90R T`\xFF\x16\x92PPP\x94\x93PPPPV[`\0\x80a(\xA9`\0\x84a+\x86V[\x90P`\x01`\x01`\xA0\x1B\x03\x81\x163\x14a(\xD4W`@Qcn\xB01_`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0cy\x93\xB9G`\xE1\x1B`\x01`\x01`\xE0\x1B\x03\x19\x86\x16\x01a)\0Wa(\xF9`\x01\x85a+\x86V[\x90Pa)\xA1V[c\xAB]\x12\x0B`\xE0\x1B`\x01`\x01`\xE0\x1B\x03\x19\x86\x16\x01a)9W`\0a)%`\x01\x86a+\x86V[\x90Pa)1\x81\x84a$>V[\x91PPa)\xA1V[`\x01`\x01`\xE0\x1B\x03\x19\x85\x16c\xBD\xA6_E`\xE0\x1B\x14\x80a)hWP`\x01`\x01`\xE0\x1B\x03\x19\x85\x16ce\x15\x14\xBF`\xE0\x1B\x14[\x80a)\x83WP`\x01`\x01`\xE0\x1B\x03\x19\x85\x16c\n\xBE\xC5\x8F`\xE0\x1B\x14[\x15a'\xA6W`\0a)\x95`\x01\x86a+\x86V[\x90Pa)1\x83\x82a$>V[`\0\x86\x81R`\x03\x88\x01` \x90\x81R`@\x80\x83 \x93\x83R\x92\x90R T`\xFF\x16\x91PP\x94\x93PPPPV[`\0\x80a)\xD73\x84a$>V[`\0\x80\x80R`\x03\x86\x01` \x90\x81R`@\x80\x83 \x93\x83R\x92\x90R T`\xFF\x16\x91PP\x92\x91PPV[```\0\x80\x85`\x01`\x01`\xA0\x1B\x03\x16\x85`@Qa*\x1B\x91\x90a5\xBBV[`\0`@Q\x80\x83\x03\x81\x85Z\xF4\x91PP=\x80`\0\x81\x14a*VW`@Q\x91P`\x1F\x19`?=\x01\x16\x82\x01`@R=\x82R=`\0` \x84\x01>a*[V[``\x91P[P\x91P\x91Pa\t\xBB\x86\x83\x83\x87a+\xF1V[`\0`\xFF`X\x83\x90\x1C\x16`\x01\x81\x11\x15a\x0E;Wa\x0E;a0\xABV[`\0\x81`\x02\x81\x11\x15a*\x9BWa*\x9Ba0\xABV[a*\xA4\x84a'\xE9V[`\x02\x81\x11\x15a*\xB5Wa*\xB5a0\xABV[\x14\x93\x92PPPV[`\0`\xFF`H\x83\x90\x1C\x16`\x03\x81\x11\x15a\x0E;Wa\x0E;a0\xABV[`\0`\t\x82\x10a*\xFBW`@Qc\xB4J\xF9\xAF`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0a+\x08\x83`\x08a4\xBDV[a+\x13\x90`\xB8a5JV[\x90P\x83\x81\x1B`\xF8\x1C`\x04\x81\x11\x15a\x13\xC1Wa\x13\xC1a0\xABV[`\0\x80\x82`\x04\x81\x11\x15a+AWa+Aa0\xABV[\x90P\x80`\xFF\x16`\0\x03a+gW`@Qc\xD8EZ\x13`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a+r`\x01\x82a5\xD7V[`\xFF\x16`\x03\x81\x11\x15a$\xC8Wa$\xC8a0\xABV[`\0a+\x93\x83` a4\xBDV[a+\x9E\x90`\x04a5JV[a+\xA9\x90` a5JV[\x82Q\x10\x15a+\xCAW`@Qc\x1D\t\x8E-`\xE2\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0a+\xD7\x84` a4\xBDV[a+\xE2\x90`\x04a5JV[\x92\x90\x92\x01` \x01Q\x93\x92PPPV[``\x83\x15a,`W\x82Q`\0\x03a,YW`\x01`\x01`\xA0\x1B\x03\x85\x16;a,YW`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`\x1D`$\x82\x01R\x7FAddress: call to non-contract\0\0\0`D\x82\x01R`d\x01a\x06BV[P\x81a\x13\xC1V[a\x13\xC1\x83\x83\x81Q\x15a,uW\x81Q\x80\x83` \x01\xFD[\x80`@QbF\x1B\xCD`\xE5\x1B\x81R`\x04\x01a\x06B\x91\x90a5\xF0V[`\x01`\x01`\xA0\x1B\x03\x81\x16\x81\x14a\x05\xF7W`\0\x80\xFD[`\0` \x82\x84\x03\x12\x15a,\xB6W`\0\x80\xFD[\x815a$\xC8\x81a,\x8FV[cNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[`@Q`\x1F\x82\x01`\x1F\x19\x16\x81\x01g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x82\x82\x10\x17\x15a-\0Wa-\0a,\xC1V[`@R\x91\x90PV[`\0g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x15a-\"Wa-\"a,\xC1V[P`\x1F\x01`\x1F\x19\x16` \x01\x90V[`\0\x82`\x1F\x83\x01\x12a-AW`\0\x80\xFD[\x815a-Ta-O\x82a-\x08V[a,\xD7V[\x81\x81R\x84` \x83\x86\x01\x01\x11\x15a-iW`\0\x80\xFD[\x81` \x85\x01` \x83\x017`\0\x91\x81\x01` \x01\x91\x90\x91R\x93\x92PPPV[`\0` \x82\x84\x03\x12\x15a-\x98W`\0\x80\xFD[\x815g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a-\xAFW`\0\x80\xFD[a\x13\xC1\x84\x82\x85\x01a-0V[`\0\x80`\0\x80`\0`\x80\x86\x88\x03\x12\x15a-\xD3W`\0\x80\xFD[\x855a-\xDE\x81a,\x8FV[\x94P` \x86\x015\x93P`@\x86\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x80\x82\x11\x15a.\x02W`\0\x80\xFD[\x81\x88\x01\x91P\x88`\x1F\x83\x01\x12a.\x16W`\0\x80\xFD[\x815\x81\x81\x11\x15a.%W`\0\x80\xFD[\x89` \x82\x85\x01\x01\x11\x15a.7W`\0\x80\xFD[` \x83\x01\x95P\x80\x94PPPP``\x86\x015`\x02\x81\x10a.UW`\0\x80\xFD[\x80\x91PP\x92\x95P\x92\x95\x90\x93PV[`\0\x80`@\x83\x85\x03\x12\x15a.vW`\0\x80\xFD[\x825a.\x81\x81a,\x8FV[\x91P` \x83\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a.\x9DW`\0\x80\xFD[a.\xA9\x85\x82\x86\x01a-0V[\x91PP\x92P\x92\x90PV[`\0[\x83\x81\x10\x15a.\xCEW\x81\x81\x01Q\x83\x82\x01R` \x01a.\xB6V[PP`\0\x91\x01RV[`\0\x81Q\x80\x84Ra.\xEF\x81` \x86\x01` \x86\x01a.\xB3V[`\x1F\x01`\x1F\x19\x16\x92\x90\x92\x01` \x01\x92\x91PPV[\x82\x15\x15\x81R`@` \x82\x01R`\0a\x13\xC1`@\x83\x01\x84a.\xD7V[`\0g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x15a/8Wa/8a,\xC1V[P`\x05\x1B` \x01\x90V[\x805`\x03\x81\x10a/QW`\0\x80\xFD[\x91\x90PV[`\0\x82`\x1F\x83\x01\x12a/gW`\0\x80\xFD[\x815` a/wa-O\x83a/\x1EV[\x82\x81R`\x05\x92\x90\x92\x1B\x84\x01\x81\x01\x91\x81\x81\x01\x90\x86\x84\x11\x15a/\x96W`\0\x80\xFD[\x82\x86\x01[\x84\x81\x10\x15a/\xB8Wa/\xAB\x81a/BV[\x83R\x91\x83\x01\x91\x83\x01a/\x9AV[P\x96\x95PPPPPPV[`\0\x80`@\x83\x85\x03\x12\x15a/\xD6W`\0\x80\xFD[\x825g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x80\x82\x11\x15a/\xEEW`\0\x80\xFD[\x81\x85\x01\x91P\x85`\x1F\x83\x01\x12a0\x02W`\0\x80\xFD[\x815` a0\x12a-O\x83a/\x1EV[\x82\x81R`\x05\x92\x90\x92\x1B\x84\x01\x81\x01\x91\x81\x81\x01\x90\x89\x84\x11\x15a01W`\0\x80\xFD[\x94\x82\x01\x94[\x83\x86\x10\x15a0fW\x855`\x01`\x01`\xE0\x1B\x03\x19\x81\x16\x81\x14a0WW`\0\x80\x81\xFD[\x82R\x94\x82\x01\x94\x90\x82\x01\x90a06V[\x96PP\x86\x015\x92PP\x80\x82\x11\x15a0|W`\0\x80\xFD[Pa.\xA9\x85\x82\x86\x01a/VV[`\0\x80`@\x83\x85\x03\x12\x15a0\x9CW`\0\x80\xFD[PP\x805\x92` \x90\x91\x015\x91PV[cNH{q`\xE0\x1B`\0R`!`\x04R`$`\0\xFD[`\x03\x81\x10a0\xD1Wa0\xD1a0\xABV[\x90RV[`@\x80\x82R\x83Q\x90\x82\x01\x81\x90R`\0\x90` \x90``\x84\x01\x90\x82\x87\x01\x84[\x82\x81\x10\x15a1\x18W\x81Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x84R\x92\x84\x01\x92\x90\x84\x01\x90`\x01\x01a0\xF2V[PPP\x83\x81\x03\x82\x85\x01R\x84Q\x80\x82R\x85\x83\x01\x91\x83\x01\x90`\0[\x81\x81\x10\x15a1TWa1D\x83\x85Qa0\xC1V[\x92\x84\x01\x92\x91\x84\x01\x91`\x01\x01a11V[P\x90\x97\x96PPPPPPPV[` \x80\x82R\x82Q\x82\x82\x01\x81\x90R`\0\x91\x90\x84\x82\x01\x90`@\x85\x01\x90\x84[\x81\x81\x10\x15a1\x99W\x83Q\x83R\x92\x84\x01\x92\x91\x84\x01\x91`\x01\x01a1}V[P\x90\x96\x95PPPPPPV[`\0` \x82\x84\x03\x12\x15a1\xB7W`\0\x80\xFD[P5\x91\x90PV[`\0\x80`\0\x80`\x80\x85\x87\x03\x12\x15a1\xD4W`\0\x80\xFD[\x845a1\xDF\x81a,\x8FV[\x93P` \x85\x015a1\xEF\x81a,\x8FV[\x92P`@\x85\x015a1\xFF\x81a,\x8FV[\x93\x96\x92\x95P\x92\x93``\x015\x92PPV[`\0\x80`\0``\x84\x86\x03\x12\x15a2$W`\0\x80\xFD[\x835a2/\x81a,\x8FV[\x92P` \x84\x015a2?\x81a,\x8FV[\x91Pa2M`@\x85\x01a/BV[\x90P\x92P\x92P\x92V[` \x81\x01a\x0E;\x82\x84a0\xC1V[`\0\x80`\0``\x84\x86\x03\x12\x15a2yW`\0\x80\xFD[\x835a2\x84\x81a,\x8FV[\x95` \x85\x015\x95P`@\x90\x94\x015\x93\x92PPPV[` \x80\x82R`,\x90\x82\x01R\x7FFunction must be called through `@\x82\x01Rk\x19\x19[\x19Y\xD8]\x19X\xD8[\x1B`\xA2\x1B``\x82\x01R`\x80\x01\x90V[` \x80\x82R`,\x90\x82\x01R\x7FFunction must be called through `@\x82\x01Rkactive proxy`\xA0\x1B``\x82\x01R`\x80\x01\x90V[`\0\x80`\0``\x84\x86\x03\x12\x15a3FW`\0\x80\xFD[\x83Qa3Q\x81a,\x8FV[` \x85\x01Q\x90\x93Pa3b\x81a,\x8FV[\x80\x92PP`@\x84\x01Q\x90P\x92P\x92P\x92V[`\0` \x82\x84\x03\x12\x15a3\x86W`\0\x80\xFD[PQ\x91\x90PV[`\0` \x82\x84\x03\x12\x15a3\x9FW`\0\x80\xFD[\x81Qa$\xC8\x81a,\x8FV[`\x01\x80`\xA0\x1B\x03\x85\x16\x81R\x83` \x82\x01R`\x80`@\x82\x01R`\0a3\xD1`\x80\x83\x01\x85a.\xD7V[\x90P`\x02\x83\x10a3\xE3Wa3\xE3a0\xABV[\x82``\x83\x01R\x95\x94PPPPPV[\x80Q\x80\x15\x15\x81\x14a/QW`\0\x80\xFD[`\0` \x82\x84\x03\x12\x15a4\x14W`\0\x80\xFD[a$\xC8\x82a3\xF2V[`\0\x80`@\x83\x85\x03\x12\x15a40W`\0\x80\xFD[a49\x83a3\xF2V[\x91P` \x83\x01Qg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a4UW`\0\x80\xFD[\x83\x01`\x1F\x81\x01\x85\x13a4fW`\0\x80\xFD[\x80Qa4ta-O\x82a-\x08V[\x81\x81R\x86` \x83\x85\x01\x01\x11\x15a4\x89W`\0\x80\xFD[a4\x9A\x82` \x83\x01` \x86\x01a.\xB3V[\x80\x93PPPP\x92P\x92\x90PV[cNH{q`\xE0\x1B`\0R`\x11`\x04R`$`\0\xFD[\x80\x82\x02\x81\x15\x82\x82\x04\x84\x14\x17a\x0E;Wa\x0E;a4\xA7V[\x81\x81\x03\x81\x81\x11\x15a\x0E;Wa\x0E;a4\xA7V[cNH{q`\xE0\x1B`\0R`2`\x04R`$`\0\xFD[`\0`\x01\x82\x01a5\x0FWa5\x0Fa4\xA7V[P`\x01\x01\x90V[`\x01`\x01`\xE0\x1B\x03\x19\x83\x16\x81R`@\x81\x01a$\xC8` \x83\x01\x84a0\xC1V[cNH{q`\xE0\x1B`\0R`1`\x04R`$`\0\xFD[\x80\x82\x01\x80\x82\x11\x15a\x0E;Wa\x0E;a4\xA7V[\x80Q` \x82\x01Q`\x01`\x01`\xE0\x1B\x03\x19\x80\x82\x16\x92\x91\x90`\x04\x83\x10\x15a5\x8CW\x80\x81\x84`\x04\x03`\x03\x1B\x1B\x83\x16\x16\x93P[PPP\x91\x90PV[\x80Q` \x80\x83\x01Q\x91\x90\x81\x10\x15a5\xB5W`\0\x19\x81` \x03`\x03\x1B\x1B\x82\x16\x91P[P\x91\x90PV[`\0\x82Qa5\xCD\x81\x84` \x87\x01a.\xB3V[\x91\x90\x91\x01\x92\x91PPV[`\xFF\x82\x81\x16\x82\x82\x16\x03\x90\x81\x11\x15a\x0E;Wa\x0E;a4\xA7V[` \x81R`\0a$\xC8` \x83\x01\x84a.\xD7V\xFE6\x08\x94\xA1;\xA1\xA3!\x06g\xC8(I-\xB9\x8D\xCA> v\xCC75\xA9 \xA3\xCAP]8+\xBCAddress: low-level delegate call failed\xA2dipfsX\"\x12 \xEC\x1BB0\x9C\xA1\xBEN\x1D\xEA\x19\xD2\x1A\xCDm6@\x96\xF0\x19\xA0\x9D\x08\x8F\xDEwZ\xB0\x84FTUdsolcC\0\x08\x13\x003",
    );
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct GranularPermission(u8);
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[automatically_derived]
        impl alloy_sol_types::private::SolTypeValue<GranularPermission> for u8 {
            #[inline]
            fn stv_to_tokens(
                &self,
            ) -> <alloy::sol_types::sol_data::Uint<
                8,
            > as alloy_sol_types::SolType>::Token<'_> {
                alloy_sol_types::private::SolTypeValue::<
                    alloy::sol_types::sol_data::Uint<8>,
                >::stv_to_tokens(self)
            }
            #[inline]
            fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
                <alloy::sol_types::sol_data::Uint<
                    8,
                > as alloy_sol_types::SolType>::tokenize(self)
                    .0
            }
            #[inline]
            fn stv_abi_encode_packed_to(
                &self,
                out: &mut alloy_sol_types::private::Vec<u8>,
            ) {
                <alloy::sol_types::sol_data::Uint<
                    8,
                > as alloy_sol_types::SolType>::abi_encode_packed_to(self, out)
            }
            #[inline]
            fn stv_abi_packed_encoded_size(&self) -> usize {
                <alloy::sol_types::sol_data::Uint<
                    8,
                > as alloy_sol_types::SolType>::abi_encoded_size(self)
            }
        }
        #[automatically_derived]
        impl GranularPermission {
            /// The Solidity type name.
            pub const NAME: &'static str = stringify!(@ name);
            /// Convert from the underlying value type.
            #[inline]
            pub const fn from_underlying(value: u8) -> Self {
                Self(value)
            }
            /// Return the underlying value.
            #[inline]
            pub const fn into_underlying(self) -> u8 {
                self.0
            }
            /// Return the single encoding of this value, delegating to the
            /// underlying type.
            #[inline]
            pub fn abi_encode(&self) -> alloy_sol_types::private::Vec<u8> {
                <Self as alloy_sol_types::SolType>::abi_encode(&self.0)
            }
            /// Return the packed encoding of this value, delegating to the
            /// underlying type.
            #[inline]
            pub fn abi_encode_packed(&self) -> alloy_sol_types::private::Vec<u8> {
                <Self as alloy_sol_types::SolType>::abi_encode_packed(&self.0)
            }
        }
        #[automatically_derived]
        impl From<u8> for GranularPermission {
            fn from(value: u8) -> Self {
                Self::from_underlying(value)
            }
        }
        #[automatically_derived]
        impl From<GranularPermission> for u8 {
            fn from(value: GranularPermission) -> Self {
                value.into_underlying()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolType for GranularPermission {
            type RustType = u8;
            type Token<'a> = <alloy::sol_types::sol_data::Uint<
                8,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SOL_NAME: &'static str = Self::NAME;
            const ENCODED_SIZE: Option<usize> = <alloy::sol_types::sol_data::Uint<
                8,
            > as alloy_sol_types::SolType>::ENCODED_SIZE;
            const PACKED_ENCODED_SIZE: Option<usize> = <alloy::sol_types::sol_data::Uint<
                8,
            > as alloy_sol_types::SolType>::PACKED_ENCODED_SIZE;
            #[inline]
            fn valid_token(token: &Self::Token<'_>) -> bool {
                Self::type_check(token).is_ok()
            }
            #[inline]
            fn type_check(token: &Self::Token<'_>) -> alloy_sol_types::Result<()> {
                <alloy::sol_types::sol_data::Uint<
                    8,
                > as alloy_sol_types::SolType>::type_check(token)
            }
            #[inline]
            fn detokenize(token: Self::Token<'_>) -> Self::RustType {
                <alloy::sol_types::sol_data::Uint<
                    8,
                > as alloy_sol_types::SolType>::detokenize(token)
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::EventTopic for GranularPermission {
            #[inline]
            fn topic_preimage_length(rust: &Self::RustType) -> usize {
                <alloy::sol_types::sol_data::Uint<
                    8,
                > as alloy_sol_types::EventTopic>::topic_preimage_length(rust)
            }
            #[inline]
            fn encode_topic_preimage(
                rust: &Self::RustType,
                out: &mut alloy_sol_types::private::Vec<u8>,
            ) {
                <alloy::sol_types::sol_data::Uint<
                    8,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(rust, out)
            }
            #[inline]
            fn encode_topic(
                rust: &Self::RustType,
            ) -> alloy_sol_types::abi::token::WordToken {
                <alloy::sol_types::sol_data::Uint<
                    8,
                > as alloy_sol_types::EventTopic>::encode_topic(rust)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct Target(alloy::sol_types::private::primitives::aliases::U256);
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[automatically_derived]
        impl alloy_sol_types::private::SolTypeValue<Target>
        for alloy::sol_types::private::primitives::aliases::U256 {
            #[inline]
            fn stv_to_tokens(
                &self,
            ) -> <alloy::sol_types::sol_data::Uint<
                256,
            > as alloy_sol_types::SolType>::Token<'_> {
                alloy_sol_types::private::SolTypeValue::<
                    alloy::sol_types::sol_data::Uint<256>,
                >::stv_to_tokens(self)
            }
            #[inline]
            fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
                <alloy::sol_types::sol_data::Uint<
                    256,
                > as alloy_sol_types::SolType>::tokenize(self)
                    .0
            }
            #[inline]
            fn stv_abi_encode_packed_to(
                &self,
                out: &mut alloy_sol_types::private::Vec<u8>,
            ) {
                <alloy::sol_types::sol_data::Uint<
                    256,
                > as alloy_sol_types::SolType>::abi_encode_packed_to(self, out)
            }
            #[inline]
            fn stv_abi_packed_encoded_size(&self) -> usize {
                <alloy::sol_types::sol_data::Uint<
                    256,
                > as alloy_sol_types::SolType>::abi_encoded_size(self)
            }
        }
        #[automatically_derived]
        impl Target {
            /// The Solidity type name.
            pub const NAME: &'static str = stringify!(@ name);
            /// Convert from the underlying value type.
            #[inline]
            pub const fn from_underlying(
                value: alloy::sol_types::private::primitives::aliases::U256,
            ) -> Self {
                Self(value)
            }
            /// Return the underlying value.
            #[inline]
            pub const fn into_underlying(
                self,
            ) -> alloy::sol_types::private::primitives::aliases::U256 {
                self.0
            }
            /// Return the single encoding of this value, delegating to the
            /// underlying type.
            #[inline]
            pub fn abi_encode(&self) -> alloy_sol_types::private::Vec<u8> {
                <Self as alloy_sol_types::SolType>::abi_encode(&self.0)
            }
            /// Return the packed encoding of this value, delegating to the
            /// underlying type.
            #[inline]
            pub fn abi_encode_packed(&self) -> alloy_sol_types::private::Vec<u8> {
                <Self as alloy_sol_types::SolType>::abi_encode_packed(&self.0)
            }
        }
        #[automatically_derived]
        impl From<alloy::sol_types::private::primitives::aliases::U256> for Target {
            fn from(
                value: alloy::sol_types::private::primitives::aliases::U256,
            ) -> Self {
                Self::from_underlying(value)
            }
        }
        #[automatically_derived]
        impl From<Target> for alloy::sol_types::private::primitives::aliases::U256 {
            fn from(value: Target) -> Self {
                value.into_underlying()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolType for Target {
            type RustType = alloy::sol_types::private::primitives::aliases::U256;
            type Token<'a> = <alloy::sol_types::sol_data::Uint<
                256,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SOL_NAME: &'static str = Self::NAME;
            const ENCODED_SIZE: Option<usize> = <alloy::sol_types::sol_data::Uint<
                256,
            > as alloy_sol_types::SolType>::ENCODED_SIZE;
            const PACKED_ENCODED_SIZE: Option<usize> = <alloy::sol_types::sol_data::Uint<
                256,
            > as alloy_sol_types::SolType>::PACKED_ENCODED_SIZE;
            #[inline]
            fn valid_token(token: &Self::Token<'_>) -> bool {
                Self::type_check(token).is_ok()
            }
            #[inline]
            fn type_check(token: &Self::Token<'_>) -> alloy_sol_types::Result<()> {
                <alloy::sol_types::sol_data::Uint<
                    256,
                > as alloy_sol_types::SolType>::type_check(token)
            }
            #[inline]
            fn detokenize(token: Self::Token<'_>) -> Self::RustType {
                <alloy::sol_types::sol_data::Uint<
                    256,
                > as alloy_sol_types::SolType>::detokenize(token)
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::EventTopic for Target {
            #[inline]
            fn topic_preimage_length(rust: &Self::RustType) -> usize {
                <alloy::sol_types::sol_data::Uint<
                    256,
                > as alloy_sol_types::EventTopic>::topic_preimage_length(rust)
            }
            #[inline]
            fn encode_topic_preimage(
                rust: &Self::RustType,
                out: &mut alloy_sol_types::private::Vec<u8>,
            ) {
                <alloy::sol_types::sol_data::Uint<
                    256,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(rust, out)
            }
            #[inline]
            fn encode_topic(
                rust: &Self::RustType,
            ) -> alloy_sol_types::abi::token::WordToken {
                <alloy::sol_types::sol_data::Uint<
                    256,
                > as alloy_sol_types::EventTopic>::encode_topic(rust)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `AddressIsZero()` and selector `0x867915ab`.
```solidity
error AddressIsZero();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct AddressIsZero;
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(
            _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
        ) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<AddressIsZero> for UnderlyingRustTuple<'_> {
            fn from(value: AddressIsZero) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for AddressIsZero {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for AddressIsZero {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "AddressIsZero()";
            const SELECTOR: [u8; 4] = [134u8, 121u8, 21u8, 171u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn abi_decode_raw_validate(data: &[u8]) -> alloy_sol_types::Result<Self> {
                <Self::Parameters<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Self::new)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `AlreadyInitialized()` and selector `0x0dc149f0`.
```solidity
error AlreadyInitialized();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct AlreadyInitialized;
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(
            _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
        ) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<AlreadyInitialized> for UnderlyingRustTuple<'_> {
            fn from(value: AlreadyInitialized) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for AlreadyInitialized {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for AlreadyInitialized {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "AlreadyInitialized()";
            const SELECTOR: [u8; 4] = [13u8, 193u8, 73u8, 240u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn abi_decode_raw_validate(data: &[u8]) -> alloy_sol_types::Result<Self> {
                <Self::Parameters<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Self::new)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `ArrayTooLong()` and selector `0xbd26cc38`.
```solidity
error ArrayTooLong();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct ArrayTooLong;
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(
            _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
        ) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<ArrayTooLong> for UnderlyingRustTuple<'_> {
            fn from(value: ArrayTooLong) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for ArrayTooLong {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for ArrayTooLong {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "ArrayTooLong()";
            const SELECTOR: [u8; 4] = [189u8, 38u8, 204u8, 56u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn abi_decode_raw_validate(data: &[u8]) -> alloy_sol_types::Result<Self> {
                <Self::Parameters<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Self::new)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `ArraysDifferentLength()` and selector `0x74f4d537`.
```solidity
error ArraysDifferentLength();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct ArraysDifferentLength;
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(
            _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
        ) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<ArraysDifferentLength> for UnderlyingRustTuple<'_> {
            fn from(value: ArraysDifferentLength) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for ArraysDifferentLength {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for ArraysDifferentLength {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "ArraysDifferentLength()";
            const SELECTOR: [u8; 4] = [116u8, 244u8, 213u8, 55u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn abi_decode_raw_validate(data: &[u8]) -> alloy_sol_types::Result<Self> {
                <Self::Parameters<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Self::new)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `CalldataOutOfBounds()` and selector `0x742638b4`.
```solidity
error CalldataOutOfBounds();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct CalldataOutOfBounds;
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(
            _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
        ) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<CalldataOutOfBounds> for UnderlyingRustTuple<'_> {
            fn from(value: CalldataOutOfBounds) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for CalldataOutOfBounds {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for CalldataOutOfBounds {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "CalldataOutOfBounds()";
            const SELECTOR: [u8; 4] = [116u8, 38u8, 56u8, 180u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn abi_decode_raw_validate(data: &[u8]) -> alloy_sol_types::Result<Self> {
                <Self::Parameters<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Self::new)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `CannotChangeOwner()` and selector `0xfd670ebe`.
```solidity
error CannotChangeOwner();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct CannotChangeOwner;
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(
            _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
        ) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<CannotChangeOwner> for UnderlyingRustTuple<'_> {
            fn from(value: CannotChangeOwner) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for CannotChangeOwner {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for CannotChangeOwner {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "CannotChangeOwner()";
            const SELECTOR: [u8; 4] = [253u8, 103u8, 14u8, 190u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn abi_decode_raw_validate(data: &[u8]) -> alloy_sol_types::Result<Self> {
                <Self::Parameters<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Self::new)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `DefaultPermissionRejected()` and selector `0x58723037`.
```solidity
error DefaultPermissionRejected();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct DefaultPermissionRejected;
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(
            _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
        ) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<DefaultPermissionRejected>
        for UnderlyingRustTuple<'_> {
            fn from(value: DefaultPermissionRejected) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>>
        for DefaultPermissionRejected {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for DefaultPermissionRejected {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "DefaultPermissionRejected()";
            const SELECTOR: [u8; 4] = [88u8, 114u8, 48u8, 55u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn abi_decode_raw_validate(data: &[u8]) -> alloy_sol_types::Result<Self> {
                <Self::Parameters<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Self::new)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `DelegateCallNotAllowed()` and selector `0x0d89438e`.
```solidity
error DelegateCallNotAllowed();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct DelegateCallNotAllowed;
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(
            _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
        ) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<DelegateCallNotAllowed> for UnderlyingRustTuple<'_> {
            fn from(value: DelegateCallNotAllowed) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for DelegateCallNotAllowed {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for DelegateCallNotAllowed {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "DelegateCallNotAllowed()";
            const SELECTOR: [u8; 4] = [13u8, 137u8, 67u8, 142u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn abi_decode_raw_validate(data: &[u8]) -> alloy_sol_types::Result<Self> {
                <Self::Parameters<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Self::new)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `FunctionSignatureTooShort()` and selector `0x4684c122`.
```solidity
error FunctionSignatureTooShort();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct FunctionSignatureTooShort;
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(
            _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
        ) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<FunctionSignatureTooShort>
        for UnderlyingRustTuple<'_> {
            fn from(value: FunctionSignatureTooShort) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>>
        for FunctionSignatureTooShort {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for FunctionSignatureTooShort {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "FunctionSignatureTooShort()";
            const SELECTOR: [u8; 4] = [70u8, 132u8, 193u8, 34u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn abi_decode_raw_validate(data: &[u8]) -> alloy_sol_types::Result<Self> {
                <Self::Parameters<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Self::new)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `GranularPermissionRejected()` and selector `0x864dd1e7`.
```solidity
error GranularPermissionRejected();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct GranularPermissionRejected;
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(
            _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
        ) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<GranularPermissionRejected>
        for UnderlyingRustTuple<'_> {
            fn from(value: GranularPermissionRejected) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>>
        for GranularPermissionRejected {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for GranularPermissionRejected {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "GranularPermissionRejected()";
            const SELECTOR: [u8; 4] = [134u8, 77u8, 209u8, 231u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn abi_decode_raw_validate(data: &[u8]) -> alloy_sol_types::Result<Self> {
                <Self::Parameters<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Self::new)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `NoMembership()` and selector `0xfd8e9f28`.
```solidity
error NoMembership();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct NoMembership;
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(
            _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
        ) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<NoMembership> for UnderlyingRustTuple<'_> {
            fn from(value: NoMembership) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for NoMembership {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for NoMembership {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "NoMembership()";
            const SELECTOR: [u8; 4] = [253u8, 142u8, 159u8, 40u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn abi_decode_raw_validate(data: &[u8]) -> alloy_sol_types::Result<Self> {
                <Self::Parameters<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Self::new)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `NodePermissionRejected()` and selector `0x6eb0315f`.
```solidity
error NodePermissionRejected();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct NodePermissionRejected;
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(
            _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
        ) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<NodePermissionRejected> for UnderlyingRustTuple<'_> {
            fn from(value: NodePermissionRejected) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for NodePermissionRejected {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for NodePermissionRejected {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "NodePermissionRejected()";
            const SELECTOR: [u8; 4] = [110u8, 176u8, 49u8, 95u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn abi_decode_raw_validate(data: &[u8]) -> alloy_sol_types::Result<Self> {
                <Self::Parameters<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Self::new)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `NonExistentKey()` and selector `0x2d0519ad`.
```solidity
error NonExistentKey();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct NonExistentKey;
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(
            _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
        ) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<NonExistentKey> for UnderlyingRustTuple<'_> {
            fn from(value: NonExistentKey) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for NonExistentKey {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for NonExistentKey {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "NonExistentKey()";
            const SELECTOR: [u8; 4] = [45u8, 5u8, 25u8, 173u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn abi_decode_raw_validate(data: &[u8]) -> alloy_sol_types::Result<Self> {
                <Self::Parameters<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Self::new)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `ParameterNotAllowed()` and selector `0x31e98246`.
```solidity
error ParameterNotAllowed();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct ParameterNotAllowed;
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(
            _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
        ) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<ParameterNotAllowed> for UnderlyingRustTuple<'_> {
            fn from(value: ParameterNotAllowed) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for ParameterNotAllowed {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for ParameterNotAllowed {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "ParameterNotAllowed()";
            const SELECTOR: [u8; 4] = [49u8, 233u8, 130u8, 70u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn abi_decode_raw_validate(data: &[u8]) -> alloy_sol_types::Result<Self> {
                <Self::Parameters<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Self::new)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `PermissionNotConfigured()` and selector `0x46ad4588`.
```solidity
error PermissionNotConfigured();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct PermissionNotConfigured;
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(
            _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
        ) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<PermissionNotConfigured> for UnderlyingRustTuple<'_> {
            fn from(value: PermissionNotConfigured) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for PermissionNotConfigured {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for PermissionNotConfigured {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "PermissionNotConfigured()";
            const SELECTOR: [u8; 4] = [70u8, 173u8, 69u8, 136u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn abi_decode_raw_validate(data: &[u8]) -> alloy_sol_types::Result<Self> {
                <Self::Parameters<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Self::new)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `PermissionNotFound()` and selector `0xd8455a13`.
```solidity
error PermissionNotFound();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct PermissionNotFound;
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(
            _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
        ) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<PermissionNotFound> for UnderlyingRustTuple<'_> {
            fn from(value: PermissionNotFound) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for PermissionNotFound {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for PermissionNotFound {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "PermissionNotFound()";
            const SELECTOR: [u8; 4] = [216u8, 69u8, 90u8, 19u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn abi_decode_raw_validate(data: &[u8]) -> alloy_sol_types::Result<Self> {
                <Self::Parameters<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Self::new)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `SafeMultisendSameAddress()` and selector `0x598a0e21`.
```solidity
error SafeMultisendSameAddress();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct SafeMultisendSameAddress;
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(
            _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
        ) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<SafeMultisendSameAddress>
        for UnderlyingRustTuple<'_> {
            fn from(value: SafeMultisendSameAddress) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>>
        for SafeMultisendSameAddress {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for SafeMultisendSameAddress {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "SafeMultisendSameAddress()";
            const SELECTOR: [u8; 4] = [89u8, 138u8, 14u8, 33u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn abi_decode_raw_validate(data: &[u8]) -> alloy_sol_types::Result<Self> {
                <Self::Parameters<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Self::new)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `SendNotAllowed()` and selector `0x09e9cd49`.
```solidity
error SendNotAllowed();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct SendNotAllowed;
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(
            _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
        ) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<SendNotAllowed> for UnderlyingRustTuple<'_> {
            fn from(value: SendNotAllowed) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for SendNotAllowed {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for SendNotAllowed {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "SendNotAllowed()";
            const SELECTOR: [u8; 4] = [9u8, 233u8, 205u8, 73u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn abi_decode_raw_validate(data: &[u8]) -> alloy_sol_types::Result<Self> {
                <Self::Parameters<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Self::new)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `TargetAddressNotAllowed()` and selector `0xef3440ac`.
```solidity
error TargetAddressNotAllowed();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct TargetAddressNotAllowed;
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(
            _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
        ) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<TargetAddressNotAllowed> for UnderlyingRustTuple<'_> {
            fn from(value: TargetAddressNotAllowed) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for TargetAddressNotAllowed {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for TargetAddressNotAllowed {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "TargetAddressNotAllowed()";
            const SELECTOR: [u8; 4] = [239u8, 52u8, 64u8, 172u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn abi_decode_raw_validate(data: &[u8]) -> alloy_sol_types::Result<Self> {
                <Self::Parameters<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Self::new)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `TargetIsNotScoped()` and selector `0x4a890321`.
```solidity
error TargetIsNotScoped();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct TargetIsNotScoped;
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(
            _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
        ) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<TargetIsNotScoped> for UnderlyingRustTuple<'_> {
            fn from(value: TargetIsNotScoped) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for TargetIsNotScoped {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for TargetIsNotScoped {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "TargetIsNotScoped()";
            const SELECTOR: [u8; 4] = [74u8, 137u8, 3u8, 33u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn abi_decode_raw_validate(data: &[u8]) -> alloy_sol_types::Result<Self> {
                <Self::Parameters<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Self::new)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `TargetIsScoped()` and selector `0xe8c07d2a`.
```solidity
error TargetIsScoped();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct TargetIsScoped;
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(
            _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
        ) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<TargetIsScoped> for UnderlyingRustTuple<'_> {
            fn from(value: TargetIsScoped) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for TargetIsScoped {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for TargetIsScoped {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "TargetIsScoped()";
            const SELECTOR: [u8; 4] = [232u8, 192u8, 125u8, 42u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn abi_decode_raw_validate(data: &[u8]) -> alloy_sol_types::Result<Self> {
                <Self::Parameters<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Self::new)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `TooManyCapabilities()` and selector `0xb44af9af`.
```solidity
error TooManyCapabilities();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct TooManyCapabilities;
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(
            _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
        ) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<TooManyCapabilities> for UnderlyingRustTuple<'_> {
            fn from(value: TooManyCapabilities) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for TooManyCapabilities {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for TooManyCapabilities {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "TooManyCapabilities()";
            const SELECTOR: [u8; 4] = [180u8, 74u8, 249u8, 175u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn abi_decode_raw_validate(data: &[u8]) -> alloy_sol_types::Result<Self> {
                <Self::Parameters<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Self::new)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `UnacceptableMultiSendOffset()` and selector `0x7ed11137`.
```solidity
error UnacceptableMultiSendOffset();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct UnacceptableMultiSendOffset;
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(
            _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
        ) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnacceptableMultiSendOffset>
        for UnderlyingRustTuple<'_> {
            fn from(value: UnacceptableMultiSendOffset) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>>
        for UnacceptableMultiSendOffset {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for UnacceptableMultiSendOffset {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "UnacceptableMultiSendOffset()";
            const SELECTOR: [u8; 4] = [126u8, 209u8, 17u8, 55u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn abi_decode_raw_validate(data: &[u8]) -> alloy_sol_types::Result<Self> {
                <Self::Parameters<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Self::new)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Custom error with signature `WithMembership()` and selector `0xe3a05a94`.
```solidity
error WithMembership();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct WithMembership;
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = ();
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = ();
        #[cfg(test)]
        #[allow(dead_code, unreachable_patterns)]
        fn _type_assertion(
            _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
        ) {
            match _t {
                alloy_sol_types::private::AssertTypeEq::<
                    <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                >(_) => {}
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<WithMembership> for UnderlyingRustTuple<'_> {
            fn from(value: WithMembership) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for WithMembership {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for WithMembership {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "WithMembership()";
            const SELECTOR: [u8; 4] = [227u8, 160u8, 90u8, 148u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn abi_decode_raw_validate(data: &[u8]) -> alloy_sol_types::Result<Self> {
                <Self::Parameters<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Self::new)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Event with signature `AdminChanged(address,address)` and selector `0x7e644d79422f17c01e4894b5f4f588d331ebfa28653d42ae832dc59e38c9798f`.
```solidity
event AdminChanged(address previousAdmin, address newAdmin);
```*/
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    #[derive(Clone)]
    pub struct AdminChanged {
        #[allow(missing_docs)]
        pub previousAdmin: alloy::sol_types::private::Address,
        #[allow(missing_docs)]
        pub newAdmin: alloy::sol_types::private::Address,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[automatically_derived]
        impl alloy_sol_types::SolEvent for AdminChanged {
            type DataTuple<'a> = (
                alloy::sol_types::sol_data::Address,
                alloy::sol_types::sol_data::Address,
            );
            type DataToken<'a> = <Self::DataTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type TopicList = (alloy_sol_types::sol_data::FixedBytes<32>,);
            const SIGNATURE: &'static str = "AdminChanged(address,address)";
            const SIGNATURE_HASH: alloy_sol_types::private::B256 = alloy_sol_types::private::B256::new([
                126u8, 100u8, 77u8, 121u8, 66u8, 47u8, 23u8, 192u8, 30u8, 72u8, 148u8,
                181u8, 244u8, 245u8, 136u8, 211u8, 49u8, 235u8, 250u8, 40u8, 101u8, 61u8,
                66u8, 174u8, 131u8, 45u8, 197u8, 158u8, 56u8, 201u8, 121u8, 143u8,
            ]);
            const ANONYMOUS: bool = false;
            #[allow(unused_variables)]
            #[inline]
            fn new(
                topics: <Self::TopicList as alloy_sol_types::SolType>::RustType,
                data: <Self::DataTuple<'_> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                Self {
                    previousAdmin: data.0,
                    newAdmin: data.1,
                }
            }
            #[inline]
            fn check_signature(
                topics: &<Self::TopicList as alloy_sol_types::SolType>::RustType,
            ) -> alloy_sol_types::Result<()> {
                if topics.0 != Self::SIGNATURE_HASH {
                    return Err(
                        alloy_sol_types::Error::invalid_event_signature_hash(
                            Self::SIGNATURE,
                            topics.0,
                            Self::SIGNATURE_HASH,
                        ),
                    );
                }
                Ok(())
            }
            #[inline]
            fn tokenize_body(&self) -> Self::DataToken<'_> {
                (
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        &self.previousAdmin,
                    ),
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        &self.newAdmin,
                    ),
                )
            }
            #[inline]
            fn topics(&self) -> <Self::TopicList as alloy_sol_types::SolType>::RustType {
                (Self::SIGNATURE_HASH.into(),)
            }
            #[inline]
            fn encode_topics_raw(
                &self,
                out: &mut [alloy_sol_types::abi::token::WordToken],
            ) -> alloy_sol_types::Result<()> {
                if out.len() < <Self::TopicList as alloy_sol_types::TopicList>::COUNT {
                    return Err(alloy_sol_types::Error::Overrun);
                }
                out[0usize] = alloy_sol_types::abi::token::WordToken(
                    Self::SIGNATURE_HASH,
                );
                Ok(())
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::private::IntoLogData for AdminChanged {
            fn to_log_data(&self) -> alloy_sol_types::private::LogData {
                From::from(self)
            }
            fn into_log_data(self) -> alloy_sol_types::private::LogData {
                From::from(&self)
            }
        }
        #[automatically_derived]
        impl From<&AdminChanged> for alloy_sol_types::private::LogData {
            #[inline]
            fn from(this: &AdminChanged) -> alloy_sol_types::private::LogData {
                alloy_sol_types::SolEvent::encode_log_data(this)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Event with signature `BeaconUpgraded(address)` and selector `0x1cf3b03a6cf19fa2baba4df148e9dcabedea7f8a5c07840e207e5c089be95d3e`.
```solidity
event BeaconUpgraded(address indexed beacon);
```*/
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    #[derive(Clone)]
    pub struct BeaconUpgraded {
        #[allow(missing_docs)]
        pub beacon: alloy::sol_types::private::Address,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[automatically_derived]
        impl alloy_sol_types::SolEvent for BeaconUpgraded {
            type DataTuple<'a> = ();
            type DataToken<'a> = <Self::DataTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type TopicList = (
                alloy_sol_types::sol_data::FixedBytes<32>,
                alloy::sol_types::sol_data::Address,
            );
            const SIGNATURE: &'static str = "BeaconUpgraded(address)";
            const SIGNATURE_HASH: alloy_sol_types::private::B256 = alloy_sol_types::private::B256::new([
                28u8, 243u8, 176u8, 58u8, 108u8, 241u8, 159u8, 162u8, 186u8, 186u8, 77u8,
                241u8, 72u8, 233u8, 220u8, 171u8, 237u8, 234u8, 127u8, 138u8, 92u8, 7u8,
                132u8, 14u8, 32u8, 126u8, 92u8, 8u8, 155u8, 233u8, 93u8, 62u8,
            ]);
            const ANONYMOUS: bool = false;
            #[allow(unused_variables)]
            #[inline]
            fn new(
                topics: <Self::TopicList as alloy_sol_types::SolType>::RustType,
                data: <Self::DataTuple<'_> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                Self { beacon: topics.1 }
            }
            #[inline]
            fn check_signature(
                topics: &<Self::TopicList as alloy_sol_types::SolType>::RustType,
            ) -> alloy_sol_types::Result<()> {
                if topics.0 != Self::SIGNATURE_HASH {
                    return Err(
                        alloy_sol_types::Error::invalid_event_signature_hash(
                            Self::SIGNATURE,
                            topics.0,
                            Self::SIGNATURE_HASH,
                        ),
                    );
                }
                Ok(())
            }
            #[inline]
            fn tokenize_body(&self) -> Self::DataToken<'_> {
                ()
            }
            #[inline]
            fn topics(&self) -> <Self::TopicList as alloy_sol_types::SolType>::RustType {
                (Self::SIGNATURE_HASH.into(), self.beacon.clone())
            }
            #[inline]
            fn encode_topics_raw(
                &self,
                out: &mut [alloy_sol_types::abi::token::WordToken],
            ) -> alloy_sol_types::Result<()> {
                if out.len() < <Self::TopicList as alloy_sol_types::TopicList>::COUNT {
                    return Err(alloy_sol_types::Error::Overrun);
                }
                out[0usize] = alloy_sol_types::abi::token::WordToken(
                    Self::SIGNATURE_HASH,
                );
                out[1usize] = <alloy::sol_types::sol_data::Address as alloy_sol_types::EventTopic>::encode_topic(
                    &self.beacon,
                );
                Ok(())
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::private::IntoLogData for BeaconUpgraded {
            fn to_log_data(&self) -> alloy_sol_types::private::LogData {
                From::from(self)
            }
            fn into_log_data(self) -> alloy_sol_types::private::LogData {
                From::from(&self)
            }
        }
        #[automatically_derived]
        impl From<&BeaconUpgraded> for alloy_sol_types::private::LogData {
            #[inline]
            fn from(this: &BeaconUpgraded) -> alloy_sol_types::private::LogData {
                alloy_sol_types::SolEvent::encode_log_data(this)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Event with signature `ExecutionFailure()` and selector `0xc24d93608a03d263ff191d7677141f5e94c496e593108f3aae0cb5b70494c4d3`.
```solidity
event ExecutionFailure();
```*/
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    #[derive(Clone)]
    pub struct ExecutionFailure;
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[automatically_derived]
        impl alloy_sol_types::SolEvent for ExecutionFailure {
            type DataTuple<'a> = ();
            type DataToken<'a> = <Self::DataTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type TopicList = (alloy_sol_types::sol_data::FixedBytes<32>,);
            const SIGNATURE: &'static str = "ExecutionFailure()";
            const SIGNATURE_HASH: alloy_sol_types::private::B256 = alloy_sol_types::private::B256::new([
                194u8, 77u8, 147u8, 96u8, 138u8, 3u8, 210u8, 99u8, 255u8, 25u8, 29u8,
                118u8, 119u8, 20u8, 31u8, 94u8, 148u8, 196u8, 150u8, 229u8, 147u8, 16u8,
                143u8, 58u8, 174u8, 12u8, 181u8, 183u8, 4u8, 148u8, 196u8, 211u8,
            ]);
            const ANONYMOUS: bool = false;
            #[allow(unused_variables)]
            #[inline]
            fn new(
                topics: <Self::TopicList as alloy_sol_types::SolType>::RustType,
                data: <Self::DataTuple<'_> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                Self {}
            }
            #[inline]
            fn check_signature(
                topics: &<Self::TopicList as alloy_sol_types::SolType>::RustType,
            ) -> alloy_sol_types::Result<()> {
                if topics.0 != Self::SIGNATURE_HASH {
                    return Err(
                        alloy_sol_types::Error::invalid_event_signature_hash(
                            Self::SIGNATURE,
                            topics.0,
                            Self::SIGNATURE_HASH,
                        ),
                    );
                }
                Ok(())
            }
            #[inline]
            fn tokenize_body(&self) -> Self::DataToken<'_> {
                ()
            }
            #[inline]
            fn topics(&self) -> <Self::TopicList as alloy_sol_types::SolType>::RustType {
                (Self::SIGNATURE_HASH.into(),)
            }
            #[inline]
            fn encode_topics_raw(
                &self,
                out: &mut [alloy_sol_types::abi::token::WordToken],
            ) -> alloy_sol_types::Result<()> {
                if out.len() < <Self::TopicList as alloy_sol_types::TopicList>::COUNT {
                    return Err(alloy_sol_types::Error::Overrun);
                }
                out[0usize] = alloy_sol_types::abi::token::WordToken(
                    Self::SIGNATURE_HASH,
                );
                Ok(())
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::private::IntoLogData for ExecutionFailure {
            fn to_log_data(&self) -> alloy_sol_types::private::LogData {
                From::from(self)
            }
            fn into_log_data(self) -> alloy_sol_types::private::LogData {
                From::from(&self)
            }
        }
        #[automatically_derived]
        impl From<&ExecutionFailure> for alloy_sol_types::private::LogData {
            #[inline]
            fn from(this: &ExecutionFailure) -> alloy_sol_types::private::LogData {
                alloy_sol_types::SolEvent::encode_log_data(this)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Event with signature `ExecutionSuccess()` and selector `0x4e2e86d21375ebcbf6e93df5ebdd5a915bf830245904c3b54f48adf0170aae4b`.
```solidity
event ExecutionSuccess();
```*/
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    #[derive(Clone)]
    pub struct ExecutionSuccess;
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[automatically_derived]
        impl alloy_sol_types::SolEvent for ExecutionSuccess {
            type DataTuple<'a> = ();
            type DataToken<'a> = <Self::DataTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type TopicList = (alloy_sol_types::sol_data::FixedBytes<32>,);
            const SIGNATURE: &'static str = "ExecutionSuccess()";
            const SIGNATURE_HASH: alloy_sol_types::private::B256 = alloy_sol_types::private::B256::new([
                78u8, 46u8, 134u8, 210u8, 19u8, 117u8, 235u8, 203u8, 246u8, 233u8, 61u8,
                245u8, 235u8, 221u8, 90u8, 145u8, 91u8, 248u8, 48u8, 36u8, 89u8, 4u8,
                195u8, 181u8, 79u8, 72u8, 173u8, 240u8, 23u8, 10u8, 174u8, 75u8,
            ]);
            const ANONYMOUS: bool = false;
            #[allow(unused_variables)]
            #[inline]
            fn new(
                topics: <Self::TopicList as alloy_sol_types::SolType>::RustType,
                data: <Self::DataTuple<'_> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                Self {}
            }
            #[inline]
            fn check_signature(
                topics: &<Self::TopicList as alloy_sol_types::SolType>::RustType,
            ) -> alloy_sol_types::Result<()> {
                if topics.0 != Self::SIGNATURE_HASH {
                    return Err(
                        alloy_sol_types::Error::invalid_event_signature_hash(
                            Self::SIGNATURE,
                            topics.0,
                            Self::SIGNATURE_HASH,
                        ),
                    );
                }
                Ok(())
            }
            #[inline]
            fn tokenize_body(&self) -> Self::DataToken<'_> {
                ()
            }
            #[inline]
            fn topics(&self) -> <Self::TopicList as alloy_sol_types::SolType>::RustType {
                (Self::SIGNATURE_HASH.into(),)
            }
            #[inline]
            fn encode_topics_raw(
                &self,
                out: &mut [alloy_sol_types::abi::token::WordToken],
            ) -> alloy_sol_types::Result<()> {
                if out.len() < <Self::TopicList as alloy_sol_types::TopicList>::COUNT {
                    return Err(alloy_sol_types::Error::Overrun);
                }
                out[0usize] = alloy_sol_types::abi::token::WordToken(
                    Self::SIGNATURE_HASH,
                );
                Ok(())
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::private::IntoLogData for ExecutionSuccess {
            fn to_log_data(&self) -> alloy_sol_types::private::LogData {
                From::from(self)
            }
            fn into_log_data(self) -> alloy_sol_types::private::LogData {
                From::from(&self)
            }
        }
        #[automatically_derived]
        impl From<&ExecutionSuccess> for alloy_sol_types::private::LogData {
            #[inline]
            fn from(this: &ExecutionSuccess) -> alloy_sol_types::private::LogData {
                alloy_sol_types::SolEvent::encode_log_data(this)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Event with signature `Initialized(uint8)` and selector `0x7f26b83ff96e1f2b6a682f133852f6798a09c465da95921460cefb3847402498`.
```solidity
event Initialized(uint8 version);
```*/
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    #[derive(Clone)]
    pub struct Initialized {
        #[allow(missing_docs)]
        pub version: u8,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[automatically_derived]
        impl alloy_sol_types::SolEvent for Initialized {
            type DataTuple<'a> = (alloy::sol_types::sol_data::Uint<8>,);
            type DataToken<'a> = <Self::DataTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type TopicList = (alloy_sol_types::sol_data::FixedBytes<32>,);
            const SIGNATURE: &'static str = "Initialized(uint8)";
            const SIGNATURE_HASH: alloy_sol_types::private::B256 = alloy_sol_types::private::B256::new([
                127u8, 38u8, 184u8, 63u8, 249u8, 110u8, 31u8, 43u8, 106u8, 104u8, 47u8,
                19u8, 56u8, 82u8, 246u8, 121u8, 138u8, 9u8, 196u8, 101u8, 218u8, 149u8,
                146u8, 20u8, 96u8, 206u8, 251u8, 56u8, 71u8, 64u8, 36u8, 152u8,
            ]);
            const ANONYMOUS: bool = false;
            #[allow(unused_variables)]
            #[inline]
            fn new(
                topics: <Self::TopicList as alloy_sol_types::SolType>::RustType,
                data: <Self::DataTuple<'_> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                Self { version: data.0 }
            }
            #[inline]
            fn check_signature(
                topics: &<Self::TopicList as alloy_sol_types::SolType>::RustType,
            ) -> alloy_sol_types::Result<()> {
                if topics.0 != Self::SIGNATURE_HASH {
                    return Err(
                        alloy_sol_types::Error::invalid_event_signature_hash(
                            Self::SIGNATURE,
                            topics.0,
                            Self::SIGNATURE_HASH,
                        ),
                    );
                }
                Ok(())
            }
            #[inline]
            fn tokenize_body(&self) -> Self::DataToken<'_> {
                (
                    <alloy::sol_types::sol_data::Uint<
                        8,
                    > as alloy_sol_types::SolType>::tokenize(&self.version),
                )
            }
            #[inline]
            fn topics(&self) -> <Self::TopicList as alloy_sol_types::SolType>::RustType {
                (Self::SIGNATURE_HASH.into(),)
            }
            #[inline]
            fn encode_topics_raw(
                &self,
                out: &mut [alloy_sol_types::abi::token::WordToken],
            ) -> alloy_sol_types::Result<()> {
                if out.len() < <Self::TopicList as alloy_sol_types::TopicList>::COUNT {
                    return Err(alloy_sol_types::Error::Overrun);
                }
                out[0usize] = alloy_sol_types::abi::token::WordToken(
                    Self::SIGNATURE_HASH,
                );
                Ok(())
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::private::IntoLogData for Initialized {
            fn to_log_data(&self) -> alloy_sol_types::private::LogData {
                From::from(self)
            }
            fn into_log_data(self) -> alloy_sol_types::private::LogData {
                From::from(&self)
            }
        }
        #[automatically_derived]
        impl From<&Initialized> for alloy_sol_types::private::LogData {
            #[inline]
            fn from(this: &Initialized) -> alloy_sol_types::private::LogData {
                alloy_sol_types::SolEvent::encode_log_data(this)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Event with signature `NodeAdded(address)` and selector `0xb25d03aaf308d7291709be1ea28b800463cf3a9a4c4a5555d7333a964c1dfebd`.
```solidity
event NodeAdded(address indexed node);
```*/
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    #[derive(Clone)]
    pub struct NodeAdded {
        #[allow(missing_docs)]
        pub node: alloy::sol_types::private::Address,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[automatically_derived]
        impl alloy_sol_types::SolEvent for NodeAdded {
            type DataTuple<'a> = ();
            type DataToken<'a> = <Self::DataTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type TopicList = (
                alloy_sol_types::sol_data::FixedBytes<32>,
                alloy::sol_types::sol_data::Address,
            );
            const SIGNATURE: &'static str = "NodeAdded(address)";
            const SIGNATURE_HASH: alloy_sol_types::private::B256 = alloy_sol_types::private::B256::new([
                178u8, 93u8, 3u8, 170u8, 243u8, 8u8, 215u8, 41u8, 23u8, 9u8, 190u8, 30u8,
                162u8, 139u8, 128u8, 4u8, 99u8, 207u8, 58u8, 154u8, 76u8, 74u8, 85u8,
                85u8, 215u8, 51u8, 58u8, 150u8, 76u8, 29u8, 254u8, 189u8,
            ]);
            const ANONYMOUS: bool = false;
            #[allow(unused_variables)]
            #[inline]
            fn new(
                topics: <Self::TopicList as alloy_sol_types::SolType>::RustType,
                data: <Self::DataTuple<'_> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                Self { node: topics.1 }
            }
            #[inline]
            fn check_signature(
                topics: &<Self::TopicList as alloy_sol_types::SolType>::RustType,
            ) -> alloy_sol_types::Result<()> {
                if topics.0 != Self::SIGNATURE_HASH {
                    return Err(
                        alloy_sol_types::Error::invalid_event_signature_hash(
                            Self::SIGNATURE,
                            topics.0,
                            Self::SIGNATURE_HASH,
                        ),
                    );
                }
                Ok(())
            }
            #[inline]
            fn tokenize_body(&self) -> Self::DataToken<'_> {
                ()
            }
            #[inline]
            fn topics(&self) -> <Self::TopicList as alloy_sol_types::SolType>::RustType {
                (Self::SIGNATURE_HASH.into(), self.node.clone())
            }
            #[inline]
            fn encode_topics_raw(
                &self,
                out: &mut [alloy_sol_types::abi::token::WordToken],
            ) -> alloy_sol_types::Result<()> {
                if out.len() < <Self::TopicList as alloy_sol_types::TopicList>::COUNT {
                    return Err(alloy_sol_types::Error::Overrun);
                }
                out[0usize] = alloy_sol_types::abi::token::WordToken(
                    Self::SIGNATURE_HASH,
                );
                out[1usize] = <alloy::sol_types::sol_data::Address as alloy_sol_types::EventTopic>::encode_topic(
                    &self.node,
                );
                Ok(())
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::private::IntoLogData for NodeAdded {
            fn to_log_data(&self) -> alloy_sol_types::private::LogData {
                From::from(self)
            }
            fn into_log_data(self) -> alloy_sol_types::private::LogData {
                From::from(&self)
            }
        }
        #[automatically_derived]
        impl From<&NodeAdded> for alloy_sol_types::private::LogData {
            #[inline]
            fn from(this: &NodeAdded) -> alloy_sol_types::private::LogData {
                alloy_sol_types::SolEvent::encode_log_data(this)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Event with signature `NodeRemoved(address)` and selector `0xcfc24166db4bb677e857cacabd1541fb2b30645021b27c5130419589b84db52b`.
```solidity
event NodeRemoved(address indexed node);
```*/
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    #[derive(Clone)]
    pub struct NodeRemoved {
        #[allow(missing_docs)]
        pub node: alloy::sol_types::private::Address,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[automatically_derived]
        impl alloy_sol_types::SolEvent for NodeRemoved {
            type DataTuple<'a> = ();
            type DataToken<'a> = <Self::DataTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type TopicList = (
                alloy_sol_types::sol_data::FixedBytes<32>,
                alloy::sol_types::sol_data::Address,
            );
            const SIGNATURE: &'static str = "NodeRemoved(address)";
            const SIGNATURE_HASH: alloy_sol_types::private::B256 = alloy_sol_types::private::B256::new([
                207u8, 194u8, 65u8, 102u8, 219u8, 75u8, 182u8, 119u8, 232u8, 87u8, 202u8,
                202u8, 189u8, 21u8, 65u8, 251u8, 43u8, 48u8, 100u8, 80u8, 33u8, 178u8,
                124u8, 81u8, 48u8, 65u8, 149u8, 137u8, 184u8, 77u8, 181u8, 43u8,
            ]);
            const ANONYMOUS: bool = false;
            #[allow(unused_variables)]
            #[inline]
            fn new(
                topics: <Self::TopicList as alloy_sol_types::SolType>::RustType,
                data: <Self::DataTuple<'_> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                Self { node: topics.1 }
            }
            #[inline]
            fn check_signature(
                topics: &<Self::TopicList as alloy_sol_types::SolType>::RustType,
            ) -> alloy_sol_types::Result<()> {
                if topics.0 != Self::SIGNATURE_HASH {
                    return Err(
                        alloy_sol_types::Error::invalid_event_signature_hash(
                            Self::SIGNATURE,
                            topics.0,
                            Self::SIGNATURE_HASH,
                        ),
                    );
                }
                Ok(())
            }
            #[inline]
            fn tokenize_body(&self) -> Self::DataToken<'_> {
                ()
            }
            #[inline]
            fn topics(&self) -> <Self::TopicList as alloy_sol_types::SolType>::RustType {
                (Self::SIGNATURE_HASH.into(), self.node.clone())
            }
            #[inline]
            fn encode_topics_raw(
                &self,
                out: &mut [alloy_sol_types::abi::token::WordToken],
            ) -> alloy_sol_types::Result<()> {
                if out.len() < <Self::TopicList as alloy_sol_types::TopicList>::COUNT {
                    return Err(alloy_sol_types::Error::Overrun);
                }
                out[0usize] = alloy_sol_types::abi::token::WordToken(
                    Self::SIGNATURE_HASH,
                );
                out[1usize] = <alloy::sol_types::sol_data::Address as alloy_sol_types::EventTopic>::encode_topic(
                    &self.node,
                );
                Ok(())
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::private::IntoLogData for NodeRemoved {
            fn to_log_data(&self) -> alloy_sol_types::private::LogData {
                From::from(self)
            }
            fn into_log_data(self) -> alloy_sol_types::private::LogData {
                From::from(&self)
            }
        }
        #[automatically_derived]
        impl From<&NodeRemoved> for alloy_sol_types::private::LogData {
            #[inline]
            fn from(this: &NodeRemoved) -> alloy_sol_types::private::LogData {
                alloy_sol_types::SolEvent::encode_log_data(this)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Event with signature `OwnershipTransferred(address,address)` and selector `0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0`.
```solidity
event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
```*/
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    #[derive(Clone)]
    pub struct OwnershipTransferred {
        #[allow(missing_docs)]
        pub previousOwner: alloy::sol_types::private::Address,
        #[allow(missing_docs)]
        pub newOwner: alloy::sol_types::private::Address,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[automatically_derived]
        impl alloy_sol_types::SolEvent for OwnershipTransferred {
            type DataTuple<'a> = ();
            type DataToken<'a> = <Self::DataTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type TopicList = (
                alloy_sol_types::sol_data::FixedBytes<32>,
                alloy::sol_types::sol_data::Address,
                alloy::sol_types::sol_data::Address,
            );
            const SIGNATURE: &'static str = "OwnershipTransferred(address,address)";
            const SIGNATURE_HASH: alloy_sol_types::private::B256 = alloy_sol_types::private::B256::new([
                139u8, 224u8, 7u8, 156u8, 83u8, 22u8, 89u8, 20u8, 19u8, 68u8, 205u8,
                31u8, 208u8, 164u8, 242u8, 132u8, 25u8, 73u8, 127u8, 151u8, 34u8, 163u8,
                218u8, 175u8, 227u8, 180u8, 24u8, 111u8, 107u8, 100u8, 87u8, 224u8,
            ]);
            const ANONYMOUS: bool = false;
            #[allow(unused_variables)]
            #[inline]
            fn new(
                topics: <Self::TopicList as alloy_sol_types::SolType>::RustType,
                data: <Self::DataTuple<'_> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                Self {
                    previousOwner: topics.1,
                    newOwner: topics.2,
                }
            }
            #[inline]
            fn check_signature(
                topics: &<Self::TopicList as alloy_sol_types::SolType>::RustType,
            ) -> alloy_sol_types::Result<()> {
                if topics.0 != Self::SIGNATURE_HASH {
                    return Err(
                        alloy_sol_types::Error::invalid_event_signature_hash(
                            Self::SIGNATURE,
                            topics.0,
                            Self::SIGNATURE_HASH,
                        ),
                    );
                }
                Ok(())
            }
            #[inline]
            fn tokenize_body(&self) -> Self::DataToken<'_> {
                ()
            }
            #[inline]
            fn topics(&self) -> <Self::TopicList as alloy_sol_types::SolType>::RustType {
                (
                    Self::SIGNATURE_HASH.into(),
                    self.previousOwner.clone(),
                    self.newOwner.clone(),
                )
            }
            #[inline]
            fn encode_topics_raw(
                &self,
                out: &mut [alloy_sol_types::abi::token::WordToken],
            ) -> alloy_sol_types::Result<()> {
                if out.len() < <Self::TopicList as alloy_sol_types::TopicList>::COUNT {
                    return Err(alloy_sol_types::Error::Overrun);
                }
                out[0usize] = alloy_sol_types::abi::token::WordToken(
                    Self::SIGNATURE_HASH,
                );
                out[1usize] = <alloy::sol_types::sol_data::Address as alloy_sol_types::EventTopic>::encode_topic(
                    &self.previousOwner,
                );
                out[2usize] = <alloy::sol_types::sol_data::Address as alloy_sol_types::EventTopic>::encode_topic(
                    &self.newOwner,
                );
                Ok(())
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::private::IntoLogData for OwnershipTransferred {
            fn to_log_data(&self) -> alloy_sol_types::private::LogData {
                From::from(self)
            }
            fn into_log_data(self) -> alloy_sol_types::private::LogData {
                From::from(&self)
            }
        }
        #[automatically_derived]
        impl From<&OwnershipTransferred> for alloy_sol_types::private::LogData {
            #[inline]
            fn from(this: &OwnershipTransferred) -> alloy_sol_types::private::LogData {
                alloy_sol_types::SolEvent::encode_log_data(this)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Event with signature `SetMultisendAddress(address)` and selector `0x5fe6aabf4e790843df43ae0e22b58620066fb389295bedc06a92df6c3b28777d`.
```solidity
event SetMultisendAddress(address indexed multisendAddress);
```*/
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    #[derive(Clone)]
    pub struct SetMultisendAddress {
        #[allow(missing_docs)]
        pub multisendAddress: alloy::sol_types::private::Address,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[automatically_derived]
        impl alloy_sol_types::SolEvent for SetMultisendAddress {
            type DataTuple<'a> = ();
            type DataToken<'a> = <Self::DataTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type TopicList = (
                alloy_sol_types::sol_data::FixedBytes<32>,
                alloy::sol_types::sol_data::Address,
            );
            const SIGNATURE: &'static str = "SetMultisendAddress(address)";
            const SIGNATURE_HASH: alloy_sol_types::private::B256 = alloy_sol_types::private::B256::new([
                95u8, 230u8, 170u8, 191u8, 78u8, 121u8, 8u8, 67u8, 223u8, 67u8, 174u8,
                14u8, 34u8, 181u8, 134u8, 32u8, 6u8, 111u8, 179u8, 137u8, 41u8, 91u8,
                237u8, 192u8, 106u8, 146u8, 223u8, 108u8, 59u8, 40u8, 119u8, 125u8,
            ]);
            const ANONYMOUS: bool = false;
            #[allow(unused_variables)]
            #[inline]
            fn new(
                topics: <Self::TopicList as alloy_sol_types::SolType>::RustType,
                data: <Self::DataTuple<'_> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                Self { multisendAddress: topics.1 }
            }
            #[inline]
            fn check_signature(
                topics: &<Self::TopicList as alloy_sol_types::SolType>::RustType,
            ) -> alloy_sol_types::Result<()> {
                if topics.0 != Self::SIGNATURE_HASH {
                    return Err(
                        alloy_sol_types::Error::invalid_event_signature_hash(
                            Self::SIGNATURE,
                            topics.0,
                            Self::SIGNATURE_HASH,
                        ),
                    );
                }
                Ok(())
            }
            #[inline]
            fn tokenize_body(&self) -> Self::DataToken<'_> {
                ()
            }
            #[inline]
            fn topics(&self) -> <Self::TopicList as alloy_sol_types::SolType>::RustType {
                (Self::SIGNATURE_HASH.into(), self.multisendAddress.clone())
            }
            #[inline]
            fn encode_topics_raw(
                &self,
                out: &mut [alloy_sol_types::abi::token::WordToken],
            ) -> alloy_sol_types::Result<()> {
                if out.len() < <Self::TopicList as alloy_sol_types::TopicList>::COUNT {
                    return Err(alloy_sol_types::Error::Overrun);
                }
                out[0usize] = alloy_sol_types::abi::token::WordToken(
                    Self::SIGNATURE_HASH,
                );
                out[1usize] = <alloy::sol_types::sol_data::Address as alloy_sol_types::EventTopic>::encode_topic(
                    &self.multisendAddress,
                );
                Ok(())
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::private::IntoLogData for SetMultisendAddress {
            fn to_log_data(&self) -> alloy_sol_types::private::LogData {
                From::from(self)
            }
            fn into_log_data(self) -> alloy_sol_types::private::LogData {
                From::from(&self)
            }
        }
        #[automatically_derived]
        impl From<&SetMultisendAddress> for alloy_sol_types::private::LogData {
            #[inline]
            fn from(this: &SetMultisendAddress) -> alloy_sol_types::private::LogData {
                alloy_sol_types::SolEvent::encode_log_data(this)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Event with signature `Upgraded(address)` and selector `0xbc7cd75a20ee27fd9adebab32041f755214dbc6bffa90cc0225b39da2e5c2d3b`.
```solidity
event Upgraded(address indexed implementation);
```*/
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    #[derive(Clone)]
    pub struct Upgraded {
        #[allow(missing_docs)]
        pub implementation: alloy::sol_types::private::Address,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[automatically_derived]
        impl alloy_sol_types::SolEvent for Upgraded {
            type DataTuple<'a> = ();
            type DataToken<'a> = <Self::DataTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type TopicList = (
                alloy_sol_types::sol_data::FixedBytes<32>,
                alloy::sol_types::sol_data::Address,
            );
            const SIGNATURE: &'static str = "Upgraded(address)";
            const SIGNATURE_HASH: alloy_sol_types::private::B256 = alloy_sol_types::private::B256::new([
                188u8, 124u8, 215u8, 90u8, 32u8, 238u8, 39u8, 253u8, 154u8, 222u8, 186u8,
                179u8, 32u8, 65u8, 247u8, 85u8, 33u8, 77u8, 188u8, 107u8, 255u8, 169u8,
                12u8, 192u8, 34u8, 91u8, 57u8, 218u8, 46u8, 92u8, 45u8, 59u8,
            ]);
            const ANONYMOUS: bool = false;
            #[allow(unused_variables)]
            #[inline]
            fn new(
                topics: <Self::TopicList as alloy_sol_types::SolType>::RustType,
                data: <Self::DataTuple<'_> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                Self { implementation: topics.1 }
            }
            #[inline]
            fn check_signature(
                topics: &<Self::TopicList as alloy_sol_types::SolType>::RustType,
            ) -> alloy_sol_types::Result<()> {
                if topics.0 != Self::SIGNATURE_HASH {
                    return Err(
                        alloy_sol_types::Error::invalid_event_signature_hash(
                            Self::SIGNATURE,
                            topics.0,
                            Self::SIGNATURE_HASH,
                        ),
                    );
                }
                Ok(())
            }
            #[inline]
            fn tokenize_body(&self) -> Self::DataToken<'_> {
                ()
            }
            #[inline]
            fn topics(&self) -> <Self::TopicList as alloy_sol_types::SolType>::RustType {
                (Self::SIGNATURE_HASH.into(), self.implementation.clone())
            }
            #[inline]
            fn encode_topics_raw(
                &self,
                out: &mut [alloy_sol_types::abi::token::WordToken],
            ) -> alloy_sol_types::Result<()> {
                if out.len() < <Self::TopicList as alloy_sol_types::TopicList>::COUNT {
                    return Err(alloy_sol_types::Error::Overrun);
                }
                out[0usize] = alloy_sol_types::abi::token::WordToken(
                    Self::SIGNATURE_HASH,
                );
                out[1usize] = <alloy::sol_types::sol_data::Address as alloy_sol_types::EventTopic>::encode_topic(
                    &self.implementation,
                );
                Ok(())
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::private::IntoLogData for Upgraded {
            fn to_log_data(&self) -> alloy_sol_types::private::LogData {
                From::from(self)
            }
            fn into_log_data(self) -> alloy_sol_types::private::LogData {
                From::from(&self)
            }
        }
        #[automatically_derived]
        impl From<&Upgraded> for alloy_sol_types::private::LogData {
            #[inline]
            fn from(this: &Upgraded) -> alloy_sol_types::private::LogData {
                alloy_sol_types::SolEvent::encode_log_data(this)
            }
        }
    };
    /**Constructor`.
```solidity
constructor();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct constructorCall {}
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = ();
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = ();
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<constructorCall> for UnderlyingRustTuple<'_> {
                fn from(value: constructorCall) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for constructorCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolConstructor for constructorCall {
            type Parameters<'a> = ();
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `addChannelsAndTokenTarget(uint256)` and selector `0xa2450f89`.
```solidity
function addChannelsAndTokenTarget(Target defaultTarget) external;
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct addChannelsAndTokenTargetCall {
        #[allow(missing_docs)]
        pub defaultTarget: <Target as alloy::sol_types::SolType>::RustType,
    }
    ///Container type for the return parameters of the [`addChannelsAndTokenTarget(uint256)`](addChannelsAndTokenTargetCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct addChannelsAndTokenTargetReturn {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (Target,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                <Target as alloy::sol_types::SolType>::RustType,
            );
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<addChannelsAndTokenTargetCall>
            for UnderlyingRustTuple<'_> {
                fn from(value: addChannelsAndTokenTargetCall) -> Self {
                    (value.defaultTarget,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for addChannelsAndTokenTargetCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { defaultTarget: tuple.0 }
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = ();
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = ();
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<addChannelsAndTokenTargetReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: addChannelsAndTokenTargetReturn) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for addChannelsAndTokenTargetReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        impl addChannelsAndTokenTargetReturn {
            fn _tokenize(
                &self,
            ) -> <addChannelsAndTokenTargetCall as alloy_sol_types::SolCall>::ReturnToken<
                '_,
            > {
                ()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for addChannelsAndTokenTargetCall {
            type Parameters<'a> = (Target,);
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = addChannelsAndTokenTargetReturn;
            type ReturnTuple<'a> = ();
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "addChannelsAndTokenTarget(uint256)";
            const SELECTOR: [u8; 4] = [162u8, 69u8, 15u8, 137u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                (<Target as alloy_sol_types::SolType>::tokenize(&self.defaultTarget),)
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                addChannelsAndTokenTargetReturn::_tokenize(ret)
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(Into::into)
            }
            #[inline]
            fn abi_decode_returns_validate(
                data: &[u8],
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Into::into)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `addNode(address)` and selector `0x9d95f1cc`.
```solidity
function addNode(address nodeAddress) external;
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct addNodeCall {
        #[allow(missing_docs)]
        pub nodeAddress: alloy::sol_types::private::Address,
    }
    ///Container type for the return parameters of the [`addNode(address)`](addNodeCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct addNodeReturn {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (alloy::sol_types::sol_data::Address,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (alloy::sol_types::private::Address,);
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<addNodeCall> for UnderlyingRustTuple<'_> {
                fn from(value: addNodeCall) -> Self {
                    (value.nodeAddress,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for addNodeCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { nodeAddress: tuple.0 }
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = ();
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = ();
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<addNodeReturn> for UnderlyingRustTuple<'_> {
                fn from(value: addNodeReturn) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for addNodeReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        impl addNodeReturn {
            fn _tokenize(
                &self,
            ) -> <addNodeCall as alloy_sol_types::SolCall>::ReturnToken<'_> {
                ()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for addNodeCall {
            type Parameters<'a> = (alloy::sol_types::sol_data::Address,);
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = addNodeReturn;
            type ReturnTuple<'a> = ();
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "addNode(address)";
            const SELECTOR: [u8; 4] = [157u8, 149u8, 241u8, 204u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                (
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        &self.nodeAddress,
                    ),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                addNodeReturn::_tokenize(ret)
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(Into::into)
            }
            #[inline]
            fn abi_decode_returns_validate(
                data: &[u8],
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Into::into)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `decodeFunctionSigsAndPermissions(bytes32,uint256)` and selector `0x60976c4b`.
```solidity
function decodeFunctionSigsAndPermissions(bytes32 encoded, uint256 length) external pure returns (bytes4[] memory functionSigs, GranularPermission[] memory permissions);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct decodeFunctionSigsAndPermissionsCall {
        #[allow(missing_docs)]
        pub encoded: alloy::sol_types::private::FixedBytes<32>,
        #[allow(missing_docs)]
        pub length: alloy::sol_types::private::primitives::aliases::U256,
    }
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`decodeFunctionSigsAndPermissions(bytes32,uint256)`](decodeFunctionSigsAndPermissionsCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct decodeFunctionSigsAndPermissionsReturn {
        #[allow(missing_docs)]
        pub functionSigs: alloy::sol_types::private::Vec<
            alloy::sol_types::private::FixedBytes<4>,
        >,
        #[allow(missing_docs)]
        pub permissions: alloy::sol_types::private::Vec<
            <GranularPermission as alloy::sol_types::SolType>::RustType,
        >,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (
                alloy::sol_types::sol_data::FixedBytes<32>,
                alloy::sol_types::sol_data::Uint<256>,
            );
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                alloy::sol_types::private::FixedBytes<32>,
                alloy::sol_types::private::primitives::aliases::U256,
            );
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<decodeFunctionSigsAndPermissionsCall>
            for UnderlyingRustTuple<'_> {
                fn from(value: decodeFunctionSigsAndPermissionsCall) -> Self {
                    (value.encoded, value.length)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for decodeFunctionSigsAndPermissionsCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {
                        encoded: tuple.0,
                        length: tuple.1,
                    }
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (
                alloy::sol_types::sol_data::Array<
                    alloy::sol_types::sol_data::FixedBytes<4>,
                >,
                alloy::sol_types::sol_data::Array<GranularPermission>,
            );
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                alloy::sol_types::private::Vec<alloy::sol_types::private::FixedBytes<4>>,
                alloy::sol_types::private::Vec<
                    <GranularPermission as alloy::sol_types::SolType>::RustType,
                >,
            );
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<decodeFunctionSigsAndPermissionsReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: decodeFunctionSigsAndPermissionsReturn) -> Self {
                    (value.functionSigs, value.permissions)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for decodeFunctionSigsAndPermissionsReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {
                        functionSigs: tuple.0,
                        permissions: tuple.1,
                    }
                }
            }
        }
        impl decodeFunctionSigsAndPermissionsReturn {
            fn _tokenize(
                &self,
            ) -> <decodeFunctionSigsAndPermissionsCall as alloy_sol_types::SolCall>::ReturnToken<
                '_,
            > {
                (
                    <alloy::sol_types::sol_data::Array<
                        alloy::sol_types::sol_data::FixedBytes<4>,
                    > as alloy_sol_types::SolType>::tokenize(&self.functionSigs),
                    <alloy::sol_types::sol_data::Array<
                        GranularPermission,
                    > as alloy_sol_types::SolType>::tokenize(&self.permissions),
                )
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for decodeFunctionSigsAndPermissionsCall {
            type Parameters<'a> = (
                alloy::sol_types::sol_data::FixedBytes<32>,
                alloy::sol_types::sol_data::Uint<256>,
            );
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = decodeFunctionSigsAndPermissionsReturn;
            type ReturnTuple<'a> = (
                alloy::sol_types::sol_data::Array<
                    alloy::sol_types::sol_data::FixedBytes<4>,
                >,
                alloy::sol_types::sol_data::Array<GranularPermission>,
            );
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "decodeFunctionSigsAndPermissions(bytes32,uint256)";
            const SELECTOR: [u8; 4] = [96u8, 151u8, 108u8, 75u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                (
                    <alloy::sol_types::sol_data::FixedBytes<
                        32,
                    > as alloy_sol_types::SolType>::tokenize(&self.encoded),
                    <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::SolType>::tokenize(&self.length),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                decodeFunctionSigsAndPermissionsReturn::_tokenize(ret)
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(Into::into)
            }
            #[inline]
            fn abi_decode_returns_validate(
                data: &[u8],
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Into::into)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `encodeFunctionSigsAndPermissions(bytes4[],uint8[])` and selector `0x56f55117`.
```solidity
function encodeFunctionSigsAndPermissions(bytes4[] memory functionSigs, GranularPermission[] memory permissions) external pure returns (bytes32 encoded, uint256 length);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct encodeFunctionSigsAndPermissionsCall {
        #[allow(missing_docs)]
        pub functionSigs: alloy::sol_types::private::Vec<
            alloy::sol_types::private::FixedBytes<4>,
        >,
        #[allow(missing_docs)]
        pub permissions: alloy::sol_types::private::Vec<
            <GranularPermission as alloy::sol_types::SolType>::RustType,
        >,
    }
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`encodeFunctionSigsAndPermissions(bytes4[],uint8[])`](encodeFunctionSigsAndPermissionsCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct encodeFunctionSigsAndPermissionsReturn {
        #[allow(missing_docs)]
        pub encoded: alloy::sol_types::private::FixedBytes<32>,
        #[allow(missing_docs)]
        pub length: alloy::sol_types::private::primitives::aliases::U256,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (
                alloy::sol_types::sol_data::Array<
                    alloy::sol_types::sol_data::FixedBytes<4>,
                >,
                alloy::sol_types::sol_data::Array<GranularPermission>,
            );
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                alloy::sol_types::private::Vec<alloy::sol_types::private::FixedBytes<4>>,
                alloy::sol_types::private::Vec<
                    <GranularPermission as alloy::sol_types::SolType>::RustType,
                >,
            );
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<encodeFunctionSigsAndPermissionsCall>
            for UnderlyingRustTuple<'_> {
                fn from(value: encodeFunctionSigsAndPermissionsCall) -> Self {
                    (value.functionSigs, value.permissions)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for encodeFunctionSigsAndPermissionsCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {
                        functionSigs: tuple.0,
                        permissions: tuple.1,
                    }
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (
                alloy::sol_types::sol_data::FixedBytes<32>,
                alloy::sol_types::sol_data::Uint<256>,
            );
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                alloy::sol_types::private::FixedBytes<32>,
                alloy::sol_types::private::primitives::aliases::U256,
            );
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<encodeFunctionSigsAndPermissionsReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: encodeFunctionSigsAndPermissionsReturn) -> Self {
                    (value.encoded, value.length)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for encodeFunctionSigsAndPermissionsReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {
                        encoded: tuple.0,
                        length: tuple.1,
                    }
                }
            }
        }
        impl encodeFunctionSigsAndPermissionsReturn {
            fn _tokenize(
                &self,
            ) -> <encodeFunctionSigsAndPermissionsCall as alloy_sol_types::SolCall>::ReturnToken<
                '_,
            > {
                (
                    <alloy::sol_types::sol_data::FixedBytes<
                        32,
                    > as alloy_sol_types::SolType>::tokenize(&self.encoded),
                    <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::SolType>::tokenize(&self.length),
                )
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for encodeFunctionSigsAndPermissionsCall {
            type Parameters<'a> = (
                alloy::sol_types::sol_data::Array<
                    alloy::sol_types::sol_data::FixedBytes<4>,
                >,
                alloy::sol_types::sol_data::Array<GranularPermission>,
            );
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = encodeFunctionSigsAndPermissionsReturn;
            type ReturnTuple<'a> = (
                alloy::sol_types::sol_data::FixedBytes<32>,
                alloy::sol_types::sol_data::Uint<256>,
            );
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "encodeFunctionSigsAndPermissions(bytes4[],uint8[])";
            const SELECTOR: [u8; 4] = [86u8, 245u8, 81u8, 23u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                (
                    <alloy::sol_types::sol_data::Array<
                        alloy::sol_types::sol_data::FixedBytes<4>,
                    > as alloy_sol_types::SolType>::tokenize(&self.functionSigs),
                    <alloy::sol_types::sol_data::Array<
                        GranularPermission,
                    > as alloy_sol_types::SolType>::tokenize(&self.permissions),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                encodeFunctionSigsAndPermissionsReturn::_tokenize(ret)
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(Into::into)
            }
            #[inline]
            fn abi_decode_returns_validate(
                data: &[u8],
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Into::into)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `execTransactionFromModule(address,uint256,bytes,uint8)` and selector `0x468721a7`.
```solidity
function execTransactionFromModule(address to, uint256 value, bytes memory data, Enum.Operation operation) external returns (bool success);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct execTransactionFromModuleCall {
        #[allow(missing_docs)]
        pub to: alloy::sol_types::private::Address,
        #[allow(missing_docs)]
        pub value: alloy::sol_types::private::primitives::aliases::U256,
        #[allow(missing_docs)]
        pub data: alloy::sol_types::private::Bytes,
        #[allow(missing_docs)]
        pub operation: <Enum::Operation as alloy::sol_types::SolType>::RustType,
    }
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`execTransactionFromModule(address,uint256,bytes,uint8)`](execTransactionFromModuleCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct execTransactionFromModuleReturn {
        #[allow(missing_docs)]
        pub success: bool,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (
                alloy::sol_types::sol_data::Address,
                alloy::sol_types::sol_data::Uint<256>,
                alloy::sol_types::sol_data::Bytes,
                Enum::Operation,
            );
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                alloy::sol_types::private::Address,
                alloy::sol_types::private::primitives::aliases::U256,
                alloy::sol_types::private::Bytes,
                <Enum::Operation as alloy::sol_types::SolType>::RustType,
            );
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<execTransactionFromModuleCall>
            for UnderlyingRustTuple<'_> {
                fn from(value: execTransactionFromModuleCall) -> Self {
                    (value.to, value.value, value.data, value.operation)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for execTransactionFromModuleCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {
                        to: tuple.0,
                        value: tuple.1,
                        data: tuple.2,
                        operation: tuple.3,
                    }
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (alloy::sol_types::sol_data::Bool,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (bool,);
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<execTransactionFromModuleReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: execTransactionFromModuleReturn) -> Self {
                    (value.success,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for execTransactionFromModuleReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { success: tuple.0 }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for execTransactionFromModuleCall {
            type Parameters<'a> = (
                alloy::sol_types::sol_data::Address,
                alloy::sol_types::sol_data::Uint<256>,
                alloy::sol_types::sol_data::Bytes,
                Enum::Operation,
            );
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = bool;
            type ReturnTuple<'a> = (alloy::sol_types::sol_data::Bool,);
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "execTransactionFromModule(address,uint256,bytes,uint8)";
            const SELECTOR: [u8; 4] = [70u8, 135u8, 33u8, 167u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                (
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        &self.to,
                    ),
                    <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::SolType>::tokenize(&self.value),
                    <alloy::sol_types::sol_data::Bytes as alloy_sol_types::SolType>::tokenize(
                        &self.data,
                    ),
                    <Enum::Operation as alloy_sol_types::SolType>::tokenize(
                        &self.operation,
                    ),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                (
                    <alloy::sol_types::sol_data::Bool as alloy_sol_types::SolType>::tokenize(
                        ret,
                    ),
                )
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(|r| {
                        let r: execTransactionFromModuleReturn = r.into();
                        r.success
                    })
            }
            #[inline]
            fn abi_decode_returns_validate(
                data: &[u8],
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(|r| {
                        let r: execTransactionFromModuleReturn = r.into();
                        r.success
                    })
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `execTransactionFromModuleReturnData(address,uint256,bytes,uint8)` and selector `0x5229073f`.
```solidity
function execTransactionFromModuleReturnData(address to, uint256 value, bytes memory data, Enum.Operation operation) external returns (bool, bytes memory);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct execTransactionFromModuleReturnDataCall {
        #[allow(missing_docs)]
        pub to: alloy::sol_types::private::Address,
        #[allow(missing_docs)]
        pub value: alloy::sol_types::private::primitives::aliases::U256,
        #[allow(missing_docs)]
        pub data: alloy::sol_types::private::Bytes,
        #[allow(missing_docs)]
        pub operation: <Enum::Operation as alloy::sol_types::SolType>::RustType,
    }
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`execTransactionFromModuleReturnData(address,uint256,bytes,uint8)`](execTransactionFromModuleReturnDataCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct execTransactionFromModuleReturnDataReturn {
        #[allow(missing_docs)]
        pub _0: bool,
        #[allow(missing_docs)]
        pub _1: alloy::sol_types::private::Bytes,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (
                alloy::sol_types::sol_data::Address,
                alloy::sol_types::sol_data::Uint<256>,
                alloy::sol_types::sol_data::Bytes,
                Enum::Operation,
            );
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                alloy::sol_types::private::Address,
                alloy::sol_types::private::primitives::aliases::U256,
                alloy::sol_types::private::Bytes,
                <Enum::Operation as alloy::sol_types::SolType>::RustType,
            );
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<execTransactionFromModuleReturnDataCall>
            for UnderlyingRustTuple<'_> {
                fn from(value: execTransactionFromModuleReturnDataCall) -> Self {
                    (value.to, value.value, value.data, value.operation)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for execTransactionFromModuleReturnDataCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {
                        to: tuple.0,
                        value: tuple.1,
                        data: tuple.2,
                        operation: tuple.3,
                    }
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (
                alloy::sol_types::sol_data::Bool,
                alloy::sol_types::sol_data::Bytes,
            );
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (bool, alloy::sol_types::private::Bytes);
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<execTransactionFromModuleReturnDataReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: execTransactionFromModuleReturnDataReturn) -> Self {
                    (value._0, value._1)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for execTransactionFromModuleReturnDataReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { _0: tuple.0, _1: tuple.1 }
                }
            }
        }
        impl execTransactionFromModuleReturnDataReturn {
            fn _tokenize(
                &self,
            ) -> <execTransactionFromModuleReturnDataCall as alloy_sol_types::SolCall>::ReturnToken<
                '_,
            > {
                (
                    <alloy::sol_types::sol_data::Bool as alloy_sol_types::SolType>::tokenize(
                        &self._0,
                    ),
                    <alloy::sol_types::sol_data::Bytes as alloy_sol_types::SolType>::tokenize(
                        &self._1,
                    ),
                )
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for execTransactionFromModuleReturnDataCall {
            type Parameters<'a> = (
                alloy::sol_types::sol_data::Address,
                alloy::sol_types::sol_data::Uint<256>,
                alloy::sol_types::sol_data::Bytes,
                Enum::Operation,
            );
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = execTransactionFromModuleReturnDataReturn;
            type ReturnTuple<'a> = (
                alloy::sol_types::sol_data::Bool,
                alloy::sol_types::sol_data::Bytes,
            );
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "execTransactionFromModuleReturnData(address,uint256,bytes,uint8)";
            const SELECTOR: [u8; 4] = [82u8, 41u8, 7u8, 63u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                (
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        &self.to,
                    ),
                    <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::SolType>::tokenize(&self.value),
                    <alloy::sol_types::sol_data::Bytes as alloy_sol_types::SolType>::tokenize(
                        &self.data,
                    ),
                    <Enum::Operation as alloy_sol_types::SolType>::tokenize(
                        &self.operation,
                    ),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                execTransactionFromModuleReturnDataReturn::_tokenize(ret)
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(Into::into)
            }
            #[inline]
            fn abi_decode_returns_validate(
                data: &[u8],
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Into::into)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `getGranularPermissions(bytes32,bytes32)` and selector `0xdc446a4a`.
```solidity
function getGranularPermissions(bytes32 capabilityKey, bytes32 pairId) external view returns (GranularPermission);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct getGranularPermissionsCall {
        #[allow(missing_docs)]
        pub capabilityKey: alloy::sol_types::private::FixedBytes<32>,
        #[allow(missing_docs)]
        pub pairId: alloy::sol_types::private::FixedBytes<32>,
    }
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`getGranularPermissions(bytes32,bytes32)`](getGranularPermissionsCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct getGranularPermissionsReturn {
        #[allow(missing_docs)]
        pub _0: <GranularPermission as alloy::sol_types::SolType>::RustType,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (
                alloy::sol_types::sol_data::FixedBytes<32>,
                alloy::sol_types::sol_data::FixedBytes<32>,
            );
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                alloy::sol_types::private::FixedBytes<32>,
                alloy::sol_types::private::FixedBytes<32>,
            );
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<getGranularPermissionsCall>
            for UnderlyingRustTuple<'_> {
                fn from(value: getGranularPermissionsCall) -> Self {
                    (value.capabilityKey, value.pairId)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for getGranularPermissionsCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {
                        capabilityKey: tuple.0,
                        pairId: tuple.1,
                    }
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (GranularPermission,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                <GranularPermission as alloy::sol_types::SolType>::RustType,
            );
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<getGranularPermissionsReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: getGranularPermissionsReturn) -> Self {
                    (value._0,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for getGranularPermissionsReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { _0: tuple.0 }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for getGranularPermissionsCall {
            type Parameters<'a> = (
                alloy::sol_types::sol_data::FixedBytes<32>,
                alloy::sol_types::sol_data::FixedBytes<32>,
            );
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = <GranularPermission as alloy::sol_types::SolType>::RustType;
            type ReturnTuple<'a> = (GranularPermission,);
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "getGranularPermissions(bytes32,bytes32)";
            const SELECTOR: [u8; 4] = [220u8, 68u8, 106u8, 74u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                (
                    <alloy::sol_types::sol_data::FixedBytes<
                        32,
                    > as alloy_sol_types::SolType>::tokenize(&self.capabilityKey),
                    <alloy::sol_types::sol_data::FixedBytes<
                        32,
                    > as alloy_sol_types::SolType>::tokenize(&self.pairId),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                (<GranularPermission as alloy_sol_types::SolType>::tokenize(ret),)
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(|r| {
                        let r: getGranularPermissionsReturn = r.into();
                        r._0
                    })
            }
            #[inline]
            fn abi_decode_returns_validate(
                data: &[u8],
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(|r| {
                        let r: getGranularPermissionsReturn = r.into();
                        r._0
                    })
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `getTargets()` and selector `0x63fe3b56`.
```solidity
function getTargets() external view returns (Target[] memory);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct getTargetsCall;
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`getTargets()`](getTargetsCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct getTargetsReturn {
        #[allow(missing_docs)]
        pub _0: alloy::sol_types::private::Vec<
            <Target as alloy::sol_types::SolType>::RustType,
        >,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = ();
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = ();
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<getTargetsCall> for UnderlyingRustTuple<'_> {
                fn from(value: getTargetsCall) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for getTargetsCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (alloy::sol_types::sol_data::Array<Target>,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                alloy::sol_types::private::Vec<
                    <Target as alloy::sol_types::SolType>::RustType,
                >,
            );
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<getTargetsReturn> for UnderlyingRustTuple<'_> {
                fn from(value: getTargetsReturn) -> Self {
                    (value._0,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for getTargetsReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { _0: tuple.0 }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for getTargetsCall {
            type Parameters<'a> = ();
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = alloy::sol_types::private::Vec<
                <Target as alloy::sol_types::SolType>::RustType,
            >;
            type ReturnTuple<'a> = (alloy::sol_types::sol_data::Array<Target>,);
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "getTargets()";
            const SELECTOR: [u8; 4] = [99u8, 254u8, 59u8, 86u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                (
                    <alloy::sol_types::sol_data::Array<
                        Target,
                    > as alloy_sol_types::SolType>::tokenize(ret),
                )
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(|r| {
                        let r: getTargetsReturn = r.into();
                        r._0
                    })
            }
            #[inline]
            fn abi_decode_returns_validate(
                data: &[u8],
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(|r| {
                        let r: getTargetsReturn = r.into();
                        r._0
                    })
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `includeNode(uint256)` and selector `0xb5736962`.
```solidity
function includeNode(Target nodeDefaultTarget) external;
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct includeNodeCall {
        #[allow(missing_docs)]
        pub nodeDefaultTarget: <Target as alloy::sol_types::SolType>::RustType,
    }
    ///Container type for the return parameters of the [`includeNode(uint256)`](includeNodeCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct includeNodeReturn {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (Target,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                <Target as alloy::sol_types::SolType>::RustType,
            );
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<includeNodeCall> for UnderlyingRustTuple<'_> {
                fn from(value: includeNodeCall) -> Self {
                    (value.nodeDefaultTarget,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for includeNodeCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { nodeDefaultTarget: tuple.0 }
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = ();
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = ();
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<includeNodeReturn> for UnderlyingRustTuple<'_> {
                fn from(value: includeNodeReturn) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for includeNodeReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        impl includeNodeReturn {
            fn _tokenize(
                &self,
            ) -> <includeNodeCall as alloy_sol_types::SolCall>::ReturnToken<'_> {
                ()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for includeNodeCall {
            type Parameters<'a> = (Target,);
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = includeNodeReturn;
            type ReturnTuple<'a> = ();
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "includeNode(uint256)";
            const SELECTOR: [u8; 4] = [181u8, 115u8, 105u8, 98u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                (
                    <Target as alloy_sol_types::SolType>::tokenize(
                        &self.nodeDefaultTarget,
                    ),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                includeNodeReturn::_tokenize(ret)
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(Into::into)
            }
            #[inline]
            fn abi_decode_returns_validate(
                data: &[u8],
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Into::into)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `initialize(bytes)` and selector `0x439fab91`.
```solidity
function initialize(bytes memory initParams) external;
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct initializeCall {
        #[allow(missing_docs)]
        pub initParams: alloy::sol_types::private::Bytes,
    }
    ///Container type for the return parameters of the [`initialize(bytes)`](initializeCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct initializeReturn {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (alloy::sol_types::sol_data::Bytes,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (alloy::sol_types::private::Bytes,);
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<initializeCall> for UnderlyingRustTuple<'_> {
                fn from(value: initializeCall) -> Self {
                    (value.initParams,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for initializeCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { initParams: tuple.0 }
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = ();
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = ();
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<initializeReturn> for UnderlyingRustTuple<'_> {
                fn from(value: initializeReturn) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for initializeReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        impl initializeReturn {
            fn _tokenize(
                &self,
            ) -> <initializeCall as alloy_sol_types::SolCall>::ReturnToken<'_> {
                ()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for initializeCall {
            type Parameters<'a> = (alloy::sol_types::sol_data::Bytes,);
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = initializeReturn;
            type ReturnTuple<'a> = ();
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "initialize(bytes)";
            const SELECTOR: [u8; 4] = [67u8, 159u8, 171u8, 145u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                (
                    <alloy::sol_types::sol_data::Bytes as alloy_sol_types::SolType>::tokenize(
                        &self.initParams,
                    ),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                initializeReturn::_tokenize(ret)
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(Into::into)
            }
            #[inline]
            fn abi_decode_returns_validate(
                data: &[u8],
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Into::into)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `isHoprNodeManagementModule()` and selector `0x4a1ba408`.
```solidity
function isHoprNodeManagementModule() external view returns (bool);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct isHoprNodeManagementModuleCall;
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`isHoprNodeManagementModule()`](isHoprNodeManagementModuleCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct isHoprNodeManagementModuleReturn {
        #[allow(missing_docs)]
        pub _0: bool,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = ();
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = ();
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<isHoprNodeManagementModuleCall>
            for UnderlyingRustTuple<'_> {
                fn from(value: isHoprNodeManagementModuleCall) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for isHoprNodeManagementModuleCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (alloy::sol_types::sol_data::Bool,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (bool,);
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<isHoprNodeManagementModuleReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: isHoprNodeManagementModuleReturn) -> Self {
                    (value._0,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for isHoprNodeManagementModuleReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { _0: tuple.0 }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for isHoprNodeManagementModuleCall {
            type Parameters<'a> = ();
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = bool;
            type ReturnTuple<'a> = (alloy::sol_types::sol_data::Bool,);
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "isHoprNodeManagementModule()";
            const SELECTOR: [u8; 4] = [74u8, 27u8, 164u8, 8u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                (
                    <alloy::sol_types::sol_data::Bool as alloy_sol_types::SolType>::tokenize(
                        ret,
                    ),
                )
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(|r| {
                        let r: isHoprNodeManagementModuleReturn = r.into();
                        r._0
                    })
            }
            #[inline]
            fn abi_decode_returns_validate(
                data: &[u8],
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(|r| {
                        let r: isHoprNodeManagementModuleReturn = r.into();
                        r._0
                    })
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `isNode(address)` and selector `0x01750152`.
```solidity
function isNode(address nodeAddress) external view returns (bool);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct isNodeCall {
        #[allow(missing_docs)]
        pub nodeAddress: alloy::sol_types::private::Address,
    }
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`isNode(address)`](isNodeCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct isNodeReturn {
        #[allow(missing_docs)]
        pub _0: bool,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (alloy::sol_types::sol_data::Address,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (alloy::sol_types::private::Address,);
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<isNodeCall> for UnderlyingRustTuple<'_> {
                fn from(value: isNodeCall) -> Self {
                    (value.nodeAddress,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for isNodeCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { nodeAddress: tuple.0 }
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (alloy::sol_types::sol_data::Bool,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (bool,);
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<isNodeReturn> for UnderlyingRustTuple<'_> {
                fn from(value: isNodeReturn) -> Self {
                    (value._0,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for isNodeReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { _0: tuple.0 }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for isNodeCall {
            type Parameters<'a> = (alloy::sol_types::sol_data::Address,);
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = bool;
            type ReturnTuple<'a> = (alloy::sol_types::sol_data::Bool,);
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "isNode(address)";
            const SELECTOR: [u8; 4] = [1u8, 117u8, 1u8, 82u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                (
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        &self.nodeAddress,
                    ),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                (
                    <alloy::sol_types::sol_data::Bool as alloy_sol_types::SolType>::tokenize(
                        ret,
                    ),
                )
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(|r| {
                        let r: isNodeReturn = r.into();
                        r._0
                    })
            }
            #[inline]
            fn abi_decode_returns_validate(
                data: &[u8],
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(|r| {
                        let r: isNodeReturn = r.into();
                        r._0
                    })
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `multisend()` and selector `0x294402cc`.
```solidity
function multisend() external view returns (address);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct multisendCall;
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`multisend()`](multisendCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct multisendReturn {
        #[allow(missing_docs)]
        pub _0: alloy::sol_types::private::Address,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = ();
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = ();
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<multisendCall> for UnderlyingRustTuple<'_> {
                fn from(value: multisendCall) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for multisendCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (alloy::sol_types::sol_data::Address,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (alloy::sol_types::private::Address,);
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<multisendReturn> for UnderlyingRustTuple<'_> {
                fn from(value: multisendReturn) -> Self {
                    (value._0,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for multisendReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { _0: tuple.0 }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for multisendCall {
            type Parameters<'a> = ();
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = alloy::sol_types::private::Address;
            type ReturnTuple<'a> = (alloy::sol_types::sol_data::Address,);
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "multisend()";
            const SELECTOR: [u8; 4] = [41u8, 68u8, 2u8, 204u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                (
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        ret,
                    ),
                )
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(|r| {
                        let r: multisendReturn = r.into();
                        r._0
                    })
            }
            #[inline]
            fn abi_decode_returns_validate(
                data: &[u8],
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(|r| {
                        let r: multisendReturn = r.into();
                        r._0
                    })
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `owner()` and selector `0x8da5cb5b`.
```solidity
function owner() external view returns (address);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct ownerCall;
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`owner()`](ownerCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct ownerReturn {
        #[allow(missing_docs)]
        pub _0: alloy::sol_types::private::Address,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = ();
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = ();
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<ownerCall> for UnderlyingRustTuple<'_> {
                fn from(value: ownerCall) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for ownerCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (alloy::sol_types::sol_data::Address,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (alloy::sol_types::private::Address,);
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<ownerReturn> for UnderlyingRustTuple<'_> {
                fn from(value: ownerReturn) -> Self {
                    (value._0,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for ownerReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { _0: tuple.0 }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for ownerCall {
            type Parameters<'a> = ();
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = alloy::sol_types::private::Address;
            type ReturnTuple<'a> = (alloy::sol_types::sol_data::Address,);
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "owner()";
            const SELECTOR: [u8; 4] = [141u8, 165u8, 203u8, 91u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                (
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        ret,
                    ),
                )
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(|r| {
                        let r: ownerReturn = r.into();
                        r._0
                    })
            }
            #[inline]
            fn abi_decode_returns_validate(
                data: &[u8],
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(|r| {
                        let r: ownerReturn = r.into();
                        r._0
                    })
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `proxiableUUID()` and selector `0x52d1902d`.
```solidity
function proxiableUUID() external view returns (bytes32);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct proxiableUUIDCall;
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`proxiableUUID()`](proxiableUUIDCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct proxiableUUIDReturn {
        #[allow(missing_docs)]
        pub _0: alloy::sol_types::private::FixedBytes<32>,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = ();
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = ();
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<proxiableUUIDCall> for UnderlyingRustTuple<'_> {
                fn from(value: proxiableUUIDCall) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for proxiableUUIDCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (alloy::sol_types::sol_data::FixedBytes<32>,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (alloy::sol_types::private::FixedBytes<32>,);
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<proxiableUUIDReturn> for UnderlyingRustTuple<'_> {
                fn from(value: proxiableUUIDReturn) -> Self {
                    (value._0,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for proxiableUUIDReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { _0: tuple.0 }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for proxiableUUIDCall {
            type Parameters<'a> = ();
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = alloy::sol_types::private::FixedBytes<32>;
            type ReturnTuple<'a> = (alloy::sol_types::sol_data::FixedBytes<32>,);
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "proxiableUUID()";
            const SELECTOR: [u8; 4] = [82u8, 209u8, 144u8, 45u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                (
                    <alloy::sol_types::sol_data::FixedBytes<
                        32,
                    > as alloy_sol_types::SolType>::tokenize(ret),
                )
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(|r| {
                        let r: proxiableUUIDReturn = r.into();
                        r._0
                    })
            }
            #[inline]
            fn abi_decode_returns_validate(
                data: &[u8],
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(|r| {
                        let r: proxiableUUIDReturn = r.into();
                        r._0
                    })
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `removeNode(address)` and selector `0xb2b99ec9`.
```solidity
function removeNode(address nodeAddress) external;
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct removeNodeCall {
        #[allow(missing_docs)]
        pub nodeAddress: alloy::sol_types::private::Address,
    }
    ///Container type for the return parameters of the [`removeNode(address)`](removeNodeCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct removeNodeReturn {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (alloy::sol_types::sol_data::Address,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (alloy::sol_types::private::Address,);
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<removeNodeCall> for UnderlyingRustTuple<'_> {
                fn from(value: removeNodeCall) -> Self {
                    (value.nodeAddress,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for removeNodeCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { nodeAddress: tuple.0 }
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = ();
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = ();
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<removeNodeReturn> for UnderlyingRustTuple<'_> {
                fn from(value: removeNodeReturn) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for removeNodeReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        impl removeNodeReturn {
            fn _tokenize(
                &self,
            ) -> <removeNodeCall as alloy_sol_types::SolCall>::ReturnToken<'_> {
                ()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for removeNodeCall {
            type Parameters<'a> = (alloy::sol_types::sol_data::Address,);
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = removeNodeReturn;
            type ReturnTuple<'a> = ();
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "removeNode(address)";
            const SELECTOR: [u8; 4] = [178u8, 185u8, 158u8, 201u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                (
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        &self.nodeAddress,
                    ),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                removeNodeReturn::_tokenize(ret)
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(Into::into)
            }
            #[inline]
            fn abi_decode_returns_validate(
                data: &[u8],
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Into::into)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `renounceOwnership()` and selector `0x715018a6`.
```solidity
function renounceOwnership() external;
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct renounceOwnershipCall;
    ///Container type for the return parameters of the [`renounceOwnership()`](renounceOwnershipCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct renounceOwnershipReturn {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = ();
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = ();
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<renounceOwnershipCall>
            for UnderlyingRustTuple<'_> {
                fn from(value: renounceOwnershipCall) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for renounceOwnershipCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = ();
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = ();
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<renounceOwnershipReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: renounceOwnershipReturn) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for renounceOwnershipReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        impl renounceOwnershipReturn {
            fn _tokenize(
                &self,
            ) -> <renounceOwnershipCall as alloy_sol_types::SolCall>::ReturnToken<'_> {
                ()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for renounceOwnershipCall {
            type Parameters<'a> = ();
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = renounceOwnershipReturn;
            type ReturnTuple<'a> = ();
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "renounceOwnership()";
            const SELECTOR: [u8; 4] = [113u8, 80u8, 24u8, 166u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                ()
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                renounceOwnershipReturn::_tokenize(ret)
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(Into::into)
            }
            #[inline]
            fn abi_decode_returns_validate(
                data: &[u8],
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Into::into)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `revokeTarget(address)` and selector `0x3401cde8`.
```solidity
function revokeTarget(address targetAddress) external;
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct revokeTargetCall {
        #[allow(missing_docs)]
        pub targetAddress: alloy::sol_types::private::Address,
    }
    ///Container type for the return parameters of the [`revokeTarget(address)`](revokeTargetCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct revokeTargetReturn {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (alloy::sol_types::sol_data::Address,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (alloy::sol_types::private::Address,);
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<revokeTargetCall> for UnderlyingRustTuple<'_> {
                fn from(value: revokeTargetCall) -> Self {
                    (value.targetAddress,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for revokeTargetCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { targetAddress: tuple.0 }
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = ();
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = ();
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<revokeTargetReturn> for UnderlyingRustTuple<'_> {
                fn from(value: revokeTargetReturn) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for revokeTargetReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        impl revokeTargetReturn {
            fn _tokenize(
                &self,
            ) -> <revokeTargetCall as alloy_sol_types::SolCall>::ReturnToken<'_> {
                ()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for revokeTargetCall {
            type Parameters<'a> = (alloy::sol_types::sol_data::Address,);
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = revokeTargetReturn;
            type ReturnTuple<'a> = ();
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "revokeTarget(address)";
            const SELECTOR: [u8; 4] = [52u8, 1u8, 205u8, 232u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                (
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        &self.targetAddress,
                    ),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                revokeTargetReturn::_tokenize(ret)
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(Into::into)
            }
            #[inline]
            fn abi_decode_returns_validate(
                data: &[u8],
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Into::into)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `scopeChannelsCapabilities(address,bytes32,bytes32)` and selector `0xfa19501d`.
```solidity
function scopeChannelsCapabilities(address targetAddress, bytes32 channelId, bytes32 encodedSigsPermissions) external;
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct scopeChannelsCapabilitiesCall {
        #[allow(missing_docs)]
        pub targetAddress: alloy::sol_types::private::Address,
        #[allow(missing_docs)]
        pub channelId: alloy::sol_types::private::FixedBytes<32>,
        #[allow(missing_docs)]
        pub encodedSigsPermissions: alloy::sol_types::private::FixedBytes<32>,
    }
    ///Container type for the return parameters of the [`scopeChannelsCapabilities(address,bytes32,bytes32)`](scopeChannelsCapabilitiesCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct scopeChannelsCapabilitiesReturn {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (
                alloy::sol_types::sol_data::Address,
                alloy::sol_types::sol_data::FixedBytes<32>,
                alloy::sol_types::sol_data::FixedBytes<32>,
            );
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                alloy::sol_types::private::Address,
                alloy::sol_types::private::FixedBytes<32>,
                alloy::sol_types::private::FixedBytes<32>,
            );
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<scopeChannelsCapabilitiesCall>
            for UnderlyingRustTuple<'_> {
                fn from(value: scopeChannelsCapabilitiesCall) -> Self {
                    (value.targetAddress, value.channelId, value.encodedSigsPermissions)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for scopeChannelsCapabilitiesCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {
                        targetAddress: tuple.0,
                        channelId: tuple.1,
                        encodedSigsPermissions: tuple.2,
                    }
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = ();
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = ();
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<scopeChannelsCapabilitiesReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: scopeChannelsCapabilitiesReturn) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for scopeChannelsCapabilitiesReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        impl scopeChannelsCapabilitiesReturn {
            fn _tokenize(
                &self,
            ) -> <scopeChannelsCapabilitiesCall as alloy_sol_types::SolCall>::ReturnToken<
                '_,
            > {
                ()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for scopeChannelsCapabilitiesCall {
            type Parameters<'a> = (
                alloy::sol_types::sol_data::Address,
                alloy::sol_types::sol_data::FixedBytes<32>,
                alloy::sol_types::sol_data::FixedBytes<32>,
            );
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = scopeChannelsCapabilitiesReturn;
            type ReturnTuple<'a> = ();
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "scopeChannelsCapabilities(address,bytes32,bytes32)";
            const SELECTOR: [u8; 4] = [250u8, 25u8, 80u8, 29u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                (
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        &self.targetAddress,
                    ),
                    <alloy::sol_types::sol_data::FixedBytes<
                        32,
                    > as alloy_sol_types::SolType>::tokenize(&self.channelId),
                    <alloy::sol_types::sol_data::FixedBytes<
                        32,
                    > as alloy_sol_types::SolType>::tokenize(
                        &self.encodedSigsPermissions,
                    ),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                scopeChannelsCapabilitiesReturn::_tokenize(ret)
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(Into::into)
            }
            #[inline]
            fn abi_decode_returns_validate(
                data: &[u8],
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Into::into)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `scopeSendCapability(address,address,uint8)` and selector `0xc68c3a83`.
```solidity
function scopeSendCapability(address nodeAddress, address beneficiary, GranularPermission permission) external;
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct scopeSendCapabilityCall {
        #[allow(missing_docs)]
        pub nodeAddress: alloy::sol_types::private::Address,
        #[allow(missing_docs)]
        pub beneficiary: alloy::sol_types::private::Address,
        #[allow(missing_docs)]
        pub permission: <GranularPermission as alloy::sol_types::SolType>::RustType,
    }
    ///Container type for the return parameters of the [`scopeSendCapability(address,address,uint8)`](scopeSendCapabilityCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct scopeSendCapabilityReturn {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (
                alloy::sol_types::sol_data::Address,
                alloy::sol_types::sol_data::Address,
                GranularPermission,
            );
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                alloy::sol_types::private::Address,
                alloy::sol_types::private::Address,
                <GranularPermission as alloy::sol_types::SolType>::RustType,
            );
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<scopeSendCapabilityCall>
            for UnderlyingRustTuple<'_> {
                fn from(value: scopeSendCapabilityCall) -> Self {
                    (value.nodeAddress, value.beneficiary, value.permission)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for scopeSendCapabilityCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {
                        nodeAddress: tuple.0,
                        beneficiary: tuple.1,
                        permission: tuple.2,
                    }
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = ();
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = ();
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<scopeSendCapabilityReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: scopeSendCapabilityReturn) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for scopeSendCapabilityReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        impl scopeSendCapabilityReturn {
            fn _tokenize(
                &self,
            ) -> <scopeSendCapabilityCall as alloy_sol_types::SolCall>::ReturnToken<'_> {
                ()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for scopeSendCapabilityCall {
            type Parameters<'a> = (
                alloy::sol_types::sol_data::Address,
                alloy::sol_types::sol_data::Address,
                GranularPermission,
            );
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = scopeSendCapabilityReturn;
            type ReturnTuple<'a> = ();
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "scopeSendCapability(address,address,uint8)";
            const SELECTOR: [u8; 4] = [198u8, 140u8, 58u8, 131u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                (
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        &self.nodeAddress,
                    ),
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        &self.beneficiary,
                    ),
                    <GranularPermission as alloy_sol_types::SolType>::tokenize(
                        &self.permission,
                    ),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                scopeSendCapabilityReturn::_tokenize(ret)
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(Into::into)
            }
            #[inline]
            fn abi_decode_returns_validate(
                data: &[u8],
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Into::into)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `scopeTargetChannels(uint256)` and selector `0x739c4b08`.
```solidity
function scopeTargetChannels(Target defaultTarget) external;
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct scopeTargetChannelsCall {
        #[allow(missing_docs)]
        pub defaultTarget: <Target as alloy::sol_types::SolType>::RustType,
    }
    ///Container type for the return parameters of the [`scopeTargetChannels(uint256)`](scopeTargetChannelsCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct scopeTargetChannelsReturn {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (Target,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                <Target as alloy::sol_types::SolType>::RustType,
            );
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<scopeTargetChannelsCall>
            for UnderlyingRustTuple<'_> {
                fn from(value: scopeTargetChannelsCall) -> Self {
                    (value.defaultTarget,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for scopeTargetChannelsCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { defaultTarget: tuple.0 }
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = ();
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = ();
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<scopeTargetChannelsReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: scopeTargetChannelsReturn) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for scopeTargetChannelsReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        impl scopeTargetChannelsReturn {
            fn _tokenize(
                &self,
            ) -> <scopeTargetChannelsCall as alloy_sol_types::SolCall>::ReturnToken<'_> {
                ()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for scopeTargetChannelsCall {
            type Parameters<'a> = (Target,);
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = scopeTargetChannelsReturn;
            type ReturnTuple<'a> = ();
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "scopeTargetChannels(uint256)";
            const SELECTOR: [u8; 4] = [115u8, 156u8, 75u8, 8u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                (<Target as alloy_sol_types::SolType>::tokenize(&self.defaultTarget),)
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                scopeTargetChannelsReturn::_tokenize(ret)
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(Into::into)
            }
            #[inline]
            fn abi_decode_returns_validate(
                data: &[u8],
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Into::into)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `scopeTargetSend(uint256)` and selector `0xdc06109d`.
```solidity
function scopeTargetSend(Target defaultTarget) external;
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct scopeTargetSendCall {
        #[allow(missing_docs)]
        pub defaultTarget: <Target as alloy::sol_types::SolType>::RustType,
    }
    ///Container type for the return parameters of the [`scopeTargetSend(uint256)`](scopeTargetSendCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct scopeTargetSendReturn {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (Target,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                <Target as alloy::sol_types::SolType>::RustType,
            );
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<scopeTargetSendCall> for UnderlyingRustTuple<'_> {
                fn from(value: scopeTargetSendCall) -> Self {
                    (value.defaultTarget,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for scopeTargetSendCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { defaultTarget: tuple.0 }
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = ();
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = ();
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<scopeTargetSendReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: scopeTargetSendReturn) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for scopeTargetSendReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        impl scopeTargetSendReturn {
            fn _tokenize(
                &self,
            ) -> <scopeTargetSendCall as alloy_sol_types::SolCall>::ReturnToken<'_> {
                ()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for scopeTargetSendCall {
            type Parameters<'a> = (Target,);
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = scopeTargetSendReturn;
            type ReturnTuple<'a> = ();
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "scopeTargetSend(uint256)";
            const SELECTOR: [u8; 4] = [220u8, 6u8, 16u8, 157u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                (<Target as alloy_sol_types::SolType>::tokenize(&self.defaultTarget),)
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                scopeTargetSendReturn::_tokenize(ret)
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(Into::into)
            }
            #[inline]
            fn abi_decode_returns_validate(
                data: &[u8],
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Into::into)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `scopeTargetToken(uint256)` and selector `0xa76c9a2f`.
```solidity
function scopeTargetToken(Target defaultTarget) external;
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct scopeTargetTokenCall {
        #[allow(missing_docs)]
        pub defaultTarget: <Target as alloy::sol_types::SolType>::RustType,
    }
    ///Container type for the return parameters of the [`scopeTargetToken(uint256)`](scopeTargetTokenCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct scopeTargetTokenReturn {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (Target,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                <Target as alloy::sol_types::SolType>::RustType,
            );
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<scopeTargetTokenCall>
            for UnderlyingRustTuple<'_> {
                fn from(value: scopeTargetTokenCall) -> Self {
                    (value.defaultTarget,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for scopeTargetTokenCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { defaultTarget: tuple.0 }
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = ();
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = ();
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<scopeTargetTokenReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: scopeTargetTokenReturn) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for scopeTargetTokenReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        impl scopeTargetTokenReturn {
            fn _tokenize(
                &self,
            ) -> <scopeTargetTokenCall as alloy_sol_types::SolCall>::ReturnToken<'_> {
                ()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for scopeTargetTokenCall {
            type Parameters<'a> = (Target,);
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = scopeTargetTokenReturn;
            type ReturnTuple<'a> = ();
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "scopeTargetToken(uint256)";
            const SELECTOR: [u8; 4] = [167u8, 108u8, 154u8, 47u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                (<Target as alloy_sol_types::SolType>::tokenize(&self.defaultTarget),)
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                scopeTargetTokenReturn::_tokenize(ret)
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(Into::into)
            }
            #[inline]
            fn abi_decode_returns_validate(
                data: &[u8],
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Into::into)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `scopeTokenCapabilities(address,address,address,bytes32)` and selector `0xc68605c8`.
```solidity
function scopeTokenCapabilities(address nodeAddress, address targetAddress, address beneficiary, bytes32 encodedSigsPermissions) external;
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct scopeTokenCapabilitiesCall {
        #[allow(missing_docs)]
        pub nodeAddress: alloy::sol_types::private::Address,
        #[allow(missing_docs)]
        pub targetAddress: alloy::sol_types::private::Address,
        #[allow(missing_docs)]
        pub beneficiary: alloy::sol_types::private::Address,
        #[allow(missing_docs)]
        pub encodedSigsPermissions: alloy::sol_types::private::FixedBytes<32>,
    }
    ///Container type for the return parameters of the [`scopeTokenCapabilities(address,address,address,bytes32)`](scopeTokenCapabilitiesCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct scopeTokenCapabilitiesReturn {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (
                alloy::sol_types::sol_data::Address,
                alloy::sol_types::sol_data::Address,
                alloy::sol_types::sol_data::Address,
                alloy::sol_types::sol_data::FixedBytes<32>,
            );
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                alloy::sol_types::private::Address,
                alloy::sol_types::private::Address,
                alloy::sol_types::private::Address,
                alloy::sol_types::private::FixedBytes<32>,
            );
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<scopeTokenCapabilitiesCall>
            for UnderlyingRustTuple<'_> {
                fn from(value: scopeTokenCapabilitiesCall) -> Self {
                    (
                        value.nodeAddress,
                        value.targetAddress,
                        value.beneficiary,
                        value.encodedSigsPermissions,
                    )
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for scopeTokenCapabilitiesCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {
                        nodeAddress: tuple.0,
                        targetAddress: tuple.1,
                        beneficiary: tuple.2,
                        encodedSigsPermissions: tuple.3,
                    }
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = ();
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = ();
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<scopeTokenCapabilitiesReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: scopeTokenCapabilitiesReturn) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for scopeTokenCapabilitiesReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        impl scopeTokenCapabilitiesReturn {
            fn _tokenize(
                &self,
            ) -> <scopeTokenCapabilitiesCall as alloy_sol_types::SolCall>::ReturnToken<
                '_,
            > {
                ()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for scopeTokenCapabilitiesCall {
            type Parameters<'a> = (
                alloy::sol_types::sol_data::Address,
                alloy::sol_types::sol_data::Address,
                alloy::sol_types::sol_data::Address,
                alloy::sol_types::sol_data::FixedBytes<32>,
            );
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = scopeTokenCapabilitiesReturn;
            type ReturnTuple<'a> = ();
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "scopeTokenCapabilities(address,address,address,bytes32)";
            const SELECTOR: [u8; 4] = [198u8, 134u8, 5u8, 200u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                (
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        &self.nodeAddress,
                    ),
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        &self.targetAddress,
                    ),
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        &self.beneficiary,
                    ),
                    <alloy::sol_types::sol_data::FixedBytes<
                        32,
                    > as alloy_sol_types::SolType>::tokenize(
                        &self.encodedSigsPermissions,
                    ),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                scopeTokenCapabilitiesReturn::_tokenize(ret)
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(Into::into)
            }
            #[inline]
            fn abi_decode_returns_validate(
                data: &[u8],
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Into::into)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `setMultisend(address)` and selector `0x8b95eccd`.
```solidity
function setMultisend(address _multisend) external;
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct setMultisendCall {
        #[allow(missing_docs)]
        pub _multisend: alloy::sol_types::private::Address,
    }
    ///Container type for the return parameters of the [`setMultisend(address)`](setMultisendCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct setMultisendReturn {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (alloy::sol_types::sol_data::Address,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (alloy::sol_types::private::Address,);
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<setMultisendCall> for UnderlyingRustTuple<'_> {
                fn from(value: setMultisendCall) -> Self {
                    (value._multisend,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for setMultisendCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { _multisend: tuple.0 }
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = ();
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = ();
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<setMultisendReturn> for UnderlyingRustTuple<'_> {
                fn from(value: setMultisendReturn) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for setMultisendReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        impl setMultisendReturn {
            fn _tokenize(
                &self,
            ) -> <setMultisendCall as alloy_sol_types::SolCall>::ReturnToken<'_> {
                ()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for setMultisendCall {
            type Parameters<'a> = (alloy::sol_types::sol_data::Address,);
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = setMultisendReturn;
            type ReturnTuple<'a> = ();
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "setMultisend(address)";
            const SELECTOR: [u8; 4] = [139u8, 149u8, 236u8, 205u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                (
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        &self._multisend,
                    ),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                setMultisendReturn::_tokenize(ret)
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(Into::into)
            }
            #[inline]
            fn abi_decode_returns_validate(
                data: &[u8],
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Into::into)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `transferOwnership(address)` and selector `0xf2fde38b`.
```solidity
function transferOwnership(address newOwner) external;
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct transferOwnershipCall {
        #[allow(missing_docs)]
        pub newOwner: alloy::sol_types::private::Address,
    }
    ///Container type for the return parameters of the [`transferOwnership(address)`](transferOwnershipCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct transferOwnershipReturn {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (alloy::sol_types::sol_data::Address,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (alloy::sol_types::private::Address,);
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<transferOwnershipCall>
            for UnderlyingRustTuple<'_> {
                fn from(value: transferOwnershipCall) -> Self {
                    (value.newOwner,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for transferOwnershipCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { newOwner: tuple.0 }
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = ();
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = ();
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<transferOwnershipReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: transferOwnershipReturn) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for transferOwnershipReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        impl transferOwnershipReturn {
            fn _tokenize(
                &self,
            ) -> <transferOwnershipCall as alloy_sol_types::SolCall>::ReturnToken<'_> {
                ()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for transferOwnershipCall {
            type Parameters<'a> = (alloy::sol_types::sol_data::Address,);
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = transferOwnershipReturn;
            type ReturnTuple<'a> = ();
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "transferOwnership(address)";
            const SELECTOR: [u8; 4] = [242u8, 253u8, 227u8, 139u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                (
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        &self.newOwner,
                    ),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                transferOwnershipReturn::_tokenize(ret)
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(Into::into)
            }
            #[inline]
            fn abi_decode_returns_validate(
                data: &[u8],
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Into::into)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `tryGetTarget(address)` and selector `0xdf4e6f8a`.
```solidity
function tryGetTarget(address targetAddress) external view returns (bool, Target);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct tryGetTargetCall {
        #[allow(missing_docs)]
        pub targetAddress: alloy::sol_types::private::Address,
    }
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`tryGetTarget(address)`](tryGetTargetCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct tryGetTargetReturn {
        #[allow(missing_docs)]
        pub _0: bool,
        #[allow(missing_docs)]
        pub _1: <Target as alloy::sol_types::SolType>::RustType,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (alloy::sol_types::sol_data::Address,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (alloy::sol_types::private::Address,);
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<tryGetTargetCall> for UnderlyingRustTuple<'_> {
                fn from(value: tryGetTargetCall) -> Self {
                    (value.targetAddress,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for tryGetTargetCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { targetAddress: tuple.0 }
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (alloy::sol_types::sol_data::Bool, Target);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                bool,
                <Target as alloy::sol_types::SolType>::RustType,
            );
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<tryGetTargetReturn> for UnderlyingRustTuple<'_> {
                fn from(value: tryGetTargetReturn) -> Self {
                    (value._0, value._1)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for tryGetTargetReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { _0: tuple.0, _1: tuple.1 }
                }
            }
        }
        impl tryGetTargetReturn {
            fn _tokenize(
                &self,
            ) -> <tryGetTargetCall as alloy_sol_types::SolCall>::ReturnToken<'_> {
                (
                    <alloy::sol_types::sol_data::Bool as alloy_sol_types::SolType>::tokenize(
                        &self._0,
                    ),
                    <Target as alloy_sol_types::SolType>::tokenize(&self._1),
                )
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for tryGetTargetCall {
            type Parameters<'a> = (alloy::sol_types::sol_data::Address,);
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = tryGetTargetReturn;
            type ReturnTuple<'a> = (alloy::sol_types::sol_data::Bool, Target);
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "tryGetTarget(address)";
            const SELECTOR: [u8; 4] = [223u8, 78u8, 111u8, 138u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                (
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        &self.targetAddress,
                    ),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                tryGetTargetReturn::_tokenize(ret)
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(Into::into)
            }
            #[inline]
            fn abi_decode_returns_validate(
                data: &[u8],
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Into::into)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `upgradeTo(address)` and selector `0x3659cfe6`.
```solidity
function upgradeTo(address newImplementation) external;
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct upgradeToCall {
        #[allow(missing_docs)]
        pub newImplementation: alloy::sol_types::private::Address,
    }
    ///Container type for the return parameters of the [`upgradeTo(address)`](upgradeToCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct upgradeToReturn {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (alloy::sol_types::sol_data::Address,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (alloy::sol_types::private::Address,);
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<upgradeToCall> for UnderlyingRustTuple<'_> {
                fn from(value: upgradeToCall) -> Self {
                    (value.newImplementation,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for upgradeToCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { newImplementation: tuple.0 }
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = ();
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = ();
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<upgradeToReturn> for UnderlyingRustTuple<'_> {
                fn from(value: upgradeToReturn) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for upgradeToReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        impl upgradeToReturn {
            fn _tokenize(
                &self,
            ) -> <upgradeToCall as alloy_sol_types::SolCall>::ReturnToken<'_> {
                ()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for upgradeToCall {
            type Parameters<'a> = (alloy::sol_types::sol_data::Address,);
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = upgradeToReturn;
            type ReturnTuple<'a> = ();
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "upgradeTo(address)";
            const SELECTOR: [u8; 4] = [54u8, 89u8, 207u8, 230u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                (
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        &self.newImplementation,
                    ),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                upgradeToReturn::_tokenize(ret)
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(Into::into)
            }
            #[inline]
            fn abi_decode_returns_validate(
                data: &[u8],
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Into::into)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `upgradeToAndCall(address,bytes)` and selector `0x4f1ef286`.
```solidity
function upgradeToAndCall(address newImplementation, bytes memory data) external payable;
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct upgradeToAndCallCall {
        #[allow(missing_docs)]
        pub newImplementation: alloy::sol_types::private::Address,
        #[allow(missing_docs)]
        pub data: alloy::sol_types::private::Bytes,
    }
    ///Container type for the return parameters of the [`upgradeToAndCall(address,bytes)`](upgradeToAndCallCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct upgradeToAndCallReturn {}
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (
                alloy::sol_types::sol_data::Address,
                alloy::sol_types::sol_data::Bytes,
            );
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                alloy::sol_types::private::Address,
                alloy::sol_types::private::Bytes,
            );
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<upgradeToAndCallCall>
            for UnderlyingRustTuple<'_> {
                fn from(value: upgradeToAndCallCall) -> Self {
                    (value.newImplementation, value.data)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for upgradeToAndCallCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {
                        newImplementation: tuple.0,
                        data: tuple.1,
                    }
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = ();
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = ();
            #[cfg(test)]
            #[allow(dead_code, unreachable_patterns)]
            fn _type_assertion(
                _t: alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>,
            ) {
                match _t {
                    alloy_sol_types::private::AssertTypeEq::<
                        <UnderlyingSolTuple as alloy_sol_types::SolType>::RustType,
                    >(_) => {}
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<upgradeToAndCallReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: upgradeToAndCallReturn) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for upgradeToAndCallReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        impl upgradeToAndCallReturn {
            fn _tokenize(
                &self,
            ) -> <upgradeToAndCallCall as alloy_sol_types::SolCall>::ReturnToken<'_> {
                ()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for upgradeToAndCallCall {
            type Parameters<'a> = (
                alloy::sol_types::sol_data::Address,
                alloy::sol_types::sol_data::Bytes,
            );
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = upgradeToAndCallReturn;
            type ReturnTuple<'a> = ();
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "upgradeToAndCall(address,bytes)";
            const SELECTOR: [u8; 4] = [79u8, 30u8, 242u8, 134u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                (
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        &self.newImplementation,
                    ),
                    <alloy::sol_types::sol_data::Bytes as alloy_sol_types::SolType>::tokenize(
                        &self.data,
                    ),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                upgradeToAndCallReturn::_tokenize(ret)
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(Into::into)
            }
            #[inline]
            fn abi_decode_returns_validate(
                data: &[u8],
            ) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence_validate(data)
                    .map(Into::into)
            }
        }
    };
    ///Container for all the [`HoprNodeManagementModule`](self) function calls.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive()]
    pub enum HoprNodeManagementModuleCalls {
        #[allow(missing_docs)]
        addChannelsAndTokenTarget(addChannelsAndTokenTargetCall),
        #[allow(missing_docs)]
        addNode(addNodeCall),
        #[allow(missing_docs)]
        decodeFunctionSigsAndPermissions(decodeFunctionSigsAndPermissionsCall),
        #[allow(missing_docs)]
        encodeFunctionSigsAndPermissions(encodeFunctionSigsAndPermissionsCall),
        #[allow(missing_docs)]
        execTransactionFromModule(execTransactionFromModuleCall),
        #[allow(missing_docs)]
        execTransactionFromModuleReturnData(execTransactionFromModuleReturnDataCall),
        #[allow(missing_docs)]
        getGranularPermissions(getGranularPermissionsCall),
        #[allow(missing_docs)]
        getTargets(getTargetsCall),
        #[allow(missing_docs)]
        includeNode(includeNodeCall),
        #[allow(missing_docs)]
        initialize(initializeCall),
        #[allow(missing_docs)]
        isHoprNodeManagementModule(isHoprNodeManagementModuleCall),
        #[allow(missing_docs)]
        isNode(isNodeCall),
        #[allow(missing_docs)]
        multisend(multisendCall),
        #[allow(missing_docs)]
        owner(ownerCall),
        #[allow(missing_docs)]
        proxiableUUID(proxiableUUIDCall),
        #[allow(missing_docs)]
        removeNode(removeNodeCall),
        #[allow(missing_docs)]
        renounceOwnership(renounceOwnershipCall),
        #[allow(missing_docs)]
        revokeTarget(revokeTargetCall),
        #[allow(missing_docs)]
        scopeChannelsCapabilities(scopeChannelsCapabilitiesCall),
        #[allow(missing_docs)]
        scopeSendCapability(scopeSendCapabilityCall),
        #[allow(missing_docs)]
        scopeTargetChannels(scopeTargetChannelsCall),
        #[allow(missing_docs)]
        scopeTargetSend(scopeTargetSendCall),
        #[allow(missing_docs)]
        scopeTargetToken(scopeTargetTokenCall),
        #[allow(missing_docs)]
        scopeTokenCapabilities(scopeTokenCapabilitiesCall),
        #[allow(missing_docs)]
        setMultisend(setMultisendCall),
        #[allow(missing_docs)]
        transferOwnership(transferOwnershipCall),
        #[allow(missing_docs)]
        tryGetTarget(tryGetTargetCall),
        #[allow(missing_docs)]
        upgradeTo(upgradeToCall),
        #[allow(missing_docs)]
        upgradeToAndCall(upgradeToAndCallCall),
    }
    #[automatically_derived]
    impl HoprNodeManagementModuleCalls {
        /// All the selectors of this enum.
        ///
        /// Note that the selectors might not be in the same order as the variants.
        /// No guarantees are made about the order of the selectors.
        ///
        /// Prefer using `SolInterface` methods instead.
        pub const SELECTORS: &'static [[u8; 4usize]] = &[
            [1u8, 117u8, 1u8, 82u8],
            [41u8, 68u8, 2u8, 204u8],
            [52u8, 1u8, 205u8, 232u8],
            [54u8, 89u8, 207u8, 230u8],
            [67u8, 159u8, 171u8, 145u8],
            [70u8, 135u8, 33u8, 167u8],
            [74u8, 27u8, 164u8, 8u8],
            [79u8, 30u8, 242u8, 134u8],
            [82u8, 41u8, 7u8, 63u8],
            [82u8, 209u8, 144u8, 45u8],
            [86u8, 245u8, 81u8, 23u8],
            [96u8, 151u8, 108u8, 75u8],
            [99u8, 254u8, 59u8, 86u8],
            [113u8, 80u8, 24u8, 166u8],
            [115u8, 156u8, 75u8, 8u8],
            [139u8, 149u8, 236u8, 205u8],
            [141u8, 165u8, 203u8, 91u8],
            [157u8, 149u8, 241u8, 204u8],
            [162u8, 69u8, 15u8, 137u8],
            [167u8, 108u8, 154u8, 47u8],
            [178u8, 185u8, 158u8, 201u8],
            [181u8, 115u8, 105u8, 98u8],
            [198u8, 134u8, 5u8, 200u8],
            [198u8, 140u8, 58u8, 131u8],
            [220u8, 6u8, 16u8, 157u8],
            [220u8, 68u8, 106u8, 74u8],
            [223u8, 78u8, 111u8, 138u8],
            [242u8, 253u8, 227u8, 139u8],
            [250u8, 25u8, 80u8, 29u8],
        ];
    }
    #[automatically_derived]
    impl alloy_sol_types::SolInterface for HoprNodeManagementModuleCalls {
        const NAME: &'static str = "HoprNodeManagementModuleCalls";
        const MIN_DATA_LENGTH: usize = 0usize;
        const COUNT: usize = 29usize;
        #[inline]
        fn selector(&self) -> [u8; 4] {
            match self {
                Self::addChannelsAndTokenTarget(_) => {
                    <addChannelsAndTokenTargetCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::addNode(_) => <addNodeCall as alloy_sol_types::SolCall>::SELECTOR,
                Self::decodeFunctionSigsAndPermissions(_) => {
                    <decodeFunctionSigsAndPermissionsCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::encodeFunctionSigsAndPermissions(_) => {
                    <encodeFunctionSigsAndPermissionsCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::execTransactionFromModule(_) => {
                    <execTransactionFromModuleCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::execTransactionFromModuleReturnData(_) => {
                    <execTransactionFromModuleReturnDataCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::getGranularPermissions(_) => {
                    <getGranularPermissionsCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::getTargets(_) => {
                    <getTargetsCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::includeNode(_) => {
                    <includeNodeCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::initialize(_) => {
                    <initializeCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::isHoprNodeManagementModule(_) => {
                    <isHoprNodeManagementModuleCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::isNode(_) => <isNodeCall as alloy_sol_types::SolCall>::SELECTOR,
                Self::multisend(_) => {
                    <multisendCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::owner(_) => <ownerCall as alloy_sol_types::SolCall>::SELECTOR,
                Self::proxiableUUID(_) => {
                    <proxiableUUIDCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::removeNode(_) => {
                    <removeNodeCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::renounceOwnership(_) => {
                    <renounceOwnershipCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::revokeTarget(_) => {
                    <revokeTargetCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::scopeChannelsCapabilities(_) => {
                    <scopeChannelsCapabilitiesCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::scopeSendCapability(_) => {
                    <scopeSendCapabilityCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::scopeTargetChannels(_) => {
                    <scopeTargetChannelsCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::scopeTargetSend(_) => {
                    <scopeTargetSendCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::scopeTargetToken(_) => {
                    <scopeTargetTokenCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::scopeTokenCapabilities(_) => {
                    <scopeTokenCapabilitiesCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::setMultisend(_) => {
                    <setMultisendCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::transferOwnership(_) => {
                    <transferOwnershipCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::tryGetTarget(_) => {
                    <tryGetTargetCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::upgradeTo(_) => {
                    <upgradeToCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::upgradeToAndCall(_) => {
                    <upgradeToAndCallCall as alloy_sol_types::SolCall>::SELECTOR
                }
            }
        }
        #[inline]
        fn selector_at(i: usize) -> ::core::option::Option<[u8; 4]> {
            Self::SELECTORS.get(i).copied()
        }
        #[inline]
        fn valid_selector(selector: [u8; 4]) -> bool {
            Self::SELECTORS.binary_search(&selector).is_ok()
        }
        #[inline]
        #[allow(non_snake_case)]
        fn abi_decode_raw(
            selector: [u8; 4],
            data: &[u8],
        ) -> alloy_sol_types::Result<Self> {
            static DECODE_SHIMS: &[fn(
                &[u8],
            ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls>] = &[
                {
                    fn isNode(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <isNodeCall as alloy_sol_types::SolCall>::abi_decode_raw(data)
                            .map(HoprNodeManagementModuleCalls::isNode)
                    }
                    isNode
                },
                {
                    fn multisend(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <multisendCall as alloy_sol_types::SolCall>::abi_decode_raw(data)
                            .map(HoprNodeManagementModuleCalls::multisend)
                    }
                    multisend
                },
                {
                    fn revokeTarget(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <revokeTargetCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::revokeTarget)
                    }
                    revokeTarget
                },
                {
                    fn upgradeTo(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <upgradeToCall as alloy_sol_types::SolCall>::abi_decode_raw(data)
                            .map(HoprNodeManagementModuleCalls::upgradeTo)
                    }
                    upgradeTo
                },
                {
                    fn initialize(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <initializeCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::initialize)
                    }
                    initialize
                },
                {
                    fn execTransactionFromModule(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <execTransactionFromModuleCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(
                                HoprNodeManagementModuleCalls::execTransactionFromModule,
                            )
                    }
                    execTransactionFromModule
                },
                {
                    fn isHoprNodeManagementModule(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <isHoprNodeManagementModuleCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(
                                HoprNodeManagementModuleCalls::isHoprNodeManagementModule,
                            )
                    }
                    isHoprNodeManagementModule
                },
                {
                    fn upgradeToAndCall(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <upgradeToAndCallCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::upgradeToAndCall)
                    }
                    upgradeToAndCall
                },
                {
                    fn execTransactionFromModuleReturnData(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <execTransactionFromModuleReturnDataCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(
                                HoprNodeManagementModuleCalls::execTransactionFromModuleReturnData,
                            )
                    }
                    execTransactionFromModuleReturnData
                },
                {
                    fn proxiableUUID(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <proxiableUUIDCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::proxiableUUID)
                    }
                    proxiableUUID
                },
                {
                    fn encodeFunctionSigsAndPermissions(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <encodeFunctionSigsAndPermissionsCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(
                                HoprNodeManagementModuleCalls::encodeFunctionSigsAndPermissions,
                            )
                    }
                    encodeFunctionSigsAndPermissions
                },
                {
                    fn decodeFunctionSigsAndPermissions(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <decodeFunctionSigsAndPermissionsCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(
                                HoprNodeManagementModuleCalls::decodeFunctionSigsAndPermissions,
                            )
                    }
                    decodeFunctionSigsAndPermissions
                },
                {
                    fn getTargets(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <getTargetsCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::getTargets)
                    }
                    getTargets
                },
                {
                    fn renounceOwnership(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <renounceOwnershipCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::renounceOwnership)
                    }
                    renounceOwnership
                },
                {
                    fn scopeTargetChannels(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <scopeTargetChannelsCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::scopeTargetChannels)
                    }
                    scopeTargetChannels
                },
                {
                    fn setMultisend(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <setMultisendCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::setMultisend)
                    }
                    setMultisend
                },
                {
                    fn owner(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <ownerCall as alloy_sol_types::SolCall>::abi_decode_raw(data)
                            .map(HoprNodeManagementModuleCalls::owner)
                    }
                    owner
                },
                {
                    fn addNode(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <addNodeCall as alloy_sol_types::SolCall>::abi_decode_raw(data)
                            .map(HoprNodeManagementModuleCalls::addNode)
                    }
                    addNode
                },
                {
                    fn addChannelsAndTokenTarget(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <addChannelsAndTokenTargetCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(
                                HoprNodeManagementModuleCalls::addChannelsAndTokenTarget,
                            )
                    }
                    addChannelsAndTokenTarget
                },
                {
                    fn scopeTargetToken(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <scopeTargetTokenCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::scopeTargetToken)
                    }
                    scopeTargetToken
                },
                {
                    fn removeNode(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <removeNodeCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::removeNode)
                    }
                    removeNode
                },
                {
                    fn includeNode(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <includeNodeCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::includeNode)
                    }
                    includeNode
                },
                {
                    fn scopeTokenCapabilities(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <scopeTokenCapabilitiesCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::scopeTokenCapabilities)
                    }
                    scopeTokenCapabilities
                },
                {
                    fn scopeSendCapability(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <scopeSendCapabilityCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::scopeSendCapability)
                    }
                    scopeSendCapability
                },
                {
                    fn scopeTargetSend(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <scopeTargetSendCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::scopeTargetSend)
                    }
                    scopeTargetSend
                },
                {
                    fn getGranularPermissions(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <getGranularPermissionsCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::getGranularPermissions)
                    }
                    getGranularPermissions
                },
                {
                    fn tryGetTarget(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <tryGetTargetCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::tryGetTarget)
                    }
                    tryGetTarget
                },
                {
                    fn transferOwnership(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <transferOwnershipCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::transferOwnership)
                    }
                    transferOwnership
                },
                {
                    fn scopeChannelsCapabilities(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <scopeChannelsCapabilitiesCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(
                                HoprNodeManagementModuleCalls::scopeChannelsCapabilities,
                            )
                    }
                    scopeChannelsCapabilities
                },
            ];
            let Ok(idx) = Self::SELECTORS.binary_search(&selector) else {
                return Err(
                    alloy_sol_types::Error::unknown_selector(
                        <Self as alloy_sol_types::SolInterface>::NAME,
                        selector,
                    ),
                );
            };
            DECODE_SHIMS[idx](data)
        }
        #[inline]
        #[allow(non_snake_case)]
        fn abi_decode_raw_validate(
            selector: [u8; 4],
            data: &[u8],
        ) -> alloy_sol_types::Result<Self> {
            static DECODE_VALIDATE_SHIMS: &[fn(
                &[u8],
            ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls>] = &[
                {
                    fn isNode(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <isNodeCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::isNode)
                    }
                    isNode
                },
                {
                    fn multisend(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <multisendCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::multisend)
                    }
                    multisend
                },
                {
                    fn revokeTarget(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <revokeTargetCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::revokeTarget)
                    }
                    revokeTarget
                },
                {
                    fn upgradeTo(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <upgradeToCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::upgradeTo)
                    }
                    upgradeTo
                },
                {
                    fn initialize(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <initializeCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::initialize)
                    }
                    initialize
                },
                {
                    fn execTransactionFromModule(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <execTransactionFromModuleCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(
                                HoprNodeManagementModuleCalls::execTransactionFromModule,
                            )
                    }
                    execTransactionFromModule
                },
                {
                    fn isHoprNodeManagementModule(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <isHoprNodeManagementModuleCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(
                                HoprNodeManagementModuleCalls::isHoprNodeManagementModule,
                            )
                    }
                    isHoprNodeManagementModule
                },
                {
                    fn upgradeToAndCall(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <upgradeToAndCallCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::upgradeToAndCall)
                    }
                    upgradeToAndCall
                },
                {
                    fn execTransactionFromModuleReturnData(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <execTransactionFromModuleReturnDataCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(
                                HoprNodeManagementModuleCalls::execTransactionFromModuleReturnData,
                            )
                    }
                    execTransactionFromModuleReturnData
                },
                {
                    fn proxiableUUID(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <proxiableUUIDCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::proxiableUUID)
                    }
                    proxiableUUID
                },
                {
                    fn encodeFunctionSigsAndPermissions(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <encodeFunctionSigsAndPermissionsCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(
                                HoprNodeManagementModuleCalls::encodeFunctionSigsAndPermissions,
                            )
                    }
                    encodeFunctionSigsAndPermissions
                },
                {
                    fn decodeFunctionSigsAndPermissions(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <decodeFunctionSigsAndPermissionsCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(
                                HoprNodeManagementModuleCalls::decodeFunctionSigsAndPermissions,
                            )
                    }
                    decodeFunctionSigsAndPermissions
                },
                {
                    fn getTargets(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <getTargetsCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::getTargets)
                    }
                    getTargets
                },
                {
                    fn renounceOwnership(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <renounceOwnershipCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::renounceOwnership)
                    }
                    renounceOwnership
                },
                {
                    fn scopeTargetChannels(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <scopeTargetChannelsCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::scopeTargetChannels)
                    }
                    scopeTargetChannels
                },
                {
                    fn setMultisend(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <setMultisendCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::setMultisend)
                    }
                    setMultisend
                },
                {
                    fn owner(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <ownerCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::owner)
                    }
                    owner
                },
                {
                    fn addNode(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <addNodeCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::addNode)
                    }
                    addNode
                },
                {
                    fn addChannelsAndTokenTarget(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <addChannelsAndTokenTargetCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(
                                HoprNodeManagementModuleCalls::addChannelsAndTokenTarget,
                            )
                    }
                    addChannelsAndTokenTarget
                },
                {
                    fn scopeTargetToken(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <scopeTargetTokenCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::scopeTargetToken)
                    }
                    scopeTargetToken
                },
                {
                    fn removeNode(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <removeNodeCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::removeNode)
                    }
                    removeNode
                },
                {
                    fn includeNode(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <includeNodeCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::includeNode)
                    }
                    includeNode
                },
                {
                    fn scopeTokenCapabilities(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <scopeTokenCapabilitiesCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::scopeTokenCapabilities)
                    }
                    scopeTokenCapabilities
                },
                {
                    fn scopeSendCapability(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <scopeSendCapabilityCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::scopeSendCapability)
                    }
                    scopeSendCapability
                },
                {
                    fn scopeTargetSend(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <scopeTargetSendCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::scopeTargetSend)
                    }
                    scopeTargetSend
                },
                {
                    fn getGranularPermissions(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <getGranularPermissionsCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::getGranularPermissions)
                    }
                    getGranularPermissions
                },
                {
                    fn tryGetTarget(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <tryGetTargetCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::tryGetTarget)
                    }
                    tryGetTarget
                },
                {
                    fn transferOwnership(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <transferOwnershipCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleCalls::transferOwnership)
                    }
                    transferOwnership
                },
                {
                    fn scopeChannelsCapabilities(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleCalls> {
                        <scopeChannelsCapabilitiesCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(
                                HoprNodeManagementModuleCalls::scopeChannelsCapabilities,
                            )
                    }
                    scopeChannelsCapabilities
                },
            ];
            let Ok(idx) = Self::SELECTORS.binary_search(&selector) else {
                return Err(
                    alloy_sol_types::Error::unknown_selector(
                        <Self as alloy_sol_types::SolInterface>::NAME,
                        selector,
                    ),
                );
            };
            DECODE_VALIDATE_SHIMS[idx](data)
        }
        #[inline]
        fn abi_encoded_size(&self) -> usize {
            match self {
                Self::addChannelsAndTokenTarget(inner) => {
                    <addChannelsAndTokenTargetCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::addNode(inner) => {
                    <addNodeCall as alloy_sol_types::SolCall>::abi_encoded_size(inner)
                }
                Self::decodeFunctionSigsAndPermissions(inner) => {
                    <decodeFunctionSigsAndPermissionsCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::encodeFunctionSigsAndPermissions(inner) => {
                    <encodeFunctionSigsAndPermissionsCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::execTransactionFromModule(inner) => {
                    <execTransactionFromModuleCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::execTransactionFromModuleReturnData(inner) => {
                    <execTransactionFromModuleReturnDataCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::getGranularPermissions(inner) => {
                    <getGranularPermissionsCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::getTargets(inner) => {
                    <getTargetsCall as alloy_sol_types::SolCall>::abi_encoded_size(inner)
                }
                Self::includeNode(inner) => {
                    <includeNodeCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::initialize(inner) => {
                    <initializeCall as alloy_sol_types::SolCall>::abi_encoded_size(inner)
                }
                Self::isHoprNodeManagementModule(inner) => {
                    <isHoprNodeManagementModuleCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::isNode(inner) => {
                    <isNodeCall as alloy_sol_types::SolCall>::abi_encoded_size(inner)
                }
                Self::multisend(inner) => {
                    <multisendCall as alloy_sol_types::SolCall>::abi_encoded_size(inner)
                }
                Self::owner(inner) => {
                    <ownerCall as alloy_sol_types::SolCall>::abi_encoded_size(inner)
                }
                Self::proxiableUUID(inner) => {
                    <proxiableUUIDCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::removeNode(inner) => {
                    <removeNodeCall as alloy_sol_types::SolCall>::abi_encoded_size(inner)
                }
                Self::renounceOwnership(inner) => {
                    <renounceOwnershipCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::revokeTarget(inner) => {
                    <revokeTargetCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::scopeChannelsCapabilities(inner) => {
                    <scopeChannelsCapabilitiesCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::scopeSendCapability(inner) => {
                    <scopeSendCapabilityCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::scopeTargetChannels(inner) => {
                    <scopeTargetChannelsCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::scopeTargetSend(inner) => {
                    <scopeTargetSendCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::scopeTargetToken(inner) => {
                    <scopeTargetTokenCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::scopeTokenCapabilities(inner) => {
                    <scopeTokenCapabilitiesCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::setMultisend(inner) => {
                    <setMultisendCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::transferOwnership(inner) => {
                    <transferOwnershipCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::tryGetTarget(inner) => {
                    <tryGetTargetCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::upgradeTo(inner) => {
                    <upgradeToCall as alloy_sol_types::SolCall>::abi_encoded_size(inner)
                }
                Self::upgradeToAndCall(inner) => {
                    <upgradeToAndCallCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
            }
        }
        #[inline]
        fn abi_encode_raw(&self, out: &mut alloy_sol_types::private::Vec<u8>) {
            match self {
                Self::addChannelsAndTokenTarget(inner) => {
                    <addChannelsAndTokenTargetCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::addNode(inner) => {
                    <addNodeCall as alloy_sol_types::SolCall>::abi_encode_raw(inner, out)
                }
                Self::decodeFunctionSigsAndPermissions(inner) => {
                    <decodeFunctionSigsAndPermissionsCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::encodeFunctionSigsAndPermissions(inner) => {
                    <encodeFunctionSigsAndPermissionsCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::execTransactionFromModule(inner) => {
                    <execTransactionFromModuleCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::execTransactionFromModuleReturnData(inner) => {
                    <execTransactionFromModuleReturnDataCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::getGranularPermissions(inner) => {
                    <getGranularPermissionsCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::getTargets(inner) => {
                    <getTargetsCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::includeNode(inner) => {
                    <includeNodeCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::initialize(inner) => {
                    <initializeCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::isHoprNodeManagementModule(inner) => {
                    <isHoprNodeManagementModuleCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::isNode(inner) => {
                    <isNodeCall as alloy_sol_types::SolCall>::abi_encode_raw(inner, out)
                }
                Self::multisend(inner) => {
                    <multisendCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::owner(inner) => {
                    <ownerCall as alloy_sol_types::SolCall>::abi_encode_raw(inner, out)
                }
                Self::proxiableUUID(inner) => {
                    <proxiableUUIDCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::removeNode(inner) => {
                    <removeNodeCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::renounceOwnership(inner) => {
                    <renounceOwnershipCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::revokeTarget(inner) => {
                    <revokeTargetCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::scopeChannelsCapabilities(inner) => {
                    <scopeChannelsCapabilitiesCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::scopeSendCapability(inner) => {
                    <scopeSendCapabilityCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::scopeTargetChannels(inner) => {
                    <scopeTargetChannelsCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::scopeTargetSend(inner) => {
                    <scopeTargetSendCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::scopeTargetToken(inner) => {
                    <scopeTargetTokenCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::scopeTokenCapabilities(inner) => {
                    <scopeTokenCapabilitiesCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::setMultisend(inner) => {
                    <setMultisendCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::transferOwnership(inner) => {
                    <transferOwnershipCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::tryGetTarget(inner) => {
                    <tryGetTargetCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::upgradeTo(inner) => {
                    <upgradeToCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::upgradeToAndCall(inner) => {
                    <upgradeToAndCallCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
            }
        }
    }
    ///Container for all the [`HoprNodeManagementModule`](self) custom errors.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Debug, PartialEq, Eq, Hash)]
    pub enum HoprNodeManagementModuleErrors {
        #[allow(missing_docs)]
        AddressIsZero(AddressIsZero),
        #[allow(missing_docs)]
        AlreadyInitialized(AlreadyInitialized),
        #[allow(missing_docs)]
        ArrayTooLong(ArrayTooLong),
        #[allow(missing_docs)]
        ArraysDifferentLength(ArraysDifferentLength),
        #[allow(missing_docs)]
        CalldataOutOfBounds(CalldataOutOfBounds),
        #[allow(missing_docs)]
        CannotChangeOwner(CannotChangeOwner),
        #[allow(missing_docs)]
        DefaultPermissionRejected(DefaultPermissionRejected),
        #[allow(missing_docs)]
        DelegateCallNotAllowed(DelegateCallNotAllowed),
        #[allow(missing_docs)]
        FunctionSignatureTooShort(FunctionSignatureTooShort),
        #[allow(missing_docs)]
        GranularPermissionRejected(GranularPermissionRejected),
        #[allow(missing_docs)]
        NoMembership(NoMembership),
        #[allow(missing_docs)]
        NodePermissionRejected(NodePermissionRejected),
        #[allow(missing_docs)]
        NonExistentKey(NonExistentKey),
        #[allow(missing_docs)]
        ParameterNotAllowed(ParameterNotAllowed),
        #[allow(missing_docs)]
        PermissionNotConfigured(PermissionNotConfigured),
        #[allow(missing_docs)]
        PermissionNotFound(PermissionNotFound),
        #[allow(missing_docs)]
        SafeMultisendSameAddress(SafeMultisendSameAddress),
        #[allow(missing_docs)]
        SendNotAllowed(SendNotAllowed),
        #[allow(missing_docs)]
        TargetAddressNotAllowed(TargetAddressNotAllowed),
        #[allow(missing_docs)]
        TargetIsNotScoped(TargetIsNotScoped),
        #[allow(missing_docs)]
        TargetIsScoped(TargetIsScoped),
        #[allow(missing_docs)]
        TooManyCapabilities(TooManyCapabilities),
        #[allow(missing_docs)]
        UnacceptableMultiSendOffset(UnacceptableMultiSendOffset),
        #[allow(missing_docs)]
        WithMembership(WithMembership),
    }
    #[automatically_derived]
    impl HoprNodeManagementModuleErrors {
        /// All the selectors of this enum.
        ///
        /// Note that the selectors might not be in the same order as the variants.
        /// No guarantees are made about the order of the selectors.
        ///
        /// Prefer using `SolInterface` methods instead.
        pub const SELECTORS: &'static [[u8; 4usize]] = &[
            [9u8, 233u8, 205u8, 73u8],
            [13u8, 137u8, 67u8, 142u8],
            [13u8, 193u8, 73u8, 240u8],
            [45u8, 5u8, 25u8, 173u8],
            [49u8, 233u8, 130u8, 70u8],
            [70u8, 132u8, 193u8, 34u8],
            [70u8, 173u8, 69u8, 136u8],
            [74u8, 137u8, 3u8, 33u8],
            [88u8, 114u8, 48u8, 55u8],
            [89u8, 138u8, 14u8, 33u8],
            [110u8, 176u8, 49u8, 95u8],
            [116u8, 38u8, 56u8, 180u8],
            [116u8, 244u8, 213u8, 55u8],
            [126u8, 209u8, 17u8, 55u8],
            [134u8, 77u8, 209u8, 231u8],
            [134u8, 121u8, 21u8, 171u8],
            [180u8, 74u8, 249u8, 175u8],
            [189u8, 38u8, 204u8, 56u8],
            [216u8, 69u8, 90u8, 19u8],
            [227u8, 160u8, 90u8, 148u8],
            [232u8, 192u8, 125u8, 42u8],
            [239u8, 52u8, 64u8, 172u8],
            [253u8, 103u8, 14u8, 190u8],
            [253u8, 142u8, 159u8, 40u8],
        ];
    }
    #[automatically_derived]
    impl alloy_sol_types::SolInterface for HoprNodeManagementModuleErrors {
        const NAME: &'static str = "HoprNodeManagementModuleErrors";
        const MIN_DATA_LENGTH: usize = 0usize;
        const COUNT: usize = 24usize;
        #[inline]
        fn selector(&self) -> [u8; 4] {
            match self {
                Self::AddressIsZero(_) => {
                    <AddressIsZero as alloy_sol_types::SolError>::SELECTOR
                }
                Self::AlreadyInitialized(_) => {
                    <AlreadyInitialized as alloy_sol_types::SolError>::SELECTOR
                }
                Self::ArrayTooLong(_) => {
                    <ArrayTooLong as alloy_sol_types::SolError>::SELECTOR
                }
                Self::ArraysDifferentLength(_) => {
                    <ArraysDifferentLength as alloy_sol_types::SolError>::SELECTOR
                }
                Self::CalldataOutOfBounds(_) => {
                    <CalldataOutOfBounds as alloy_sol_types::SolError>::SELECTOR
                }
                Self::CannotChangeOwner(_) => {
                    <CannotChangeOwner as alloy_sol_types::SolError>::SELECTOR
                }
                Self::DefaultPermissionRejected(_) => {
                    <DefaultPermissionRejected as alloy_sol_types::SolError>::SELECTOR
                }
                Self::DelegateCallNotAllowed(_) => {
                    <DelegateCallNotAllowed as alloy_sol_types::SolError>::SELECTOR
                }
                Self::FunctionSignatureTooShort(_) => {
                    <FunctionSignatureTooShort as alloy_sol_types::SolError>::SELECTOR
                }
                Self::GranularPermissionRejected(_) => {
                    <GranularPermissionRejected as alloy_sol_types::SolError>::SELECTOR
                }
                Self::NoMembership(_) => {
                    <NoMembership as alloy_sol_types::SolError>::SELECTOR
                }
                Self::NodePermissionRejected(_) => {
                    <NodePermissionRejected as alloy_sol_types::SolError>::SELECTOR
                }
                Self::NonExistentKey(_) => {
                    <NonExistentKey as alloy_sol_types::SolError>::SELECTOR
                }
                Self::ParameterNotAllowed(_) => {
                    <ParameterNotAllowed as alloy_sol_types::SolError>::SELECTOR
                }
                Self::PermissionNotConfigured(_) => {
                    <PermissionNotConfigured as alloy_sol_types::SolError>::SELECTOR
                }
                Self::PermissionNotFound(_) => {
                    <PermissionNotFound as alloy_sol_types::SolError>::SELECTOR
                }
                Self::SafeMultisendSameAddress(_) => {
                    <SafeMultisendSameAddress as alloy_sol_types::SolError>::SELECTOR
                }
                Self::SendNotAllowed(_) => {
                    <SendNotAllowed as alloy_sol_types::SolError>::SELECTOR
                }
                Self::TargetAddressNotAllowed(_) => {
                    <TargetAddressNotAllowed as alloy_sol_types::SolError>::SELECTOR
                }
                Self::TargetIsNotScoped(_) => {
                    <TargetIsNotScoped as alloy_sol_types::SolError>::SELECTOR
                }
                Self::TargetIsScoped(_) => {
                    <TargetIsScoped as alloy_sol_types::SolError>::SELECTOR
                }
                Self::TooManyCapabilities(_) => {
                    <TooManyCapabilities as alloy_sol_types::SolError>::SELECTOR
                }
                Self::UnacceptableMultiSendOffset(_) => {
                    <UnacceptableMultiSendOffset as alloy_sol_types::SolError>::SELECTOR
                }
                Self::WithMembership(_) => {
                    <WithMembership as alloy_sol_types::SolError>::SELECTOR
                }
            }
        }
        #[inline]
        fn selector_at(i: usize) -> ::core::option::Option<[u8; 4]> {
            Self::SELECTORS.get(i).copied()
        }
        #[inline]
        fn valid_selector(selector: [u8; 4]) -> bool {
            Self::SELECTORS.binary_search(&selector).is_ok()
        }
        #[inline]
        #[allow(non_snake_case)]
        fn abi_decode_raw(
            selector: [u8; 4],
            data: &[u8],
        ) -> alloy_sol_types::Result<Self> {
            static DECODE_SHIMS: &[fn(
                &[u8],
            ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors>] = &[
                {
                    fn SendNotAllowed(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <SendNotAllowed as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::SendNotAllowed)
                    }
                    SendNotAllowed
                },
                {
                    fn DelegateCallNotAllowed(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <DelegateCallNotAllowed as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::DelegateCallNotAllowed)
                    }
                    DelegateCallNotAllowed
                },
                {
                    fn AlreadyInitialized(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <AlreadyInitialized as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::AlreadyInitialized)
                    }
                    AlreadyInitialized
                },
                {
                    fn NonExistentKey(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <NonExistentKey as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::NonExistentKey)
                    }
                    NonExistentKey
                },
                {
                    fn ParameterNotAllowed(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <ParameterNotAllowed as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::ParameterNotAllowed)
                    }
                    ParameterNotAllowed
                },
                {
                    fn FunctionSignatureTooShort(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <FunctionSignatureTooShort as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(
                                HoprNodeManagementModuleErrors::FunctionSignatureTooShort,
                            )
                    }
                    FunctionSignatureTooShort
                },
                {
                    fn PermissionNotConfigured(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <PermissionNotConfigured as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::PermissionNotConfigured)
                    }
                    PermissionNotConfigured
                },
                {
                    fn TargetIsNotScoped(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <TargetIsNotScoped as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::TargetIsNotScoped)
                    }
                    TargetIsNotScoped
                },
                {
                    fn DefaultPermissionRejected(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <DefaultPermissionRejected as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(
                                HoprNodeManagementModuleErrors::DefaultPermissionRejected,
                            )
                    }
                    DefaultPermissionRejected
                },
                {
                    fn SafeMultisendSameAddress(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <SafeMultisendSameAddress as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(
                                HoprNodeManagementModuleErrors::SafeMultisendSameAddress,
                            )
                    }
                    SafeMultisendSameAddress
                },
                {
                    fn NodePermissionRejected(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <NodePermissionRejected as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::NodePermissionRejected)
                    }
                    NodePermissionRejected
                },
                {
                    fn CalldataOutOfBounds(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <CalldataOutOfBounds as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::CalldataOutOfBounds)
                    }
                    CalldataOutOfBounds
                },
                {
                    fn ArraysDifferentLength(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <ArraysDifferentLength as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::ArraysDifferentLength)
                    }
                    ArraysDifferentLength
                },
                {
                    fn UnacceptableMultiSendOffset(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <UnacceptableMultiSendOffset as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(
                                HoprNodeManagementModuleErrors::UnacceptableMultiSendOffset,
                            )
                    }
                    UnacceptableMultiSendOffset
                },
                {
                    fn GranularPermissionRejected(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <GranularPermissionRejected as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(
                                HoprNodeManagementModuleErrors::GranularPermissionRejected,
                            )
                    }
                    GranularPermissionRejected
                },
                {
                    fn AddressIsZero(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <AddressIsZero as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::AddressIsZero)
                    }
                    AddressIsZero
                },
                {
                    fn TooManyCapabilities(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <TooManyCapabilities as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::TooManyCapabilities)
                    }
                    TooManyCapabilities
                },
                {
                    fn ArrayTooLong(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <ArrayTooLong as alloy_sol_types::SolError>::abi_decode_raw(data)
                            .map(HoprNodeManagementModuleErrors::ArrayTooLong)
                    }
                    ArrayTooLong
                },
                {
                    fn PermissionNotFound(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <PermissionNotFound as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::PermissionNotFound)
                    }
                    PermissionNotFound
                },
                {
                    fn WithMembership(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <WithMembership as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::WithMembership)
                    }
                    WithMembership
                },
                {
                    fn TargetIsScoped(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <TargetIsScoped as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::TargetIsScoped)
                    }
                    TargetIsScoped
                },
                {
                    fn TargetAddressNotAllowed(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <TargetAddressNotAllowed as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::TargetAddressNotAllowed)
                    }
                    TargetAddressNotAllowed
                },
                {
                    fn CannotChangeOwner(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <CannotChangeOwner as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::CannotChangeOwner)
                    }
                    CannotChangeOwner
                },
                {
                    fn NoMembership(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <NoMembership as alloy_sol_types::SolError>::abi_decode_raw(data)
                            .map(HoprNodeManagementModuleErrors::NoMembership)
                    }
                    NoMembership
                },
            ];
            let Ok(idx) = Self::SELECTORS.binary_search(&selector) else {
                return Err(
                    alloy_sol_types::Error::unknown_selector(
                        <Self as alloy_sol_types::SolInterface>::NAME,
                        selector,
                    ),
                );
            };
            DECODE_SHIMS[idx](data)
        }
        #[inline]
        #[allow(non_snake_case)]
        fn abi_decode_raw_validate(
            selector: [u8; 4],
            data: &[u8],
        ) -> alloy_sol_types::Result<Self> {
            static DECODE_VALIDATE_SHIMS: &[fn(
                &[u8],
            ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors>] = &[
                {
                    fn SendNotAllowed(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <SendNotAllowed as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::SendNotAllowed)
                    }
                    SendNotAllowed
                },
                {
                    fn DelegateCallNotAllowed(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <DelegateCallNotAllowed as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::DelegateCallNotAllowed)
                    }
                    DelegateCallNotAllowed
                },
                {
                    fn AlreadyInitialized(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <AlreadyInitialized as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::AlreadyInitialized)
                    }
                    AlreadyInitialized
                },
                {
                    fn NonExistentKey(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <NonExistentKey as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::NonExistentKey)
                    }
                    NonExistentKey
                },
                {
                    fn ParameterNotAllowed(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <ParameterNotAllowed as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::ParameterNotAllowed)
                    }
                    ParameterNotAllowed
                },
                {
                    fn FunctionSignatureTooShort(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <FunctionSignatureTooShort as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(
                                HoprNodeManagementModuleErrors::FunctionSignatureTooShort,
                            )
                    }
                    FunctionSignatureTooShort
                },
                {
                    fn PermissionNotConfigured(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <PermissionNotConfigured as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::PermissionNotConfigured)
                    }
                    PermissionNotConfigured
                },
                {
                    fn TargetIsNotScoped(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <TargetIsNotScoped as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::TargetIsNotScoped)
                    }
                    TargetIsNotScoped
                },
                {
                    fn DefaultPermissionRejected(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <DefaultPermissionRejected as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(
                                HoprNodeManagementModuleErrors::DefaultPermissionRejected,
                            )
                    }
                    DefaultPermissionRejected
                },
                {
                    fn SafeMultisendSameAddress(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <SafeMultisendSameAddress as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(
                                HoprNodeManagementModuleErrors::SafeMultisendSameAddress,
                            )
                    }
                    SafeMultisendSameAddress
                },
                {
                    fn NodePermissionRejected(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <NodePermissionRejected as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::NodePermissionRejected)
                    }
                    NodePermissionRejected
                },
                {
                    fn CalldataOutOfBounds(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <CalldataOutOfBounds as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::CalldataOutOfBounds)
                    }
                    CalldataOutOfBounds
                },
                {
                    fn ArraysDifferentLength(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <ArraysDifferentLength as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::ArraysDifferentLength)
                    }
                    ArraysDifferentLength
                },
                {
                    fn UnacceptableMultiSendOffset(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <UnacceptableMultiSendOffset as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(
                                HoprNodeManagementModuleErrors::UnacceptableMultiSendOffset,
                            )
                    }
                    UnacceptableMultiSendOffset
                },
                {
                    fn GranularPermissionRejected(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <GranularPermissionRejected as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(
                                HoprNodeManagementModuleErrors::GranularPermissionRejected,
                            )
                    }
                    GranularPermissionRejected
                },
                {
                    fn AddressIsZero(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <AddressIsZero as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::AddressIsZero)
                    }
                    AddressIsZero
                },
                {
                    fn TooManyCapabilities(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <TooManyCapabilities as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::TooManyCapabilities)
                    }
                    TooManyCapabilities
                },
                {
                    fn ArrayTooLong(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <ArrayTooLong as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::ArrayTooLong)
                    }
                    ArrayTooLong
                },
                {
                    fn PermissionNotFound(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <PermissionNotFound as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::PermissionNotFound)
                    }
                    PermissionNotFound
                },
                {
                    fn WithMembership(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <WithMembership as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::WithMembership)
                    }
                    WithMembership
                },
                {
                    fn TargetIsScoped(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <TargetIsScoped as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::TargetIsScoped)
                    }
                    TargetIsScoped
                },
                {
                    fn TargetAddressNotAllowed(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <TargetAddressNotAllowed as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::TargetAddressNotAllowed)
                    }
                    TargetAddressNotAllowed
                },
                {
                    fn CannotChangeOwner(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <CannotChangeOwner as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::CannotChangeOwner)
                    }
                    CannotChangeOwner
                },
                {
                    fn NoMembership(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprNodeManagementModuleErrors> {
                        <NoMembership as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprNodeManagementModuleErrors::NoMembership)
                    }
                    NoMembership
                },
            ];
            let Ok(idx) = Self::SELECTORS.binary_search(&selector) else {
                return Err(
                    alloy_sol_types::Error::unknown_selector(
                        <Self as alloy_sol_types::SolInterface>::NAME,
                        selector,
                    ),
                );
            };
            DECODE_VALIDATE_SHIMS[idx](data)
        }
        #[inline]
        fn abi_encoded_size(&self) -> usize {
            match self {
                Self::AddressIsZero(inner) => {
                    <AddressIsZero as alloy_sol_types::SolError>::abi_encoded_size(inner)
                }
                Self::AlreadyInitialized(inner) => {
                    <AlreadyInitialized as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::ArrayTooLong(inner) => {
                    <ArrayTooLong as alloy_sol_types::SolError>::abi_encoded_size(inner)
                }
                Self::ArraysDifferentLength(inner) => {
                    <ArraysDifferentLength as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::CalldataOutOfBounds(inner) => {
                    <CalldataOutOfBounds as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::CannotChangeOwner(inner) => {
                    <CannotChangeOwner as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::DefaultPermissionRejected(inner) => {
                    <DefaultPermissionRejected as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::DelegateCallNotAllowed(inner) => {
                    <DelegateCallNotAllowed as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::FunctionSignatureTooShort(inner) => {
                    <FunctionSignatureTooShort as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::GranularPermissionRejected(inner) => {
                    <GranularPermissionRejected as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::NoMembership(inner) => {
                    <NoMembership as alloy_sol_types::SolError>::abi_encoded_size(inner)
                }
                Self::NodePermissionRejected(inner) => {
                    <NodePermissionRejected as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::NonExistentKey(inner) => {
                    <NonExistentKey as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::ParameterNotAllowed(inner) => {
                    <ParameterNotAllowed as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::PermissionNotConfigured(inner) => {
                    <PermissionNotConfigured as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::PermissionNotFound(inner) => {
                    <PermissionNotFound as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::SafeMultisendSameAddress(inner) => {
                    <SafeMultisendSameAddress as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::SendNotAllowed(inner) => {
                    <SendNotAllowed as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::TargetAddressNotAllowed(inner) => {
                    <TargetAddressNotAllowed as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::TargetIsNotScoped(inner) => {
                    <TargetIsNotScoped as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::TargetIsScoped(inner) => {
                    <TargetIsScoped as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::TooManyCapabilities(inner) => {
                    <TooManyCapabilities as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::UnacceptableMultiSendOffset(inner) => {
                    <UnacceptableMultiSendOffset as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::WithMembership(inner) => {
                    <WithMembership as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
            }
        }
        #[inline]
        fn abi_encode_raw(&self, out: &mut alloy_sol_types::private::Vec<u8>) {
            match self {
                Self::AddressIsZero(inner) => {
                    <AddressIsZero as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::AlreadyInitialized(inner) => {
                    <AlreadyInitialized as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::ArrayTooLong(inner) => {
                    <ArrayTooLong as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::ArraysDifferentLength(inner) => {
                    <ArraysDifferentLength as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::CalldataOutOfBounds(inner) => {
                    <CalldataOutOfBounds as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::CannotChangeOwner(inner) => {
                    <CannotChangeOwner as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::DefaultPermissionRejected(inner) => {
                    <DefaultPermissionRejected as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::DelegateCallNotAllowed(inner) => {
                    <DelegateCallNotAllowed as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::FunctionSignatureTooShort(inner) => {
                    <FunctionSignatureTooShort as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::GranularPermissionRejected(inner) => {
                    <GranularPermissionRejected as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::NoMembership(inner) => {
                    <NoMembership as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::NodePermissionRejected(inner) => {
                    <NodePermissionRejected as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::NonExistentKey(inner) => {
                    <NonExistentKey as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::ParameterNotAllowed(inner) => {
                    <ParameterNotAllowed as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::PermissionNotConfigured(inner) => {
                    <PermissionNotConfigured as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::PermissionNotFound(inner) => {
                    <PermissionNotFound as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::SafeMultisendSameAddress(inner) => {
                    <SafeMultisendSameAddress as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::SendNotAllowed(inner) => {
                    <SendNotAllowed as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::TargetAddressNotAllowed(inner) => {
                    <TargetAddressNotAllowed as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::TargetIsNotScoped(inner) => {
                    <TargetIsNotScoped as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::TargetIsScoped(inner) => {
                    <TargetIsScoped as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::TooManyCapabilities(inner) => {
                    <TooManyCapabilities as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::UnacceptableMultiSendOffset(inner) => {
                    <UnacceptableMultiSendOffset as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::WithMembership(inner) => {
                    <WithMembership as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
            }
        }
    }
    ///Container for all the [`HoprNodeManagementModule`](self) events.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Debug, PartialEq, Eq, Hash)]
    pub enum HoprNodeManagementModuleEvents {
        #[allow(missing_docs)]
        AdminChanged(AdminChanged),
        #[allow(missing_docs)]
        BeaconUpgraded(BeaconUpgraded),
        #[allow(missing_docs)]
        ExecutionFailure(ExecutionFailure),
        #[allow(missing_docs)]
        ExecutionSuccess(ExecutionSuccess),
        #[allow(missing_docs)]
        Initialized(Initialized),
        #[allow(missing_docs)]
        NodeAdded(NodeAdded),
        #[allow(missing_docs)]
        NodeRemoved(NodeRemoved),
        #[allow(missing_docs)]
        OwnershipTransferred(OwnershipTransferred),
        #[allow(missing_docs)]
        SetMultisendAddress(SetMultisendAddress),
        #[allow(missing_docs)]
        Upgraded(Upgraded),
    }
    #[automatically_derived]
    impl HoprNodeManagementModuleEvents {
        /// All the selectors of this enum.
        ///
        /// Note that the selectors might not be in the same order as the variants.
        /// No guarantees are made about the order of the selectors.
        ///
        /// Prefer using `SolInterface` methods instead.
        pub const SELECTORS: &'static [[u8; 32usize]] = &[
            [
                28u8, 243u8, 176u8, 58u8, 108u8, 241u8, 159u8, 162u8, 186u8, 186u8, 77u8,
                241u8, 72u8, 233u8, 220u8, 171u8, 237u8, 234u8, 127u8, 138u8, 92u8, 7u8,
                132u8, 14u8, 32u8, 126u8, 92u8, 8u8, 155u8, 233u8, 93u8, 62u8,
            ],
            [
                78u8, 46u8, 134u8, 210u8, 19u8, 117u8, 235u8, 203u8, 246u8, 233u8, 61u8,
                245u8, 235u8, 221u8, 90u8, 145u8, 91u8, 248u8, 48u8, 36u8, 89u8, 4u8,
                195u8, 181u8, 79u8, 72u8, 173u8, 240u8, 23u8, 10u8, 174u8, 75u8,
            ],
            [
                95u8, 230u8, 170u8, 191u8, 78u8, 121u8, 8u8, 67u8, 223u8, 67u8, 174u8,
                14u8, 34u8, 181u8, 134u8, 32u8, 6u8, 111u8, 179u8, 137u8, 41u8, 91u8,
                237u8, 192u8, 106u8, 146u8, 223u8, 108u8, 59u8, 40u8, 119u8, 125u8,
            ],
            [
                126u8, 100u8, 77u8, 121u8, 66u8, 47u8, 23u8, 192u8, 30u8, 72u8, 148u8,
                181u8, 244u8, 245u8, 136u8, 211u8, 49u8, 235u8, 250u8, 40u8, 101u8, 61u8,
                66u8, 174u8, 131u8, 45u8, 197u8, 158u8, 56u8, 201u8, 121u8, 143u8,
            ],
            [
                127u8, 38u8, 184u8, 63u8, 249u8, 110u8, 31u8, 43u8, 106u8, 104u8, 47u8,
                19u8, 56u8, 82u8, 246u8, 121u8, 138u8, 9u8, 196u8, 101u8, 218u8, 149u8,
                146u8, 20u8, 96u8, 206u8, 251u8, 56u8, 71u8, 64u8, 36u8, 152u8,
            ],
            [
                139u8, 224u8, 7u8, 156u8, 83u8, 22u8, 89u8, 20u8, 19u8, 68u8, 205u8,
                31u8, 208u8, 164u8, 242u8, 132u8, 25u8, 73u8, 127u8, 151u8, 34u8, 163u8,
                218u8, 175u8, 227u8, 180u8, 24u8, 111u8, 107u8, 100u8, 87u8, 224u8,
            ],
            [
                178u8, 93u8, 3u8, 170u8, 243u8, 8u8, 215u8, 41u8, 23u8, 9u8, 190u8, 30u8,
                162u8, 139u8, 128u8, 4u8, 99u8, 207u8, 58u8, 154u8, 76u8, 74u8, 85u8,
                85u8, 215u8, 51u8, 58u8, 150u8, 76u8, 29u8, 254u8, 189u8,
            ],
            [
                188u8, 124u8, 215u8, 90u8, 32u8, 238u8, 39u8, 253u8, 154u8, 222u8, 186u8,
                179u8, 32u8, 65u8, 247u8, 85u8, 33u8, 77u8, 188u8, 107u8, 255u8, 169u8,
                12u8, 192u8, 34u8, 91u8, 57u8, 218u8, 46u8, 92u8, 45u8, 59u8,
            ],
            [
                194u8, 77u8, 147u8, 96u8, 138u8, 3u8, 210u8, 99u8, 255u8, 25u8, 29u8,
                118u8, 119u8, 20u8, 31u8, 94u8, 148u8, 196u8, 150u8, 229u8, 147u8, 16u8,
                143u8, 58u8, 174u8, 12u8, 181u8, 183u8, 4u8, 148u8, 196u8, 211u8,
            ],
            [
                207u8, 194u8, 65u8, 102u8, 219u8, 75u8, 182u8, 119u8, 232u8, 87u8, 202u8,
                202u8, 189u8, 21u8, 65u8, 251u8, 43u8, 48u8, 100u8, 80u8, 33u8, 178u8,
                124u8, 81u8, 48u8, 65u8, 149u8, 137u8, 184u8, 77u8, 181u8, 43u8,
            ],
        ];
    }
    #[automatically_derived]
    impl alloy_sol_types::SolEventInterface for HoprNodeManagementModuleEvents {
        const NAME: &'static str = "HoprNodeManagementModuleEvents";
        const COUNT: usize = 10usize;
        fn decode_raw_log(
            topics: &[alloy_sol_types::Word],
            data: &[u8],
        ) -> alloy_sol_types::Result<Self> {
            match topics.first().copied() {
                Some(<AdminChanged as alloy_sol_types::SolEvent>::SIGNATURE_HASH) => {
                    <AdminChanged as alloy_sol_types::SolEvent>::decode_raw_log(
                            topics,
                            data,
                        )
                        .map(Self::AdminChanged)
                }
                Some(<BeaconUpgraded as alloy_sol_types::SolEvent>::SIGNATURE_HASH) => {
                    <BeaconUpgraded as alloy_sol_types::SolEvent>::decode_raw_log(
                            topics,
                            data,
                        )
                        .map(Self::BeaconUpgraded)
                }
                Some(<ExecutionFailure as alloy_sol_types::SolEvent>::SIGNATURE_HASH) => {
                    <ExecutionFailure as alloy_sol_types::SolEvent>::decode_raw_log(
                            topics,
                            data,
                        )
                        .map(Self::ExecutionFailure)
                }
                Some(<ExecutionSuccess as alloy_sol_types::SolEvent>::SIGNATURE_HASH) => {
                    <ExecutionSuccess as alloy_sol_types::SolEvent>::decode_raw_log(
                            topics,
                            data,
                        )
                        .map(Self::ExecutionSuccess)
                }
                Some(<Initialized as alloy_sol_types::SolEvent>::SIGNATURE_HASH) => {
                    <Initialized as alloy_sol_types::SolEvent>::decode_raw_log(
                            topics,
                            data,
                        )
                        .map(Self::Initialized)
                }
                Some(<NodeAdded as alloy_sol_types::SolEvent>::SIGNATURE_HASH) => {
                    <NodeAdded as alloy_sol_types::SolEvent>::decode_raw_log(
                            topics,
                            data,
                        )
                        .map(Self::NodeAdded)
                }
                Some(<NodeRemoved as alloy_sol_types::SolEvent>::SIGNATURE_HASH) => {
                    <NodeRemoved as alloy_sol_types::SolEvent>::decode_raw_log(
                            topics,
                            data,
                        )
                        .map(Self::NodeRemoved)
                }
                Some(
                    <OwnershipTransferred as alloy_sol_types::SolEvent>::SIGNATURE_HASH,
                ) => {
                    <OwnershipTransferred as alloy_sol_types::SolEvent>::decode_raw_log(
                            topics,
                            data,
                        )
                        .map(Self::OwnershipTransferred)
                }
                Some(
                    <SetMultisendAddress as alloy_sol_types::SolEvent>::SIGNATURE_HASH,
                ) => {
                    <SetMultisendAddress as alloy_sol_types::SolEvent>::decode_raw_log(
                            topics,
                            data,
                        )
                        .map(Self::SetMultisendAddress)
                }
                Some(<Upgraded as alloy_sol_types::SolEvent>::SIGNATURE_HASH) => {
                    <Upgraded as alloy_sol_types::SolEvent>::decode_raw_log(topics, data)
                        .map(Self::Upgraded)
                }
                _ => {
                    alloy_sol_types::private::Err(alloy_sol_types::Error::InvalidLog {
                        name: <Self as alloy_sol_types::SolEventInterface>::NAME,
                        log: alloy_sol_types::private::Box::new(
                            alloy_sol_types::private::LogData::new_unchecked(
                                topics.to_vec(),
                                data.to_vec().into(),
                            ),
                        ),
                    })
                }
            }
        }
    }
    #[automatically_derived]
    impl alloy_sol_types::private::IntoLogData for HoprNodeManagementModuleEvents {
        fn to_log_data(&self) -> alloy_sol_types::private::LogData {
            match self {
                Self::AdminChanged(inner) => {
                    alloy_sol_types::private::IntoLogData::to_log_data(inner)
                }
                Self::BeaconUpgraded(inner) => {
                    alloy_sol_types::private::IntoLogData::to_log_data(inner)
                }
                Self::ExecutionFailure(inner) => {
                    alloy_sol_types::private::IntoLogData::to_log_data(inner)
                }
                Self::ExecutionSuccess(inner) => {
                    alloy_sol_types::private::IntoLogData::to_log_data(inner)
                }
                Self::Initialized(inner) => {
                    alloy_sol_types::private::IntoLogData::to_log_data(inner)
                }
                Self::NodeAdded(inner) => {
                    alloy_sol_types::private::IntoLogData::to_log_data(inner)
                }
                Self::NodeRemoved(inner) => {
                    alloy_sol_types::private::IntoLogData::to_log_data(inner)
                }
                Self::OwnershipTransferred(inner) => {
                    alloy_sol_types::private::IntoLogData::to_log_data(inner)
                }
                Self::SetMultisendAddress(inner) => {
                    alloy_sol_types::private::IntoLogData::to_log_data(inner)
                }
                Self::Upgraded(inner) => {
                    alloy_sol_types::private::IntoLogData::to_log_data(inner)
                }
            }
        }
        fn into_log_data(self) -> alloy_sol_types::private::LogData {
            match self {
                Self::AdminChanged(inner) => {
                    alloy_sol_types::private::IntoLogData::into_log_data(inner)
                }
                Self::BeaconUpgraded(inner) => {
                    alloy_sol_types::private::IntoLogData::into_log_data(inner)
                }
                Self::ExecutionFailure(inner) => {
                    alloy_sol_types::private::IntoLogData::into_log_data(inner)
                }
                Self::ExecutionSuccess(inner) => {
                    alloy_sol_types::private::IntoLogData::into_log_data(inner)
                }
                Self::Initialized(inner) => {
                    alloy_sol_types::private::IntoLogData::into_log_data(inner)
                }
                Self::NodeAdded(inner) => {
                    alloy_sol_types::private::IntoLogData::into_log_data(inner)
                }
                Self::NodeRemoved(inner) => {
                    alloy_sol_types::private::IntoLogData::into_log_data(inner)
                }
                Self::OwnershipTransferred(inner) => {
                    alloy_sol_types::private::IntoLogData::into_log_data(inner)
                }
                Self::SetMultisendAddress(inner) => {
                    alloy_sol_types::private::IntoLogData::into_log_data(inner)
                }
                Self::Upgraded(inner) => {
                    alloy_sol_types::private::IntoLogData::into_log_data(inner)
                }
            }
        }
    }
    use alloy::contract as alloy_contract;
    /**Creates a new wrapper around an on-chain [`HoprNodeManagementModule`](self) contract instance.

See the [wrapper's documentation](`HoprNodeManagementModuleInstance`) for more details.*/
    #[inline]
    pub const fn new<
        P: alloy_contract::private::Provider<N>,
        N: alloy_contract::private::Network,
    >(
        address: alloy_sol_types::private::Address,
        provider: P,
    ) -> HoprNodeManagementModuleInstance<P, N> {
        HoprNodeManagementModuleInstance::<P, N>::new(address, provider)
    }
    /**Deploys this contract using the given `provider` and constructor arguments, if any.

Returns a new instance of the contract, if the deployment was successful.

For more fine-grained control over the deployment process, use [`deploy_builder`] instead.*/
    #[inline]
    pub fn deploy<
        P: alloy_contract::private::Provider<N>,
        N: alloy_contract::private::Network,
    >(
        provider: P,
    ) -> impl ::core::future::Future<
        Output = alloy_contract::Result<HoprNodeManagementModuleInstance<P, N>>,
    > {
        HoprNodeManagementModuleInstance::<P, N>::deploy(provider)
    }
    /**Creates a `RawCallBuilder` for deploying this contract using the given `provider`
and constructor arguments, if any.

This is a simple wrapper around creating a `RawCallBuilder` with the data set to
the bytecode concatenated with the constructor's ABI-encoded arguments.*/
    #[inline]
    pub fn deploy_builder<
        P: alloy_contract::private::Provider<N>,
        N: alloy_contract::private::Network,
    >(provider: P) -> alloy_contract::RawCallBuilder<P, N> {
        HoprNodeManagementModuleInstance::<P, N>::deploy_builder(provider)
    }
    /**A [`HoprNodeManagementModule`](self) instance.

Contains type-safe methods for interacting with an on-chain instance of the
[`HoprNodeManagementModule`](self) contract located at a given `address`, using a given
provider `P`.

If the contract bytecode is available (see the [`sol!`](alloy_sol_types::sol!)
documentation on how to provide it), the `deploy` and `deploy_builder` methods can
be used to deploy a new instance of the contract.

See the [module-level documentation](self) for all the available methods.*/
    #[derive(Clone)]
    pub struct HoprNodeManagementModuleInstance<
        P,
        N = alloy_contract::private::Ethereum,
    > {
        address: alloy_sol_types::private::Address,
        provider: P,
        _network: ::core::marker::PhantomData<N>,
    }
    #[automatically_derived]
    impl<P, N> ::core::fmt::Debug for HoprNodeManagementModuleInstance<P, N> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple("HoprNodeManagementModuleInstance")
                .field(&self.address)
                .finish()
        }
    }
    /// Instantiation and getters/setters.
    #[automatically_derived]
    impl<
        P: alloy_contract::private::Provider<N>,
        N: alloy_contract::private::Network,
    > HoprNodeManagementModuleInstance<P, N> {
        /**Creates a new wrapper around an on-chain [`HoprNodeManagementModule`](self) contract instance.

See the [wrapper's documentation](`HoprNodeManagementModuleInstance`) for more details.*/
        #[inline]
        pub const fn new(
            address: alloy_sol_types::private::Address,
            provider: P,
        ) -> Self {
            Self {
                address,
                provider,
                _network: ::core::marker::PhantomData,
            }
        }
        /**Deploys this contract using the given `provider` and constructor arguments, if any.

Returns a new instance of the contract, if the deployment was successful.

For more fine-grained control over the deployment process, use [`deploy_builder`] instead.*/
        #[inline]
        pub async fn deploy(
            provider: P,
        ) -> alloy_contract::Result<HoprNodeManagementModuleInstance<P, N>> {
            let call_builder = Self::deploy_builder(provider);
            let contract_address = call_builder.deploy().await?;
            Ok(Self::new(contract_address, call_builder.provider))
        }
        /**Creates a `RawCallBuilder` for deploying this contract using the given `provider`
and constructor arguments, if any.

This is a simple wrapper around creating a `RawCallBuilder` with the data set to
the bytecode concatenated with the constructor's ABI-encoded arguments.*/
        #[inline]
        pub fn deploy_builder(provider: P) -> alloy_contract::RawCallBuilder<P, N> {
            alloy_contract::RawCallBuilder::new_raw_deploy(
                provider,
                ::core::clone::Clone::clone(&BYTECODE),
            )
        }
        /// Returns a reference to the address.
        #[inline]
        pub const fn address(&self) -> &alloy_sol_types::private::Address {
            &self.address
        }
        /// Sets the address.
        #[inline]
        pub fn set_address(&mut self, address: alloy_sol_types::private::Address) {
            self.address = address;
        }
        /// Sets the address and returns `self`.
        pub fn at(mut self, address: alloy_sol_types::private::Address) -> Self {
            self.set_address(address);
            self
        }
        /// Returns a reference to the provider.
        #[inline]
        pub const fn provider(&self) -> &P {
            &self.provider
        }
    }
    impl<P: ::core::clone::Clone, N> HoprNodeManagementModuleInstance<&P, N> {
        /// Clones the provider and returns a new instance with the cloned provider.
        #[inline]
        pub fn with_cloned_provider(self) -> HoprNodeManagementModuleInstance<P, N> {
            HoprNodeManagementModuleInstance {
                address: self.address,
                provider: ::core::clone::Clone::clone(&self.provider),
                _network: ::core::marker::PhantomData,
            }
        }
    }
    /// Function calls.
    #[automatically_derived]
    impl<
        P: alloy_contract::private::Provider<N>,
        N: alloy_contract::private::Network,
    > HoprNodeManagementModuleInstance<P, N> {
        /// Creates a new call builder using this contract instance's provider and address.
        ///
        /// Note that the call can be any function call, not just those defined in this
        /// contract. Prefer using the other methods for building type-safe contract calls.
        pub fn call_builder<C: alloy_sol_types::SolCall>(
            &self,
            call: &C,
        ) -> alloy_contract::SolCallBuilder<&P, C, N> {
            alloy_contract::SolCallBuilder::new_sol(&self.provider, &self.address, call)
        }
        ///Creates a new call builder for the [`addChannelsAndTokenTarget`] function.
        pub fn addChannelsAndTokenTarget(
            &self,
            defaultTarget: <Target as alloy::sol_types::SolType>::RustType,
        ) -> alloy_contract::SolCallBuilder<&P, addChannelsAndTokenTargetCall, N> {
            self.call_builder(
                &addChannelsAndTokenTargetCall {
                    defaultTarget,
                },
            )
        }
        ///Creates a new call builder for the [`addNode`] function.
        pub fn addNode(
            &self,
            nodeAddress: alloy::sol_types::private::Address,
        ) -> alloy_contract::SolCallBuilder<&P, addNodeCall, N> {
            self.call_builder(&addNodeCall { nodeAddress })
        }
        ///Creates a new call builder for the [`decodeFunctionSigsAndPermissions`] function.
        pub fn decodeFunctionSigsAndPermissions(
            &self,
            encoded: alloy::sol_types::private::FixedBytes<32>,
            length: alloy::sol_types::private::primitives::aliases::U256,
        ) -> alloy_contract::SolCallBuilder<
            &P,
            decodeFunctionSigsAndPermissionsCall,
            N,
        > {
            self.call_builder(
                &decodeFunctionSigsAndPermissionsCall {
                    encoded,
                    length,
                },
            )
        }
        ///Creates a new call builder for the [`encodeFunctionSigsAndPermissions`] function.
        pub fn encodeFunctionSigsAndPermissions(
            &self,
            functionSigs: alloy::sol_types::private::Vec<
                alloy::sol_types::private::FixedBytes<4>,
            >,
            permissions: alloy::sol_types::private::Vec<
                <GranularPermission as alloy::sol_types::SolType>::RustType,
            >,
        ) -> alloy_contract::SolCallBuilder<
            &P,
            encodeFunctionSigsAndPermissionsCall,
            N,
        > {
            self.call_builder(
                &encodeFunctionSigsAndPermissionsCall {
                    functionSigs,
                    permissions,
                },
            )
        }
        ///Creates a new call builder for the [`execTransactionFromModule`] function.
        pub fn execTransactionFromModule(
            &self,
            to: alloy::sol_types::private::Address,
            value: alloy::sol_types::private::primitives::aliases::U256,
            data: alloy::sol_types::private::Bytes,
            operation: <Enum::Operation as alloy::sol_types::SolType>::RustType,
        ) -> alloy_contract::SolCallBuilder<&P, execTransactionFromModuleCall, N> {
            self.call_builder(
                &execTransactionFromModuleCall {
                    to,
                    value,
                    data,
                    operation,
                },
            )
        }
        ///Creates a new call builder for the [`execTransactionFromModuleReturnData`] function.
        pub fn execTransactionFromModuleReturnData(
            &self,
            to: alloy::sol_types::private::Address,
            value: alloy::sol_types::private::primitives::aliases::U256,
            data: alloy::sol_types::private::Bytes,
            operation: <Enum::Operation as alloy::sol_types::SolType>::RustType,
        ) -> alloy_contract::SolCallBuilder<
            &P,
            execTransactionFromModuleReturnDataCall,
            N,
        > {
            self.call_builder(
                &execTransactionFromModuleReturnDataCall {
                    to,
                    value,
                    data,
                    operation,
                },
            )
        }
        ///Creates a new call builder for the [`getGranularPermissions`] function.
        pub fn getGranularPermissions(
            &self,
            capabilityKey: alloy::sol_types::private::FixedBytes<32>,
            pairId: alloy::sol_types::private::FixedBytes<32>,
        ) -> alloy_contract::SolCallBuilder<&P, getGranularPermissionsCall, N> {
            self.call_builder(
                &getGranularPermissionsCall {
                    capabilityKey,
                    pairId,
                },
            )
        }
        ///Creates a new call builder for the [`getTargets`] function.
        pub fn getTargets(
            &self,
        ) -> alloy_contract::SolCallBuilder<&P, getTargetsCall, N> {
            self.call_builder(&getTargetsCall)
        }
        ///Creates a new call builder for the [`includeNode`] function.
        pub fn includeNode(
            &self,
            nodeDefaultTarget: <Target as alloy::sol_types::SolType>::RustType,
        ) -> alloy_contract::SolCallBuilder<&P, includeNodeCall, N> {
            self.call_builder(
                &includeNodeCall {
                    nodeDefaultTarget,
                },
            )
        }
        ///Creates a new call builder for the [`initialize`] function.
        pub fn initialize(
            &self,
            initParams: alloy::sol_types::private::Bytes,
        ) -> alloy_contract::SolCallBuilder<&P, initializeCall, N> {
            self.call_builder(&initializeCall { initParams })
        }
        ///Creates a new call builder for the [`isHoprNodeManagementModule`] function.
        pub fn isHoprNodeManagementModule(
            &self,
        ) -> alloy_contract::SolCallBuilder<&P, isHoprNodeManagementModuleCall, N> {
            self.call_builder(&isHoprNodeManagementModuleCall)
        }
        ///Creates a new call builder for the [`isNode`] function.
        pub fn isNode(
            &self,
            nodeAddress: alloy::sol_types::private::Address,
        ) -> alloy_contract::SolCallBuilder<&P, isNodeCall, N> {
            self.call_builder(&isNodeCall { nodeAddress })
        }
        ///Creates a new call builder for the [`multisend`] function.
        pub fn multisend(&self) -> alloy_contract::SolCallBuilder<&P, multisendCall, N> {
            self.call_builder(&multisendCall)
        }
        ///Creates a new call builder for the [`owner`] function.
        pub fn owner(&self) -> alloy_contract::SolCallBuilder<&P, ownerCall, N> {
            self.call_builder(&ownerCall)
        }
        ///Creates a new call builder for the [`proxiableUUID`] function.
        pub fn proxiableUUID(
            &self,
        ) -> alloy_contract::SolCallBuilder<&P, proxiableUUIDCall, N> {
            self.call_builder(&proxiableUUIDCall)
        }
        ///Creates a new call builder for the [`removeNode`] function.
        pub fn removeNode(
            &self,
            nodeAddress: alloy::sol_types::private::Address,
        ) -> alloy_contract::SolCallBuilder<&P, removeNodeCall, N> {
            self.call_builder(&removeNodeCall { nodeAddress })
        }
        ///Creates a new call builder for the [`renounceOwnership`] function.
        pub fn renounceOwnership(
            &self,
        ) -> alloy_contract::SolCallBuilder<&P, renounceOwnershipCall, N> {
            self.call_builder(&renounceOwnershipCall)
        }
        ///Creates a new call builder for the [`revokeTarget`] function.
        pub fn revokeTarget(
            &self,
            targetAddress: alloy::sol_types::private::Address,
        ) -> alloy_contract::SolCallBuilder<&P, revokeTargetCall, N> {
            self.call_builder(&revokeTargetCall { targetAddress })
        }
        ///Creates a new call builder for the [`scopeChannelsCapabilities`] function.
        pub fn scopeChannelsCapabilities(
            &self,
            targetAddress: alloy::sol_types::private::Address,
            channelId: alloy::sol_types::private::FixedBytes<32>,
            encodedSigsPermissions: alloy::sol_types::private::FixedBytes<32>,
        ) -> alloy_contract::SolCallBuilder<&P, scopeChannelsCapabilitiesCall, N> {
            self.call_builder(
                &scopeChannelsCapabilitiesCall {
                    targetAddress,
                    channelId,
                    encodedSigsPermissions,
                },
            )
        }
        ///Creates a new call builder for the [`scopeSendCapability`] function.
        pub fn scopeSendCapability(
            &self,
            nodeAddress: alloy::sol_types::private::Address,
            beneficiary: alloy::sol_types::private::Address,
            permission: <GranularPermission as alloy::sol_types::SolType>::RustType,
        ) -> alloy_contract::SolCallBuilder<&P, scopeSendCapabilityCall, N> {
            self.call_builder(
                &scopeSendCapabilityCall {
                    nodeAddress,
                    beneficiary,
                    permission,
                },
            )
        }
        ///Creates a new call builder for the [`scopeTargetChannels`] function.
        pub fn scopeTargetChannels(
            &self,
            defaultTarget: <Target as alloy::sol_types::SolType>::RustType,
        ) -> alloy_contract::SolCallBuilder<&P, scopeTargetChannelsCall, N> {
            self.call_builder(
                &scopeTargetChannelsCall {
                    defaultTarget,
                },
            )
        }
        ///Creates a new call builder for the [`scopeTargetSend`] function.
        pub fn scopeTargetSend(
            &self,
            defaultTarget: <Target as alloy::sol_types::SolType>::RustType,
        ) -> alloy_contract::SolCallBuilder<&P, scopeTargetSendCall, N> {
            self.call_builder(
                &scopeTargetSendCall {
                    defaultTarget,
                },
            )
        }
        ///Creates a new call builder for the [`scopeTargetToken`] function.
        pub fn scopeTargetToken(
            &self,
            defaultTarget: <Target as alloy::sol_types::SolType>::RustType,
        ) -> alloy_contract::SolCallBuilder<&P, scopeTargetTokenCall, N> {
            self.call_builder(
                &scopeTargetTokenCall {
                    defaultTarget,
                },
            )
        }
        ///Creates a new call builder for the [`scopeTokenCapabilities`] function.
        pub fn scopeTokenCapabilities(
            &self,
            nodeAddress: alloy::sol_types::private::Address,
            targetAddress: alloy::sol_types::private::Address,
            beneficiary: alloy::sol_types::private::Address,
            encodedSigsPermissions: alloy::sol_types::private::FixedBytes<32>,
        ) -> alloy_contract::SolCallBuilder<&P, scopeTokenCapabilitiesCall, N> {
            self.call_builder(
                &scopeTokenCapabilitiesCall {
                    nodeAddress,
                    targetAddress,
                    beneficiary,
                    encodedSigsPermissions,
                },
            )
        }
        ///Creates a new call builder for the [`setMultisend`] function.
        pub fn setMultisend(
            &self,
            _multisend: alloy::sol_types::private::Address,
        ) -> alloy_contract::SolCallBuilder<&P, setMultisendCall, N> {
            self.call_builder(&setMultisendCall { _multisend })
        }
        ///Creates a new call builder for the [`transferOwnership`] function.
        pub fn transferOwnership(
            &self,
            newOwner: alloy::sol_types::private::Address,
        ) -> alloy_contract::SolCallBuilder<&P, transferOwnershipCall, N> {
            self.call_builder(&transferOwnershipCall { newOwner })
        }
        ///Creates a new call builder for the [`tryGetTarget`] function.
        pub fn tryGetTarget(
            &self,
            targetAddress: alloy::sol_types::private::Address,
        ) -> alloy_contract::SolCallBuilder<&P, tryGetTargetCall, N> {
            self.call_builder(&tryGetTargetCall { targetAddress })
        }
        ///Creates a new call builder for the [`upgradeTo`] function.
        pub fn upgradeTo(
            &self,
            newImplementation: alloy::sol_types::private::Address,
        ) -> alloy_contract::SolCallBuilder<&P, upgradeToCall, N> {
            self.call_builder(&upgradeToCall { newImplementation })
        }
        ///Creates a new call builder for the [`upgradeToAndCall`] function.
        pub fn upgradeToAndCall(
            &self,
            newImplementation: alloy::sol_types::private::Address,
            data: alloy::sol_types::private::Bytes,
        ) -> alloy_contract::SolCallBuilder<&P, upgradeToAndCallCall, N> {
            self.call_builder(
                &upgradeToAndCallCall {
                    newImplementation,
                    data,
                },
            )
        }
    }
    /// Event filters.
    #[automatically_derived]
    impl<
        P: alloy_contract::private::Provider<N>,
        N: alloy_contract::private::Network,
    > HoprNodeManagementModuleInstance<P, N> {
        /// Creates a new event filter using this contract instance's provider and address.
        ///
        /// Note that the type can be any event, not just those defined in this contract.
        /// Prefer using the other methods for building type-safe event filters.
        pub fn event_filter<E: alloy_sol_types::SolEvent>(
            &self,
        ) -> alloy_contract::Event<&P, E, N> {
            alloy_contract::Event::new_sol(&self.provider, &self.address)
        }
        ///Creates a new event filter for the [`AdminChanged`] event.
        pub fn AdminChanged_filter(&self) -> alloy_contract::Event<&P, AdminChanged, N> {
            self.event_filter::<AdminChanged>()
        }
        ///Creates a new event filter for the [`BeaconUpgraded`] event.
        pub fn BeaconUpgraded_filter(
            &self,
        ) -> alloy_contract::Event<&P, BeaconUpgraded, N> {
            self.event_filter::<BeaconUpgraded>()
        }
        ///Creates a new event filter for the [`ExecutionFailure`] event.
        pub fn ExecutionFailure_filter(
            &self,
        ) -> alloy_contract::Event<&P, ExecutionFailure, N> {
            self.event_filter::<ExecutionFailure>()
        }
        ///Creates a new event filter for the [`ExecutionSuccess`] event.
        pub fn ExecutionSuccess_filter(
            &self,
        ) -> alloy_contract::Event<&P, ExecutionSuccess, N> {
            self.event_filter::<ExecutionSuccess>()
        }
        ///Creates a new event filter for the [`Initialized`] event.
        pub fn Initialized_filter(&self) -> alloy_contract::Event<&P, Initialized, N> {
            self.event_filter::<Initialized>()
        }
        ///Creates a new event filter for the [`NodeAdded`] event.
        pub fn NodeAdded_filter(&self) -> alloy_contract::Event<&P, NodeAdded, N> {
            self.event_filter::<NodeAdded>()
        }
        ///Creates a new event filter for the [`NodeRemoved`] event.
        pub fn NodeRemoved_filter(&self) -> alloy_contract::Event<&P, NodeRemoved, N> {
            self.event_filter::<NodeRemoved>()
        }
        ///Creates a new event filter for the [`OwnershipTransferred`] event.
        pub fn OwnershipTransferred_filter(
            &self,
        ) -> alloy_contract::Event<&P, OwnershipTransferred, N> {
            self.event_filter::<OwnershipTransferred>()
        }
        ///Creates a new event filter for the [`SetMultisendAddress`] event.
        pub fn SetMultisendAddress_filter(
            &self,
        ) -> alloy_contract::Event<&P, SetMultisendAddress, N> {
            self.event_filter::<SetMultisendAddress>()
        }
        ///Creates a new event filter for the [`Upgraded`] event.
        pub fn Upgraded_filter(&self) -> alloy_contract::Event<&P, Upgraded, N> {
            self.event_filter::<Upgraded>()
        }
    }
}
