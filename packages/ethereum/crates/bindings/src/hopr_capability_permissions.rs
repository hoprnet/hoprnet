pub use hopr_capability_permissions::*;
/// This module was auto-generated with ethers-rs Abigen.
/// More information at: <https://github.com/gakonst/ethers-rs>
#[allow(
    clippy::enum_variant_names,
    clippy::too_many_arguments,
    clippy::upper_case_acronyms,
    clippy::type_complexity,
    dead_code,
    non_camel_case_types,
)]
pub mod hopr_capability_permissions {
    #[rustfmt::skip]
    const __ABI: &str = "[{\"inputs\":[],\"type\":\"error\",\"name\":\"AddressIsZero\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"ArrayTooLong\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"ArraysDifferentLength\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"CalldataOutOfBounds\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"DefaultPermissionRejected\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"DelegateCallNotAllowed\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"FunctionSignatureTooShort\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"GranularPermissionRejected\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"NoMembership\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"NodePermissionRejected\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"ParameterNotAllowed\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"PermissionNotConfigured\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"SendNotAllowed\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"TargetAddressNotAllowed\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"TargetIsNotScoped\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"TargetIsScoped\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"UnacceptableMultiSendOffset\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"targetAddress\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"RevokedTarget\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"targetAddress\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"bytes32\",\"name\":\"channelId\",\"type\":\"bytes32\",\"components\":[],\"indexed\":true},{\"internalType\":\"bytes4\",\"name\":\"selector\",\"type\":\"bytes4\",\"components\":[],\"indexed\":false},{\"internalType\":\"enum GranularPermission\",\"name\":\"permission\",\"type\":\"uint8\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"ScopedGranularChannelCapability\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"nodeAddress\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"recipientAddress\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"enum GranularPermission\",\"name\":\"permission\",\"type\":\"uint8\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"ScopedGranularSendCapability\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"nodeAddress\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"targetAddress\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"recipientAddress\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"bytes4\",\"name\":\"selector\",\"type\":\"bytes4\",\"components\":[],\"indexed\":false},{\"internalType\":\"enum GranularPermission\",\"name\":\"permission\",\"type\":\"uint8\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"ScopedGranularTokenCapability\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"targetAddress\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"Target\",\"name\":\"target\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"ScopedTargetChannels\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"targetAddress\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"Target\",\"name\":\"target\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"ScopedTargetSend\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"targetAddress\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"Target\",\"name\":\"target\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"ScopedTargetToken\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"APPROVE_SELECTOR\",\"outputs\":[{\"internalType\":\"bytes4\",\"name\":\"\",\"type\":\"bytes4\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"CLOSE_INCOMING_CHANNEL_SELECTOR\",\"outputs\":[{\"internalType\":\"bytes4\",\"name\":\"\",\"type\":\"bytes4\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"FINALIZE_OUTGOING_CHANNEL_CLOSURE_SELECTOR\",\"outputs\":[{\"internalType\":\"bytes4\",\"name\":\"\",\"type\":\"bytes4\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"FUND_CHANNEL_SELECTOR\",\"outputs\":[{\"internalType\":\"bytes4\",\"name\":\"\",\"type\":\"bytes4\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"INITIATE_OUTGOING_CHANNEL_CLOSURE_SELECTOR\",\"outputs\":[{\"internalType\":\"bytes4\",\"name\":\"\",\"type\":\"bytes4\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"REDEEM_TICKET_SELECTOR\",\"outputs\":[{\"internalType\":\"bytes4\",\"name\":\"\",\"type\":\"bytes4\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"SEND_SELECTOR\",\"outputs\":[{\"internalType\":\"bytes4\",\"name\":\"\",\"type\":\"bytes4\",\"components\":[]}]}]";
    ///The parsed JSON ABI of the contract.
    pub static HOPRCAPABILITYPERMISSIONS_ABI: ::ethers::contract::Lazy<
        ::ethers::core::abi::Abi,
    > = ::ethers::contract::Lazy::new(|| {
        ::ethers::core::utils::__serde_json::from_str(__ABI)
            .expect("ABI is always valid")
    });
    #[rustfmt::skip]
    const __BYTECODE: &[u8] = &[
        97,
        1,
        49,
        97,
        0,
        58,
        96,
        11,
        130,
        130,
        130,
        57,
        128,
        81,
        96,
        0,
        26,
        96,
        115,
        20,
        97,
        0,
        45,
        87,
        99,
        78,
        72,
        123,
        113,
        96,
        224,
        27,
        96,
        0,
        82,
        96,
        0,
        96,
        4,
        82,
        96,
        36,
        96,
        0,
        253,
        91,
        48,
        96,
        0,
        82,
        96,
        115,
        129,
        83,
        130,
        129,
        243,
        254,
        115,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        48,
        20,
        96,
        128,
        96,
        64,
        82,
        96,
        4,
        54,
        16,
        96,
        126,
        87,
        96,
        0,
        53,
        96,
        224,
        28,
        128,
        99,
        122,
        117,
        113,
        21,
        17,
        96,
        95,
        87,
        128,
        99,
        122,
        117,
        113,
        21,
        20,
        96,
        199,
        87,
        128,
        99,
        129,
        103,
        182,
        126,
        20,
        96,
        212,
        87,
        128,
        99,
        160,
        134,
        88,
        57,
        20,
        96,
        225,
        87,
        128,
        99,
        182,
        119,
        164,
        15,
        20,
        96,
        238,
        87,
        96,
        0,
        128,
        253,
        91,
        128,
        99,
        81,
        210,
        97,
        75,
        20,
        96,
        131,
        87,
        128,
        99,
        93,
        170,
        158,
        36,
        20,
        96,
        173,
        87,
        128,
        99,
        103,
        138,
        94,
        251,
        20,
        96,
        186,
        87,
        91,
        96,
        0,
        128,
        253,
        91,
        96,
        144,
        99,
        6,
        108,
        70,
        185,
        96,
        225,
        27,
        129,
        86,
        91,
        96,
        64,
        81,
        96,
        1,
        96,
        1,
        96,
        224,
        27,
        3,
        25,
        144,
        145,
        22,
        129,
        82,
        96,
        32,
        1,
        96,
        64,
        81,
        128,
        145,
        3,
        144,
        243,
        91,
        96,
        144,
        99,
        189,
        166,
        95,
        69,
        96,
        224,
        27,
        129,
        86,
        91,
        96,
        144,
        99,
        84,
        162,
        237,
        245,
        96,
        224,
        27,
        129,
        86,
        91,
        96,
        144,
        99,
        77,
        236,
        221,
        227,
        96,
        225,
        27,
        129,
        86,
        91,
        96,
        144,
        99,
        101,
        21,
        20,
        191,
        96,
        224,
        27,
        129,
        86,
        91,
        96,
        144,
        99,
        9,
        94,
        167,
        179,
        96,
        224,
        27,
        129,
        86,
        91,
        96,
        144,
        99,
        10,
        190,
        197,
        143,
        96,
        224,
        27,
        129,
        86,
        254,
        162,
        100,
        105,
        112,
        102,
        115,
        88,
        34,
        18,
        32,
        22,
        96,
        69,
        157,
        17,
        229,
        173,
        46,
        143,
        17,
        115,
        49,
        227,
        93,
        42,
        79,
        84,
        135,
        103,
        226,
        201,
        186,
        142,
        204,
        205,
        86,
        84,
        214,
        141,
        212,
        150,
        83,
        100,
        115,
        111,
        108,
        99,
        67,
        0,
        8,
        19,
        0,
        51,
    ];
    ///The bytecode of the contract.
    pub static HOPRCAPABILITYPERMISSIONS_BYTECODE: ::ethers::core::types::Bytes = ::ethers::core::types::Bytes::from_static(
        __BYTECODE,
    );
    #[rustfmt::skip]
    const __DEPLOYED_BYTECODE: &[u8] = &[
        115,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        48,
        20,
        96,
        128,
        96,
        64,
        82,
        96,
        4,
        54,
        16,
        96,
        126,
        87,
        96,
        0,
        53,
        96,
        224,
        28,
        128,
        99,
        122,
        117,
        113,
        21,
        17,
        96,
        95,
        87,
        128,
        99,
        122,
        117,
        113,
        21,
        20,
        96,
        199,
        87,
        128,
        99,
        129,
        103,
        182,
        126,
        20,
        96,
        212,
        87,
        128,
        99,
        160,
        134,
        88,
        57,
        20,
        96,
        225,
        87,
        128,
        99,
        182,
        119,
        164,
        15,
        20,
        96,
        238,
        87,
        96,
        0,
        128,
        253,
        91,
        128,
        99,
        81,
        210,
        97,
        75,
        20,
        96,
        131,
        87,
        128,
        99,
        93,
        170,
        158,
        36,
        20,
        96,
        173,
        87,
        128,
        99,
        103,
        138,
        94,
        251,
        20,
        96,
        186,
        87,
        91,
        96,
        0,
        128,
        253,
        91,
        96,
        144,
        99,
        6,
        108,
        70,
        185,
        96,
        225,
        27,
        129,
        86,
        91,
        96,
        64,
        81,
        96,
        1,
        96,
        1,
        96,
        224,
        27,
        3,
        25,
        144,
        145,
        22,
        129,
        82,
        96,
        32,
        1,
        96,
        64,
        81,
        128,
        145,
        3,
        144,
        243,
        91,
        96,
        144,
        99,
        189,
        166,
        95,
        69,
        96,
        224,
        27,
        129,
        86,
        91,
        96,
        144,
        99,
        84,
        162,
        237,
        245,
        96,
        224,
        27,
        129,
        86,
        91,
        96,
        144,
        99,
        77,
        236,
        221,
        227,
        96,
        225,
        27,
        129,
        86,
        91,
        96,
        144,
        99,
        101,
        21,
        20,
        191,
        96,
        224,
        27,
        129,
        86,
        91,
        96,
        144,
        99,
        9,
        94,
        167,
        179,
        96,
        224,
        27,
        129,
        86,
        91,
        96,
        144,
        99,
        10,
        190,
        197,
        143,
        96,
        224,
        27,
        129,
        86,
        254,
        162,
        100,
        105,
        112,
        102,
        115,
        88,
        34,
        18,
        32,
        22,
        96,
        69,
        157,
        17,
        229,
        173,
        46,
        143,
        17,
        115,
        49,
        227,
        93,
        42,
        79,
        84,
        135,
        103,
        226,
        201,
        186,
        142,
        204,
        205,
        86,
        84,
        214,
        141,
        212,
        150,
        83,
        100,
        115,
        111,
        108,
        99,
        67,
        0,
        8,
        19,
        0,
        51,
    ];
    ///The deployed bytecode of the contract.
    pub static HOPRCAPABILITYPERMISSIONS_DEPLOYED_BYTECODE: ::ethers::core::types::Bytes = ::ethers::core::types::Bytes::from_static(
        __DEPLOYED_BYTECODE,
    );
    pub struct HoprCapabilityPermissions<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for HoprCapabilityPermissions<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for HoprCapabilityPermissions<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for HoprCapabilityPermissions<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for HoprCapabilityPermissions<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(stringify!(HoprCapabilityPermissions))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> HoprCapabilityPermissions<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    HOPRCAPABILITYPERMISSIONS_ABI.clone(),
                    client,
                ),
            )
        }
        /// Constructs the general purpose `Deployer` instance based on the provided constructor arguments and sends it.
        /// Returns a new instance of a deployer that returns an instance of this contract after sending the transaction
        ///
        /// Notes:
        /// - If there are no constructor arguments, you should pass `()` as the argument.
        /// - The default poll duration is 7 seconds.
        /// - The default number of confirmations is 1 block.
        ///
        ///
        /// # Example
        ///
        /// Generate contract bindings with `abigen!` and deploy a new contract instance.
        ///
        /// *Note*: this requires a `bytecode` and `abi` object in the `greeter.json` artifact.
        ///
        /// ```ignore
        /// # async fn deploy<M: ethers::providers::Middleware>(client: ::std::sync::Arc<M>) {
        ///     abigen!(Greeter, "../greeter.json");
        ///
        ///    let greeter_contract = Greeter::deploy(client, "Hello world!".to_string()).unwrap().send().await.unwrap();
        ///    let msg = greeter_contract.greet().call().await.unwrap();
        /// # }
        /// ```
        pub fn deploy<T: ::ethers::core::abi::Tokenize>(
            client: ::std::sync::Arc<M>,
            constructor_args: T,
        ) -> ::core::result::Result<
            ::ethers::contract::builders::ContractDeployer<M, Self>,
            ::ethers::contract::ContractError<M>,
        > {
            let factory = ::ethers::contract::ContractFactory::new(
                HOPRCAPABILITYPERMISSIONS_ABI.clone(),
                HOPRCAPABILITYPERMISSIONS_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ::ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        ///Calls the contract's `APPROVE_SELECTOR` (0xa0865839) function
        pub fn approve_selector(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 4]> {
            self.0
                .method_hash([160, 134, 88, 57], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `CLOSE_INCOMING_CHANNEL_SELECTOR` (0x678a5efb) function
        pub fn close_incoming_channel_selector(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 4]> {
            self.0
                .method_hash([103, 138, 94, 251], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `FINALIZE_OUTGOING_CHANNEL_CLOSURE_SELECTOR` (0x8167b67e) function
        pub fn finalize_outgoing_channel_closure_selector(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 4]> {
            self.0
                .method_hash([129, 103, 182, 126], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `FUND_CHANNEL_SELECTOR` (0xb677a40f) function
        pub fn fund_channel_selector(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 4]> {
            self.0
                .method_hash([182, 119, 164, 15], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `INITIATE_OUTGOING_CHANNEL_CLOSURE_SELECTOR` (0x5daa9e24) function
        pub fn initiate_outgoing_channel_closure_selector(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 4]> {
            self.0
                .method_hash([93, 170, 158, 36], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `REDEEM_TICKET_SELECTOR` (0x51d2614b) function
        pub fn redeem_ticket_selector(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 4]> {
            self.0
                .method_hash([81, 210, 97, 75], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `SEND_SELECTOR` (0x7a757115) function
        pub fn send_selector(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 4]> {
            self.0
                .method_hash([122, 117, 113, 21], ())
                .expect("method not found (this should never happen)")
        }
        ///Gets the contract's `RevokedTarget` event
        pub fn revoked_target_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            RevokedTargetFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `ScopedGranularChannelCapability` event
        pub fn scoped_granular_channel_capability_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ScopedGranularChannelCapabilityFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `ScopedGranularSendCapability` event
        pub fn scoped_granular_send_capability_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ScopedGranularSendCapabilityFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `ScopedGranularTokenCapability` event
        pub fn scoped_granular_token_capability_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ScopedGranularTokenCapabilityFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `ScopedTargetChannels` event
        pub fn scoped_target_channels_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ScopedTargetChannelsFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `ScopedTargetSend` event
        pub fn scoped_target_send_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ScopedTargetSendFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `ScopedTargetToken` event
        pub fn scoped_target_token_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ScopedTargetTokenFilter,
        > {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            HoprCapabilityPermissionsEvents,
        > {
            self.0.event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for HoprCapabilityPermissions<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Custom Error type `AddressIsZero` with signature `AddressIsZero()` and selector `0x867915ab`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "AddressIsZero", abi = "AddressIsZero()")]
    pub struct AddressIsZero;
    ///Custom Error type `ArrayTooLong` with signature `ArrayTooLong()` and selector `0xbd26cc38`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "ArrayTooLong", abi = "ArrayTooLong()")]
    pub struct ArrayTooLong;
    ///Custom Error type `ArraysDifferentLength` with signature `ArraysDifferentLength()` and selector `0x74f4d537`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "ArraysDifferentLength", abi = "ArraysDifferentLength()")]
    pub struct ArraysDifferentLength;
    ///Custom Error type `CalldataOutOfBounds` with signature `CalldataOutOfBounds()` and selector `0x742638b4`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "CalldataOutOfBounds", abi = "CalldataOutOfBounds()")]
    pub struct CalldataOutOfBounds;
    ///Custom Error type `DefaultPermissionRejected` with signature `DefaultPermissionRejected()` and selector `0x58723037`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "DefaultPermissionRejected", abi = "DefaultPermissionRejected()")]
    pub struct DefaultPermissionRejected;
    ///Custom Error type `DelegateCallNotAllowed` with signature `DelegateCallNotAllowed()` and selector `0x0d89438e`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "DelegateCallNotAllowed", abi = "DelegateCallNotAllowed()")]
    pub struct DelegateCallNotAllowed;
    ///Custom Error type `FunctionSignatureTooShort` with signature `FunctionSignatureTooShort()` and selector `0x4684c122`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "FunctionSignatureTooShort", abi = "FunctionSignatureTooShort()")]
    pub struct FunctionSignatureTooShort;
    ///Custom Error type `GranularPermissionRejected` with signature `GranularPermissionRejected()` and selector `0x864dd1e7`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "GranularPermissionRejected",
        abi = "GranularPermissionRejected()"
    )]
    pub struct GranularPermissionRejected;
    ///Custom Error type `NoMembership` with signature `NoMembership()` and selector `0xfd8e9f28`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "NoMembership", abi = "NoMembership()")]
    pub struct NoMembership;
    ///Custom Error type `NodePermissionRejected` with signature `NodePermissionRejected()` and selector `0x6eb0315f`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "NodePermissionRejected", abi = "NodePermissionRejected()")]
    pub struct NodePermissionRejected;
    ///Custom Error type `ParameterNotAllowed` with signature `ParameterNotAllowed()` and selector `0x31e98246`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "ParameterNotAllowed", abi = "ParameterNotAllowed()")]
    pub struct ParameterNotAllowed;
    ///Custom Error type `PermissionNotConfigured` with signature `PermissionNotConfigured()` and selector `0x46ad4588`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "PermissionNotConfigured", abi = "PermissionNotConfigured()")]
    pub struct PermissionNotConfigured;
    ///Custom Error type `SendNotAllowed` with signature `SendNotAllowed()` and selector `0x09e9cd49`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "SendNotAllowed", abi = "SendNotAllowed()")]
    pub struct SendNotAllowed;
    ///Custom Error type `TargetAddressNotAllowed` with signature `TargetAddressNotAllowed()` and selector `0xef3440ac`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "TargetAddressNotAllowed", abi = "TargetAddressNotAllowed()")]
    pub struct TargetAddressNotAllowed;
    ///Custom Error type `TargetIsNotScoped` with signature `TargetIsNotScoped()` and selector `0x4a890321`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "TargetIsNotScoped", abi = "TargetIsNotScoped()")]
    pub struct TargetIsNotScoped;
    ///Custom Error type `TargetIsScoped` with signature `TargetIsScoped()` and selector `0xe8c07d2a`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "TargetIsScoped", abi = "TargetIsScoped()")]
    pub struct TargetIsScoped;
    ///Custom Error type `UnacceptableMultiSendOffset` with signature `UnacceptableMultiSendOffset()` and selector `0x7ed11137`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "UnacceptableMultiSendOffset",
        abi = "UnacceptableMultiSendOffset()"
    )]
    pub struct UnacceptableMultiSendOffset;
    ///Container type for all of the contract's custom errors
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum HoprCapabilityPermissionsErrors {
        AddressIsZero(AddressIsZero),
        ArrayTooLong(ArrayTooLong),
        ArraysDifferentLength(ArraysDifferentLength),
        CalldataOutOfBounds(CalldataOutOfBounds),
        DefaultPermissionRejected(DefaultPermissionRejected),
        DelegateCallNotAllowed(DelegateCallNotAllowed),
        FunctionSignatureTooShort(FunctionSignatureTooShort),
        GranularPermissionRejected(GranularPermissionRejected),
        NoMembership(NoMembership),
        NodePermissionRejected(NodePermissionRejected),
        ParameterNotAllowed(ParameterNotAllowed),
        PermissionNotConfigured(PermissionNotConfigured),
        SendNotAllowed(SendNotAllowed),
        TargetAddressNotAllowed(TargetAddressNotAllowed),
        TargetIsNotScoped(TargetIsNotScoped),
        TargetIsScoped(TargetIsScoped),
        UnacceptableMultiSendOffset(UnacceptableMultiSendOffset),
        /// The standard solidity revert string, with selector
        /// Error(string) -- 0x08c379a0
        RevertString(::std::string::String),
    }
    impl ::ethers::core::abi::AbiDecode for HoprCapabilityPermissionsErrors {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded)
                = <::std::string::String as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::RevertString(decoded));
            }
            if let Ok(decoded)
                = <AddressIsZero as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::AddressIsZero(decoded));
            }
            if let Ok(decoded)
                = <ArrayTooLong as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ArrayTooLong(decoded));
            }
            if let Ok(decoded)
                = <ArraysDifferentLength as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::ArraysDifferentLength(decoded));
            }
            if let Ok(decoded)
                = <CalldataOutOfBounds as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::CalldataOutOfBounds(decoded));
            }
            if let Ok(decoded)
                = <DefaultPermissionRejected as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::DefaultPermissionRejected(decoded));
            }
            if let Ok(decoded)
                = <DelegateCallNotAllowed as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::DelegateCallNotAllowed(decoded));
            }
            if let Ok(decoded)
                = <FunctionSignatureTooShort as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::FunctionSignatureTooShort(decoded));
            }
            if let Ok(decoded)
                = <GranularPermissionRejected as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::GranularPermissionRejected(decoded));
            }
            if let Ok(decoded)
                = <NoMembership as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::NoMembership(decoded));
            }
            if let Ok(decoded)
                = <NodePermissionRejected as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::NodePermissionRejected(decoded));
            }
            if let Ok(decoded)
                = <ParameterNotAllowed as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ParameterNotAllowed(decoded));
            }
            if let Ok(decoded)
                = <PermissionNotConfigured as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::PermissionNotConfigured(decoded));
            }
            if let Ok(decoded)
                = <SendNotAllowed as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::SendNotAllowed(decoded));
            }
            if let Ok(decoded)
                = <TargetAddressNotAllowed as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::TargetAddressNotAllowed(decoded));
            }
            if let Ok(decoded)
                = <TargetIsNotScoped as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::TargetIsNotScoped(decoded));
            }
            if let Ok(decoded)
                = <TargetIsScoped as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::TargetIsScoped(decoded));
            }
            if let Ok(decoded)
                = <UnacceptableMultiSendOffset as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::UnacceptableMultiSendOffset(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for HoprCapabilityPermissionsErrors {
        fn encode(self) -> ::std::vec::Vec<u8> {
            match self {
                Self::AddressIsZero(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ArrayTooLong(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ArraysDifferentLength(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CalldataOutOfBounds(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::DefaultPermissionRejected(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::DelegateCallNotAllowed(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::FunctionSignatureTooShort(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GranularPermissionRejected(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NoMembership(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NodePermissionRejected(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ParameterNotAllowed(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PermissionNotConfigured(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SendNotAllowed(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::TargetAddressNotAllowed(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::TargetIsNotScoped(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::TargetIsScoped(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::UnacceptableMultiSendOffset(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RevertString(s) => ::ethers::core::abi::AbiEncode::encode(s),
            }
        }
    }
    impl ::ethers::contract::ContractRevert for HoprCapabilityPermissionsErrors {
        fn valid_selector(selector: [u8; 4]) -> bool {
            match selector {
                [0x08, 0xc3, 0x79, 0xa0] => true,
                _ if selector
                    == <AddressIsZero as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <ArrayTooLong as ::ethers::contract::EthError>::selector() => true,
                _ if selector
                    == <ArraysDifferentLength as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <CalldataOutOfBounds as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <DefaultPermissionRejected as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <DelegateCallNotAllowed as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <FunctionSignatureTooShort as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <GranularPermissionRejected as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NoMembership as ::ethers::contract::EthError>::selector() => true,
                _ if selector
                    == <NodePermissionRejected as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <ParameterNotAllowed as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <PermissionNotConfigured as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <SendNotAllowed as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <TargetAddressNotAllowed as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <TargetIsNotScoped as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <TargetIsScoped as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <UnacceptableMultiSendOffset as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ => false,
            }
        }
    }
    impl ::core::fmt::Display for HoprCapabilityPermissionsErrors {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AddressIsZero(element) => ::core::fmt::Display::fmt(element, f),
                Self::ArrayTooLong(element) => ::core::fmt::Display::fmt(element, f),
                Self::ArraysDifferentLength(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CalldataOutOfBounds(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::DefaultPermissionRejected(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::DelegateCallNotAllowed(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::FunctionSignatureTooShort(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GranularPermissionRejected(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NoMembership(element) => ::core::fmt::Display::fmt(element, f),
                Self::NodePermissionRejected(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ParameterNotAllowed(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::PermissionNotConfigured(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SendNotAllowed(element) => ::core::fmt::Display::fmt(element, f),
                Self::TargetAddressNotAllowed(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::TargetIsNotScoped(element) => ::core::fmt::Display::fmt(element, f),
                Self::TargetIsScoped(element) => ::core::fmt::Display::fmt(element, f),
                Self::UnacceptableMultiSendOffset(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RevertString(s) => ::core::fmt::Display::fmt(s, f),
            }
        }
    }
    impl ::core::convert::From<::std::string::String>
    for HoprCapabilityPermissionsErrors {
        fn from(value: String) -> Self {
            Self::RevertString(value)
        }
    }
    impl ::core::convert::From<AddressIsZero> for HoprCapabilityPermissionsErrors {
        fn from(value: AddressIsZero) -> Self {
            Self::AddressIsZero(value)
        }
    }
    impl ::core::convert::From<ArrayTooLong> for HoprCapabilityPermissionsErrors {
        fn from(value: ArrayTooLong) -> Self {
            Self::ArrayTooLong(value)
        }
    }
    impl ::core::convert::From<ArraysDifferentLength>
    for HoprCapabilityPermissionsErrors {
        fn from(value: ArraysDifferentLength) -> Self {
            Self::ArraysDifferentLength(value)
        }
    }
    impl ::core::convert::From<CalldataOutOfBounds> for HoprCapabilityPermissionsErrors {
        fn from(value: CalldataOutOfBounds) -> Self {
            Self::CalldataOutOfBounds(value)
        }
    }
    impl ::core::convert::From<DefaultPermissionRejected>
    for HoprCapabilityPermissionsErrors {
        fn from(value: DefaultPermissionRejected) -> Self {
            Self::DefaultPermissionRejected(value)
        }
    }
    impl ::core::convert::From<DelegateCallNotAllowed>
    for HoprCapabilityPermissionsErrors {
        fn from(value: DelegateCallNotAllowed) -> Self {
            Self::DelegateCallNotAllowed(value)
        }
    }
    impl ::core::convert::From<FunctionSignatureTooShort>
    for HoprCapabilityPermissionsErrors {
        fn from(value: FunctionSignatureTooShort) -> Self {
            Self::FunctionSignatureTooShort(value)
        }
    }
    impl ::core::convert::From<GranularPermissionRejected>
    for HoprCapabilityPermissionsErrors {
        fn from(value: GranularPermissionRejected) -> Self {
            Self::GranularPermissionRejected(value)
        }
    }
    impl ::core::convert::From<NoMembership> for HoprCapabilityPermissionsErrors {
        fn from(value: NoMembership) -> Self {
            Self::NoMembership(value)
        }
    }
    impl ::core::convert::From<NodePermissionRejected>
    for HoprCapabilityPermissionsErrors {
        fn from(value: NodePermissionRejected) -> Self {
            Self::NodePermissionRejected(value)
        }
    }
    impl ::core::convert::From<ParameterNotAllowed> for HoprCapabilityPermissionsErrors {
        fn from(value: ParameterNotAllowed) -> Self {
            Self::ParameterNotAllowed(value)
        }
    }
    impl ::core::convert::From<PermissionNotConfigured>
    for HoprCapabilityPermissionsErrors {
        fn from(value: PermissionNotConfigured) -> Self {
            Self::PermissionNotConfigured(value)
        }
    }
    impl ::core::convert::From<SendNotAllowed> for HoprCapabilityPermissionsErrors {
        fn from(value: SendNotAllowed) -> Self {
            Self::SendNotAllowed(value)
        }
    }
    impl ::core::convert::From<TargetAddressNotAllowed>
    for HoprCapabilityPermissionsErrors {
        fn from(value: TargetAddressNotAllowed) -> Self {
            Self::TargetAddressNotAllowed(value)
        }
    }
    impl ::core::convert::From<TargetIsNotScoped> for HoprCapabilityPermissionsErrors {
        fn from(value: TargetIsNotScoped) -> Self {
            Self::TargetIsNotScoped(value)
        }
    }
    impl ::core::convert::From<TargetIsScoped> for HoprCapabilityPermissionsErrors {
        fn from(value: TargetIsScoped) -> Self {
            Self::TargetIsScoped(value)
        }
    }
    impl ::core::convert::From<UnacceptableMultiSendOffset>
    for HoprCapabilityPermissionsErrors {
        fn from(value: UnacceptableMultiSendOffset) -> Self {
            Self::UnacceptableMultiSendOffset(value)
        }
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "RevokedTarget", abi = "RevokedTarget(address)")]
    pub struct RevokedTargetFilter {
        #[ethevent(indexed)]
        pub target_address: ::ethers::core::types::Address,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "ScopedGranularChannelCapability",
        abi = "ScopedGranularChannelCapability(address,bytes32,bytes4,uint8)"
    )]
    pub struct ScopedGranularChannelCapabilityFilter {
        #[ethevent(indexed)]
        pub target_address: ::ethers::core::types::Address,
        #[ethevent(indexed)]
        pub channel_id: [u8; 32],
        pub selector: [u8; 4],
        pub permission: u8,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "ScopedGranularSendCapability",
        abi = "ScopedGranularSendCapability(address,address,uint8)"
    )]
    pub struct ScopedGranularSendCapabilityFilter {
        #[ethevent(indexed)]
        pub node_address: ::ethers::core::types::Address,
        #[ethevent(indexed)]
        pub recipient_address: ::ethers::core::types::Address,
        pub permission: u8,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "ScopedGranularTokenCapability",
        abi = "ScopedGranularTokenCapability(address,address,address,bytes4,uint8)"
    )]
    pub struct ScopedGranularTokenCapabilityFilter {
        #[ethevent(indexed)]
        pub node_address: ::ethers::core::types::Address,
        #[ethevent(indexed)]
        pub target_address: ::ethers::core::types::Address,
        #[ethevent(indexed)]
        pub recipient_address: ::ethers::core::types::Address,
        pub selector: [u8; 4],
        pub permission: u8,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "ScopedTargetChannels",
        abi = "ScopedTargetChannels(address,uint256)"
    )]
    pub struct ScopedTargetChannelsFilter {
        #[ethevent(indexed)]
        pub target_address: ::ethers::core::types::Address,
        pub target: ::ethers::core::types::U256,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "ScopedTargetSend", abi = "ScopedTargetSend(address,uint256)")]
    pub struct ScopedTargetSendFilter {
        #[ethevent(indexed)]
        pub target_address: ::ethers::core::types::Address,
        pub target: ::ethers::core::types::U256,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "ScopedTargetToken", abi = "ScopedTargetToken(address,uint256)")]
    pub struct ScopedTargetTokenFilter {
        #[ethevent(indexed)]
        pub target_address: ::ethers::core::types::Address,
        pub target: ::ethers::core::types::U256,
    }
    ///Container type for all of the contract's events
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum HoprCapabilityPermissionsEvents {
        RevokedTargetFilter(RevokedTargetFilter),
        ScopedGranularChannelCapabilityFilter(ScopedGranularChannelCapabilityFilter),
        ScopedGranularSendCapabilityFilter(ScopedGranularSendCapabilityFilter),
        ScopedGranularTokenCapabilityFilter(ScopedGranularTokenCapabilityFilter),
        ScopedTargetChannelsFilter(ScopedTargetChannelsFilter),
        ScopedTargetSendFilter(ScopedTargetSendFilter),
        ScopedTargetTokenFilter(ScopedTargetTokenFilter),
    }
    impl ::ethers::contract::EthLogDecode for HoprCapabilityPermissionsEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::Error> {
            if let Ok(decoded) = RevokedTargetFilter::decode_log(log) {
                return Ok(HoprCapabilityPermissionsEvents::RevokedTargetFilter(decoded));
            }
            if let Ok(decoded) = ScopedGranularChannelCapabilityFilter::decode_log(log) {
                return Ok(
                    HoprCapabilityPermissionsEvents::ScopedGranularChannelCapabilityFilter(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded) = ScopedGranularSendCapabilityFilter::decode_log(log) {
                return Ok(
                    HoprCapabilityPermissionsEvents::ScopedGranularSendCapabilityFilter(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded) = ScopedGranularTokenCapabilityFilter::decode_log(log) {
                return Ok(
                    HoprCapabilityPermissionsEvents::ScopedGranularTokenCapabilityFilter(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded) = ScopedTargetChannelsFilter::decode_log(log) {
                return Ok(
                    HoprCapabilityPermissionsEvents::ScopedTargetChannelsFilter(decoded),
                );
            }
            if let Ok(decoded) = ScopedTargetSendFilter::decode_log(log) {
                return Ok(
                    HoprCapabilityPermissionsEvents::ScopedTargetSendFilter(decoded),
                );
            }
            if let Ok(decoded) = ScopedTargetTokenFilter::decode_log(log) {
                return Ok(
                    HoprCapabilityPermissionsEvents::ScopedTargetTokenFilter(decoded),
                );
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::core::fmt::Display for HoprCapabilityPermissionsEvents {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::RevokedTargetFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ScopedGranularChannelCapabilityFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ScopedGranularSendCapabilityFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ScopedGranularTokenCapabilityFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ScopedTargetChannelsFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ScopedTargetSendFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ScopedTargetTokenFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<RevokedTargetFilter> for HoprCapabilityPermissionsEvents {
        fn from(value: RevokedTargetFilter) -> Self {
            Self::RevokedTargetFilter(value)
        }
    }
    impl ::core::convert::From<ScopedGranularChannelCapabilityFilter>
    for HoprCapabilityPermissionsEvents {
        fn from(value: ScopedGranularChannelCapabilityFilter) -> Self {
            Self::ScopedGranularChannelCapabilityFilter(value)
        }
    }
    impl ::core::convert::From<ScopedGranularSendCapabilityFilter>
    for HoprCapabilityPermissionsEvents {
        fn from(value: ScopedGranularSendCapabilityFilter) -> Self {
            Self::ScopedGranularSendCapabilityFilter(value)
        }
    }
    impl ::core::convert::From<ScopedGranularTokenCapabilityFilter>
    for HoprCapabilityPermissionsEvents {
        fn from(value: ScopedGranularTokenCapabilityFilter) -> Self {
            Self::ScopedGranularTokenCapabilityFilter(value)
        }
    }
    impl ::core::convert::From<ScopedTargetChannelsFilter>
    for HoprCapabilityPermissionsEvents {
        fn from(value: ScopedTargetChannelsFilter) -> Self {
            Self::ScopedTargetChannelsFilter(value)
        }
    }
    impl ::core::convert::From<ScopedTargetSendFilter>
    for HoprCapabilityPermissionsEvents {
        fn from(value: ScopedTargetSendFilter) -> Self {
            Self::ScopedTargetSendFilter(value)
        }
    }
    impl ::core::convert::From<ScopedTargetTokenFilter>
    for HoprCapabilityPermissionsEvents {
        fn from(value: ScopedTargetTokenFilter) -> Self {
            Self::ScopedTargetTokenFilter(value)
        }
    }
    ///Container type for all input parameters for the `APPROVE_SELECTOR` function with signature `APPROVE_SELECTOR()` and selector `0xa0865839`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "APPROVE_SELECTOR", abi = "APPROVE_SELECTOR()")]
    pub struct ApproveSelectorCall;
    ///Container type for all input parameters for the `CLOSE_INCOMING_CHANNEL_SELECTOR` function with signature `CLOSE_INCOMING_CHANNEL_SELECTOR()` and selector `0x678a5efb`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "CLOSE_INCOMING_CHANNEL_SELECTOR",
        abi = "CLOSE_INCOMING_CHANNEL_SELECTOR()"
    )]
    pub struct CloseIncomingChannelSelectorCall;
    ///Container type for all input parameters for the `FINALIZE_OUTGOING_CHANNEL_CLOSURE_SELECTOR` function with signature `FINALIZE_OUTGOING_CHANNEL_CLOSURE_SELECTOR()` and selector `0x8167b67e`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "FINALIZE_OUTGOING_CHANNEL_CLOSURE_SELECTOR",
        abi = "FINALIZE_OUTGOING_CHANNEL_CLOSURE_SELECTOR()"
    )]
    pub struct FinalizeOutgoingChannelClosureSelectorCall;
    ///Container type for all input parameters for the `FUND_CHANNEL_SELECTOR` function with signature `FUND_CHANNEL_SELECTOR()` and selector `0xb677a40f`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "FUND_CHANNEL_SELECTOR", abi = "FUND_CHANNEL_SELECTOR()")]
    pub struct FundChannelSelectorCall;
    ///Container type for all input parameters for the `INITIATE_OUTGOING_CHANNEL_CLOSURE_SELECTOR` function with signature `INITIATE_OUTGOING_CHANNEL_CLOSURE_SELECTOR()` and selector `0x5daa9e24`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "INITIATE_OUTGOING_CHANNEL_CLOSURE_SELECTOR",
        abi = "INITIATE_OUTGOING_CHANNEL_CLOSURE_SELECTOR()"
    )]
    pub struct InitiateOutgoingChannelClosureSelectorCall;
    ///Container type for all input parameters for the `REDEEM_TICKET_SELECTOR` function with signature `REDEEM_TICKET_SELECTOR()` and selector `0x51d2614b`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "REDEEM_TICKET_SELECTOR", abi = "REDEEM_TICKET_SELECTOR()")]
    pub struct RedeemTicketSelectorCall;
    ///Container type for all input parameters for the `SEND_SELECTOR` function with signature `SEND_SELECTOR()` and selector `0x7a757115`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "SEND_SELECTOR", abi = "SEND_SELECTOR()")]
    pub struct SendSelectorCall;
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum HoprCapabilityPermissionsCalls {
        ApproveSelector(ApproveSelectorCall),
        CloseIncomingChannelSelector(CloseIncomingChannelSelectorCall),
        FinalizeOutgoingChannelClosureSelector(
            FinalizeOutgoingChannelClosureSelectorCall,
        ),
        FundChannelSelector(FundChannelSelectorCall),
        InitiateOutgoingChannelClosureSelector(
            InitiateOutgoingChannelClosureSelectorCall,
        ),
        RedeemTicketSelector(RedeemTicketSelectorCall),
        SendSelector(SendSelectorCall),
    }
    impl ::ethers::core::abi::AbiDecode for HoprCapabilityPermissionsCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded)
                = <ApproveSelectorCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ApproveSelector(decoded));
            }
            if let Ok(decoded)
                = <CloseIncomingChannelSelectorCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::CloseIncomingChannelSelector(decoded));
            }
            if let Ok(decoded)
                = <FinalizeOutgoingChannelClosureSelectorCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::FinalizeOutgoingChannelClosureSelector(decoded));
            }
            if let Ok(decoded)
                = <FundChannelSelectorCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::FundChannelSelector(decoded));
            }
            if let Ok(decoded)
                = <InitiateOutgoingChannelClosureSelectorCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::InitiateOutgoingChannelClosureSelector(decoded));
            }
            if let Ok(decoded)
                = <RedeemTicketSelectorCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::RedeemTicketSelector(decoded));
            }
            if let Ok(decoded)
                = <SendSelectorCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::SendSelector(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for HoprCapabilityPermissionsCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::ApproveSelector(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CloseIncomingChannelSelector(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::FinalizeOutgoingChannelClosureSelector(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::FundChannelSelector(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InitiateOutgoingChannelClosureSelector(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RedeemTicketSelector(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SendSelector(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for HoprCapabilityPermissionsCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::ApproveSelector(element) => ::core::fmt::Display::fmt(element, f),
                Self::CloseIncomingChannelSelector(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::FinalizeOutgoingChannelClosureSelector(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::FundChannelSelector(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InitiateOutgoingChannelClosureSelector(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RedeemTicketSelector(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SendSelector(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<ApproveSelectorCall> for HoprCapabilityPermissionsCalls {
        fn from(value: ApproveSelectorCall) -> Self {
            Self::ApproveSelector(value)
        }
    }
    impl ::core::convert::From<CloseIncomingChannelSelectorCall>
    for HoprCapabilityPermissionsCalls {
        fn from(value: CloseIncomingChannelSelectorCall) -> Self {
            Self::CloseIncomingChannelSelector(value)
        }
    }
    impl ::core::convert::From<FinalizeOutgoingChannelClosureSelectorCall>
    for HoprCapabilityPermissionsCalls {
        fn from(value: FinalizeOutgoingChannelClosureSelectorCall) -> Self {
            Self::FinalizeOutgoingChannelClosureSelector(value)
        }
    }
    impl ::core::convert::From<FundChannelSelectorCall>
    for HoprCapabilityPermissionsCalls {
        fn from(value: FundChannelSelectorCall) -> Self {
            Self::FundChannelSelector(value)
        }
    }
    impl ::core::convert::From<InitiateOutgoingChannelClosureSelectorCall>
    for HoprCapabilityPermissionsCalls {
        fn from(value: InitiateOutgoingChannelClosureSelectorCall) -> Self {
            Self::InitiateOutgoingChannelClosureSelector(value)
        }
    }
    impl ::core::convert::From<RedeemTicketSelectorCall>
    for HoprCapabilityPermissionsCalls {
        fn from(value: RedeemTicketSelectorCall) -> Self {
            Self::RedeemTicketSelector(value)
        }
    }
    impl ::core::convert::From<SendSelectorCall> for HoprCapabilityPermissionsCalls {
        fn from(value: SendSelectorCall) -> Self {
            Self::SendSelector(value)
        }
    }
    ///Container type for all return fields from the `APPROVE_SELECTOR` function with signature `APPROVE_SELECTOR()` and selector `0xa0865839`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ApproveSelectorReturn(pub [u8; 4]);
    ///Container type for all return fields from the `CLOSE_INCOMING_CHANNEL_SELECTOR` function with signature `CLOSE_INCOMING_CHANNEL_SELECTOR()` and selector `0x678a5efb`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct CloseIncomingChannelSelectorReturn(pub [u8; 4]);
    ///Container type for all return fields from the `FINALIZE_OUTGOING_CHANNEL_CLOSURE_SELECTOR` function with signature `FINALIZE_OUTGOING_CHANNEL_CLOSURE_SELECTOR()` and selector `0x8167b67e`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct FinalizeOutgoingChannelClosureSelectorReturn(pub [u8; 4]);
    ///Container type for all return fields from the `FUND_CHANNEL_SELECTOR` function with signature `FUND_CHANNEL_SELECTOR()` and selector `0xb677a40f`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct FundChannelSelectorReturn(pub [u8; 4]);
    ///Container type for all return fields from the `INITIATE_OUTGOING_CHANNEL_CLOSURE_SELECTOR` function with signature `INITIATE_OUTGOING_CHANNEL_CLOSURE_SELECTOR()` and selector `0x5daa9e24`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct InitiateOutgoingChannelClosureSelectorReturn(pub [u8; 4]);
    ///Container type for all return fields from the `REDEEM_TICKET_SELECTOR` function with signature `REDEEM_TICKET_SELECTOR()` and selector `0x51d2614b`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct RedeemTicketSelectorReturn(pub [u8; 4]);
    ///Container type for all return fields from the `SEND_SELECTOR` function with signature `SEND_SELECTOR()` and selector `0x7a757115`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct SendSelectorReturn(pub [u8; 4]);
}
