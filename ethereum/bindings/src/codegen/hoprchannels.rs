///Module containing a contract's types and functions.
/**

```solidity
library HoprCrypto {
    struct CompactSignature { bytes32 r; bytes32 vs; }
    struct VRFParameters { uint256 vx; uint256 vy; uint256 s; uint256 h; uint256 sBx; uint256 sBy; uint256 hVx; uint256 hVy; }
}
```*/
#[allow(
    non_camel_case_types,
    non_snake_case,
    clippy::pub_underscore_fields,
    clippy::style,
    clippy::empty_structs_with_brackets
)]
pub mod HoprCrypto {
    use super::*;
    use alloy::sol_types as alloy_sol_types;
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**```solidity
struct CompactSignature { bytes32 r; bytes32 vs; }
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct CompactSignature {
        #[allow(missing_docs)]
        pub r: alloy::sol_types::private::FixedBytes<32>,
        #[allow(missing_docs)]
        pub vs: alloy::sol_types::private::FixedBytes<32>,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
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
        impl ::core::convert::From<CompactSignature> for UnderlyingRustTuple<'_> {
            fn from(value: CompactSignature) -> Self {
                (value.r, value.vs)
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for CompactSignature {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self { r: tuple.0, vs: tuple.1 }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolValue for CompactSignature {
            type SolType = Self;
        }
        #[automatically_derived]
        impl alloy_sol_types::private::SolTypeValue<Self> for CompactSignature {
            #[inline]
            fn stv_to_tokens(&self) -> <Self as alloy_sol_types::SolType>::Token<'_> {
                (
                    <alloy::sol_types::sol_data::FixedBytes<
                        32,
                    > as alloy_sol_types::SolType>::tokenize(&self.r),
                    <alloy::sol_types::sol_data::FixedBytes<
                        32,
                    > as alloy_sol_types::SolType>::tokenize(&self.vs),
                )
            }
            #[inline]
            fn stv_abi_encoded_size(&self) -> usize {
                if let Some(size) = <Self as alloy_sol_types::SolType>::ENCODED_SIZE {
                    return size;
                }
                let tuple = <UnderlyingRustTuple<
                    '_,
                > as ::core::convert::From<Self>>::from(self.clone());
                <UnderlyingSolTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_encoded_size(&tuple)
            }
            #[inline]
            fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
                <Self as alloy_sol_types::SolStruct>::eip712_hash_struct(self)
            }
            #[inline]
            fn stv_abi_encode_packed_to(
                &self,
                out: &mut alloy_sol_types::private::Vec<u8>,
            ) {
                let tuple = <UnderlyingRustTuple<
                    '_,
                > as ::core::convert::From<Self>>::from(self.clone());
                <UnderlyingSolTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_encode_packed_to(&tuple, out)
            }
            #[inline]
            fn stv_abi_packed_encoded_size(&self) -> usize {
                if let Some(size) = <Self as alloy_sol_types::SolType>::PACKED_ENCODED_SIZE {
                    return size;
                }
                let tuple = <UnderlyingRustTuple<
                    '_,
                > as ::core::convert::From<Self>>::from(self.clone());
                <UnderlyingSolTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_packed_encoded_size(&tuple)
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolType for CompactSignature {
            type RustType = Self;
            type Token<'a> = <UnderlyingSolTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SOL_NAME: &'static str = <Self as alloy_sol_types::SolStruct>::NAME;
            const ENCODED_SIZE: Option<usize> = <UnderlyingSolTuple<
                '_,
            > as alloy_sol_types::SolType>::ENCODED_SIZE;
            const PACKED_ENCODED_SIZE: Option<usize> = <UnderlyingSolTuple<
                '_,
            > as alloy_sol_types::SolType>::PACKED_ENCODED_SIZE;
            #[inline]
            fn valid_token(token: &Self::Token<'_>) -> bool {
                <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::valid_token(token)
            }
            #[inline]
            fn detokenize(token: Self::Token<'_>) -> Self::RustType {
                let tuple = <UnderlyingSolTuple<
                    '_,
                > as alloy_sol_types::SolType>::detokenize(token);
                <Self as ::core::convert::From<UnderlyingRustTuple<'_>>>::from(tuple)
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolStruct for CompactSignature {
            const NAME: &'static str = "CompactSignature";
            #[inline]
            fn eip712_root_type() -> alloy_sol_types::private::Cow<'static, str> {
                alloy_sol_types::private::Cow::Borrowed(
                    "CompactSignature(bytes32 r,bytes32 vs)",
                )
            }
            #[inline]
            fn eip712_components() -> alloy_sol_types::private::Vec<
                alloy_sol_types::private::Cow<'static, str>,
            > {
                alloy_sol_types::private::Vec::new()
            }
            #[inline]
            fn eip712_encode_type() -> alloy_sol_types::private::Cow<'static, str> {
                <Self as alloy_sol_types::SolStruct>::eip712_root_type()
            }
            #[inline]
            fn eip712_encode_data(&self) -> alloy_sol_types::private::Vec<u8> {
                [
                    <alloy::sol_types::sol_data::FixedBytes<
                        32,
                    > as alloy_sol_types::SolType>::eip712_data_word(&self.r)
                        .0,
                    <alloy::sol_types::sol_data::FixedBytes<
                        32,
                    > as alloy_sol_types::SolType>::eip712_data_word(&self.vs)
                        .0,
                ]
                    .concat()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::EventTopic for CompactSignature {
            #[inline]
            fn topic_preimage_length(rust: &Self::RustType) -> usize {
                0usize
                    + <alloy::sol_types::sol_data::FixedBytes<
                        32,
                    > as alloy_sol_types::EventTopic>::topic_preimage_length(&rust.r)
                    + <alloy::sol_types::sol_data::FixedBytes<
                        32,
                    > as alloy_sol_types::EventTopic>::topic_preimage_length(&rust.vs)
            }
            #[inline]
            fn encode_topic_preimage(
                rust: &Self::RustType,
                out: &mut alloy_sol_types::private::Vec<u8>,
            ) {
                out.reserve(
                    <Self as alloy_sol_types::EventTopic>::topic_preimage_length(rust),
                );
                <alloy::sol_types::sol_data::FixedBytes<
                    32,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(&rust.r, out);
                <alloy::sol_types::sol_data::FixedBytes<
                    32,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(&rust.vs, out);
            }
            #[inline]
            fn encode_topic(
                rust: &Self::RustType,
            ) -> alloy_sol_types::abi::token::WordToken {
                let mut out = alloy_sol_types::private::Vec::new();
                <Self as alloy_sol_types::EventTopic>::encode_topic_preimage(
                    rust,
                    &mut out,
                );
                alloy_sol_types::abi::token::WordToken(
                    alloy_sol_types::private::keccak256(out),
                )
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**```solidity
struct VRFParameters { uint256 vx; uint256 vy; uint256 s; uint256 h; uint256 sBx; uint256 sBy; uint256 hVx; uint256 hVy; }
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct VRFParameters {
        #[allow(missing_docs)]
        pub vx: alloy::sol_types::private::primitives::aliases::U256,
        #[allow(missing_docs)]
        pub vy: alloy::sol_types::private::primitives::aliases::U256,
        #[allow(missing_docs)]
        pub s: alloy::sol_types::private::primitives::aliases::U256,
        #[allow(missing_docs)]
        pub h: alloy::sol_types::private::primitives::aliases::U256,
        #[allow(missing_docs)]
        pub sBx: alloy::sol_types::private::primitives::aliases::U256,
        #[allow(missing_docs)]
        pub sBy: alloy::sol_types::private::primitives::aliases::U256,
        #[allow(missing_docs)]
        pub hVx: alloy::sol_types::private::primitives::aliases::U256,
        #[allow(missing_docs)]
        pub hVy: alloy::sol_types::private::primitives::aliases::U256,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = (
            alloy::sol_types::sol_data::Uint<256>,
            alloy::sol_types::sol_data::Uint<256>,
            alloy::sol_types::sol_data::Uint<256>,
            alloy::sol_types::sol_data::Uint<256>,
            alloy::sol_types::sol_data::Uint<256>,
            alloy::sol_types::sol_data::Uint<256>,
            alloy::sol_types::sol_data::Uint<256>,
            alloy::sol_types::sol_data::Uint<256>,
        );
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = (
            alloy::sol_types::private::primitives::aliases::U256,
            alloy::sol_types::private::primitives::aliases::U256,
            alloy::sol_types::private::primitives::aliases::U256,
            alloy::sol_types::private::primitives::aliases::U256,
            alloy::sol_types::private::primitives::aliases::U256,
            alloy::sol_types::private::primitives::aliases::U256,
            alloy::sol_types::private::primitives::aliases::U256,
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
        impl ::core::convert::From<VRFParameters> for UnderlyingRustTuple<'_> {
            fn from(value: VRFParameters) -> Self {
                (
                    value.vx,
                    value.vy,
                    value.s,
                    value.h,
                    value.sBx,
                    value.sBy,
                    value.hVx,
                    value.hVy,
                )
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for VRFParameters {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self {
                    vx: tuple.0,
                    vy: tuple.1,
                    s: tuple.2,
                    h: tuple.3,
                    sBx: tuple.4,
                    sBy: tuple.5,
                    hVx: tuple.6,
                    hVy: tuple.7,
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolValue for VRFParameters {
            type SolType = Self;
        }
        #[automatically_derived]
        impl alloy_sol_types::private::SolTypeValue<Self> for VRFParameters {
            #[inline]
            fn stv_to_tokens(&self) -> <Self as alloy_sol_types::SolType>::Token<'_> {
                (
                    <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::SolType>::tokenize(&self.vx),
                    <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::SolType>::tokenize(&self.vy),
                    <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::SolType>::tokenize(&self.s),
                    <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::SolType>::tokenize(&self.h),
                    <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::SolType>::tokenize(&self.sBx),
                    <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::SolType>::tokenize(&self.sBy),
                    <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::SolType>::tokenize(&self.hVx),
                    <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::SolType>::tokenize(&self.hVy),
                )
            }
            #[inline]
            fn stv_abi_encoded_size(&self) -> usize {
                if let Some(size) = <Self as alloy_sol_types::SolType>::ENCODED_SIZE {
                    return size;
                }
                let tuple = <UnderlyingRustTuple<
                    '_,
                > as ::core::convert::From<Self>>::from(self.clone());
                <UnderlyingSolTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_encoded_size(&tuple)
            }
            #[inline]
            fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
                <Self as alloy_sol_types::SolStruct>::eip712_hash_struct(self)
            }
            #[inline]
            fn stv_abi_encode_packed_to(
                &self,
                out: &mut alloy_sol_types::private::Vec<u8>,
            ) {
                let tuple = <UnderlyingRustTuple<
                    '_,
                > as ::core::convert::From<Self>>::from(self.clone());
                <UnderlyingSolTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_encode_packed_to(&tuple, out)
            }
            #[inline]
            fn stv_abi_packed_encoded_size(&self) -> usize {
                if let Some(size) = <Self as alloy_sol_types::SolType>::PACKED_ENCODED_SIZE {
                    return size;
                }
                let tuple = <UnderlyingRustTuple<
                    '_,
                > as ::core::convert::From<Self>>::from(self.clone());
                <UnderlyingSolTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_packed_encoded_size(&tuple)
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolType for VRFParameters {
            type RustType = Self;
            type Token<'a> = <UnderlyingSolTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SOL_NAME: &'static str = <Self as alloy_sol_types::SolStruct>::NAME;
            const ENCODED_SIZE: Option<usize> = <UnderlyingSolTuple<
                '_,
            > as alloy_sol_types::SolType>::ENCODED_SIZE;
            const PACKED_ENCODED_SIZE: Option<usize> = <UnderlyingSolTuple<
                '_,
            > as alloy_sol_types::SolType>::PACKED_ENCODED_SIZE;
            #[inline]
            fn valid_token(token: &Self::Token<'_>) -> bool {
                <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::valid_token(token)
            }
            #[inline]
            fn detokenize(token: Self::Token<'_>) -> Self::RustType {
                let tuple = <UnderlyingSolTuple<
                    '_,
                > as alloy_sol_types::SolType>::detokenize(token);
                <Self as ::core::convert::From<UnderlyingRustTuple<'_>>>::from(tuple)
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolStruct for VRFParameters {
            const NAME: &'static str = "VRFParameters";
            #[inline]
            fn eip712_root_type() -> alloy_sol_types::private::Cow<'static, str> {
                alloy_sol_types::private::Cow::Borrowed(
                    "VRFParameters(uint256 vx,uint256 vy,uint256 s,uint256 h,uint256 sBx,uint256 sBy,uint256 hVx,uint256 hVy)",
                )
            }
            #[inline]
            fn eip712_components() -> alloy_sol_types::private::Vec<
                alloy_sol_types::private::Cow<'static, str>,
            > {
                alloy_sol_types::private::Vec::new()
            }
            #[inline]
            fn eip712_encode_type() -> alloy_sol_types::private::Cow<'static, str> {
                <Self as alloy_sol_types::SolStruct>::eip712_root_type()
            }
            #[inline]
            fn eip712_encode_data(&self) -> alloy_sol_types::private::Vec<u8> {
                [
                    <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::SolType>::eip712_data_word(&self.vx)
                        .0,
                    <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::SolType>::eip712_data_word(&self.vy)
                        .0,
                    <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::SolType>::eip712_data_word(&self.s)
                        .0,
                    <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::SolType>::eip712_data_word(&self.h)
                        .0,
                    <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::SolType>::eip712_data_word(&self.sBx)
                        .0,
                    <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::SolType>::eip712_data_word(&self.sBy)
                        .0,
                    <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::SolType>::eip712_data_word(&self.hVx)
                        .0,
                    <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::SolType>::eip712_data_word(&self.hVy)
                        .0,
                ]
                    .concat()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::EventTopic for VRFParameters {
            #[inline]
            fn topic_preimage_length(rust: &Self::RustType) -> usize {
                0usize
                    + <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::EventTopic>::topic_preimage_length(&rust.vx)
                    + <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::EventTopic>::topic_preimage_length(&rust.vy)
                    + <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::EventTopic>::topic_preimage_length(&rust.s)
                    + <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::EventTopic>::topic_preimage_length(&rust.h)
                    + <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::EventTopic>::topic_preimage_length(&rust.sBx)
                    + <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::EventTopic>::topic_preimage_length(&rust.sBy)
                    + <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::EventTopic>::topic_preimage_length(&rust.hVx)
                    + <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::EventTopic>::topic_preimage_length(&rust.hVy)
            }
            #[inline]
            fn encode_topic_preimage(
                rust: &Self::RustType,
                out: &mut alloy_sol_types::private::Vec<u8>,
            ) {
                out.reserve(
                    <Self as alloy_sol_types::EventTopic>::topic_preimage_length(rust),
                );
                <alloy::sol_types::sol_data::Uint<
                    256,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(&rust.vx, out);
                <alloy::sol_types::sol_data::Uint<
                    256,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(&rust.vy, out);
                <alloy::sol_types::sol_data::Uint<
                    256,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(&rust.s, out);
                <alloy::sol_types::sol_data::Uint<
                    256,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(&rust.h, out);
                <alloy::sol_types::sol_data::Uint<
                    256,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(&rust.sBx, out);
                <alloy::sol_types::sol_data::Uint<
                    256,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(&rust.sBy, out);
                <alloy::sol_types::sol_data::Uint<
                    256,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(&rust.hVx, out);
                <alloy::sol_types::sol_data::Uint<
                    256,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(&rust.hVy, out);
            }
            #[inline]
            fn encode_topic(
                rust: &Self::RustType,
            ) -> alloy_sol_types::abi::token::WordToken {
                let mut out = alloy_sol_types::private::Vec::new();
                <Self as alloy_sol_types::EventTopic>::encode_topic_preimage(
                    rust,
                    &mut out,
                );
                alloy_sol_types::abi::token::WordToken(
                    alloy_sol_types::private::keccak256(out),
                )
            }
        }
    };
    use alloy::contract as alloy_contract;
    /**Creates a new wrapper around an on-chain [`HoprCrypto`](self) contract instance.

See the [wrapper's documentation](`HoprCryptoInstance`) for more details.*/
    #[inline]
    pub const fn new<
        P: alloy_contract::private::Provider<N>,
        N: alloy_contract::private::Network,
    >(
        address: alloy_sol_types::private::Address,
        provider: P,
    ) -> HoprCryptoInstance<P, N> {
        HoprCryptoInstance::<P, N>::new(address, provider)
    }
    /**A [`HoprCrypto`](self) instance.

Contains type-safe methods for interacting with an on-chain instance of the
[`HoprCrypto`](self) contract located at a given `address`, using a given
provider `P`.

If the contract bytecode is available (see the [`sol!`](alloy_sol_types::sol!)
documentation on how to provide it), the `deploy` and `deploy_builder` methods can
be used to deploy a new instance of the contract.

See the [module-level documentation](self) for all the available methods.*/
    #[derive(Clone)]
    pub struct HoprCryptoInstance<P, N = alloy_contract::private::Ethereum> {
        address: alloy_sol_types::private::Address,
        provider: P,
        _network: ::core::marker::PhantomData<N>,
    }
    #[automatically_derived]
    impl<P, N> ::core::fmt::Debug for HoprCryptoInstance<P, N> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple("HoprCryptoInstance").field(&self.address).finish()
        }
    }
    /// Instantiation and getters/setters.
    #[automatically_derived]
    impl<
        P: alloy_contract::private::Provider<N>,
        N: alloy_contract::private::Network,
    > HoprCryptoInstance<P, N> {
        /**Creates a new wrapper around an on-chain [`HoprCrypto`](self) contract instance.

See the [wrapper's documentation](`HoprCryptoInstance`) for more details.*/
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
    impl<P: ::core::clone::Clone, N> HoprCryptoInstance<&P, N> {
        /// Clones the provider and returns a new instance with the cloned provider.
        #[inline]
        pub fn with_cloned_provider(self) -> HoprCryptoInstance<P, N> {
            HoprCryptoInstance {
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
    > HoprCryptoInstance<P, N> {
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
    > HoprCryptoInstance<P, N> {
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
library HoprCrypto {
    struct CompactSignature {
        bytes32 r;
        bytes32 vs;
    }
    struct VRFParameters {
        uint256 vx;
        uint256 vy;
        uint256 s;
        uint256 h;
        uint256 sBx;
        uint256 sBy;
        uint256 hVx;
        uint256 hVy;
    }
}

interface HoprChannels {
    type ChannelStatus is uint8;
    type Balance is uint96;
    type ChannelEpoch is uint24;
    type TicketIndex is uint48;
    type TicketIndexOffset is uint32;
    type Timestamp is uint32;
    type WinProb is uint56;
    struct RedeemableTicket {
        TicketData data;
        HoprCrypto.CompactSignature signature;
        uint256 porSecret;
    }
    struct TicketData {
        bytes32 channelId;
        Balance amount;
        TicketIndex ticketIndex;
        TicketIndexOffset indexOffset;
        ChannelEpoch epoch;
        WinProb winProb;
    }

    error AlreadyInitialized();
    error BalanceExceedsGlobalPerChannelAllowance();
    error ContractNotResponsible();
    error InsufficientChannelBalance();
    error InvalidAggregatedTicketInterval();
    error InvalidBalance();
    error InvalidCurvePoint();
    error InvalidFieldElement();
    error InvalidNoticePeriod();
    error InvalidPointWitness();
    error InvalidSafeAddress();
    error InvalidTicketSignature();
    error InvalidTokenRecipient();
    error InvalidTokensReceivedUsage();
    error InvalidVRFProof();
    error MultiSigUninitialized();
    error NoticePeriodNotDue();
    error SourceEqualsDestination();
    error TicketIsNotAWin();
    error TokenTransferFailed();
    error WrongChannelState(string reason);
    error WrongToken();
    error ZeroAddress(string reason);

    event ChannelBalanceDecreased(bytes32 indexed channelId, Balance newBalance);
    event ChannelBalanceIncreased(bytes32 indexed channelId, Balance newBalance);
    event ChannelClosed(bytes32 indexed channelId);
    event ChannelOpened(address indexed source, address indexed destination);
    event DomainSeparatorUpdated(bytes32 indexed domainSeparator);
    event LedgerDomainSeparatorUpdated(bytes32 indexed ledgerDomainSeparator);
    event OutgoingChannelClosureInitiated(bytes32 indexed channelId, Timestamp closureTime);
    event TicketRedeemed(bytes32 indexed channelId, TicketIndex newTicketIndex);

    constructor(address _token, Timestamp _noticePeriodChannelClosure, address _safeRegistry);

    function ERC777_HOOK_FUND_CHANNEL_MULTI_SIZE() external view returns (uint256);
    function ERC777_HOOK_FUND_CHANNEL_SIZE() external view returns (uint256);
    function LEDGER_VERSION() external view returns (string memory);
    function MAX_USED_BALANCE() external view returns (Balance);
    function MIN_USED_BALANCE() external view returns (Balance);
    function TOKENS_RECIPIENT_INTERFACE_HASH() external view returns (bytes32);
    function VERSION() external view returns (string memory);
    function _currentBlockTimestamp() external view returns (Timestamp);
    function _getChannelId(address source, address destination) external pure returns (bytes32);
    function _getTicketHash(RedeemableTicket memory redeemable) external view returns (bytes32);
    function _isWinningTicket(bytes32 ticketHash, RedeemableTicket memory redeemable, HoprCrypto.VRFParameters memory params) external pure returns (bool);
    function canImplementInterfaceForAddress(bytes32 interfaceHash, address account) external view returns (bytes32);
    function channels(bytes32) external view returns (Balance balance, TicketIndex ticketIndex, Timestamp closureTime, ChannelEpoch epoch, ChannelStatus status);
    function closeIncomingChannel(address source) external;
    function closeIncomingChannelSafe(address selfAddress, address source) external;
    function domainSeparator() external view returns (bytes32);
    function finalizeOutgoingChannelClosure(address destination) external;
    function finalizeOutgoingChannelClosureSafe(address selfAddress, address destination) external;
    function fundChannel(address account, Balance amount) external;
    function fundChannelSafe(address selfAddress, address account, Balance amount) external;
    function initiateOutgoingChannelClosure(address destination) external;
    function initiateOutgoingChannelClosureSafe(address selfAddress, address destination) external;
    function ledgerDomainSeparator() external view returns (bytes32);
    function multicall(bytes[] memory data) external returns (bytes[] memory results);
    function noticePeriodChannelClosure() external view returns (Timestamp);
    function redeemTicket(RedeemableTicket memory redeemable, HoprCrypto.VRFParameters memory params) external;
    function redeemTicketSafe(address selfAddress, RedeemableTicket memory redeemable, HoprCrypto.VRFParameters memory params) external;
    function token() external view returns (address);
    function tokensReceived(address, address from, address to, uint256 amount, bytes memory userData, bytes memory) external;
    function updateDomainSeparator() external;
    function updateLedgerDomainSeparator() external;
}
```

...which was generated by the following JSON ABI:
```json
[
  {
    "type": "constructor",
    "inputs": [
      {
        "name": "_token",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "_noticePeriodChannelClosure",
        "type": "uint32",
        "internalType": "HoprChannels.Timestamp"
      },
      {
        "name": "_safeRegistry",
        "type": "address",
        "internalType": "contract HoprNodeSafeRegistry"
      }
    ],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "ERC777_HOOK_FUND_CHANNEL_MULTI_SIZE",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "uint256",
        "internalType": "uint256"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "ERC777_HOOK_FUND_CHANNEL_SIZE",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "uint256",
        "internalType": "uint256"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "LEDGER_VERSION",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "string",
        "internalType": "string"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "MAX_USED_BALANCE",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "uint96",
        "internalType": "HoprChannels.Balance"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "MIN_USED_BALANCE",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "uint96",
        "internalType": "HoprChannels.Balance"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "TOKENS_RECIPIENT_INTERFACE_HASH",
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
    "name": "VERSION",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "string",
        "internalType": "string"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "_currentBlockTimestamp",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "uint32",
        "internalType": "HoprChannels.Timestamp"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "_getChannelId",
    "inputs": [
      {
        "name": "source",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "destination",
        "type": "address",
        "internalType": "address"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bytes32",
        "internalType": "bytes32"
      }
    ],
    "stateMutability": "pure"
  },
  {
    "type": "function",
    "name": "_getTicketHash",
    "inputs": [
      {
        "name": "redeemable",
        "type": "tuple",
        "internalType": "struct HoprChannels.RedeemableTicket",
        "components": [
          {
            "name": "data",
            "type": "tuple",
            "internalType": "struct HoprChannels.TicketData",
            "components": [
              {
                "name": "channelId",
                "type": "bytes32",
                "internalType": "bytes32"
              },
              {
                "name": "amount",
                "type": "uint96",
                "internalType": "HoprChannels.Balance"
              },
              {
                "name": "ticketIndex",
                "type": "uint48",
                "internalType": "HoprChannels.TicketIndex"
              },
              {
                "name": "indexOffset",
                "type": "uint32",
                "internalType": "HoprChannels.TicketIndexOffset"
              },
              {
                "name": "epoch",
                "type": "uint24",
                "internalType": "HoprChannels.ChannelEpoch"
              },
              {
                "name": "winProb",
                "type": "uint56",
                "internalType": "HoprChannels.WinProb"
              }
            ]
          },
          {
            "name": "signature",
            "type": "tuple",
            "internalType": "struct HoprCrypto.CompactSignature",
            "components": [
              {
                "name": "r",
                "type": "bytes32",
                "internalType": "bytes32"
              },
              {
                "name": "vs",
                "type": "bytes32",
                "internalType": "bytes32"
              }
            ]
          },
          {
            "name": "porSecret",
            "type": "uint256",
            "internalType": "uint256"
          }
        ]
      }
    ],
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
    "name": "_isWinningTicket",
    "inputs": [
      {
        "name": "ticketHash",
        "type": "bytes32",
        "internalType": "bytes32"
      },
      {
        "name": "redeemable",
        "type": "tuple",
        "internalType": "struct HoprChannels.RedeemableTicket",
        "components": [
          {
            "name": "data",
            "type": "tuple",
            "internalType": "struct HoprChannels.TicketData",
            "components": [
              {
                "name": "channelId",
                "type": "bytes32",
                "internalType": "bytes32"
              },
              {
                "name": "amount",
                "type": "uint96",
                "internalType": "HoprChannels.Balance"
              },
              {
                "name": "ticketIndex",
                "type": "uint48",
                "internalType": "HoprChannels.TicketIndex"
              },
              {
                "name": "indexOffset",
                "type": "uint32",
                "internalType": "HoprChannels.TicketIndexOffset"
              },
              {
                "name": "epoch",
                "type": "uint24",
                "internalType": "HoprChannels.ChannelEpoch"
              },
              {
                "name": "winProb",
                "type": "uint56",
                "internalType": "HoprChannels.WinProb"
              }
            ]
          },
          {
            "name": "signature",
            "type": "tuple",
            "internalType": "struct HoprCrypto.CompactSignature",
            "components": [
              {
                "name": "r",
                "type": "bytes32",
                "internalType": "bytes32"
              },
              {
                "name": "vs",
                "type": "bytes32",
                "internalType": "bytes32"
              }
            ]
          },
          {
            "name": "porSecret",
            "type": "uint256",
            "internalType": "uint256"
          }
        ]
      },
      {
        "name": "params",
        "type": "tuple",
        "internalType": "struct HoprCrypto.VRFParameters",
        "components": [
          {
            "name": "vx",
            "type": "uint256",
            "internalType": "uint256"
          },
          {
            "name": "vy",
            "type": "uint256",
            "internalType": "uint256"
          },
          {
            "name": "s",
            "type": "uint256",
            "internalType": "uint256"
          },
          {
            "name": "h",
            "type": "uint256",
            "internalType": "uint256"
          },
          {
            "name": "sBx",
            "type": "uint256",
            "internalType": "uint256"
          },
          {
            "name": "sBy",
            "type": "uint256",
            "internalType": "uint256"
          },
          {
            "name": "hVx",
            "type": "uint256",
            "internalType": "uint256"
          },
          {
            "name": "hVy",
            "type": "uint256",
            "internalType": "uint256"
          }
        ]
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bool",
        "internalType": "bool"
      }
    ],
    "stateMutability": "pure"
  },
  {
    "type": "function",
    "name": "canImplementInterfaceForAddress",
    "inputs": [
      {
        "name": "interfaceHash",
        "type": "bytes32",
        "internalType": "bytes32"
      },
      {
        "name": "account",
        "type": "address",
        "internalType": "address"
      }
    ],
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
    "name": "channels",
    "inputs": [
      {
        "name": "",
        "type": "bytes32",
        "internalType": "bytes32"
      }
    ],
    "outputs": [
      {
        "name": "balance",
        "type": "uint96",
        "internalType": "HoprChannels.Balance"
      },
      {
        "name": "ticketIndex",
        "type": "uint48",
        "internalType": "HoprChannels.TicketIndex"
      },
      {
        "name": "closureTime",
        "type": "uint32",
        "internalType": "HoprChannels.Timestamp"
      },
      {
        "name": "epoch",
        "type": "uint24",
        "internalType": "HoprChannels.ChannelEpoch"
      },
      {
        "name": "status",
        "type": "uint8",
        "internalType": "enum HoprChannels.ChannelStatus"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "closeIncomingChannel",
    "inputs": [
      {
        "name": "source",
        "type": "address",
        "internalType": "address"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "closeIncomingChannelSafe",
    "inputs": [
      {
        "name": "selfAddress",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "source",
        "type": "address",
        "internalType": "address"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "domainSeparator",
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
    "name": "finalizeOutgoingChannelClosure",
    "inputs": [
      {
        "name": "destination",
        "type": "address",
        "internalType": "address"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "finalizeOutgoingChannelClosureSafe",
    "inputs": [
      {
        "name": "selfAddress",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "destination",
        "type": "address",
        "internalType": "address"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "fundChannel",
    "inputs": [
      {
        "name": "account",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "amount",
        "type": "uint96",
        "internalType": "HoprChannels.Balance"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "fundChannelSafe",
    "inputs": [
      {
        "name": "selfAddress",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "account",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "amount",
        "type": "uint96",
        "internalType": "HoprChannels.Balance"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "initiateOutgoingChannelClosure",
    "inputs": [
      {
        "name": "destination",
        "type": "address",
        "internalType": "address"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "initiateOutgoingChannelClosureSafe",
    "inputs": [
      {
        "name": "selfAddress",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "destination",
        "type": "address",
        "internalType": "address"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "ledgerDomainSeparator",
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
    "name": "multicall",
    "inputs": [
      {
        "name": "data",
        "type": "bytes[]",
        "internalType": "bytes[]"
      }
    ],
    "outputs": [
      {
        "name": "results",
        "type": "bytes[]",
        "internalType": "bytes[]"
      }
    ],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "noticePeriodChannelClosure",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "uint32",
        "internalType": "HoprChannels.Timestamp"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "redeemTicket",
    "inputs": [
      {
        "name": "redeemable",
        "type": "tuple",
        "internalType": "struct HoprChannels.RedeemableTicket",
        "components": [
          {
            "name": "data",
            "type": "tuple",
            "internalType": "struct HoprChannels.TicketData",
            "components": [
              {
                "name": "channelId",
                "type": "bytes32",
                "internalType": "bytes32"
              },
              {
                "name": "amount",
                "type": "uint96",
                "internalType": "HoprChannels.Balance"
              },
              {
                "name": "ticketIndex",
                "type": "uint48",
                "internalType": "HoprChannels.TicketIndex"
              },
              {
                "name": "indexOffset",
                "type": "uint32",
                "internalType": "HoprChannels.TicketIndexOffset"
              },
              {
                "name": "epoch",
                "type": "uint24",
                "internalType": "HoprChannels.ChannelEpoch"
              },
              {
                "name": "winProb",
                "type": "uint56",
                "internalType": "HoprChannels.WinProb"
              }
            ]
          },
          {
            "name": "signature",
            "type": "tuple",
            "internalType": "struct HoprCrypto.CompactSignature",
            "components": [
              {
                "name": "r",
                "type": "bytes32",
                "internalType": "bytes32"
              },
              {
                "name": "vs",
                "type": "bytes32",
                "internalType": "bytes32"
              }
            ]
          },
          {
            "name": "porSecret",
            "type": "uint256",
            "internalType": "uint256"
          }
        ]
      },
      {
        "name": "params",
        "type": "tuple",
        "internalType": "struct HoprCrypto.VRFParameters",
        "components": [
          {
            "name": "vx",
            "type": "uint256",
            "internalType": "uint256"
          },
          {
            "name": "vy",
            "type": "uint256",
            "internalType": "uint256"
          },
          {
            "name": "s",
            "type": "uint256",
            "internalType": "uint256"
          },
          {
            "name": "h",
            "type": "uint256",
            "internalType": "uint256"
          },
          {
            "name": "sBx",
            "type": "uint256",
            "internalType": "uint256"
          },
          {
            "name": "sBy",
            "type": "uint256",
            "internalType": "uint256"
          },
          {
            "name": "hVx",
            "type": "uint256",
            "internalType": "uint256"
          },
          {
            "name": "hVy",
            "type": "uint256",
            "internalType": "uint256"
          }
        ]
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "redeemTicketSafe",
    "inputs": [
      {
        "name": "selfAddress",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "redeemable",
        "type": "tuple",
        "internalType": "struct HoprChannels.RedeemableTicket",
        "components": [
          {
            "name": "data",
            "type": "tuple",
            "internalType": "struct HoprChannels.TicketData",
            "components": [
              {
                "name": "channelId",
                "type": "bytes32",
                "internalType": "bytes32"
              },
              {
                "name": "amount",
                "type": "uint96",
                "internalType": "HoprChannels.Balance"
              },
              {
                "name": "ticketIndex",
                "type": "uint48",
                "internalType": "HoprChannels.TicketIndex"
              },
              {
                "name": "indexOffset",
                "type": "uint32",
                "internalType": "HoprChannels.TicketIndexOffset"
              },
              {
                "name": "epoch",
                "type": "uint24",
                "internalType": "HoprChannels.ChannelEpoch"
              },
              {
                "name": "winProb",
                "type": "uint56",
                "internalType": "HoprChannels.WinProb"
              }
            ]
          },
          {
            "name": "signature",
            "type": "tuple",
            "internalType": "struct HoprCrypto.CompactSignature",
            "components": [
              {
                "name": "r",
                "type": "bytes32",
                "internalType": "bytes32"
              },
              {
                "name": "vs",
                "type": "bytes32",
                "internalType": "bytes32"
              }
            ]
          },
          {
            "name": "porSecret",
            "type": "uint256",
            "internalType": "uint256"
          }
        ]
      },
      {
        "name": "params",
        "type": "tuple",
        "internalType": "struct HoprCrypto.VRFParameters",
        "components": [
          {
            "name": "vx",
            "type": "uint256",
            "internalType": "uint256"
          },
          {
            "name": "vy",
            "type": "uint256",
            "internalType": "uint256"
          },
          {
            "name": "s",
            "type": "uint256",
            "internalType": "uint256"
          },
          {
            "name": "h",
            "type": "uint256",
            "internalType": "uint256"
          },
          {
            "name": "sBx",
            "type": "uint256",
            "internalType": "uint256"
          },
          {
            "name": "sBy",
            "type": "uint256",
            "internalType": "uint256"
          },
          {
            "name": "hVx",
            "type": "uint256",
            "internalType": "uint256"
          },
          {
            "name": "hVy",
            "type": "uint256",
            "internalType": "uint256"
          }
        ]
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "token",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "address",
        "internalType": "contract IERC20"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "tokensReceived",
    "inputs": [
      {
        "name": "",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "from",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "to",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "amount",
        "type": "uint256",
        "internalType": "uint256"
      },
      {
        "name": "userData",
        "type": "bytes",
        "internalType": "bytes"
      },
      {
        "name": "",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "updateDomainSeparator",
    "inputs": [],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "updateLedgerDomainSeparator",
    "inputs": [],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "event",
    "name": "ChannelBalanceDecreased",
    "inputs": [
      {
        "name": "channelId",
        "type": "bytes32",
        "indexed": true,
        "internalType": "bytes32"
      },
      {
        "name": "newBalance",
        "type": "uint96",
        "indexed": false,
        "internalType": "HoprChannels.Balance"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "ChannelBalanceIncreased",
    "inputs": [
      {
        "name": "channelId",
        "type": "bytes32",
        "indexed": true,
        "internalType": "bytes32"
      },
      {
        "name": "newBalance",
        "type": "uint96",
        "indexed": false,
        "internalType": "HoprChannels.Balance"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "ChannelClosed",
    "inputs": [
      {
        "name": "channelId",
        "type": "bytes32",
        "indexed": true,
        "internalType": "bytes32"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "ChannelOpened",
    "inputs": [
      {
        "name": "source",
        "type": "address",
        "indexed": true,
        "internalType": "address"
      },
      {
        "name": "destination",
        "type": "address",
        "indexed": true,
        "internalType": "address"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "DomainSeparatorUpdated",
    "inputs": [
      {
        "name": "domainSeparator",
        "type": "bytes32",
        "indexed": true,
        "internalType": "bytes32"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "LedgerDomainSeparatorUpdated",
    "inputs": [
      {
        "name": "ledgerDomainSeparator",
        "type": "bytes32",
        "indexed": true,
        "internalType": "bytes32"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "OutgoingChannelClosureInitiated",
    "inputs": [
      {
        "name": "channelId",
        "type": "bytes32",
        "indexed": true,
        "internalType": "bytes32"
      },
      {
        "name": "closureTime",
        "type": "uint32",
        "indexed": false,
        "internalType": "HoprChannels.Timestamp"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "TicketRedeemed",
    "inputs": [
      {
        "name": "channelId",
        "type": "bytes32",
        "indexed": true,
        "internalType": "bytes32"
      },
      {
        "name": "newTicketIndex",
        "type": "uint48",
        "indexed": false,
        "internalType": "HoprChannels.TicketIndex"
      }
    ],
    "anonymous": false
  },
  {
    "type": "error",
    "name": "AlreadyInitialized",
    "inputs": []
  },
  {
    "type": "error",
    "name": "BalanceExceedsGlobalPerChannelAllowance",
    "inputs": []
  },
  {
    "type": "error",
    "name": "ContractNotResponsible",
    "inputs": []
  },
  {
    "type": "error",
    "name": "InsufficientChannelBalance",
    "inputs": []
  },
  {
    "type": "error",
    "name": "InvalidAggregatedTicketInterval",
    "inputs": []
  },
  {
    "type": "error",
    "name": "InvalidBalance",
    "inputs": []
  },
  {
    "type": "error",
    "name": "InvalidCurvePoint",
    "inputs": []
  },
  {
    "type": "error",
    "name": "InvalidFieldElement",
    "inputs": []
  },
  {
    "type": "error",
    "name": "InvalidNoticePeriod",
    "inputs": []
  },
  {
    "type": "error",
    "name": "InvalidPointWitness",
    "inputs": []
  },
  {
    "type": "error",
    "name": "InvalidSafeAddress",
    "inputs": []
  },
  {
    "type": "error",
    "name": "InvalidTicketSignature",
    "inputs": []
  },
  {
    "type": "error",
    "name": "InvalidTokenRecipient",
    "inputs": []
  },
  {
    "type": "error",
    "name": "InvalidTokensReceivedUsage",
    "inputs": []
  },
  {
    "type": "error",
    "name": "InvalidVRFProof",
    "inputs": []
  },
  {
    "type": "error",
    "name": "MultiSigUninitialized",
    "inputs": []
  },
  {
    "type": "error",
    "name": "NoticePeriodNotDue",
    "inputs": []
  },
  {
    "type": "error",
    "name": "SourceEqualsDestination",
    "inputs": []
  },
  {
    "type": "error",
    "name": "TicketIsNotAWin",
    "inputs": []
  },
  {
    "type": "error",
    "name": "TokenTransferFailed",
    "inputs": []
  },
  {
    "type": "error",
    "name": "WrongChannelState",
    "inputs": [
      {
        "name": "reason",
        "type": "string",
        "internalType": "string"
      }
    ]
  },
  {
    "type": "error",
    "name": "WrongToken",
    "inputs": []
  },
  {
    "type": "error",
    "name": "ZeroAddress",
    "inputs": [
      {
        "name": "reason",
        "type": "string",
        "internalType": "string"
      }
    ]
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
pub mod HoprChannels {
    use super::*;
    use alloy::sol_types as alloy_sol_types;
    /// The creation / init bytecode of the contract.
    ///
    /// ```text
    ///0x6004805460ff60a01b191690556000610140819052610154819052610160819052610174819052604061012081905260a08190526101a08290526101b49190915260286101808190526101c890915260c0523480156200005e57600080fd5b5060405162004927380380620049278339810160408190526200008191620004e9565b6305265c0060808190526040516001600160601b03193060601b16602082015260340160408051808303601f190181529190528051602091820120901c600160e01b4263ffffffff90811682029290921760018190556001600160e01b038082169183900490931690910217600255620000fc906200024f16565b508163ffffffff16600003620001255760405163f9ee910760e01b815260040160405180910390fd5b6001600160a01b038316620001805760405162461bcd60e51b815260206004820152601760248201527f746f6b656e206d757374206e6f7420626520656d707479000000000000000000604482015260640160405180910390fd5b6200018b816200034e565b6001600160a01b03831660e05263ffffffff8216610100526040516329965a1d60e01b815230600482018190527fb281fc8c12954d22544db45de3159a39272895b169a852b314f9cc762e44c53b60248301526044820152731820a4b7618bde71dce8cdc73aab6c95905fad24906329965a1d90606401600060405180830381600087803b1580156200021d57600080fd5b505af115801562000232573d6000803e3d6000fd5b5050505062000246620003ca60201b60201c565b50505062000545565b604080518082018252600a8152692437b8392632b233b2b960b11b6020918201528151808301835260058152640312e302e360dc1b90820152815160008051602062004907833981519152818301527f6cd681790c78c220517b099a737f8e85f69e797abe4e2910fb189b61db4bf2cd818401527f06c015bd22b4c69690933c1058878ebdfef31f9aaae40bbe86d8a09fe1b2972c60608201524660808201523060a0808301919091528351808303909101815260c0909101909252815191012060035481146200034b57600381905560405181907fa43fad83920fd09445855e854e73c9c532e17402c9ceb09993a2392843a5bdb990600090a25b50565b600454600160a01b900460ff1615620003795760405162dc149f60e41b815260040160405180910390fd5b6001600160a01b038116620003a15760405163474ebe2f60e11b815260040160405180910390fd5b600480546001600160a01b039092166001600160a81b031990921691909117600160a01b179055565b604080518082018252600c81526b486f70724368616e6e656c7360a01b6020918201528151808301835260058152640322e302e360dc1b90820152815160008051602062004907833981519152918101919091527f84e6908f343601d9ce9fb60d8250394eb8a51c56f1876bc1e017c97acd6567f2918101919091527fb4bcb154e38601c389396fa918314da42d4626f13ef6d0ceb07e5f5d26b2fbc360608201524660808201523060a082015260009060c00160405160208183030381529060405280519060200120905060055481146200034b57600581905560405181907f771f5240ae5fd8a7640d3fb82fa70aab2fb1dbf35f2ef464f8509946717664c590600090a250565b6001600160a01b03811681146200034b57600080fd5b600080600060608486031215620004ff57600080fd5b83516200050c81620004d3565b602085015190935063ffffffff811681146200052757600080fd5b60408501519092506200053a81620004d3565b809150509250925092565b60805160a05160c05160e0516101005161433e620005c9600039600081816103d9015261262d0152600081816104d701528181610566015281816109120152818161160101528181612089015281816122f801526124dc0152600081816102a801526105d501526000818161032e015261073001526000612751015261433e6000f3fe608060405234801561001057600080fd5b50600436106101e45760003560e01c80637c8e28da1161010f578063c966c4fe116100a2578063fc0c546a11610071578063fc0c546a146104d2578063fc55309a14610511578063fcb7796f14610524578063ffa1ad741461053757600080fd5b8063c966c4fe14610487578063dc96fd5014610490578063ddad190214610498578063f698da25146104c957600080fd5b8063ac9650d8116100de578063ac9650d81461043b578063b920deed1461045b578063bda65f4514610461578063be9babdc1461047457600080fd5b80637c8e28da146103c157806387352d65146103d457806389ccfe89146104105780638c3710c91461041857600080fd5b806329392e3211610187578063651514bf11610156578063651514bf146102ef57806372581cc01461030257806378d8016d146103295780637a7ebd7b1461035057600080fd5b806329392e321461028357806344dae6f8146102a357806354a2edf5146102ca5780635d2f07c5146102dd57600080fd5b80631a7ffe7a116101c35780631a7ffe7a1461022457806323cb3ac01461023757806324086cc21461024a578063249cb3fa1461027057600080fd5b806223de29146101e95780630abec58f146101fe5780630cd88d7214610211575b600080fd5b6101fc6101f7366004613a87565b61055b565b005b6101fc61020c366004613b54565b610817565b6101fc61021f366004613bc7565b6109af565b6101fc610232366004613c07565b610a80565b6101fc610245366004613c07565b610b50565b61025d610258366004613c2b565b610c1d565b6040519081526020015b60405180910390f35b61025d61027e366004613c48565b610d8a565b61028b600181565b6040516001600160601b039091168152602001610267565b61025d7f000000000000000000000000000000000000000000000000000000000000000081565b6101fc6102d8366004613c78565b610de4565b61028b6a084595161401484a00000081565b6101fc6102fd366004613c78565b610eb9565b61025d7fb281fc8c12954d22544db45de3159a39272895b169a852b314f9cc762e44c53b81565b61025d7f000000000000000000000000000000000000000000000000000000000000000081565b6103b061035e366004613ca6565b6006602052600090815260409020546001600160601b03811690600160601b810465ffffffffffff1690600160901b810463ffffffff1690600160b01b810462ffffff1690600160c81b900460ff1685565b604051610267959493929190613cd5565b6101fc6103cf366004613c07565b610f89565b6103fb7f000000000000000000000000000000000000000000000000000000000000000081565b60405163ffffffff9091168152602001610267565b6101fc611056565b61042b610426366004613d38565b61116f565b6040519015158152602001610267565b61044e610449366004613d5f565b6111f1565b6040516102679190613e24565b426103fb565b6101fc61046f366004613c78565b6112e6565b61025d610482366004613c78565b6113b6565b61025d60035481565b6101fc6113fb565b6104bc604051806040016040528060058152602001640312e302e360dc1b81525081565b6040516102679190613e86565b61025d60055481565b6104f97f000000000000000000000000000000000000000000000000000000000000000081565b6040516001600160a01b039091168152602001610267565b6101fc61051f366004613e99565b611509565b6101fc610532366004613ece565b61169c565b6104bc604051806040016040528060058152602001640322e302e360dc1b81525081565b336001600160a01b037f000000000000000000000000000000000000000000000000000000000000000016146105a457604051635079ff7560e11b815260040160405180910390fd5b6001600160a01b03861630146105cd57604051631738922160e31b815260040160405180910390fd5b821561080d577f0000000000000000000000000000000000000000000000000000000000000000830361072e576001600160601b038511156106225760405163293ceef960e21b815260040160405180910390fd5b600480546040516302265e3160e61b81528635606090811c9382018490526014880135901c916000916001600160a01b03909116906389978c4090602401602060405180830381865afa15801561067d573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906106a19190613efc565b9050826001600160a01b03168a6001600160a01b0316036106e9576001600160a01b038116156106e45760405163acd5a82360e01b815260040160405180910390fd5b61071b565b896001600160a01b0316816001600160a01b03161461071b5760405163acd5a82360e01b815260040160405180910390fd5b61072683838a61176a565b50505061080d565b7f000000000000000000000000000000000000000000000000000000000000000083036107f4578335606090811c90601486013560a090811c916020880135901c906034880135901c88158061079957506107956001600160601b03808316908516613f2f565b8914155b156107b75760405163c52e3eff60e01b815260040160405180910390fd5b6001600160601b038316156107d1576107d184838561176a565b6001600160601b038116156107eb576107eb82858361176a565b5050505061080d565b604051630d3dcde560e31b815260040160405180910390fd5b5050505050505050565b6004548390600160a01b900460ff16610843576040516308a9441960e31b815260040160405180910390fd5b600480546040516302265e3160e61b81526001600160a01b03848116938201939093523392909116906389978c4090602401602060405180830381865afa158015610892573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906108b69190613efc565b6001600160a01b0316146108dd5760405163acd5a82360e01b815260040160405180910390fd5b6108e884848461176a565b6040516323b872dd60e01b81523360048201523060248201526001600160601b03831660448201527f00000000000000000000000000000000000000000000000000000000000000006001600160a01b0316906323b872dd906064016020604051808303816000875af1158015610963573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906109879190613f42565b15156001146109a95760405163022e258160e11b815260040160405180910390fd5b50505050565b6004548390600160a01b900460ff166109db576040516308a9441960e31b815260040160405180910390fd5b600480546040516302265e3160e61b81526001600160a01b03848116938201939093523392909116906389978c4090602401602060405180830381865afa158015610a2a573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610a4e9190613efc565b6001600160a01b031614610a755760405163acd5a82360e01b815260040160405180910390fd5b6109a9848484611b16565b600454600160a01b900460ff16610aaa576040516308a9441960e31b815260040160405180910390fd5b600480546040516302265e3160e61b815233928101929092526000916001600160a01b03909116906389978c4090602401602060405180830381865afa158015610af8573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610b1c9190613efc565b6001600160a01b031614610b435760405163acd5a82360e01b815260040160405180910390fd5b610b4d3382612213565b50565b600454600160a01b900460ff16610b7a576040516308a9441960e31b815260040160405180910390fd5b600480546040516302265e3160e61b815233928101929092526000916001600160a01b03909116906389978c4090602401602060405180830381865afa158015610bc8573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610bec9190613efc565b6001600160a01b031614610c135760405163acd5a82360e01b815260040160405180910390fd5b610b4d338261238f565b600080610c2e836101000135612513565b90506000610c4260c0850160a08601613f64565b66ffffffffffffff166038610c5d60a0870160808801613f8d565b62ffffff16901b6050610c766080880160608901613fb2565b63ffffffff16901b6070610c906060890160408a01613fd8565b65ffffffffffff16901b60a0610cac60408a0160208b01614000565b6001600160601b0316901b171717179050600063fcb7796f60e01b85600001600001358385604051602001610d0193929190928352602083019190915260601b6001600160601b031916604082015260540190565b60408051808303601f1901815282825280516020918201206001600160e01b0319949094168184015282820193909352805180830382018152606083018252805190840120600554601960f81b6080850152600160f81b6081850152608284015260a2808401919091528151808403909101815260c29092019052805191012095945050505050565b6000828152602081815260408083206001600160a01b038516845290915281205460ff16610db9576000610ddb565b7fa2ef4600d742022d532d4747cb3547474667d6f13804902513b2ec01c848f4b45b90505b92915050565b6004548290600160a01b900460ff16610e10576040516308a9441960e31b815260040160405180910390fd5b600480546040516302265e3160e61b81526001600160a01b03848116938201939093523392909116906389978c4090602401602060405180830381865afa158015610e5f573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610e839190613efc565b6001600160a01b031614610eaa5760405163acd5a82360e01b815260040160405180910390fd5b610eb48383612213565b505050565b6004548290600160a01b900460ff16610ee5576040516308a9441960e31b815260040160405180910390fd5b600480546040516302265e3160e61b81526001600160a01b03848116938201939093523392909116906389978c4090602401602060405180830381865afa158015610f34573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610f589190613efc565b6001600160a01b031614610f7f5760405163acd5a82360e01b815260040160405180910390fd5b610eb4838361238f565b600454600160a01b900460ff16610fb3576040516308a9441960e31b815260040160405180910390fd5b600480546040516302265e3160e61b815233928101929092526000916001600160a01b03909116906389978c4090602401602060405180830381865afa158015611001573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906110259190613efc565b6001600160a01b03161461104c5760405163acd5a82360e01b815260040160405180910390fd5b610b4d33826125d0565b604080518082018252600c81526b486f70724368616e6e656c7360a01b6020918201528151808301835260058152640322e302e360dc1b9082015281517f8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f918101919091527f84e6908f343601d9ce9fb60d8250394eb8a51c56f1876bc1e017c97acd6567f2918101919091527fb4bcb154e38601c389396fa918314da42d4626f13ef6d0ceb07e5f5d26b2fbc360608201524660808201523060a082015260009060c0016040516020818303038152906040528051906020012090506005548114610b4d57600581905560405181907f771f5240ae5fd8a7640d3fb82fa70aab2fb1dbf35f2ef464f8509946717664c590600090a250565b604080516020808201869052833582840152838101356060830152610100850135608083015260c0808601803560a08086019190915260e0808901358487015286518087039094018452909401909452805191012060009260c89190911c916111da91908601613f64565b66ffffffffffffff90811691161115949350505050565b60608167ffffffffffffffff81111561120c5761120c61401b565b60405190808252806020026020018201604052801561123f57816020015b606081526020019060019003908161122a5790505b50905060005b828110156112df576112af3085858481811061126357611263614031565b90506020028101906112759190614047565b8080601f01602080910402602001604051908101604052809392919081815260200183838082843760009201919091525061272092505050565b8282815181106112c1576112c1614031565b602002602001018190525080806112d79061408e565b915050611245565b5092915050565b6004548290600160a01b900460ff16611312576040516308a9441960e31b815260040160405180910390fd5b600480546040516302265e3160e61b81526001600160a01b03848116938201939093523392909116906389978c4090602401602060405180830381865afa158015611361573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906113859190613efc565b6001600160a01b0316146113ac5760405163acd5a82360e01b815260040160405180910390fd5b610eb483836125d0565b6040516001600160601b0319606084811b8216602084015283901b16603482015260009060480160405160208183030381529060405280519060200120905092915050565b604080518082018252600a8152692437b8392632b233b2b960b11b6020918201528151808301835260058152640312e302e360dc1b9082015281517f8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f818301527f6cd681790c78c220517b099a737f8e85f69e797abe4e2910fb189b61db4bf2cd818401527f06c015bd22b4c69690933c1058878ebdfef31f9aaae40bbe86d8a09fe1b2972c60608201524660808201523060a0808301919091528351808303909101815260c090910190925281519101206003548114610b4d57600381905560405181907fa43fad83920fd09445855e854e73c9c532e17402c9ceb09993a2392843a5bdb990600090a250565b600454600160a01b900460ff16611533576040516308a9441960e31b815260040160405180910390fd5b600480546040516302265e3160e61b815233928101929092526000916001600160a01b03909116906389978c4090602401602060405180830381865afa158015611581573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906115a59190613efc565b6001600160a01b0316146115cc5760405163acd5a82360e01b815260040160405180910390fd5b6115d733838361176a565b6040516323b872dd60e01b81523360048201523060248201526001600160601b03821660448201527f00000000000000000000000000000000000000000000000000000000000000006001600160a01b0316906323b872dd906064016020604051808303816000875af1158015611652573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906116769190613f42565b15156001146116985760405163022e258160e11b815260040160405180910390fd5b5050565b600454600160a01b900460ff166116c6576040516308a9441960e31b815260040160405180910390fd5b600480546040516302265e3160e61b815233928101929092526000916001600160a01b03909116906389978c4090602401602060405180830381865afa158015611714573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906117389190613efc565b6001600160a01b03161461175f5760405163acd5a82360e01b815260040160405180910390fd5b611698338383611b16565b8060016001600160601b03821610156117965760405163c52e3eff60e01b815260040160405180910390fd5b6a084595161401484a0000006001600160601b03821611156117cb5760405163293ceef960e21b815260040160405180910390fd5b8383806001600160a01b0316826001600160a01b0316036117ff57604051634bd1d76960e11b815260040160405180910390fd5b6001600160a01b03821661185b5760405163eac0d38960e01b815260206004820152601860248201527f736f75726365206d757374206e6f7420626520656d707479000000000000000060448201526064015b60405180910390fd5b6001600160a01b0381166118b25760405163eac0d38960e01b815260206004820152601d60248201527f64657374696e6174696f6e206d757374206e6f7420626520656d7074790000006044820152606401611852565b60006118be87876113b6565b600081815260066020526040902090915060028154600160c81b900460ff1660028111156118ee576118ee613cbf565b0361194f5760405163499463c160e01b815260206004820152602a60248201527f63616e6e6f742066756e642061206368616e6e656c20746861742077696c6c2060448201526931b637b9b29039b7b7b760b11b6064820152608401611852565b80546119659087906001600160601b03166140a7565b81546001600160601b0319166001600160601b039190911617815560008154600160c81b900460ff16600281111561199f5761199f613cbf565b03611aaa5780546119bd90600160b01b900462ffffff1660016140c7565b815462ffffff91909116600160b01b026dff00000000000000ffffffffffff60601b19166dffffffff00000000ffffffffffff60601b1990911617600160c81b178155604080517fdd90f938230335e59dc925c57ecb0e27a28c2d87356e31f00cd5554abd6c1b2d602082015260608a811b6001600160601b03199081169383019390935289901b9091166054820152611a69906068015b604051602081830303815290604052612745565b866001600160a01b0316886001600160a01b03167fdd90f938230335e59dc925c57ecb0e27a28c2d87356e31f00cd5554abd6c1b2d60405160405180910390a35b8054604051611add91611a55916000805160206142c98339815191529186916001600160601b03909116906020016140e3565b80546040516001600160601b03909116815282906000805160206142c98339815191529060200160405180910390a25050505050505050565b611b266040830160208401614000565b60016001600160601b0382161015611b515760405163c52e3eff60e01b815260040160405180910390fd5b6a084595161401484a0000006001600160601b0382161115611b865760405163293ceef960e21b815260040160405180910390fd5b826101000135611b958161282b565b611bb257604051633ae4ed6b60e01b815260040160405180910390fd5b8335600090815260066020526040902060018154600160c81b900460ff166002811115611be157611be1613cbf565b14158015611c0c575060028154600160c81b900460ff166002811115611c0957611c09613cbf565b14155b15611c745760405163499463c160e01b815260206004820152603160248201527f7370656e64696e67206368616e6e656c206d757374206265204f50454e206f726044820152702050454e44494e475f544f5f434c4f534560781b6064820152608401611852565b611c8460a0860160808701613f8d565b8154600160b01b900462ffffff908116911614611ce45760405163499463c160e01b815260206004820152601860248201527f6368616e6e656c2065706f6368206d757374206d6174636800000000000000006044820152606401611852565b6000611cf66060870160408801613fd8565b90506000611d0a6080880160608901613fb2565b8354909150600160601b900465ffffffffffff16600163ffffffff83161080611d4257508065ffffffffffff168365ffffffffffff16105b15611d605760405163686e1e0f60e11b815260040160405180910390fd5b611d706040890160208a01614000565b84546001600160601b0391821691161015611d9e57604051632c51d8db60e21b815260040160405180910390fd5b6000611da989610c1d565b9050611db6818a8a61116f565b611dd35760405163ee835c8960e01b815260040160405180910390fd5b600060405180606001604052808381526020018c6001600160a01b03168152602001600554604051602001611e0a91815260200190565b60408051601f1981840301815291905290529050611e36611e30368b90038b018b614106565b8261284d565b611e53576040516312bfb7b760e31b815260040160405180910390fd5b6000611e688360c08d013560e08e0135612ad6565b90508a35611e76828e6113b6565b14611e94576040516366eea9ab60e11b815260040160405180910390fd5b611ea463ffffffff8616876141a4565b875465ffffffffffff91909116600160601b0265ffffffffffff60601b19909116178755611ed860408c0160208d01614000565b8754611eed91906001600160601b03166141c3565b87546001600160601b0319166001600160601b03919091169081178855604051611f4291611a55917f22e2a422a8860656a3a33cfa1daf771e76798ce5649747957235025de12e0b24918f35916020016140e3565b86546040516001600160601b0390911681528b35907f22e2a422a8860656a3a33cfa1daf771e76798ce5649747957235025de12e0b249060200160405180910390a26000611f908d836113b6565b9050600060066000838152602001908152602001600020905061201c7f7165e2ebc7ce35cc98cb7666f9945b3617f3f36326b76d18937ba5fecf18739a8e600001600001358b600001600c9054906101000a900465ffffffffffff16604051602001611a5593929190928352602083019190915260d01b6001600160d01b031916604082015260460190565b8854604051600160601b90910465ffffffffffff1681528d35907f7165e2ebc7ce35cc98cb7666f9945b3617f3f36326b76d18937ba5fecf18739a9060200160405180910390a260008154600160c81b900460ff16600281111561208257612082613cbf565b0361216c577f00000000000000000000000000000000000000000000000000000000000000006001600160a01b031663a9059cbb338f60000160200160208101906120cd9190614000565b6040516001600160e01b031960e085901b1681526001600160a01b0390921660048301526001600160601b031660248201526044016020604051808303816000875af1158015612121573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906121459190613f42565b15156001146121675760405163022e258160e11b815260040160405180910390fd5b612203565b61217c60408e0160208f01614000565b815461219191906001600160601b03166140a7565b81546001600160601b0319166001600160601b039190911690811782556040516121d391611a55916000805160206142c98339815191529186916020016140e3565b80546040516001600160601b03909116815282906000805160206142c98339815191529060200160405180910390a25b5050505050505050505050505050565b600061221f82846113b6565b60008181526006602052604081209192508154600160c81b900460ff16600281111561224d5761224d613cbf565b0361226b5760405163499463c160e01b8152600401611852906141e3565b8054600163ff00000160b01b031981168255604080516000805160206142e983398151915260208201529081018490526001600160601b03909116906122b390606001611a55565b60405183906000805160206142e983398151915290600090a280156123885760405163a9059cbb60e01b81526001600160a01b038581166004830152602482018390527f0000000000000000000000000000000000000000000000000000000000000000169063a9059cbb906044015b6020604051808303816000875af1158015612342573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906123669190613f42565b15156001146123885760405163022e258160e11b815260040160405180910390fd5b5050505050565b600061239b83836113b6565b600081815260066020526040902090915060028154600160c81b900460ff1660028111156123cb576123cb613cbf565b146124285760405163499463c160e01b815260206004820152602660248201527f6368616e6e656c207374617465206d7573742062652050454e44494e475f544f6044820152655f434c4f534560d01b6064820152608401611852565b805463ffffffff428116600160901b9092041610612459576040516338b2019560e11b815260040160405180910390fd5b8054600163ff00000160b01b031981168255604080516000805160206142e983398151915260208201529081018490526001600160601b03909116906124a190606001611a55565b60405183906000805160206142e983398151915290600090a280156123885760405163a9059cbb60e01b8152336004820152602481018290527f00000000000000000000000000000000000000000000000000000000000000006001600160a01b03169063a9059cbb90604401612323565b6000600181601b7f79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f8179870014551231950b75fc4402da1732fc9bebe197f79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f8179887096040805160008152602081018083529590955260ff909316928401929092526060830152608082015260a0016020604051602081039080840390855afa1580156125bf573d6000803e3d6000fd5b5050604051601f1901519392505050565b60006125dc83836113b6565b60008181526006602052604081209192508154600160c81b900460ff16600281111561260a5761260a613cbf565b036126285760405163499463c160e01b8152600401611852906141e3565b6126527f000000000000000000000000000000000000000000000000000000000000000042614233565b8154600160c91b67ff000000ffffffff60901b1990911660ff60c81b19600160901b63ffffffff949094168402161717808355604080517f07b5c950597fc3bed92e2ad37fa84f701655acb372982e486f5fad3607f04a5c602082015290810185905291900460e01b6001600160e01b03191660608201526126d690606401611a55565b8054604051600160901b90910463ffffffff16815282907f07b5c950597fc3bed92e2ad37fa84f701655acb372982e486f5fad3607f04a5c9060200160405180910390a250505050565b6060610ddb83836040518060600160405280602781526020016142a260279139612afc565b600154600090612783907f000000000000000000000000000000000000000000000000000000000000000090600160e01b900463ffffffff16613f2f565b42111561278e575060015b600354600154835160208086019190912060408051808401959095524360e01b6001600160e01b0319169085015291901b63ffffffff19166044830152606082015260800160408051601f19818403018152919052805160209182012063ffffffff4216600160e01b02911c1760015580156116985750506001546001600160e01b038116600160e01b9182900463ffffffff1690910217600255565b6000811580610dde57505070014551231950b75fc4402da1732fc9bebe191190565b60006401000003d019836060015110158061287257506401000003d019836040015110155b1561289057604051633ae4ed6b60e01b815260040160405180910390fd5b6128a283600001518460200151612b74565b6128bf57604051633922a54160e11b815260040160405180910390fd5b600080612911846020015185600001516040516020016128f892919060609290921b6001600160601b0319168252601482015260340190565b6040516020818303038152906040528560400151612b9f565b91509150600061292686604001518484612c25565b905061296186608001518760a00151604080516020808201949094528082019290925280518083038201815260609092019052805191012090565b6001600160a01b0316816001600160a01b03161461299257604051631dbfb9b360e31b815260040160405180910390fd5b60006129ab876060015188600001518960200151612c25565b90506129e68760c001518860e00151604080516020808201949094528082019290925280518083038201815260609092019052805191012090565b6001600160a01b0316816001600160a01b031614612a1757604051631dbfb9b360e31b815260040160405180910390fd5b600080612a4989608001518a60a001518b60c001518c60e001516401000003d019612a429190614250565b6000612cc4565b6020808b01518c518d8301518d51604051969850949650600095612ac195612aa8958a928a92910160609690961b6001600160601b03191686526014860194909452603485019290925260548401526074830152609482015260b40190565b6040516020818303038152906040528a60400151612e4b565b60608b01511497505050505050505092915050565b6000806000612ae6868686612ebc565b91509150612af381612ef5565b50949350505050565b6060600080856001600160a01b031685604051612b199190614263565b600060405180830381855af49150503d8060008114612b54576040519150601f19603f3d011682016040523d82523d6000602084013e612b59565b606091505b5091509150612b6a8683838761303f565b9695505050505050565b60006401000003d01980846401000003d019868709096007086401000003d019838409149392505050565b600080600080612baf86866130c0565b91509150600080612bbf8461317c565b91509150600080612bcf8561317c565b91509150600080612c03868686867f3f8731abdd661adca08a5558f0f5d272e953d363cb6f0e5d405447c01a444533612cc4565b91509150612c11828261343e565b9950995050505050505050505b9250929050565b600080612c3360028461427f565b600003612c425750601b612c46565b50601c5b60016000828670014551231950b75fc4402da1732fc9bebe19888a096040805160008152602081018083529590955260ff909316928401929092526060830152608082015260a0016020604051602081039080840390855afa158015612cb0573d6000803e3d6000fd5b5050604051601f1901519695505050505050565b600080838614198588141615612cd957600080fd5b600080858814878a141660018114612cf6578015612d7357612dee565b6401000003d019866401000003d0198b60020908915060405160208152602080820152602060408201528260608201526401000003d21960808201526401000003d01960a082015260208160c0836005600019fa612d5357600080fd5b6401000003d01981516401000003d019808e8f0960030909935050612dee565b6401000003d0198a6401000003d019038908915060405160208152602080820152602060408201528260608201526401000003d21960808201526401000003d01960a082015260208160c0836005600019fa612dce57600080fd5b6401000003d01981516401000003d0198c6401000003d019038b08099350505b50506401000003d01980896401000003d01903886401000003d01903086401000003d0198384090892506401000003d019876401000003d019036401000003d01980866401000003d019038c088409089150509550959350505050565b6000806000612e5a858561372b565b9150915060405160308152602080820152602060408201528260608201528160808201526001609082015270014551231950b75fc4402da1732fc9bebe1960b082015260208160d0836005600019fa612eb257600080fd5b5195945050505050565b6000806001600160ff1b03831681612ed960ff86901c601b613f2f565b9050612ee78782888561382b565b935093505050935093915050565b6000816004811115612f0957612f09613cbf565b03612f115750565b6001816004811115612f2557612f25613cbf565b03612f725760405162461bcd60e51b815260206004820152601860248201527f45434453413a20696e76616c6964207369676e617475726500000000000000006044820152606401611852565b6002816004811115612f8657612f86613cbf565b03612fd35760405162461bcd60e51b815260206004820152601f60248201527f45434453413a20696e76616c6964207369676e6174757265206c656e677468006044820152606401611852565b6003816004811115612fe757612fe7613cbf565b03610b4d5760405162461bcd60e51b815260206004820152602260248201527f45434453413a20696e76616c6964207369676e6174757265202773272076616c604482015261756560f01b6064820152608401611852565b606083156130ae5782516000036130a7576001600160a01b0385163b6130a75760405162461bcd60e51b815260206004820152601d60248201527f416464726573733a2063616c6c20746f206e6f6e2d636f6e74726163740000006044820152606401611852565b50816130b8565b6130b883836138ef565b949350505050565b60008060008060006130d28787613919565b9250925092506040516030815260208082015260206040820152836060820152826080820152600160908201526401000003d01960b082015260208160d0836005600019fa61312057600080fd5b80519550506040516030815260208082015282605082015260206040820152816070820152600160908201526401000003d01960b082015260208160d0836005600019fa61316d57600080fd5b80519450505050509250929050565b6000806401000003d0198384096401000003d019816401000003db190990506401000003d0198182096401000003d01982820890506401000003d019600182086401000003d0196106eb8209905060008215600181146131e15780156131ef576131fb565b6401000003db1991506131fb565b836401000003d0190391505b506401000003d019817f3f8731abdd661adca08a5558f0f5d272e953d363cb6f0e5d405447c01a4445330990506401000003d01982830992506401000003d0198182096401000003d019817f3f8731abdd661adca08a5558f0f5d272e953d363cb6f0e5d405447c01a444533096401000003d01981860894506401000003d01984860994506401000003d01983830991506401000003d019826106eb0990506401000003d0198186089450506401000003d01983860996506000806401000003d0198384096401000003d0198488096401000003d0198183099150604051602081526020808201526020604082015282606082015263400000f5600160fe1b0360808201526401000003d01960a082015260208160c0836005600019fa61332157600080fd5b6401000003d01982825109925050506401000003d0197f31fdf302724013e57ad13fb38f842afeec184f00a74789dd286729c8303c4a5982096401000003d0198283096401000003d0198682099050888114600181146133865780156133925761339a565b6001945083955061339a565b600094508295505b505050506401000003d0198a880997506401000003d019828909975080156133c3578498508197505b5050506002850660028806146133df57846401000003d0190394505b604051935060208452602080850152602060408501528060608501525050506401000003d21960808201526401000003d01960a082015260208160c0836005600019fa61342b57600080fd5b6401000003d01981518409925050915091565b6000806401000003d0198485096401000003d0198186096401000003d019807f8e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38daaaaa8c76401000003d019897f07d3d4c80bc321d5b9f315cea7fd44c5d595d2fc0bf63b92dfff1044f17c658109086401000003d01980857f534c328d23f234e6e2a413deca25caece4506144037c40314ecbd0b53d9dd262096401000003d019857f8e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38daaaaa88c0908086401000003d0197fd35771193d94918a9ca34ccbb7b640dd86cd409542f8487d9fe6b745781eb49b6401000003d019808a7fedadc6f64383dc1df7c4b2d51b54225406d36b641f5e41bbc52a56612a8c6d140986080860405160208152602080820152602060408201528160608201526401000003d21960808201526401000003d01960a082015260208160c0836005600019fa61359d57600080fd5b805191506401000003d01982840996506401000003d019807f4bda12f684bda12f684bda12f684bda12f684bda12f684bda12f684b8e38e23c6401000003d0198c7fc75e0c32d5cb7c0fa9d0a54b12a0a6d5647ab046d686da6fdffc90fc201d71a309086401000003d01980887f29a6194691f91a73715209ef6512e576722830a201be2018a765e85a9ecee931096401000003d019887f2f684bda12f684bda12f684bda12f684bda12f684bda12f684bda12f38e38d8409080892506401000003d019806401000006c4196401000003d0198c7f7a06534bb8bdb49fd5e9e6632722c2989467c1bfc8e8d978dfb425d2685c257309086401000003d01980887f6484aa716545ca2cf3a70c3fa8fe337e0a3d21162f0d6299a7bf8192bfd2a76f098708089450604051905060208152602080820152602060408201528460608201526401000003d21960808201526401000003d01960a082015260208160c0836005600019fa61370d57600080fd5b5193506401000003d019905083818389090993505050509250929050565b60008060ff8351111561373d57600080fd5b60006040516088602060005b885181101561376a5788820151848401526020928301929182019101613749565b505060898751019050603081830153600201602060005b87518110156137a25787820151848401526020928301929182019101613781565b5050608b8651885101019050855181830153508551855101608c018120915050604051818152600160208201536021602060005b87518110156137f757878201518484015260209283019291820191016137d6565b5050508451855160210182015384516022018120935083821881526002602082015384516022018120925050509250929050565b6000807f7fffffffffffffffffffffffffffffff5d576e7357a4501ddfe92f46681b20a083111561386257506000905060036138e6565b6040805160008082526020820180845289905260ff881692820192909252606081018690526080810185905260019060a0016020604051602081039080840390855afa1580156138b6573d6000803e3d6000fd5b5050604051601f1901519150506001600160a01b0381166138df576000600192509250506138e6565b9150600090505b94509492505050565b8151156138ff5781518083602001fd5b8060405162461bcd60e51b81526004016118529190613e86565b600080600060ff8451111561392d57600080fd5b60006040516088602060005b895181101561395a5789820151848401526020928301929182019101613939565b505060898851019050606081830153600201602060005b88518110156139925788820151848401526020928301929182019101613971565b5050608b8751895101019050865181830153508651865101608c018120915050604051818152600160208201536021602060005b88518110156139e757888201518484015260209283019291820191016139c6565b5050508551865160210182015385516022018120945084821881526002602082015385516022018120935083821881526003602082015385516022018120925050509250925092565b6001600160a01b0381168114610b4d57600080fd5b60008083601f840112613a5757600080fd5b50813567ffffffffffffffff811115613a6f57600080fd5b602083019150836020828501011115612c1e57600080fd5b60008060008060008060008060c0898b031215613aa357600080fd5b8835613aae81613a30565b97506020890135613abe81613a30565b96506040890135613ace81613a30565b955060608901359450608089013567ffffffffffffffff80821115613af257600080fd5b613afe8c838d01613a45565b909650945060a08b0135915080821115613b1757600080fd5b50613b248b828c01613a45565b999c989b5096995094979396929594505050565b80356001600160601b0381168114613b4f57600080fd5b919050565b600080600060608486031215613b6957600080fd5b8335613b7481613a30565b92506020840135613b8481613a30565b9150613b9260408501613b38565b90509250925092565b60006101208284031215613bae57600080fd5b50919050565b60006101008284031215613bae57600080fd5b60008060006102408486031215613bdd57600080fd5b8335613be881613a30565b9250613bf78560208601613b9b565b9150613b92856101408601613bb4565b600060208284031215613c1957600080fd5b8135613c2481613a30565b9392505050565b60006101208284031215613c3e57600080fd5b610ddb8383613b9b565b60008060408385031215613c5b57600080fd5b823591506020830135613c6d81613a30565b809150509250929050565b60008060408385031215613c8b57600080fd5b8235613c9681613a30565b91506020830135613c6d81613a30565b600060208284031215613cb857600080fd5b5035919050565b634e487b7160e01b600052602160045260246000fd5b6001600160601b038616815265ffffffffffff8516602082015263ffffffff8416604082015262ffffff8316606082015260a0810160038310613d2857634e487b7160e01b600052602160045260246000fd5b8260808301529695505050505050565b60008060006102408486031215613d4e57600080fd5b83359250613bf78560208601613b9b565b60008060208385031215613d7257600080fd5b823567ffffffffffffffff80821115613d8a57600080fd5b818501915085601f830112613d9e57600080fd5b813581811115613dad57600080fd5b8660208260051b8501011115613dc257600080fd5b60209290920196919550909350505050565b60005b83811015613def578181015183820152602001613dd7565b50506000910152565b60008151808452613e10816020860160208601613dd4565b601f01601f19169290920160200192915050565b6000602080830181845280855180835260408601915060408160051b870101925083870160005b82811015613e7957603f19888603018452613e67858351613df8565b94509285019290850190600101613e4b565b5092979650505050505050565b602081526000610ddb6020830184613df8565b60008060408385031215613eac57600080fd5b8235613eb781613a30565b9150613ec560208401613b38565b90509250929050565b6000806102208385031215613ee257600080fd5b613eec8484613b9b565b9150613ec5846101208501613bb4565b600060208284031215613f0e57600080fd5b8151613c2481613a30565b634e487b7160e01b600052601160045260246000fd5b80820180821115610dde57610dde613f19565b600060208284031215613f5457600080fd5b81518015158114613c2457600080fd5b600060208284031215613f7657600080fd5b813566ffffffffffffff81168114613c2457600080fd5b600060208284031215613f9f57600080fd5b813562ffffff81168114613c2457600080fd5b600060208284031215613fc457600080fd5b813563ffffffff81168114613c2457600080fd5b600060208284031215613fea57600080fd5b813565ffffffffffff81168114613c2457600080fd5b60006020828403121561401257600080fd5b610ddb82613b38565b634e487b7160e01b600052604160045260246000fd5b634e487b7160e01b600052603260045260246000fd5b6000808335601e1984360301811261405e57600080fd5b83018035915067ffffffffffffffff82111561407957600080fd5b602001915036819003821315612c1e57600080fd5b6000600182016140a0576140a0613f19565b5060010190565b6001600160601b038181168382160190808211156112df576112df613f19565b62ffffff8181168382160190808211156112df576112df613f19565b928352602083019190915260a01b6001600160a01b0319166040820152604c0190565b600061010080838503121561411a57600080fd5b6040519081019067ffffffffffffffff8211818310171561414b57634e487b7160e01b600052604160045260246000fd5b81604052833581526020840135602082015260408401356040820152606084013560608201526080840135608082015260a084013560a082015260c084013560c082015260e084013560e0820152809250505092915050565b65ffffffffffff8181168382160190808211156112df576112df613f19565b6001600160601b038281168282160390808211156112df576112df613f19565b60208082526030908201527f6368616e6e656c206d7573742068617665207374617465204f50454e206f722060408201526f50454e44494e475f544f5f434c4f534560801b606082015260800190565b63ffffffff8181168382160190808211156112df576112df613f19565b81810381811115610dde57610dde613f19565b60008251614275818460208701613dd4565b9190910192915050565b60008261429c57634e487b7160e01b600052601260045260246000fd5b50069056fe416464726573733a206c6f772d6c6576656c2064656c65676174652063616c6c206661696c65645fa17246d3a5d68d42baa94cde33042180b783a399c02bf63ac2076e0f708738ceeab2eef998c17fe96f30f83fbf3c55fc5047f6e40c55a0cf72d236e9d2ba72a26469706673582212202343980d92998edaee11a6676235eb75899e111708ed0a2a33545ccdcb0e050364736f6c634300081300338b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f
    /// ```
    #[rustfmt::skip]
    #[allow(clippy::all)]
    pub static BYTECODE: alloy_sol_types::private::Bytes = alloy_sol_types::private::Bytes::from_static(
        b"`\x04\x80T`\xFF`\xA0\x1B\x19\x16\x90U`\0a\x01@\x81\x90Ra\x01T\x81\x90Ra\x01`\x81\x90Ra\x01t\x81\x90R`@a\x01 \x81\x90R`\xA0\x81\x90Ra\x01\xA0\x82\x90Ra\x01\xB4\x91\x90\x91R`(a\x01\x80\x81\x90Ra\x01\xC8\x90\x91R`\xC0R4\x80\x15b\0\0^W`\0\x80\xFD[P`@Qb\0I'8\x03\x80b\0I'\x839\x81\x01`@\x81\x90Rb\0\0\x81\x91b\0\x04\xE9V[c\x05&\\\0`\x80\x81\x90R`@Q`\x01`\x01``\x1B\x03\x190``\x1B\x16` \x82\x01R`4\x01`@\x80Q\x80\x83\x03`\x1F\x19\x01\x81R\x91\x90R\x80Q` \x91\x82\x01 \x90\x1C`\x01`\xE0\x1BBc\xFF\xFF\xFF\xFF\x90\x81\x16\x82\x02\x92\x90\x92\x17`\x01\x81\x90U`\x01`\x01`\xE0\x1B\x03\x80\x82\x16\x91\x83\x90\x04\x90\x93\x16\x90\x91\x02\x17`\x02Ub\0\0\xFC\x90b\0\x02O\x16V[P\x81c\xFF\xFF\xFF\xFF\x16`\0\x03b\0\x01%W`@Qc\xF9\xEE\x91\x07`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01`\x01`\xA0\x1B\x03\x83\x16b\0\x01\x80W`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`\x17`$\x82\x01R\x7Ftoken must not be empty\0\0\0\0\0\0\0\0\0`D\x82\x01R`d\x01`@Q\x80\x91\x03\x90\xFD[b\0\x01\x8B\x81b\0\x03NV[`\x01`\x01`\xA0\x1B\x03\x83\x16`\xE0Rc\xFF\xFF\xFF\xFF\x82\x16a\x01\0R`@Qc)\x96Z\x1D`\xE0\x1B\x81R0`\x04\x82\x01\x81\x90R\x7F\xB2\x81\xFC\x8C\x12\x95M\"TM\xB4]\xE3\x15\x9A9'(\x95\xB1i\xA8R\xB3\x14\xF9\xCCv.D\xC5;`$\x83\x01R`D\x82\x01Rs\x18 \xA4\xB7a\x8B\xDEq\xDC\xE8\xCD\xC7:\xABl\x95\x90_\xAD$\x90c)\x96Z\x1D\x90`d\x01`\0`@Q\x80\x83\x03\x81`\0\x87\x80;\x15\x80\x15b\0\x02\x1DW`\0\x80\xFD[PZ\xF1\x15\x80\x15b\0\x022W=`\0\x80>=`\0\xFD[PPPPb\0\x02Fb\0\x03\xCA` \x1B` \x1CV[PPPb\0\x05EV[`@\x80Q\x80\x82\x01\x82R`\n\x81Ri$7\xB89&2\xB23\xB2\xB9`\xB1\x1B` \x91\x82\x01R\x81Q\x80\x83\x01\x83R`\x05\x81Rd\x03\x12\xE3\x02\xE3`\xDC\x1B\x90\x82\x01R\x81Q`\0\x80Q` b\0I\x07\x839\x81Q\x91R\x81\x83\x01R\x7Fl\xD6\x81y\x0Cx\xC2 Q{\t\x9As\x7F\x8E\x85\xF6\x9Eyz\xBEN)\x10\xFB\x18\x9Ba\xDBK\xF2\xCD\x81\x84\x01R\x7F\x06\xC0\x15\xBD\"\xB4\xC6\x96\x90\x93<\x10X\x87\x8E\xBD\xFE\xF3\x1F\x9A\xAA\xE4\x0B\xBE\x86\xD8\xA0\x9F\xE1\xB2\x97,``\x82\x01RF`\x80\x82\x01R0`\xA0\x80\x83\x01\x91\x90\x91R\x83Q\x80\x83\x03\x90\x91\x01\x81R`\xC0\x90\x91\x01\x90\x92R\x81Q\x91\x01 `\x03T\x81\x14b\0\x03KW`\x03\x81\x90U`@Q\x81\x90\x7F\xA4?\xAD\x83\x92\x0F\xD0\x94E\x85^\x85Ns\xC9\xC52\xE1t\x02\xC9\xCE\xB0\x99\x93\xA29(C\xA5\xBD\xB9\x90`\0\x90\xA2[PV[`\x04T`\x01`\xA0\x1B\x90\x04`\xFF\x16\x15b\0\x03yW`@Qb\xDC\x14\x9F`\xE4\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01`\x01`\xA0\x1B\x03\x81\x16b\0\x03\xA1W`@QcGN\xBE/`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x04\x80T`\x01`\x01`\xA0\x1B\x03\x90\x92\x16`\x01`\x01`\xA8\x1B\x03\x19\x90\x92\x16\x91\x90\x91\x17`\x01`\xA0\x1B\x17\x90UV[`@\x80Q\x80\x82\x01\x82R`\x0C\x81RkHoprChannels`\xA0\x1B` \x91\x82\x01R\x81Q\x80\x83\x01\x83R`\x05\x81Rd\x03\"\xE3\x02\xE3`\xDC\x1B\x90\x82\x01R\x81Q`\0\x80Q` b\0I\x07\x839\x81Q\x91R\x91\x81\x01\x91\x90\x91R\x7F\x84\xE6\x90\x8F46\x01\xD9\xCE\x9F\xB6\r\x82P9N\xB8\xA5\x1CV\xF1\x87k\xC1\xE0\x17\xC9z\xCDeg\xF2\x91\x81\x01\x91\x90\x91R\x7F\xB4\xBC\xB1T\xE3\x86\x01\xC3\x899o\xA9\x181M\xA4-F&\xF1>\xF6\xD0\xCE\xB0~_]&\xB2\xFB\xC3``\x82\x01RF`\x80\x82\x01R0`\xA0\x82\x01R`\0\x90`\xC0\x01`@Q` \x81\x83\x03\x03\x81R\x90`@R\x80Q\x90` \x01 \x90P`\x05T\x81\x14b\0\x03KW`\x05\x81\x90U`@Q\x81\x90\x7Fw\x1FR@\xAE_\xD8\xA7d\r?\xB8/\xA7\n\xAB/\xB1\xDB\xF3_.\xF4d\xF8P\x99Fqvd\xC5\x90`\0\x90\xA2PV[`\x01`\x01`\xA0\x1B\x03\x81\x16\x81\x14b\0\x03KW`\0\x80\xFD[`\0\x80`\0``\x84\x86\x03\x12\x15b\0\x04\xFFW`\0\x80\xFD[\x83Qb\0\x05\x0C\x81b\0\x04\xD3V[` \x85\x01Q\x90\x93Pc\xFF\xFF\xFF\xFF\x81\x16\x81\x14b\0\x05'W`\0\x80\xFD[`@\x85\x01Q\x90\x92Pb\0\x05:\x81b\0\x04\xD3V[\x80\x91PP\x92P\x92P\x92V[`\x80Q`\xA0Q`\xC0Q`\xE0Qa\x01\0QaC>b\0\x05\xC9`\09`\0\x81\x81a\x03\xD9\x01Ra&-\x01R`\0\x81\x81a\x04\xD7\x01R\x81\x81a\x05f\x01R\x81\x81a\t\x12\x01R\x81\x81a\x16\x01\x01R\x81\x81a \x89\x01R\x81\x81a\"\xF8\x01Ra$\xDC\x01R`\0\x81\x81a\x02\xA8\x01Ra\x05\xD5\x01R`\0\x81\x81a\x03.\x01Ra\x070\x01R`\0a'Q\x01RaC>`\0\xF3\xFE`\x80`@R4\x80\x15a\0\x10W`\0\x80\xFD[P`\x046\x10a\x01\xE4W`\x005`\xE0\x1C\x80c|\x8E(\xDA\x11a\x01\x0FW\x80c\xC9f\xC4\xFE\x11a\0\xA2W\x80c\xFC\x0CTj\x11a\0qW\x80c\xFC\x0CTj\x14a\x04\xD2W\x80c\xFCU0\x9A\x14a\x05\x11W\x80c\xFC\xB7yo\x14a\x05$W\x80c\xFF\xA1\xADt\x14a\x057W`\0\x80\xFD[\x80c\xC9f\xC4\xFE\x14a\x04\x87W\x80c\xDC\x96\xFDP\x14a\x04\x90W\x80c\xDD\xAD\x19\x02\x14a\x04\x98W\x80c\xF6\x98\xDA%\x14a\x04\xC9W`\0\x80\xFD[\x80c\xAC\x96P\xD8\x11a\0\xDEW\x80c\xAC\x96P\xD8\x14a\x04;W\x80c\xB9 \xDE\xED\x14a\x04[W\x80c\xBD\xA6_E\x14a\x04aW\x80c\xBE\x9B\xAB\xDC\x14a\x04tW`\0\x80\xFD[\x80c|\x8E(\xDA\x14a\x03\xC1W\x80c\x875-e\x14a\x03\xD4W\x80c\x89\xCC\xFE\x89\x14a\x04\x10W\x80c\x8C7\x10\xC9\x14a\x04\x18W`\0\x80\xFD[\x80c)9.2\x11a\x01\x87W\x80ce\x15\x14\xBF\x11a\x01VW\x80ce\x15\x14\xBF\x14a\x02\xEFW\x80crX\x1C\xC0\x14a\x03\x02W\x80cx\xD8\x01m\x14a\x03)W\x80cz~\xBD{\x14a\x03PW`\0\x80\xFD[\x80c)9.2\x14a\x02\x83W\x80cD\xDA\xE6\xF8\x14a\x02\xA3W\x80cT\xA2\xED\xF5\x14a\x02\xCAW\x80c]/\x07\xC5\x14a\x02\xDDW`\0\x80\xFD[\x80c\x1A\x7F\xFEz\x11a\x01\xC3W\x80c\x1A\x7F\xFEz\x14a\x02$W\x80c#\xCB:\xC0\x14a\x027W\x80c$\x08l\xC2\x14a\x02JW\x80c$\x9C\xB3\xFA\x14a\x02pW`\0\x80\xFD[\x80b#\xDE)\x14a\x01\xE9W\x80c\n\xBE\xC5\x8F\x14a\x01\xFEW\x80c\x0C\xD8\x8Dr\x14a\x02\x11W[`\0\x80\xFD[a\x01\xFCa\x01\xF76`\x04a:\x87V[a\x05[V[\0[a\x01\xFCa\x02\x0C6`\x04a;TV[a\x08\x17V[a\x01\xFCa\x02\x1F6`\x04a;\xC7V[a\t\xAFV[a\x01\xFCa\x0226`\x04a<\x07V[a\n\x80V[a\x01\xFCa\x02E6`\x04a<\x07V[a\x0BPV[a\x02]a\x02X6`\x04a<+V[a\x0C\x1DV[`@Q\x90\x81R` \x01[`@Q\x80\x91\x03\x90\xF3[a\x02]a\x02~6`\x04a<HV[a\r\x8AV[a\x02\x8B`\x01\x81V[`@Q`\x01`\x01``\x1B\x03\x90\x91\x16\x81R` \x01a\x02gV[a\x02]\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81V[a\x01\xFCa\x02\xD86`\x04a<xV[a\r\xE4V[a\x02\x8Bj\x08E\x95\x16\x14\x01HJ\0\0\0\x81V[a\x01\xFCa\x02\xFD6`\x04a<xV[a\x0E\xB9V[a\x02]\x7F\xB2\x81\xFC\x8C\x12\x95M\"TM\xB4]\xE3\x15\x9A9'(\x95\xB1i\xA8R\xB3\x14\xF9\xCCv.D\xC5;\x81V[a\x02]\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81V[a\x03\xB0a\x03^6`\x04a<\xA6V[`\x06` R`\0\x90\x81R`@\x90 T`\x01`\x01``\x1B\x03\x81\x16\x90`\x01``\x1B\x81\x04e\xFF\xFF\xFF\xFF\xFF\xFF\x16\x90`\x01`\x90\x1B\x81\x04c\xFF\xFF\xFF\xFF\x16\x90`\x01`\xB0\x1B\x81\x04b\xFF\xFF\xFF\x16\x90`\x01`\xC8\x1B\x90\x04`\xFF\x16\x85V[`@Qa\x02g\x95\x94\x93\x92\x91\x90a<\xD5V[a\x01\xFCa\x03\xCF6`\x04a<\x07V[a\x0F\x89V[a\x03\xFB\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81V[`@Qc\xFF\xFF\xFF\xFF\x90\x91\x16\x81R` \x01a\x02gV[a\x01\xFCa\x10VV[a\x04+a\x04&6`\x04a=8V[a\x11oV[`@Q\x90\x15\x15\x81R` \x01a\x02gV[a\x04Na\x04I6`\x04a=_V[a\x11\xF1V[`@Qa\x02g\x91\x90a>$V[Ba\x03\xFBV[a\x01\xFCa\x04o6`\x04a<xV[a\x12\xE6V[a\x02]a\x04\x826`\x04a<xV[a\x13\xB6V[a\x02]`\x03T\x81V[a\x01\xFCa\x13\xFBV[a\x04\xBC`@Q\x80`@\x01`@R\x80`\x05\x81R` \x01d\x03\x12\xE3\x02\xE3`\xDC\x1B\x81RP\x81V[`@Qa\x02g\x91\x90a>\x86V[a\x02]`\x05T\x81V[a\x04\xF9\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81V[`@Q`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x81R` \x01a\x02gV[a\x01\xFCa\x05\x1F6`\x04a>\x99V[a\x15\tV[a\x01\xFCa\x0526`\x04a>\xCEV[a\x16\x9CV[a\x04\xBC`@Q\x80`@\x01`@R\x80`\x05\x81R` \x01d\x03\"\xE3\x02\xE3`\xDC\x1B\x81RP\x81V[3`\x01`\x01`\xA0\x1B\x03\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x16\x14a\x05\xA4W`@QcPy\xFFu`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01`\x01`\xA0\x1B\x03\x86\x160\x14a\x05\xCDW`@Qc\x178\x92!`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x82\x15a\x08\rW\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x83\x03a\x07.W`\x01`\x01``\x1B\x03\x85\x11\x15a\x06\"W`@Qc)<\xEE\xF9`\xE2\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x04\x80T`@Qc\x02&^1`\xE6\x1B\x81R\x865``\x90\x81\x1C\x93\x82\x01\x84\x90R`\x14\x88\x015\x90\x1C\x91`\0\x91`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x90c\x89\x97\x8C@\x90`$\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x06}W=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x06\xA1\x91\x90a>\xFCV[\x90P\x82`\x01`\x01`\xA0\x1B\x03\x16\x8A`\x01`\x01`\xA0\x1B\x03\x16\x03a\x06\xE9W`\x01`\x01`\xA0\x1B\x03\x81\x16\x15a\x06\xE4W`@Qc\xAC\xD5\xA8#`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x07\x1BV[\x89`\x01`\x01`\xA0\x1B\x03\x16\x81`\x01`\x01`\xA0\x1B\x03\x16\x14a\x07\x1BW`@Qc\xAC\xD5\xA8#`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x07&\x83\x83\x8Aa\x17jV[PPPa\x08\rV[\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x83\x03a\x07\xF4W\x835``\x90\x81\x1C\x90`\x14\x86\x015`\xA0\x90\x81\x1C\x91` \x88\x015\x90\x1C\x90`4\x88\x015\x90\x1C\x88\x15\x80a\x07\x99WPa\x07\x95`\x01`\x01``\x1B\x03\x80\x83\x16\x90\x85\x16a?/V[\x89\x14\x15[\x15a\x07\xB7W`@Qc\xC5.>\xFF`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01`\x01``\x1B\x03\x83\x16\x15a\x07\xD1Wa\x07\xD1\x84\x83\x85a\x17jV[`\x01`\x01``\x1B\x03\x81\x16\x15a\x07\xEBWa\x07\xEB\x82\x85\x83a\x17jV[PPPPa\x08\rV[`@Qc\r=\xCD\xE5`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[PPPPPPPPV[`\x04T\x83\x90`\x01`\xA0\x1B\x90\x04`\xFF\x16a\x08CW`@Qc\x08\xA9D\x19`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x04\x80T`@Qc\x02&^1`\xE6\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x84\x81\x16\x93\x82\x01\x93\x90\x93R3\x92\x90\x91\x16\x90c\x89\x97\x8C@\x90`$\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x08\x92W=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x08\xB6\x91\x90a>\xFCV[`\x01`\x01`\xA0\x1B\x03\x16\x14a\x08\xDDW`@Qc\xAC\xD5\xA8#`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x08\xE8\x84\x84\x84a\x17jV[`@Qc#\xB8r\xDD`\xE0\x1B\x81R3`\x04\x82\x01R0`$\x82\x01R`\x01`\x01``\x1B\x03\x83\x16`D\x82\x01R\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\x01`\x01`\xA0\x1B\x03\x16\x90c#\xB8r\xDD\x90`d\x01` `@Q\x80\x83\x03\x81`\0\x87Z\xF1\x15\x80\x15a\tcW=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\t\x87\x91\x90a?BV[\x15\x15`\x01\x14a\t\xA9W`@Qc\x02.%\x81`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[PPPPV[`\x04T\x83\x90`\x01`\xA0\x1B\x90\x04`\xFF\x16a\t\xDBW`@Qc\x08\xA9D\x19`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x04\x80T`@Qc\x02&^1`\xE6\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x84\x81\x16\x93\x82\x01\x93\x90\x93R3\x92\x90\x91\x16\x90c\x89\x97\x8C@\x90`$\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\n*W=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\nN\x91\x90a>\xFCV[`\x01`\x01`\xA0\x1B\x03\x16\x14a\nuW`@Qc\xAC\xD5\xA8#`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\t\xA9\x84\x84\x84a\x1B\x16V[`\x04T`\x01`\xA0\x1B\x90\x04`\xFF\x16a\n\xAAW`@Qc\x08\xA9D\x19`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x04\x80T`@Qc\x02&^1`\xE6\x1B\x81R3\x92\x81\x01\x92\x90\x92R`\0\x91`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x90c\x89\x97\x8C@\x90`$\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\n\xF8W=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x0B\x1C\x91\x90a>\xFCV[`\x01`\x01`\xA0\x1B\x03\x16\x14a\x0BCW`@Qc\xAC\xD5\xA8#`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x0BM3\x82a\"\x13V[PV[`\x04T`\x01`\xA0\x1B\x90\x04`\xFF\x16a\x0BzW`@Qc\x08\xA9D\x19`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x04\x80T`@Qc\x02&^1`\xE6\x1B\x81R3\x92\x81\x01\x92\x90\x92R`\0\x91`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x90c\x89\x97\x8C@\x90`$\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x0B\xC8W=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x0B\xEC\x91\x90a>\xFCV[`\x01`\x01`\xA0\x1B\x03\x16\x14a\x0C\x13W`@Qc\xAC\xD5\xA8#`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x0BM3\x82a#\x8FV[`\0\x80a\x0C.\x83a\x01\0\x015a%\x13V[\x90P`\0a\x0CB`\xC0\x85\x01`\xA0\x86\x01a?dV[f\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16`8a\x0C]`\xA0\x87\x01`\x80\x88\x01a?\x8DV[b\xFF\xFF\xFF\x16\x90\x1B`Pa\x0Cv`\x80\x88\x01``\x89\x01a?\xB2V[c\xFF\xFF\xFF\xFF\x16\x90\x1B`pa\x0C\x90``\x89\x01`@\x8A\x01a?\xD8V[e\xFF\xFF\xFF\xFF\xFF\xFF\x16\x90\x1B`\xA0a\x0C\xAC`@\x8A\x01` \x8B\x01a@\0V[`\x01`\x01``\x1B\x03\x16\x90\x1B\x17\x17\x17\x17\x90P`\0c\xFC\xB7yo`\xE0\x1B\x85`\0\x01`\0\x015\x83\x85`@Q` \x01a\r\x01\x93\x92\x91\x90\x92\x83R` \x83\x01\x91\x90\x91R``\x1B`\x01`\x01``\x1B\x03\x19\x16`@\x82\x01R`T\x01\x90V[`@\x80Q\x80\x83\x03`\x1F\x19\x01\x81R\x82\x82R\x80Q` \x91\x82\x01 `\x01`\x01`\xE0\x1B\x03\x19\x94\x90\x94\x16\x81\x84\x01R\x82\x82\x01\x93\x90\x93R\x80Q\x80\x83\x03\x82\x01\x81R``\x83\x01\x82R\x80Q\x90\x84\x01 `\x05T`\x19`\xF8\x1B`\x80\x85\x01R`\x01`\xF8\x1B`\x81\x85\x01R`\x82\x84\x01R`\xA2\x80\x84\x01\x91\x90\x91R\x81Q\x80\x84\x03\x90\x91\x01\x81R`\xC2\x90\x92\x01\x90R\x80Q\x91\x01 \x95\x94PPPPPV[`\0\x82\x81R` \x81\x81R`@\x80\x83 `\x01`\x01`\xA0\x1B\x03\x85\x16\x84R\x90\x91R\x81 T`\xFF\x16a\r\xB9W`\0a\r\xDBV[\x7F\xA2\xEFF\0\xD7B\x02-S-GG\xCB5GGFg\xD6\xF18\x04\x90%\x13\xB2\xEC\x01\xC8H\xF4\xB4[\x90P[\x92\x91PPV[`\x04T\x82\x90`\x01`\xA0\x1B\x90\x04`\xFF\x16a\x0E\x10W`@Qc\x08\xA9D\x19`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x04\x80T`@Qc\x02&^1`\xE6\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x84\x81\x16\x93\x82\x01\x93\x90\x93R3\x92\x90\x91\x16\x90c\x89\x97\x8C@\x90`$\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x0E_W=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x0E\x83\x91\x90a>\xFCV[`\x01`\x01`\xA0\x1B\x03\x16\x14a\x0E\xAAW`@Qc\xAC\xD5\xA8#`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x0E\xB4\x83\x83a\"\x13V[PPPV[`\x04T\x82\x90`\x01`\xA0\x1B\x90\x04`\xFF\x16a\x0E\xE5W`@Qc\x08\xA9D\x19`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x04\x80T`@Qc\x02&^1`\xE6\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x84\x81\x16\x93\x82\x01\x93\x90\x93R3\x92\x90\x91\x16\x90c\x89\x97\x8C@\x90`$\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x0F4W=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x0FX\x91\x90a>\xFCV[`\x01`\x01`\xA0\x1B\x03\x16\x14a\x0F\x7FW`@Qc\xAC\xD5\xA8#`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x0E\xB4\x83\x83a#\x8FV[`\x04T`\x01`\xA0\x1B\x90\x04`\xFF\x16a\x0F\xB3W`@Qc\x08\xA9D\x19`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x04\x80T`@Qc\x02&^1`\xE6\x1B\x81R3\x92\x81\x01\x92\x90\x92R`\0\x91`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x90c\x89\x97\x8C@\x90`$\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x10\x01W=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x10%\x91\x90a>\xFCV[`\x01`\x01`\xA0\x1B\x03\x16\x14a\x10LW`@Qc\xAC\xD5\xA8#`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x0BM3\x82a%\xD0V[`@\x80Q\x80\x82\x01\x82R`\x0C\x81RkHoprChannels`\xA0\x1B` \x91\x82\x01R\x81Q\x80\x83\x01\x83R`\x05\x81Rd\x03\"\xE3\x02\xE3`\xDC\x1B\x90\x82\x01R\x81Q\x7F\x8Bs\xC3\xC6\x9B\xB8\xFE=Q.\xCCL\xF7Y\xCCy#\x9F{\x17\x9B\x0F\xFA\xCA\xA9\xA7]R+9@\x0F\x91\x81\x01\x91\x90\x91R\x7F\x84\xE6\x90\x8F46\x01\xD9\xCE\x9F\xB6\r\x82P9N\xB8\xA5\x1CV\xF1\x87k\xC1\xE0\x17\xC9z\xCDeg\xF2\x91\x81\x01\x91\x90\x91R\x7F\xB4\xBC\xB1T\xE3\x86\x01\xC3\x899o\xA9\x181M\xA4-F&\xF1>\xF6\xD0\xCE\xB0~_]&\xB2\xFB\xC3``\x82\x01RF`\x80\x82\x01R0`\xA0\x82\x01R`\0\x90`\xC0\x01`@Q` \x81\x83\x03\x03\x81R\x90`@R\x80Q\x90` \x01 \x90P`\x05T\x81\x14a\x0BMW`\x05\x81\x90U`@Q\x81\x90\x7Fw\x1FR@\xAE_\xD8\xA7d\r?\xB8/\xA7\n\xAB/\xB1\xDB\xF3_.\xF4d\xF8P\x99Fqvd\xC5\x90`\0\x90\xA2PV[`@\x80Q` \x80\x82\x01\x86\x90R\x835\x82\x84\x01R\x83\x81\x015``\x83\x01Ra\x01\0\x85\x015`\x80\x83\x01R`\xC0\x80\x86\x01\x805`\xA0\x80\x86\x01\x91\x90\x91R`\xE0\x80\x89\x015\x84\x87\x01R\x86Q\x80\x87\x03\x90\x94\x01\x84R\x90\x94\x01\x90\x94R\x80Q\x91\x01 `\0\x92`\xC8\x91\x90\x91\x1C\x91a\x11\xDA\x91\x90\x86\x01a?dV[f\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x90\x81\x16\x91\x16\x11\x15\x94\x93PPPPV[``\x81g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x12\x0CWa\x12\x0Ca@\x1BV[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x12?W\x81` \x01[``\x81R` \x01\x90`\x01\x90\x03\x90\x81a\x12*W\x90P[P\x90P`\0[\x82\x81\x10\x15a\x12\xDFWa\x12\xAF0\x85\x85\x84\x81\x81\x10a\x12cWa\x12ca@1V[\x90P` \x02\x81\x01\x90a\x12u\x91\x90a@GV[\x80\x80`\x1F\x01` \x80\x91\x04\x02` \x01`@Q\x90\x81\x01`@R\x80\x93\x92\x91\x90\x81\x81R` \x01\x83\x83\x80\x82\x847`\0\x92\x01\x91\x90\x91RPa' \x92PPPV[\x82\x82\x81Q\x81\x10a\x12\xC1Wa\x12\xC1a@1V[` \x02` \x01\x01\x81\x90RP\x80\x80a\x12\xD7\x90a@\x8EV[\x91PPa\x12EV[P\x92\x91PPV[`\x04T\x82\x90`\x01`\xA0\x1B\x90\x04`\xFF\x16a\x13\x12W`@Qc\x08\xA9D\x19`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x04\x80T`@Qc\x02&^1`\xE6\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x84\x81\x16\x93\x82\x01\x93\x90\x93R3\x92\x90\x91\x16\x90c\x89\x97\x8C@\x90`$\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x13aW=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x13\x85\x91\x90a>\xFCV[`\x01`\x01`\xA0\x1B\x03\x16\x14a\x13\xACW`@Qc\xAC\xD5\xA8#`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x0E\xB4\x83\x83a%\xD0V[`@Q`\x01`\x01``\x1B\x03\x19``\x84\x81\x1B\x82\x16` \x84\x01R\x83\x90\x1B\x16`4\x82\x01R`\0\x90`H\x01`@Q` \x81\x83\x03\x03\x81R\x90`@R\x80Q\x90` \x01 \x90P\x92\x91PPV[`@\x80Q\x80\x82\x01\x82R`\n\x81Ri$7\xB89&2\xB23\xB2\xB9`\xB1\x1B` \x91\x82\x01R\x81Q\x80\x83\x01\x83R`\x05\x81Rd\x03\x12\xE3\x02\xE3`\xDC\x1B\x90\x82\x01R\x81Q\x7F\x8Bs\xC3\xC6\x9B\xB8\xFE=Q.\xCCL\xF7Y\xCCy#\x9F{\x17\x9B\x0F\xFA\xCA\xA9\xA7]R+9@\x0F\x81\x83\x01R\x7Fl\xD6\x81y\x0Cx\xC2 Q{\t\x9As\x7F\x8E\x85\xF6\x9Eyz\xBEN)\x10\xFB\x18\x9Ba\xDBK\xF2\xCD\x81\x84\x01R\x7F\x06\xC0\x15\xBD\"\xB4\xC6\x96\x90\x93<\x10X\x87\x8E\xBD\xFE\xF3\x1F\x9A\xAA\xE4\x0B\xBE\x86\xD8\xA0\x9F\xE1\xB2\x97,``\x82\x01RF`\x80\x82\x01R0`\xA0\x80\x83\x01\x91\x90\x91R\x83Q\x80\x83\x03\x90\x91\x01\x81R`\xC0\x90\x91\x01\x90\x92R\x81Q\x91\x01 `\x03T\x81\x14a\x0BMW`\x03\x81\x90U`@Q\x81\x90\x7F\xA4?\xAD\x83\x92\x0F\xD0\x94E\x85^\x85Ns\xC9\xC52\xE1t\x02\xC9\xCE\xB0\x99\x93\xA29(C\xA5\xBD\xB9\x90`\0\x90\xA2PV[`\x04T`\x01`\xA0\x1B\x90\x04`\xFF\x16a\x153W`@Qc\x08\xA9D\x19`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x04\x80T`@Qc\x02&^1`\xE6\x1B\x81R3\x92\x81\x01\x92\x90\x92R`\0\x91`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x90c\x89\x97\x8C@\x90`$\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x15\x81W=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x15\xA5\x91\x90a>\xFCV[`\x01`\x01`\xA0\x1B\x03\x16\x14a\x15\xCCW`@Qc\xAC\xD5\xA8#`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x15\xD73\x83\x83a\x17jV[`@Qc#\xB8r\xDD`\xE0\x1B\x81R3`\x04\x82\x01R0`$\x82\x01R`\x01`\x01``\x1B\x03\x82\x16`D\x82\x01R\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\x01`\x01`\xA0\x1B\x03\x16\x90c#\xB8r\xDD\x90`d\x01` `@Q\x80\x83\x03\x81`\0\x87Z\xF1\x15\x80\x15a\x16RW=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x16v\x91\x90a?BV[\x15\x15`\x01\x14a\x16\x98W`@Qc\x02.%\x81`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[PPV[`\x04T`\x01`\xA0\x1B\x90\x04`\xFF\x16a\x16\xC6W`@Qc\x08\xA9D\x19`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x04\x80T`@Qc\x02&^1`\xE6\x1B\x81R3\x92\x81\x01\x92\x90\x92R`\0\x91`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x90c\x89\x97\x8C@\x90`$\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x17\x14W=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x178\x91\x90a>\xFCV[`\x01`\x01`\xA0\x1B\x03\x16\x14a\x17_W`@Qc\xAC\xD5\xA8#`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x16\x983\x83\x83a\x1B\x16V[\x80`\x01`\x01`\x01``\x1B\x03\x82\x16\x10\x15a\x17\x96W`@Qc\xC5.>\xFF`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[j\x08E\x95\x16\x14\x01HJ\0\0\0`\x01`\x01``\x1B\x03\x82\x16\x11\x15a\x17\xCBW`@Qc)<\xEE\xF9`\xE2\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x83\x83\x80`\x01`\x01`\xA0\x1B\x03\x16\x82`\x01`\x01`\xA0\x1B\x03\x16\x03a\x17\xFFW`@QcK\xD1\xD7i`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01`\x01`\xA0\x1B\x03\x82\x16a\x18[W`@Qc\xEA\xC0\xD3\x89`\xE0\x1B\x81R` `\x04\x82\x01R`\x18`$\x82\x01R\x7Fsource must not be empty\0\0\0\0\0\0\0\0`D\x82\x01R`d\x01[`@Q\x80\x91\x03\x90\xFD[`\x01`\x01`\xA0\x1B\x03\x81\x16a\x18\xB2W`@Qc\xEA\xC0\xD3\x89`\xE0\x1B\x81R` `\x04\x82\x01R`\x1D`$\x82\x01R\x7Fdestination must not be empty\0\0\0`D\x82\x01R`d\x01a\x18RV[`\0a\x18\xBE\x87\x87a\x13\xB6V[`\0\x81\x81R`\x06` R`@\x90 \x90\x91P`\x02\x81T`\x01`\xC8\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a\x18\xEEWa\x18\xEEa<\xBFV[\x03a\x19OW`@QcI\x94c\xC1`\xE0\x1B\x81R` `\x04\x82\x01R`*`$\x82\x01R\x7Fcannot fund a channel that will `D\x82\x01Ri1\xB67\xB9\xB2\x909\xB7\xB7\xB7`\xB1\x1B`d\x82\x01R`\x84\x01a\x18RV[\x80Ta\x19e\x90\x87\x90`\x01`\x01``\x1B\x03\x16a@\xA7V[\x81T`\x01`\x01``\x1B\x03\x19\x16`\x01`\x01``\x1B\x03\x91\x90\x91\x16\x17\x81U`\0\x81T`\x01`\xC8\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a\x19\x9FWa\x19\x9Fa<\xBFV[\x03a\x1A\xAAW\x80Ta\x19\xBD\x90`\x01`\xB0\x1B\x90\x04b\xFF\xFF\xFF\x16`\x01a@\xC7V[\x81Tb\xFF\xFF\xFF\x91\x90\x91\x16`\x01`\xB0\x1B\x02m\xFF\0\0\0\0\0\0\0\xFF\xFF\xFF\xFF\xFF\xFF``\x1B\x19\x16m\xFF\xFF\xFF\xFF\0\0\0\0\xFF\xFF\xFF\xFF\xFF\xFF``\x1B\x19\x90\x91\x16\x17`\x01`\xC8\x1B\x17\x81U`@\x80Q\x7F\xDD\x90\xF98#\x035\xE5\x9D\xC9%\xC5~\xCB\x0E'\xA2\x8C-\x875n1\xF0\x0C\xD5UJ\xBDl\x1B-` \x82\x01R``\x8A\x81\x1B`\x01`\x01``\x1B\x03\x19\x90\x81\x16\x93\x83\x01\x93\x90\x93R\x89\x90\x1B\x90\x91\x16`T\x82\x01Ra\x1Ai\x90`h\x01[`@Q` \x81\x83\x03\x03\x81R\x90`@Ra'EV[\x86`\x01`\x01`\xA0\x1B\x03\x16\x88`\x01`\x01`\xA0\x1B\x03\x16\x7F\xDD\x90\xF98#\x035\xE5\x9D\xC9%\xC5~\xCB\x0E'\xA2\x8C-\x875n1\xF0\x0C\xD5UJ\xBDl\x1B-`@Q`@Q\x80\x91\x03\x90\xA3[\x80T`@Qa\x1A\xDD\x91a\x1AU\x91`\0\x80Q` aB\xC9\x839\x81Q\x91R\x91\x86\x91`\x01`\x01``\x1B\x03\x90\x91\x16\x90` \x01a@\xE3V[\x80T`@Q`\x01`\x01``\x1B\x03\x90\x91\x16\x81R\x82\x90`\0\x80Q` aB\xC9\x839\x81Q\x91R\x90` \x01`@Q\x80\x91\x03\x90\xA2PPPPPPPPV[a\x1B&`@\x83\x01` \x84\x01a@\0V[`\x01`\x01`\x01``\x1B\x03\x82\x16\x10\x15a\x1BQW`@Qc\xC5.>\xFF`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[j\x08E\x95\x16\x14\x01HJ\0\0\0`\x01`\x01``\x1B\x03\x82\x16\x11\x15a\x1B\x86W`@Qc)<\xEE\xF9`\xE2\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x82a\x01\0\x015a\x1B\x95\x81a(+V[a\x1B\xB2W`@Qc:\xE4\xEDk`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x835`\0\x90\x81R`\x06` R`@\x90 `\x01\x81T`\x01`\xC8\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a\x1B\xE1Wa\x1B\xE1a<\xBFV[\x14\x15\x80\x15a\x1C\x0CWP`\x02\x81T`\x01`\xC8\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a\x1C\tWa\x1C\ta<\xBFV[\x14\x15[\x15a\x1CtW`@QcI\x94c\xC1`\xE0\x1B\x81R` `\x04\x82\x01R`1`$\x82\x01R\x7Fspending channel must be OPEN or`D\x82\x01Rp PENDING_TO_CLOSE`x\x1B`d\x82\x01R`\x84\x01a\x18RV[a\x1C\x84`\xA0\x86\x01`\x80\x87\x01a?\x8DV[\x81T`\x01`\xB0\x1B\x90\x04b\xFF\xFF\xFF\x90\x81\x16\x91\x16\x14a\x1C\xE4W`@QcI\x94c\xC1`\xE0\x1B\x81R` `\x04\x82\x01R`\x18`$\x82\x01R\x7Fchannel epoch must match\0\0\0\0\0\0\0\0`D\x82\x01R`d\x01a\x18RV[`\0a\x1C\xF6``\x87\x01`@\x88\x01a?\xD8V[\x90P`\0a\x1D\n`\x80\x88\x01``\x89\x01a?\xB2V[\x83T\x90\x91P`\x01``\x1B\x90\x04e\xFF\xFF\xFF\xFF\xFF\xFF\x16`\x01c\xFF\xFF\xFF\xFF\x83\x16\x10\x80a\x1DBWP\x80e\xFF\xFF\xFF\xFF\xFF\xFF\x16\x83e\xFF\xFF\xFF\xFF\xFF\xFF\x16\x10[\x15a\x1D`W`@Qchn\x1E\x0F`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x1Dp`@\x89\x01` \x8A\x01a@\0V[\x84T`\x01`\x01``\x1B\x03\x91\x82\x16\x91\x16\x10\x15a\x1D\x9EW`@Qc,Q\xD8\xDB`\xE2\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0a\x1D\xA9\x89a\x0C\x1DV[\x90Pa\x1D\xB6\x81\x8A\x8Aa\x11oV[a\x1D\xD3W`@Qc\xEE\x83\\\x89`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0`@Q\x80``\x01`@R\x80\x83\x81R` \x01\x8C`\x01`\x01`\xA0\x1B\x03\x16\x81R` \x01`\x05T`@Q` \x01a\x1E\n\x91\x81R` \x01\x90V[`@\x80Q`\x1F\x19\x81\x84\x03\x01\x81R\x91\x90R\x90R\x90Pa\x1E6a\x1E06\x8B\x90\x03\x8B\x01\x8BaA\x06V[\x82a(MV[a\x1ESW`@Qc\x12\xBF\xB7\xB7`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0a\x1Eh\x83`\xC0\x8D\x015`\xE0\x8E\x015a*\xD6V[\x90P\x8A5a\x1Ev\x82\x8Ea\x13\xB6V[\x14a\x1E\x94W`@Qcf\xEE\xA9\xAB`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x1E\xA4c\xFF\xFF\xFF\xFF\x86\x16\x87aA\xA4V[\x87Te\xFF\xFF\xFF\xFF\xFF\xFF\x91\x90\x91\x16`\x01``\x1B\x02e\xFF\xFF\xFF\xFF\xFF\xFF``\x1B\x19\x90\x91\x16\x17\x87Ua\x1E\xD8`@\x8C\x01` \x8D\x01a@\0V[\x87Ta\x1E\xED\x91\x90`\x01`\x01``\x1B\x03\x16aA\xC3V[\x87T`\x01`\x01``\x1B\x03\x19\x16`\x01`\x01``\x1B\x03\x91\x90\x91\x16\x90\x81\x17\x88U`@Qa\x1FB\x91a\x1AU\x91\x7F\"\xE2\xA4\"\xA8\x86\x06V\xA3\xA3<\xFA\x1D\xAFw\x1Evy\x8C\xE5d\x97G\x95r5\x02]\xE1.\x0B$\x91\x8F5\x91` \x01a@\xE3V[\x86T`@Q`\x01`\x01``\x1B\x03\x90\x91\x16\x81R\x8B5\x90\x7F\"\xE2\xA4\"\xA8\x86\x06V\xA3\xA3<\xFA\x1D\xAFw\x1Evy\x8C\xE5d\x97G\x95r5\x02]\xE1.\x0B$\x90` \x01`@Q\x80\x91\x03\x90\xA2`\0a\x1F\x90\x8D\x83a\x13\xB6V[\x90P`\0`\x06`\0\x83\x81R` \x01\x90\x81R` \x01`\0 \x90Pa \x1C\x7Fqe\xE2\xEB\xC7\xCE5\xCC\x98\xCBvf\xF9\x94[6\x17\xF3\xF3c&\xB7m\x18\x93{\xA5\xFE\xCF\x18s\x9A\x8E`\0\x01`\0\x015\x8B`\0\x01`\x0C\x90T\x90a\x01\0\n\x90\x04e\xFF\xFF\xFF\xFF\xFF\xFF\x16`@Q` \x01a\x1AU\x93\x92\x91\x90\x92\x83R` \x83\x01\x91\x90\x91R`\xD0\x1B`\x01`\x01`\xD0\x1B\x03\x19\x16`@\x82\x01R`F\x01\x90V[\x88T`@Q`\x01``\x1B\x90\x91\x04e\xFF\xFF\xFF\xFF\xFF\xFF\x16\x81R\x8D5\x90\x7Fqe\xE2\xEB\xC7\xCE5\xCC\x98\xCBvf\xF9\x94[6\x17\xF3\xF3c&\xB7m\x18\x93{\xA5\xFE\xCF\x18s\x9A\x90` \x01`@Q\x80\x91\x03\x90\xA2`\0\x81T`\x01`\xC8\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a \x82Wa \x82a<\xBFV[\x03a!lW\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\x01`\x01`\xA0\x1B\x03\x16c\xA9\x05\x9C\xBB3\x8F`\0\x01` \x01` \x81\x01\x90a \xCD\x91\x90a@\0V[`@Q`\x01`\x01`\xE0\x1B\x03\x19`\xE0\x85\x90\x1B\x16\x81R`\x01`\x01`\xA0\x1B\x03\x90\x92\x16`\x04\x83\x01R`\x01`\x01``\x1B\x03\x16`$\x82\x01R`D\x01` `@Q\x80\x83\x03\x81`\0\x87Z\xF1\x15\x80\x15a!!W=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a!E\x91\x90a?BV[\x15\x15`\x01\x14a!gW`@Qc\x02.%\x81`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\"\x03V[a!|`@\x8E\x01` \x8F\x01a@\0V[\x81Ta!\x91\x91\x90`\x01`\x01``\x1B\x03\x16a@\xA7V[\x81T`\x01`\x01``\x1B\x03\x19\x16`\x01`\x01``\x1B\x03\x91\x90\x91\x16\x90\x81\x17\x82U`@Qa!\xD3\x91a\x1AU\x91`\0\x80Q` aB\xC9\x839\x81Q\x91R\x91\x86\x91` \x01a@\xE3V[\x80T`@Q`\x01`\x01``\x1B\x03\x90\x91\x16\x81R\x82\x90`\0\x80Q` aB\xC9\x839\x81Q\x91R\x90` \x01`@Q\x80\x91\x03\x90\xA2[PPPPPPPPPPPPPPV[`\0a\"\x1F\x82\x84a\x13\xB6V[`\0\x81\x81R`\x06` R`@\x81 \x91\x92P\x81T`\x01`\xC8\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a\"MWa\"Ma<\xBFV[\x03a\"kW`@QcI\x94c\xC1`\xE0\x1B\x81R`\x04\x01a\x18R\x90aA\xE3V[\x80T`\x01c\xFF\0\0\x01`\xB0\x1B\x03\x19\x81\x16\x82U`@\x80Q`\0\x80Q` aB\xE9\x839\x81Q\x91R` \x82\x01R\x90\x81\x01\x84\x90R`\x01`\x01``\x1B\x03\x90\x91\x16\x90a\"\xB3\x90``\x01a\x1AUV[`@Q\x83\x90`\0\x80Q` aB\xE9\x839\x81Q\x91R\x90`\0\x90\xA2\x80\x15a#\x88W`@Qc\xA9\x05\x9C\xBB`\xE0\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x85\x81\x16`\x04\x83\x01R`$\x82\x01\x83\x90R\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x16\x90c\xA9\x05\x9C\xBB\x90`D\x01[` `@Q\x80\x83\x03\x81`\0\x87Z\xF1\x15\x80\x15a#BW=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a#f\x91\x90a?BV[\x15\x15`\x01\x14a#\x88W`@Qc\x02.%\x81`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[PPPPPV[`\0a#\x9B\x83\x83a\x13\xB6V[`\0\x81\x81R`\x06` R`@\x90 \x90\x91P`\x02\x81T`\x01`\xC8\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a#\xCBWa#\xCBa<\xBFV[\x14a$(W`@QcI\x94c\xC1`\xE0\x1B\x81R` `\x04\x82\x01R`&`$\x82\x01R\x7Fchannel state must be PENDING_TO`D\x82\x01Re_CLOSE`\xD0\x1B`d\x82\x01R`\x84\x01a\x18RV[\x80Tc\xFF\xFF\xFF\xFFB\x81\x16`\x01`\x90\x1B\x90\x92\x04\x16\x10a$YW`@Qc8\xB2\x01\x95`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x80T`\x01c\xFF\0\0\x01`\xB0\x1B\x03\x19\x81\x16\x82U`@\x80Q`\0\x80Q` aB\xE9\x839\x81Q\x91R` \x82\x01R\x90\x81\x01\x84\x90R`\x01`\x01``\x1B\x03\x90\x91\x16\x90a$\xA1\x90``\x01a\x1AUV[`@Q\x83\x90`\0\x80Q` aB\xE9\x839\x81Q\x91R\x90`\0\x90\xA2\x80\x15a#\x88W`@Qc\xA9\x05\x9C\xBB`\xE0\x1B\x81R3`\x04\x82\x01R`$\x81\x01\x82\x90R\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\x01`\x01`\xA0\x1B\x03\x16\x90c\xA9\x05\x9C\xBB\x90`D\x01a##V[`\0`\x01\x81`\x1B\x7Fy\xBEf~\xF9\xDC\xBB\xACU\xA0b\x95\xCE\x87\x0B\x07\x02\x9B\xFC\xDB-\xCE(\xD9Y\xF2\x81[\x16\xF8\x17\x98p\x01EQ#\x19P\xB7_\xC4@-\xA1s/\xC9\xBE\xBE\x19\x7Fy\xBEf~\xF9\xDC\xBB\xACU\xA0b\x95\xCE\x87\x0B\x07\x02\x9B\xFC\xDB-\xCE(\xD9Y\xF2\x81[\x16\xF8\x17\x98\x87\t`@\x80Q`\0\x81R` \x81\x01\x80\x83R\x95\x90\x95R`\xFF\x90\x93\x16\x92\x84\x01\x92\x90\x92R``\x83\x01R`\x80\x82\x01R`\xA0\x01` `@Q` \x81\x03\x90\x80\x84\x03\x90\x85Z\xFA\x15\x80\x15a%\xBFW=`\0\x80>=`\0\xFD[PP`@Q`\x1F\x19\x01Q\x93\x92PPPV[`\0a%\xDC\x83\x83a\x13\xB6V[`\0\x81\x81R`\x06` R`@\x81 \x91\x92P\x81T`\x01`\xC8\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a&\nWa&\na<\xBFV[\x03a&(W`@QcI\x94c\xC1`\xE0\x1B\x81R`\x04\x01a\x18R\x90aA\xE3V[a&R\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0BaB3V[\x81T`\x01`\xC9\x1Bg\xFF\0\0\0\xFF\xFF\xFF\xFF`\x90\x1B\x19\x90\x91\x16`\xFF`\xC8\x1B\x19`\x01`\x90\x1Bc\xFF\xFF\xFF\xFF\x94\x90\x94\x16\x84\x02\x16\x17\x17\x80\x83U`@\x80Q\x7F\x07\xB5\xC9PY\x7F\xC3\xBE\xD9.*\xD3\x7F\xA8Op\x16U\xAC\xB3r\x98.Ho_\xAD6\x07\xF0J\\` \x82\x01R\x90\x81\x01\x85\x90R\x91\x90\x04`\xE0\x1B`\x01`\x01`\xE0\x1B\x03\x19\x16``\x82\x01Ra&\xD6\x90`d\x01a\x1AUV[\x80T`@Q`\x01`\x90\x1B\x90\x91\x04c\xFF\xFF\xFF\xFF\x16\x81R\x82\x90\x7F\x07\xB5\xC9PY\x7F\xC3\xBE\xD9.*\xD3\x7F\xA8Op\x16U\xAC\xB3r\x98.Ho_\xAD6\x07\xF0J\\\x90` \x01`@Q\x80\x91\x03\x90\xA2PPPPV[``a\r\xDB\x83\x83`@Q\x80``\x01`@R\x80`'\x81R` \x01aB\xA2`'\x919a*\xFCV[`\x01T`\0\x90a'\x83\x90\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x90`\x01`\xE0\x1B\x90\x04c\xFF\xFF\xFF\xFF\x16a?/V[B\x11\x15a'\x8EWP`\x01[`\x03T`\x01T\x83Q` \x80\x86\x01\x91\x90\x91 `@\x80Q\x80\x84\x01\x95\x90\x95RC`\xE0\x1B`\x01`\x01`\xE0\x1B\x03\x19\x16\x90\x85\x01R\x91\x90\x1Bc\xFF\xFF\xFF\xFF\x19\x16`D\x83\x01R``\x82\x01R`\x80\x01`@\x80Q`\x1F\x19\x81\x84\x03\x01\x81R\x91\x90R\x80Q` \x91\x82\x01 c\xFF\xFF\xFF\xFFB\x16`\x01`\xE0\x1B\x02\x91\x1C\x17`\x01U\x80\x15a\x16\x98WPP`\x01T`\x01`\x01`\xE0\x1B\x03\x81\x16`\x01`\xE0\x1B\x91\x82\x90\x04c\xFF\xFF\xFF\xFF\x16\x90\x91\x02\x17`\x02UV[`\0\x81\x15\x80a\r\xDEWPPp\x01EQ#\x19P\xB7_\xC4@-\xA1s/\xC9\xBE\xBE\x19\x11\x90V[`\0d\x01\0\0\x03\xD0\x19\x83``\x01Q\x10\x15\x80a(rWPd\x01\0\0\x03\xD0\x19\x83`@\x01Q\x10\x15[\x15a(\x90W`@Qc:\xE4\xEDk`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a(\xA2\x83`\0\x01Q\x84` \x01Qa+tV[a(\xBFW`@Qc9\"\xA5A`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0\x80a)\x11\x84` \x01Q\x85`\0\x01Q`@Q` \x01a(\xF8\x92\x91\x90``\x92\x90\x92\x1B`\x01`\x01``\x1B\x03\x19\x16\x82R`\x14\x82\x01R`4\x01\x90V[`@Q` \x81\x83\x03\x03\x81R\x90`@R\x85`@\x01Qa+\x9FV[\x91P\x91P`\0a)&\x86`@\x01Q\x84\x84a,%V[\x90Pa)a\x86`\x80\x01Q\x87`\xA0\x01Q`@\x80Q` \x80\x82\x01\x94\x90\x94R\x80\x82\x01\x92\x90\x92R\x80Q\x80\x83\x03\x82\x01\x81R``\x90\x92\x01\x90R\x80Q\x91\x01 \x90V[`\x01`\x01`\xA0\x1B\x03\x16\x81`\x01`\x01`\xA0\x1B\x03\x16\x14a)\x92W`@Qc\x1D\xBF\xB9\xB3`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0a)\xAB\x87``\x01Q\x88`\0\x01Q\x89` \x01Qa,%V[\x90Pa)\xE6\x87`\xC0\x01Q\x88`\xE0\x01Q`@\x80Q` \x80\x82\x01\x94\x90\x94R\x80\x82\x01\x92\x90\x92R\x80Q\x80\x83\x03\x82\x01\x81R``\x90\x92\x01\x90R\x80Q\x91\x01 \x90V[`\x01`\x01`\xA0\x1B\x03\x16\x81`\x01`\x01`\xA0\x1B\x03\x16\x14a*\x17W`@Qc\x1D\xBF\xB9\xB3`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0\x80a*I\x89`\x80\x01Q\x8A`\xA0\x01Q\x8B`\xC0\x01Q\x8C`\xE0\x01Qd\x01\0\0\x03\xD0\x19a*B\x91\x90aBPV[`\0a,\xC4V[` \x80\x8B\x01Q\x8CQ\x8D\x83\x01Q\x8DQ`@Q\x96\x98P\x94\x96P`\0\x95a*\xC1\x95a*\xA8\x95\x8A\x92\x8A\x92\x91\x01``\x96\x90\x96\x1B`\x01`\x01``\x1B\x03\x19\x16\x86R`\x14\x86\x01\x94\x90\x94R`4\x85\x01\x92\x90\x92R`T\x84\x01R`t\x83\x01R`\x94\x82\x01R`\xB4\x01\x90V[`@Q` \x81\x83\x03\x03\x81R\x90`@R\x8A`@\x01Qa.KV[``\x8B\x01Q\x14\x97PPPPPPPP\x92\x91PPV[`\0\x80`\0a*\xE6\x86\x86\x86a.\xBCV[\x91P\x91Pa*\xF3\x81a.\xF5V[P\x94\x93PPPPV[```\0\x80\x85`\x01`\x01`\xA0\x1B\x03\x16\x85`@Qa+\x19\x91\x90aBcV[`\0`@Q\x80\x83\x03\x81\x85Z\xF4\x91PP=\x80`\0\x81\x14a+TW`@Q\x91P`\x1F\x19`?=\x01\x16\x82\x01`@R=\x82R=`\0` \x84\x01>a+YV[``\x91P[P\x91P\x91Pa+j\x86\x83\x83\x87a0?V[\x96\x95PPPPPPV[`\0d\x01\0\0\x03\xD0\x19\x80\x84d\x01\0\0\x03\xD0\x19\x86\x87\t\t`\x07\x08d\x01\0\0\x03\xD0\x19\x83\x84\t\x14\x93\x92PPPV[`\0\x80`\0\x80a+\xAF\x86\x86a0\xC0V[\x91P\x91P`\0\x80a+\xBF\x84a1|V[\x91P\x91P`\0\x80a+\xCF\x85a1|V[\x91P\x91P`\0\x80a,\x03\x86\x86\x86\x86\x7F?\x871\xAB\xDDf\x1A\xDC\xA0\x8AUX\xF0\xF5\xD2r\xE9S\xD3c\xCBo\x0E]@TG\xC0\x1ADE3a,\xC4V[\x91P\x91Pa,\x11\x82\x82a4>V[\x99P\x99PPPPPPPPP[\x92P\x92\x90PV[`\0\x80a,3`\x02\x84aB\x7FV[`\0\x03a,BWP`\x1Ba,FV[P`\x1C[`\x01`\0\x82\x86p\x01EQ#\x19P\xB7_\xC4@-\xA1s/\xC9\xBE\xBE\x19\x88\x8A\t`@\x80Q`\0\x81R` \x81\x01\x80\x83R\x95\x90\x95R`\xFF\x90\x93\x16\x92\x84\x01\x92\x90\x92R``\x83\x01R`\x80\x82\x01R`\xA0\x01` `@Q` \x81\x03\x90\x80\x84\x03\x90\x85Z\xFA\x15\x80\x15a,\xB0W=`\0\x80>=`\0\xFD[PP`@Q`\x1F\x19\x01Q\x96\x95PPPPPPV[`\0\x80\x83\x86\x14\x19\x85\x88\x14\x16\x15a,\xD9W`\0\x80\xFD[`\0\x80\x85\x88\x14\x87\x8A\x14\x16`\x01\x81\x14a,\xF6W\x80\x15a-sWa-\xEEV[d\x01\0\0\x03\xD0\x19\x86d\x01\0\0\x03\xD0\x19\x8B`\x02\t\x08\x91P`@Q` \x81R` \x80\x82\x01R` `@\x82\x01R\x82``\x82\x01Rd\x01\0\0\x03\xD2\x19`\x80\x82\x01Rd\x01\0\0\x03\xD0\x19`\xA0\x82\x01R` \x81`\xC0\x83`\x05`\0\x19\xFAa-SW`\0\x80\xFD[d\x01\0\0\x03\xD0\x19\x81Qd\x01\0\0\x03\xD0\x19\x80\x8E\x8F\t`\x03\t\t\x93PPa-\xEEV[d\x01\0\0\x03\xD0\x19\x8Ad\x01\0\0\x03\xD0\x19\x03\x89\x08\x91P`@Q` \x81R` \x80\x82\x01R` `@\x82\x01R\x82``\x82\x01Rd\x01\0\0\x03\xD2\x19`\x80\x82\x01Rd\x01\0\0\x03\xD0\x19`\xA0\x82\x01R` \x81`\xC0\x83`\x05`\0\x19\xFAa-\xCEW`\0\x80\xFD[d\x01\0\0\x03\xD0\x19\x81Qd\x01\0\0\x03\xD0\x19\x8Cd\x01\0\0\x03\xD0\x19\x03\x8B\x08\t\x93PP[PPd\x01\0\0\x03\xD0\x19\x80\x89d\x01\0\0\x03\xD0\x19\x03\x88d\x01\0\0\x03\xD0\x19\x03\x08d\x01\0\0\x03\xD0\x19\x83\x84\t\x08\x92Pd\x01\0\0\x03\xD0\x19\x87d\x01\0\0\x03\xD0\x19\x03d\x01\0\0\x03\xD0\x19\x80\x86d\x01\0\0\x03\xD0\x19\x03\x8C\x08\x84\t\x08\x91PP\x95P\x95\x93PPPPV[`\0\x80`\0a.Z\x85\x85a7+V[\x91P\x91P`@Q`0\x81R` \x80\x82\x01R` `@\x82\x01R\x82``\x82\x01R\x81`\x80\x82\x01R`\x01`\x90\x82\x01Rp\x01EQ#\x19P\xB7_\xC4@-\xA1s/\xC9\xBE\xBE\x19`\xB0\x82\x01R` \x81`\xD0\x83`\x05`\0\x19\xFAa.\xB2W`\0\x80\xFD[Q\x95\x94PPPPPV[`\0\x80`\x01`\x01`\xFF\x1B\x03\x83\x16\x81a.\xD9`\xFF\x86\x90\x1C`\x1Ba?/V[\x90Pa.\xE7\x87\x82\x88\x85a8+V[\x93P\x93PPP\x93P\x93\x91PPV[`\0\x81`\x04\x81\x11\x15a/\tWa/\ta<\xBFV[\x03a/\x11WPV[`\x01\x81`\x04\x81\x11\x15a/%Wa/%a<\xBFV[\x03a/rW`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`\x18`$\x82\x01R\x7FECDSA: invalid signature\0\0\0\0\0\0\0\0`D\x82\x01R`d\x01a\x18RV[`\x02\x81`\x04\x81\x11\x15a/\x86Wa/\x86a<\xBFV[\x03a/\xD3W`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`\x1F`$\x82\x01R\x7FECDSA: invalid signature length\0`D\x82\x01R`d\x01a\x18RV[`\x03\x81`\x04\x81\x11\x15a/\xE7Wa/\xE7a<\xBFV[\x03a\x0BMW`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`\"`$\x82\x01R\x7FECDSA: invalid signature 's' val`D\x82\x01Raue`\xF0\x1B`d\x82\x01R`\x84\x01a\x18RV[``\x83\x15a0\xAEW\x82Q`\0\x03a0\xA7W`\x01`\x01`\xA0\x1B\x03\x85\x16;a0\xA7W`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`\x1D`$\x82\x01R\x7FAddress: call to non-contract\0\0\0`D\x82\x01R`d\x01a\x18RV[P\x81a0\xB8V[a0\xB8\x83\x83a8\xEFV[\x94\x93PPPPV[`\0\x80`\0\x80`\0a0\xD2\x87\x87a9\x19V[\x92P\x92P\x92P`@Q`0\x81R` \x80\x82\x01R` `@\x82\x01R\x83``\x82\x01R\x82`\x80\x82\x01R`\x01`\x90\x82\x01Rd\x01\0\0\x03\xD0\x19`\xB0\x82\x01R` \x81`\xD0\x83`\x05`\0\x19\xFAa1 W`\0\x80\xFD[\x80Q\x95PP`@Q`0\x81R` \x80\x82\x01R\x82`P\x82\x01R` `@\x82\x01R\x81`p\x82\x01R`\x01`\x90\x82\x01Rd\x01\0\0\x03\xD0\x19`\xB0\x82\x01R` \x81`\xD0\x83`\x05`\0\x19\xFAa1mW`\0\x80\xFD[\x80Q\x94PPPPP\x92P\x92\x90PV[`\0\x80d\x01\0\0\x03\xD0\x19\x83\x84\td\x01\0\0\x03\xD0\x19\x81d\x01\0\0\x03\xDB\x19\t\x90Pd\x01\0\0\x03\xD0\x19\x81\x82\td\x01\0\0\x03\xD0\x19\x82\x82\x08\x90Pd\x01\0\0\x03\xD0\x19`\x01\x82\x08d\x01\0\0\x03\xD0\x19a\x06\xEB\x82\t\x90P`\0\x82\x15`\x01\x81\x14a1\xE1W\x80\x15a1\xEFWa1\xFBV[d\x01\0\0\x03\xDB\x19\x91Pa1\xFBV[\x83d\x01\0\0\x03\xD0\x19\x03\x91P[Pd\x01\0\0\x03\xD0\x19\x81\x7F?\x871\xAB\xDDf\x1A\xDC\xA0\x8AUX\xF0\xF5\xD2r\xE9S\xD3c\xCBo\x0E]@TG\xC0\x1ADE3\t\x90Pd\x01\0\0\x03\xD0\x19\x82\x83\t\x92Pd\x01\0\0\x03\xD0\x19\x81\x82\td\x01\0\0\x03\xD0\x19\x81\x7F?\x871\xAB\xDDf\x1A\xDC\xA0\x8AUX\xF0\xF5\xD2r\xE9S\xD3c\xCBo\x0E]@TG\xC0\x1ADE3\td\x01\0\0\x03\xD0\x19\x81\x86\x08\x94Pd\x01\0\0\x03\xD0\x19\x84\x86\t\x94Pd\x01\0\0\x03\xD0\x19\x83\x83\t\x91Pd\x01\0\0\x03\xD0\x19\x82a\x06\xEB\t\x90Pd\x01\0\0\x03\xD0\x19\x81\x86\x08\x94PPd\x01\0\0\x03\xD0\x19\x83\x86\t\x96P`\0\x80d\x01\0\0\x03\xD0\x19\x83\x84\td\x01\0\0\x03\xD0\x19\x84\x88\td\x01\0\0\x03\xD0\x19\x81\x83\t\x91P`@Q` \x81R` \x80\x82\x01R` `@\x82\x01R\x82``\x82\x01Rc@\0\0\xF5`\x01`\xFE\x1B\x03`\x80\x82\x01Rd\x01\0\0\x03\xD0\x19`\xA0\x82\x01R` \x81`\xC0\x83`\x05`\0\x19\xFAa3!W`\0\x80\xFD[d\x01\0\0\x03\xD0\x19\x82\x82Q\t\x92PPPd\x01\0\0\x03\xD0\x19\x7F1\xFD\xF3\x02r@\x13\xE5z\xD1?\xB3\x8F\x84*\xFE\xEC\x18O\0\xA7G\x89\xDD(g)\xC80<JY\x82\td\x01\0\0\x03\xD0\x19\x82\x83\td\x01\0\0\x03\xD0\x19\x86\x82\t\x90P\x88\x81\x14`\x01\x81\x14a3\x86W\x80\x15a3\x92Wa3\x9AV[`\x01\x94P\x83\x95Pa3\x9AV[`\0\x94P\x82\x95P[PPPPd\x01\0\0\x03\xD0\x19\x8A\x88\t\x97Pd\x01\0\0\x03\xD0\x19\x82\x89\t\x97P\x80\x15a3\xC3W\x84\x98P\x81\x97P[PPP`\x02\x85\x06`\x02\x88\x06\x14a3\xDFW\x84d\x01\0\0\x03\xD0\x19\x03\x94P[`@Q\x93P` \x84R` \x80\x85\x01R` `@\x85\x01R\x80``\x85\x01RPPPd\x01\0\0\x03\xD2\x19`\x80\x82\x01Rd\x01\0\0\x03\xD0\x19`\xA0\x82\x01R` \x81`\xC0\x83`\x05`\0\x19\xFAa4+W`\0\x80\xFD[d\x01\0\0\x03\xD0\x19\x81Q\x84\t\x92PP\x91P\x91V[`\0\x80d\x01\0\0\x03\xD0\x19\x84\x85\td\x01\0\0\x03\xD0\x19\x81\x86\td\x01\0\0\x03\xD0\x19\x80\x7F\x8E8\xE3\x8E8\xE3\x8E8\xE3\x8E8\xE3\x8E8\xE3\x8E8\xE3\x8E8\xE3\x8E8\xE3\x8E8\xE3\x8D\xAA\xAA\xA8\xC7d\x01\0\0\x03\xD0\x19\x89\x7F\x07\xD3\xD4\xC8\x0B\xC3!\xD5\xB9\xF3\x15\xCE\xA7\xFDD\xC5\xD5\x95\xD2\xFC\x0B\xF6;\x92\xDF\xFF\x10D\xF1|e\x81\t\x08d\x01\0\0\x03\xD0\x19\x80\x85\x7FSL2\x8D#\xF24\xE6\xE2\xA4\x13\xDE\xCA%\xCA\xEC\xE4PaD\x03|@1N\xCB\xD0\xB5=\x9D\xD2b\td\x01\0\0\x03\xD0\x19\x85\x7F\x8E8\xE3\x8E8\xE3\x8E8\xE3\x8E8\xE3\x8E8\xE3\x8E8\xE3\x8E8\xE3\x8E8\xE3\x8E8\xE3\x8D\xAA\xAA\xA8\x8C\t\x08\x08d\x01\0\0\x03\xD0\x19\x7F\xD3Wq\x19=\x94\x91\x8A\x9C\xA3L\xCB\xB7\xB6@\xDD\x86\xCD@\x95B\xF8H}\x9F\xE6\xB7Ex\x1E\xB4\x9Bd\x01\0\0\x03\xD0\x19\x80\x8A\x7F\xED\xAD\xC6\xF6C\x83\xDC\x1D\xF7\xC4\xB2\xD5\x1BT\"T\x06\xD3kd\x1F^A\xBB\xC5*Va*\x8Cm\x14\t\x86\x08\x08`@Q` \x81R` \x80\x82\x01R` `@\x82\x01R\x81``\x82\x01Rd\x01\0\0\x03\xD2\x19`\x80\x82\x01Rd\x01\0\0\x03\xD0\x19`\xA0\x82\x01R` \x81`\xC0\x83`\x05`\0\x19\xFAa5\x9DW`\0\x80\xFD[\x80Q\x91Pd\x01\0\0\x03\xD0\x19\x82\x84\t\x96Pd\x01\0\0\x03\xD0\x19\x80\x7FK\xDA\x12\xF6\x84\xBD\xA1/hK\xDA\x12\xF6\x84\xBD\xA1/hK\xDA\x12\xF6\x84\xBD\xA1/hK\x8E8\xE2<d\x01\0\0\x03\xD0\x19\x8C\x7F\xC7^\x0C2\xD5\xCB|\x0F\xA9\xD0\xA5K\x12\xA0\xA6\xD5dz\xB0F\xD6\x86\xDAo\xDF\xFC\x90\xFC \x1Dq\xA3\t\x08d\x01\0\0\x03\xD0\x19\x80\x88\x7F)\xA6\x19F\x91\xF9\x1AsqR\t\xEFe\x12\xE5vr(0\xA2\x01\xBE \x18\xA7e\xE8Z\x9E\xCE\xE91\td\x01\0\0\x03\xD0\x19\x88\x7F/hK\xDA\x12\xF6\x84\xBD\xA1/hK\xDA\x12\xF6\x84\xBD\xA1/hK\xDA\x12\xF6\x84\xBD\xA1/8\xE3\x8D\x84\t\x08\x08\x92Pd\x01\0\0\x03\xD0\x19\x80d\x01\0\0\x06\xC4\x19d\x01\0\0\x03\xD0\x19\x8C\x7Fz\x06SK\xB8\xBD\xB4\x9F\xD5\xE9\xE6c'\"\xC2\x98\x94g\xC1\xBF\xC8\xE8\xD9x\xDF\xB4%\xD2h\\%s\t\x08d\x01\0\0\x03\xD0\x19\x80\x88\x7Fd\x84\xAAqeE\xCA,\xF3\xA7\x0C?\xA8\xFE3~\n=!\x16/\rb\x99\xA7\xBF\x81\x92\xBF\xD2\xA7o\t\x87\x08\x08\x94P`@Q\x90P` \x81R` \x80\x82\x01R` `@\x82\x01R\x84``\x82\x01Rd\x01\0\0\x03\xD2\x19`\x80\x82\x01Rd\x01\0\0\x03\xD0\x19`\xA0\x82\x01R` \x81`\xC0\x83`\x05`\0\x19\xFAa7\rW`\0\x80\xFD[Q\x93Pd\x01\0\0\x03\xD0\x19\x90P\x83\x81\x83\x89\t\t\x93PPPP\x92P\x92\x90PV[`\0\x80`\xFF\x83Q\x11\x15a7=W`\0\x80\xFD[`\0`@Q`\x88` `\0[\x88Q\x81\x10\x15a7jW\x88\x82\x01Q\x84\x84\x01R` \x92\x83\x01\x92\x91\x82\x01\x91\x01a7IV[PP`\x89\x87Q\x01\x90P`0\x81\x83\x01S`\x02\x01` `\0[\x87Q\x81\x10\x15a7\xA2W\x87\x82\x01Q\x84\x84\x01R` \x92\x83\x01\x92\x91\x82\x01\x91\x01a7\x81V[PP`\x8B\x86Q\x88Q\x01\x01\x90P\x85Q\x81\x83\x01SP\x85Q\x85Q\x01`\x8C\x01\x81 \x91PP`@Q\x81\x81R`\x01` \x82\x01S`!` `\0[\x87Q\x81\x10\x15a7\xF7W\x87\x82\x01Q\x84\x84\x01R` \x92\x83\x01\x92\x91\x82\x01\x91\x01a7\xD6V[PPP\x84Q\x85Q`!\x01\x82\x01S\x84Q`\"\x01\x81 \x93P\x83\x82\x18\x81R`\x02` \x82\x01S\x84Q`\"\x01\x81 \x92PPP\x92P\x92\x90PV[`\0\x80\x7F\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF]WnsW\xA4P\x1D\xDF\xE9/Fh\x1B \xA0\x83\x11\x15a8bWP`\0\x90P`\x03a8\xE6V[`@\x80Q`\0\x80\x82R` \x82\x01\x80\x84R\x89\x90R`\xFF\x88\x16\x92\x82\x01\x92\x90\x92R``\x81\x01\x86\x90R`\x80\x81\x01\x85\x90R`\x01\x90`\xA0\x01` `@Q` \x81\x03\x90\x80\x84\x03\x90\x85Z\xFA\x15\x80\x15a8\xB6W=`\0\x80>=`\0\xFD[PP`@Q`\x1F\x19\x01Q\x91PP`\x01`\x01`\xA0\x1B\x03\x81\x16a8\xDFW`\0`\x01\x92P\x92PPa8\xE6V[\x91P`\0\x90P[\x94P\x94\x92PPPV[\x81Q\x15a8\xFFW\x81Q\x80\x83` \x01\xFD[\x80`@QbF\x1B\xCD`\xE5\x1B\x81R`\x04\x01a\x18R\x91\x90a>\x86V[`\0\x80`\0`\xFF\x84Q\x11\x15a9-W`\0\x80\xFD[`\0`@Q`\x88` `\0[\x89Q\x81\x10\x15a9ZW\x89\x82\x01Q\x84\x84\x01R` \x92\x83\x01\x92\x91\x82\x01\x91\x01a99V[PP`\x89\x88Q\x01\x90P``\x81\x83\x01S`\x02\x01` `\0[\x88Q\x81\x10\x15a9\x92W\x88\x82\x01Q\x84\x84\x01R` \x92\x83\x01\x92\x91\x82\x01\x91\x01a9qV[PP`\x8B\x87Q\x89Q\x01\x01\x90P\x86Q\x81\x83\x01SP\x86Q\x86Q\x01`\x8C\x01\x81 \x91PP`@Q\x81\x81R`\x01` \x82\x01S`!` `\0[\x88Q\x81\x10\x15a9\xE7W\x88\x82\x01Q\x84\x84\x01R` \x92\x83\x01\x92\x91\x82\x01\x91\x01a9\xC6V[PPP\x85Q\x86Q`!\x01\x82\x01S\x85Q`\"\x01\x81 \x94P\x84\x82\x18\x81R`\x02` \x82\x01S\x85Q`\"\x01\x81 \x93P\x83\x82\x18\x81R`\x03` \x82\x01S\x85Q`\"\x01\x81 \x92PPP\x92P\x92P\x92V[`\x01`\x01`\xA0\x1B\x03\x81\x16\x81\x14a\x0BMW`\0\x80\xFD[`\0\x80\x83`\x1F\x84\x01\x12a:WW`\0\x80\xFD[P\x815g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a:oW`\0\x80\xFD[` \x83\x01\x91P\x83` \x82\x85\x01\x01\x11\x15a,\x1EW`\0\x80\xFD[`\0\x80`\0\x80`\0\x80`\0\x80`\xC0\x89\x8B\x03\x12\x15a:\xA3W`\0\x80\xFD[\x885a:\xAE\x81a:0V[\x97P` \x89\x015a:\xBE\x81a:0V[\x96P`@\x89\x015a:\xCE\x81a:0V[\x95P``\x89\x015\x94P`\x80\x89\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x80\x82\x11\x15a:\xF2W`\0\x80\xFD[a:\xFE\x8C\x83\x8D\x01a:EV[\x90\x96P\x94P`\xA0\x8B\x015\x91P\x80\x82\x11\x15a;\x17W`\0\x80\xFD[Pa;$\x8B\x82\x8C\x01a:EV[\x99\x9C\x98\x9BP\x96\x99P\x94\x97\x93\x96\x92\x95\x94PPPV[\x805`\x01`\x01``\x1B\x03\x81\x16\x81\x14a;OW`\0\x80\xFD[\x91\x90PV[`\0\x80`\0``\x84\x86\x03\x12\x15a;iW`\0\x80\xFD[\x835a;t\x81a:0V[\x92P` \x84\x015a;\x84\x81a:0V[\x91Pa;\x92`@\x85\x01a;8V[\x90P\x92P\x92P\x92V[`\0a\x01 \x82\x84\x03\x12\x15a;\xAEW`\0\x80\xFD[P\x91\x90PV[`\0a\x01\0\x82\x84\x03\x12\x15a;\xAEW`\0\x80\xFD[`\0\x80`\0a\x02@\x84\x86\x03\x12\x15a;\xDDW`\0\x80\xFD[\x835a;\xE8\x81a:0V[\x92Pa;\xF7\x85` \x86\x01a;\x9BV[\x91Pa;\x92\x85a\x01@\x86\x01a;\xB4V[`\0` \x82\x84\x03\x12\x15a<\x19W`\0\x80\xFD[\x815a<$\x81a:0V[\x93\x92PPPV[`\0a\x01 \x82\x84\x03\x12\x15a<>W`\0\x80\xFD[a\r\xDB\x83\x83a;\x9BV[`\0\x80`@\x83\x85\x03\x12\x15a<[W`\0\x80\xFD[\x825\x91P` \x83\x015a<m\x81a:0V[\x80\x91PP\x92P\x92\x90PV[`\0\x80`@\x83\x85\x03\x12\x15a<\x8BW`\0\x80\xFD[\x825a<\x96\x81a:0V[\x91P` \x83\x015a<m\x81a:0V[`\0` \x82\x84\x03\x12\x15a<\xB8W`\0\x80\xFD[P5\x91\x90PV[cNH{q`\xE0\x1B`\0R`!`\x04R`$`\0\xFD[`\x01`\x01``\x1B\x03\x86\x16\x81Re\xFF\xFF\xFF\xFF\xFF\xFF\x85\x16` \x82\x01Rc\xFF\xFF\xFF\xFF\x84\x16`@\x82\x01Rb\xFF\xFF\xFF\x83\x16``\x82\x01R`\xA0\x81\x01`\x03\x83\x10a=(WcNH{q`\xE0\x1B`\0R`!`\x04R`$`\0\xFD[\x82`\x80\x83\x01R\x96\x95PPPPPPV[`\0\x80`\0a\x02@\x84\x86\x03\x12\x15a=NW`\0\x80\xFD[\x835\x92Pa;\xF7\x85` \x86\x01a;\x9BV[`\0\x80` \x83\x85\x03\x12\x15a=rW`\0\x80\xFD[\x825g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x80\x82\x11\x15a=\x8AW`\0\x80\xFD[\x81\x85\x01\x91P\x85`\x1F\x83\x01\x12a=\x9EW`\0\x80\xFD[\x815\x81\x81\x11\x15a=\xADW`\0\x80\xFD[\x86` \x82`\x05\x1B\x85\x01\x01\x11\x15a=\xC2W`\0\x80\xFD[` \x92\x90\x92\x01\x96\x91\x95P\x90\x93PPPPV[`\0[\x83\x81\x10\x15a=\xEFW\x81\x81\x01Q\x83\x82\x01R` \x01a=\xD7V[PP`\0\x91\x01RV[`\0\x81Q\x80\x84Ra>\x10\x81` \x86\x01` \x86\x01a=\xD4V[`\x1F\x01`\x1F\x19\x16\x92\x90\x92\x01` \x01\x92\x91PPV[`\0` \x80\x83\x01\x81\x84R\x80\x85Q\x80\x83R`@\x86\x01\x91P`@\x81`\x05\x1B\x87\x01\x01\x92P\x83\x87\x01`\0[\x82\x81\x10\x15a>yW`?\x19\x88\x86\x03\x01\x84Ra>g\x85\x83Qa=\xF8V[\x94P\x92\x85\x01\x92\x90\x85\x01\x90`\x01\x01a>KV[P\x92\x97\x96PPPPPPPV[` \x81R`\0a\r\xDB` \x83\x01\x84a=\xF8V[`\0\x80`@\x83\x85\x03\x12\x15a>\xACW`\0\x80\xFD[\x825a>\xB7\x81a:0V[\x91Pa>\xC5` \x84\x01a;8V[\x90P\x92P\x92\x90PV[`\0\x80a\x02 \x83\x85\x03\x12\x15a>\xE2W`\0\x80\xFD[a>\xEC\x84\x84a;\x9BV[\x91Pa>\xC5\x84a\x01 \x85\x01a;\xB4V[`\0` \x82\x84\x03\x12\x15a?\x0EW`\0\x80\xFD[\x81Qa<$\x81a:0V[cNH{q`\xE0\x1B`\0R`\x11`\x04R`$`\0\xFD[\x80\x82\x01\x80\x82\x11\x15a\r\xDEWa\r\xDEa?\x19V[`\0` \x82\x84\x03\x12\x15a?TW`\0\x80\xFD[\x81Q\x80\x15\x15\x81\x14a<$W`\0\x80\xFD[`\0` \x82\x84\x03\x12\x15a?vW`\0\x80\xFD[\x815f\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x16\x81\x14a<$W`\0\x80\xFD[`\0` \x82\x84\x03\x12\x15a?\x9FW`\0\x80\xFD[\x815b\xFF\xFF\xFF\x81\x16\x81\x14a<$W`\0\x80\xFD[`\0` \x82\x84\x03\x12\x15a?\xC4W`\0\x80\xFD[\x815c\xFF\xFF\xFF\xFF\x81\x16\x81\x14a<$W`\0\x80\xFD[`\0` \x82\x84\x03\x12\x15a?\xEAW`\0\x80\xFD[\x815e\xFF\xFF\xFF\xFF\xFF\xFF\x81\x16\x81\x14a<$W`\0\x80\xFD[`\0` \x82\x84\x03\x12\x15a@\x12W`\0\x80\xFD[a\r\xDB\x82a;8V[cNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[cNH{q`\xE0\x1B`\0R`2`\x04R`$`\0\xFD[`\0\x80\x835`\x1E\x19\x846\x03\x01\x81\x12a@^W`\0\x80\xFD[\x83\x01\x805\x91Pg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x15a@yW`\0\x80\xFD[` \x01\x91P6\x81\x90\x03\x82\x13\x15a,\x1EW`\0\x80\xFD[`\0`\x01\x82\x01a@\xA0Wa@\xA0a?\x19V[P`\x01\x01\x90V[`\x01`\x01``\x1B\x03\x81\x81\x16\x83\x82\x16\x01\x90\x80\x82\x11\x15a\x12\xDFWa\x12\xDFa?\x19V[b\xFF\xFF\xFF\x81\x81\x16\x83\x82\x16\x01\x90\x80\x82\x11\x15a\x12\xDFWa\x12\xDFa?\x19V[\x92\x83R` \x83\x01\x91\x90\x91R`\xA0\x1B`\x01`\x01`\xA0\x1B\x03\x19\x16`@\x82\x01R`L\x01\x90V[`\0a\x01\0\x80\x83\x85\x03\x12\x15aA\x1AW`\0\x80\xFD[`@Q\x90\x81\x01\x90g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x81\x83\x10\x17\x15aAKWcNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[\x81`@R\x835\x81R` \x84\x015` \x82\x01R`@\x84\x015`@\x82\x01R``\x84\x015``\x82\x01R`\x80\x84\x015`\x80\x82\x01R`\xA0\x84\x015`\xA0\x82\x01R`\xC0\x84\x015`\xC0\x82\x01R`\xE0\x84\x015`\xE0\x82\x01R\x80\x92PPP\x92\x91PPV[e\xFF\xFF\xFF\xFF\xFF\xFF\x81\x81\x16\x83\x82\x16\x01\x90\x80\x82\x11\x15a\x12\xDFWa\x12\xDFa?\x19V[`\x01`\x01``\x1B\x03\x82\x81\x16\x82\x82\x16\x03\x90\x80\x82\x11\x15a\x12\xDFWa\x12\xDFa?\x19V[` \x80\x82R`0\x90\x82\x01R\x7Fchannel must have state OPEN or `@\x82\x01RoPENDING_TO_CLOSE`\x80\x1B``\x82\x01R`\x80\x01\x90V[c\xFF\xFF\xFF\xFF\x81\x81\x16\x83\x82\x16\x01\x90\x80\x82\x11\x15a\x12\xDFWa\x12\xDFa?\x19V[\x81\x81\x03\x81\x81\x11\x15a\r\xDEWa\r\xDEa?\x19V[`\0\x82QaBu\x81\x84` \x87\x01a=\xD4V[\x91\x90\x91\x01\x92\x91PPV[`\0\x82aB\x9CWcNH{q`\xE0\x1B`\0R`\x12`\x04R`$`\0\xFD[P\x06\x90V\xFEAddress: low-level delegate call failed_\xA1rF\xD3\xA5\xD6\x8DB\xBA\xA9L\xDE3\x04!\x80\xB7\x83\xA3\x99\xC0+\xF6:\xC2\x07n\x0Fp\x878\xCE\xEA\xB2\xEE\xF9\x98\xC1\x7F\xE9o0\xF8?\xBF<U\xFCPG\xF6\xE4\x0CU\xA0\xCFr\xD26\xE9\xD2\xBAr\xA2dipfsX\"\x12 #C\x98\r\x92\x99\x8E\xDA\xEE\x11\xA6gb5\xEBu\x89\x9E\x11\x17\x08\xED\n*3T\\\xCD\xCB\x0E\x05\x03dsolcC\0\x08\x13\x003\x8Bs\xC3\xC6\x9B\xB8\xFE=Q.\xCCL\xF7Y\xCCy#\x9F{\x17\x9B\x0F\xFA\xCA\xA9\xA7]R+9@\x0F",
    );
    /// The runtime bytecode of the contract, as deployed on the network.
    ///
    /// ```text
    ///0x608060405234801561001057600080fd5b50600436106101e45760003560e01c80637c8e28da1161010f578063c966c4fe116100a2578063fc0c546a11610071578063fc0c546a146104d2578063fc55309a14610511578063fcb7796f14610524578063ffa1ad741461053757600080fd5b8063c966c4fe14610487578063dc96fd5014610490578063ddad190214610498578063f698da25146104c957600080fd5b8063ac9650d8116100de578063ac9650d81461043b578063b920deed1461045b578063bda65f4514610461578063be9babdc1461047457600080fd5b80637c8e28da146103c157806387352d65146103d457806389ccfe89146104105780638c3710c91461041857600080fd5b806329392e3211610187578063651514bf11610156578063651514bf146102ef57806372581cc01461030257806378d8016d146103295780637a7ebd7b1461035057600080fd5b806329392e321461028357806344dae6f8146102a357806354a2edf5146102ca5780635d2f07c5146102dd57600080fd5b80631a7ffe7a116101c35780631a7ffe7a1461022457806323cb3ac01461023757806324086cc21461024a578063249cb3fa1461027057600080fd5b806223de29146101e95780630abec58f146101fe5780630cd88d7214610211575b600080fd5b6101fc6101f7366004613a87565b61055b565b005b6101fc61020c366004613b54565b610817565b6101fc61021f366004613bc7565b6109af565b6101fc610232366004613c07565b610a80565b6101fc610245366004613c07565b610b50565b61025d610258366004613c2b565b610c1d565b6040519081526020015b60405180910390f35b61025d61027e366004613c48565b610d8a565b61028b600181565b6040516001600160601b039091168152602001610267565b61025d7f000000000000000000000000000000000000000000000000000000000000000081565b6101fc6102d8366004613c78565b610de4565b61028b6a084595161401484a00000081565b6101fc6102fd366004613c78565b610eb9565b61025d7fb281fc8c12954d22544db45de3159a39272895b169a852b314f9cc762e44c53b81565b61025d7f000000000000000000000000000000000000000000000000000000000000000081565b6103b061035e366004613ca6565b6006602052600090815260409020546001600160601b03811690600160601b810465ffffffffffff1690600160901b810463ffffffff1690600160b01b810462ffffff1690600160c81b900460ff1685565b604051610267959493929190613cd5565b6101fc6103cf366004613c07565b610f89565b6103fb7f000000000000000000000000000000000000000000000000000000000000000081565b60405163ffffffff9091168152602001610267565b6101fc611056565b61042b610426366004613d38565b61116f565b6040519015158152602001610267565b61044e610449366004613d5f565b6111f1565b6040516102679190613e24565b426103fb565b6101fc61046f366004613c78565b6112e6565b61025d610482366004613c78565b6113b6565b61025d60035481565b6101fc6113fb565b6104bc604051806040016040528060058152602001640312e302e360dc1b81525081565b6040516102679190613e86565b61025d60055481565b6104f97f000000000000000000000000000000000000000000000000000000000000000081565b6040516001600160a01b039091168152602001610267565b6101fc61051f366004613e99565b611509565b6101fc610532366004613ece565b61169c565b6104bc604051806040016040528060058152602001640322e302e360dc1b81525081565b336001600160a01b037f000000000000000000000000000000000000000000000000000000000000000016146105a457604051635079ff7560e11b815260040160405180910390fd5b6001600160a01b03861630146105cd57604051631738922160e31b815260040160405180910390fd5b821561080d577f0000000000000000000000000000000000000000000000000000000000000000830361072e576001600160601b038511156106225760405163293ceef960e21b815260040160405180910390fd5b600480546040516302265e3160e61b81528635606090811c9382018490526014880135901c916000916001600160a01b03909116906389978c4090602401602060405180830381865afa15801561067d573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906106a19190613efc565b9050826001600160a01b03168a6001600160a01b0316036106e9576001600160a01b038116156106e45760405163acd5a82360e01b815260040160405180910390fd5b61071b565b896001600160a01b0316816001600160a01b03161461071b5760405163acd5a82360e01b815260040160405180910390fd5b61072683838a61176a565b50505061080d565b7f000000000000000000000000000000000000000000000000000000000000000083036107f4578335606090811c90601486013560a090811c916020880135901c906034880135901c88158061079957506107956001600160601b03808316908516613f2f565b8914155b156107b75760405163c52e3eff60e01b815260040160405180910390fd5b6001600160601b038316156107d1576107d184838561176a565b6001600160601b038116156107eb576107eb82858361176a565b5050505061080d565b604051630d3dcde560e31b815260040160405180910390fd5b5050505050505050565b6004548390600160a01b900460ff16610843576040516308a9441960e31b815260040160405180910390fd5b600480546040516302265e3160e61b81526001600160a01b03848116938201939093523392909116906389978c4090602401602060405180830381865afa158015610892573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906108b69190613efc565b6001600160a01b0316146108dd5760405163acd5a82360e01b815260040160405180910390fd5b6108e884848461176a565b6040516323b872dd60e01b81523360048201523060248201526001600160601b03831660448201527f00000000000000000000000000000000000000000000000000000000000000006001600160a01b0316906323b872dd906064016020604051808303816000875af1158015610963573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906109879190613f42565b15156001146109a95760405163022e258160e11b815260040160405180910390fd5b50505050565b6004548390600160a01b900460ff166109db576040516308a9441960e31b815260040160405180910390fd5b600480546040516302265e3160e61b81526001600160a01b03848116938201939093523392909116906389978c4090602401602060405180830381865afa158015610a2a573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610a4e9190613efc565b6001600160a01b031614610a755760405163acd5a82360e01b815260040160405180910390fd5b6109a9848484611b16565b600454600160a01b900460ff16610aaa576040516308a9441960e31b815260040160405180910390fd5b600480546040516302265e3160e61b815233928101929092526000916001600160a01b03909116906389978c4090602401602060405180830381865afa158015610af8573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610b1c9190613efc565b6001600160a01b031614610b435760405163acd5a82360e01b815260040160405180910390fd5b610b4d3382612213565b50565b600454600160a01b900460ff16610b7a576040516308a9441960e31b815260040160405180910390fd5b600480546040516302265e3160e61b815233928101929092526000916001600160a01b03909116906389978c4090602401602060405180830381865afa158015610bc8573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610bec9190613efc565b6001600160a01b031614610c135760405163acd5a82360e01b815260040160405180910390fd5b610b4d338261238f565b600080610c2e836101000135612513565b90506000610c4260c0850160a08601613f64565b66ffffffffffffff166038610c5d60a0870160808801613f8d565b62ffffff16901b6050610c766080880160608901613fb2565b63ffffffff16901b6070610c906060890160408a01613fd8565b65ffffffffffff16901b60a0610cac60408a0160208b01614000565b6001600160601b0316901b171717179050600063fcb7796f60e01b85600001600001358385604051602001610d0193929190928352602083019190915260601b6001600160601b031916604082015260540190565b60408051808303601f1901815282825280516020918201206001600160e01b0319949094168184015282820193909352805180830382018152606083018252805190840120600554601960f81b6080850152600160f81b6081850152608284015260a2808401919091528151808403909101815260c29092019052805191012095945050505050565b6000828152602081815260408083206001600160a01b038516845290915281205460ff16610db9576000610ddb565b7fa2ef4600d742022d532d4747cb3547474667d6f13804902513b2ec01c848f4b45b90505b92915050565b6004548290600160a01b900460ff16610e10576040516308a9441960e31b815260040160405180910390fd5b600480546040516302265e3160e61b81526001600160a01b03848116938201939093523392909116906389978c4090602401602060405180830381865afa158015610e5f573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610e839190613efc565b6001600160a01b031614610eaa5760405163acd5a82360e01b815260040160405180910390fd5b610eb48383612213565b505050565b6004548290600160a01b900460ff16610ee5576040516308a9441960e31b815260040160405180910390fd5b600480546040516302265e3160e61b81526001600160a01b03848116938201939093523392909116906389978c4090602401602060405180830381865afa158015610f34573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610f589190613efc565b6001600160a01b031614610f7f5760405163acd5a82360e01b815260040160405180910390fd5b610eb4838361238f565b600454600160a01b900460ff16610fb3576040516308a9441960e31b815260040160405180910390fd5b600480546040516302265e3160e61b815233928101929092526000916001600160a01b03909116906389978c4090602401602060405180830381865afa158015611001573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906110259190613efc565b6001600160a01b03161461104c5760405163acd5a82360e01b815260040160405180910390fd5b610b4d33826125d0565b604080518082018252600c81526b486f70724368616e6e656c7360a01b6020918201528151808301835260058152640322e302e360dc1b9082015281517f8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f918101919091527f84e6908f343601d9ce9fb60d8250394eb8a51c56f1876bc1e017c97acd6567f2918101919091527fb4bcb154e38601c389396fa918314da42d4626f13ef6d0ceb07e5f5d26b2fbc360608201524660808201523060a082015260009060c0016040516020818303038152906040528051906020012090506005548114610b4d57600581905560405181907f771f5240ae5fd8a7640d3fb82fa70aab2fb1dbf35f2ef464f8509946717664c590600090a250565b604080516020808201869052833582840152838101356060830152610100850135608083015260c0808601803560a08086019190915260e0808901358487015286518087039094018452909401909452805191012060009260c89190911c916111da91908601613f64565b66ffffffffffffff90811691161115949350505050565b60608167ffffffffffffffff81111561120c5761120c61401b565b60405190808252806020026020018201604052801561123f57816020015b606081526020019060019003908161122a5790505b50905060005b828110156112df576112af3085858481811061126357611263614031565b90506020028101906112759190614047565b8080601f01602080910402602001604051908101604052809392919081815260200183838082843760009201919091525061272092505050565b8282815181106112c1576112c1614031565b602002602001018190525080806112d79061408e565b915050611245565b5092915050565b6004548290600160a01b900460ff16611312576040516308a9441960e31b815260040160405180910390fd5b600480546040516302265e3160e61b81526001600160a01b03848116938201939093523392909116906389978c4090602401602060405180830381865afa158015611361573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906113859190613efc565b6001600160a01b0316146113ac5760405163acd5a82360e01b815260040160405180910390fd5b610eb483836125d0565b6040516001600160601b0319606084811b8216602084015283901b16603482015260009060480160405160208183030381529060405280519060200120905092915050565b604080518082018252600a8152692437b8392632b233b2b960b11b6020918201528151808301835260058152640312e302e360dc1b9082015281517f8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f818301527f6cd681790c78c220517b099a737f8e85f69e797abe4e2910fb189b61db4bf2cd818401527f06c015bd22b4c69690933c1058878ebdfef31f9aaae40bbe86d8a09fe1b2972c60608201524660808201523060a0808301919091528351808303909101815260c090910190925281519101206003548114610b4d57600381905560405181907fa43fad83920fd09445855e854e73c9c532e17402c9ceb09993a2392843a5bdb990600090a250565b600454600160a01b900460ff16611533576040516308a9441960e31b815260040160405180910390fd5b600480546040516302265e3160e61b815233928101929092526000916001600160a01b03909116906389978c4090602401602060405180830381865afa158015611581573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906115a59190613efc565b6001600160a01b0316146115cc5760405163acd5a82360e01b815260040160405180910390fd5b6115d733838361176a565b6040516323b872dd60e01b81523360048201523060248201526001600160601b03821660448201527f00000000000000000000000000000000000000000000000000000000000000006001600160a01b0316906323b872dd906064016020604051808303816000875af1158015611652573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906116769190613f42565b15156001146116985760405163022e258160e11b815260040160405180910390fd5b5050565b600454600160a01b900460ff166116c6576040516308a9441960e31b815260040160405180910390fd5b600480546040516302265e3160e61b815233928101929092526000916001600160a01b03909116906389978c4090602401602060405180830381865afa158015611714573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906117389190613efc565b6001600160a01b03161461175f5760405163acd5a82360e01b815260040160405180910390fd5b611698338383611b16565b8060016001600160601b03821610156117965760405163c52e3eff60e01b815260040160405180910390fd5b6a084595161401484a0000006001600160601b03821611156117cb5760405163293ceef960e21b815260040160405180910390fd5b8383806001600160a01b0316826001600160a01b0316036117ff57604051634bd1d76960e11b815260040160405180910390fd5b6001600160a01b03821661185b5760405163eac0d38960e01b815260206004820152601860248201527f736f75726365206d757374206e6f7420626520656d707479000000000000000060448201526064015b60405180910390fd5b6001600160a01b0381166118b25760405163eac0d38960e01b815260206004820152601d60248201527f64657374696e6174696f6e206d757374206e6f7420626520656d7074790000006044820152606401611852565b60006118be87876113b6565b600081815260066020526040902090915060028154600160c81b900460ff1660028111156118ee576118ee613cbf565b0361194f5760405163499463c160e01b815260206004820152602a60248201527f63616e6e6f742066756e642061206368616e6e656c20746861742077696c6c2060448201526931b637b9b29039b7b7b760b11b6064820152608401611852565b80546119659087906001600160601b03166140a7565b81546001600160601b0319166001600160601b039190911617815560008154600160c81b900460ff16600281111561199f5761199f613cbf565b03611aaa5780546119bd90600160b01b900462ffffff1660016140c7565b815462ffffff91909116600160b01b026dff00000000000000ffffffffffff60601b19166dffffffff00000000ffffffffffff60601b1990911617600160c81b178155604080517fdd90f938230335e59dc925c57ecb0e27a28c2d87356e31f00cd5554abd6c1b2d602082015260608a811b6001600160601b03199081169383019390935289901b9091166054820152611a69906068015b604051602081830303815290604052612745565b866001600160a01b0316886001600160a01b03167fdd90f938230335e59dc925c57ecb0e27a28c2d87356e31f00cd5554abd6c1b2d60405160405180910390a35b8054604051611add91611a55916000805160206142c98339815191529186916001600160601b03909116906020016140e3565b80546040516001600160601b03909116815282906000805160206142c98339815191529060200160405180910390a25050505050505050565b611b266040830160208401614000565b60016001600160601b0382161015611b515760405163c52e3eff60e01b815260040160405180910390fd5b6a084595161401484a0000006001600160601b0382161115611b865760405163293ceef960e21b815260040160405180910390fd5b826101000135611b958161282b565b611bb257604051633ae4ed6b60e01b815260040160405180910390fd5b8335600090815260066020526040902060018154600160c81b900460ff166002811115611be157611be1613cbf565b14158015611c0c575060028154600160c81b900460ff166002811115611c0957611c09613cbf565b14155b15611c745760405163499463c160e01b815260206004820152603160248201527f7370656e64696e67206368616e6e656c206d757374206265204f50454e206f726044820152702050454e44494e475f544f5f434c4f534560781b6064820152608401611852565b611c8460a0860160808701613f8d565b8154600160b01b900462ffffff908116911614611ce45760405163499463c160e01b815260206004820152601860248201527f6368616e6e656c2065706f6368206d757374206d6174636800000000000000006044820152606401611852565b6000611cf66060870160408801613fd8565b90506000611d0a6080880160608901613fb2565b8354909150600160601b900465ffffffffffff16600163ffffffff83161080611d4257508065ffffffffffff168365ffffffffffff16105b15611d605760405163686e1e0f60e11b815260040160405180910390fd5b611d706040890160208a01614000565b84546001600160601b0391821691161015611d9e57604051632c51d8db60e21b815260040160405180910390fd5b6000611da989610c1d565b9050611db6818a8a61116f565b611dd35760405163ee835c8960e01b815260040160405180910390fd5b600060405180606001604052808381526020018c6001600160a01b03168152602001600554604051602001611e0a91815260200190565b60408051601f1981840301815291905290529050611e36611e30368b90038b018b614106565b8261284d565b611e53576040516312bfb7b760e31b815260040160405180910390fd5b6000611e688360c08d013560e08e0135612ad6565b90508a35611e76828e6113b6565b14611e94576040516366eea9ab60e11b815260040160405180910390fd5b611ea463ffffffff8616876141a4565b875465ffffffffffff91909116600160601b0265ffffffffffff60601b19909116178755611ed860408c0160208d01614000565b8754611eed91906001600160601b03166141c3565b87546001600160601b0319166001600160601b03919091169081178855604051611f4291611a55917f22e2a422a8860656a3a33cfa1daf771e76798ce5649747957235025de12e0b24918f35916020016140e3565b86546040516001600160601b0390911681528b35907f22e2a422a8860656a3a33cfa1daf771e76798ce5649747957235025de12e0b249060200160405180910390a26000611f908d836113b6565b9050600060066000838152602001908152602001600020905061201c7f7165e2ebc7ce35cc98cb7666f9945b3617f3f36326b76d18937ba5fecf18739a8e600001600001358b600001600c9054906101000a900465ffffffffffff16604051602001611a5593929190928352602083019190915260d01b6001600160d01b031916604082015260460190565b8854604051600160601b90910465ffffffffffff1681528d35907f7165e2ebc7ce35cc98cb7666f9945b3617f3f36326b76d18937ba5fecf18739a9060200160405180910390a260008154600160c81b900460ff16600281111561208257612082613cbf565b0361216c577f00000000000000000000000000000000000000000000000000000000000000006001600160a01b031663a9059cbb338f60000160200160208101906120cd9190614000565b6040516001600160e01b031960e085901b1681526001600160a01b0390921660048301526001600160601b031660248201526044016020604051808303816000875af1158015612121573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906121459190613f42565b15156001146121675760405163022e258160e11b815260040160405180910390fd5b612203565b61217c60408e0160208f01614000565b815461219191906001600160601b03166140a7565b81546001600160601b0319166001600160601b039190911690811782556040516121d391611a55916000805160206142c98339815191529186916020016140e3565b80546040516001600160601b03909116815282906000805160206142c98339815191529060200160405180910390a25b5050505050505050505050505050565b600061221f82846113b6565b60008181526006602052604081209192508154600160c81b900460ff16600281111561224d5761224d613cbf565b0361226b5760405163499463c160e01b8152600401611852906141e3565b8054600163ff00000160b01b031981168255604080516000805160206142e983398151915260208201529081018490526001600160601b03909116906122b390606001611a55565b60405183906000805160206142e983398151915290600090a280156123885760405163a9059cbb60e01b81526001600160a01b038581166004830152602482018390527f0000000000000000000000000000000000000000000000000000000000000000169063a9059cbb906044015b6020604051808303816000875af1158015612342573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906123669190613f42565b15156001146123885760405163022e258160e11b815260040160405180910390fd5b5050505050565b600061239b83836113b6565b600081815260066020526040902090915060028154600160c81b900460ff1660028111156123cb576123cb613cbf565b146124285760405163499463c160e01b815260206004820152602660248201527f6368616e6e656c207374617465206d7573742062652050454e44494e475f544f6044820152655f434c4f534560d01b6064820152608401611852565b805463ffffffff428116600160901b9092041610612459576040516338b2019560e11b815260040160405180910390fd5b8054600163ff00000160b01b031981168255604080516000805160206142e983398151915260208201529081018490526001600160601b03909116906124a190606001611a55565b60405183906000805160206142e983398151915290600090a280156123885760405163a9059cbb60e01b8152336004820152602481018290527f00000000000000000000000000000000000000000000000000000000000000006001600160a01b03169063a9059cbb90604401612323565b6000600181601b7f79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f8179870014551231950b75fc4402da1732fc9bebe197f79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f8179887096040805160008152602081018083529590955260ff909316928401929092526060830152608082015260a0016020604051602081039080840390855afa1580156125bf573d6000803e3d6000fd5b5050604051601f1901519392505050565b60006125dc83836113b6565b60008181526006602052604081209192508154600160c81b900460ff16600281111561260a5761260a613cbf565b036126285760405163499463c160e01b8152600401611852906141e3565b6126527f000000000000000000000000000000000000000000000000000000000000000042614233565b8154600160c91b67ff000000ffffffff60901b1990911660ff60c81b19600160901b63ffffffff949094168402161717808355604080517f07b5c950597fc3bed92e2ad37fa84f701655acb372982e486f5fad3607f04a5c602082015290810185905291900460e01b6001600160e01b03191660608201526126d690606401611a55565b8054604051600160901b90910463ffffffff16815282907f07b5c950597fc3bed92e2ad37fa84f701655acb372982e486f5fad3607f04a5c9060200160405180910390a250505050565b6060610ddb83836040518060600160405280602781526020016142a260279139612afc565b600154600090612783907f000000000000000000000000000000000000000000000000000000000000000090600160e01b900463ffffffff16613f2f565b42111561278e575060015b600354600154835160208086019190912060408051808401959095524360e01b6001600160e01b0319169085015291901b63ffffffff19166044830152606082015260800160408051601f19818403018152919052805160209182012063ffffffff4216600160e01b02911c1760015580156116985750506001546001600160e01b038116600160e01b9182900463ffffffff1690910217600255565b6000811580610dde57505070014551231950b75fc4402da1732fc9bebe191190565b60006401000003d019836060015110158061287257506401000003d019836040015110155b1561289057604051633ae4ed6b60e01b815260040160405180910390fd5b6128a283600001518460200151612b74565b6128bf57604051633922a54160e11b815260040160405180910390fd5b600080612911846020015185600001516040516020016128f892919060609290921b6001600160601b0319168252601482015260340190565b6040516020818303038152906040528560400151612b9f565b91509150600061292686604001518484612c25565b905061296186608001518760a00151604080516020808201949094528082019290925280518083038201815260609092019052805191012090565b6001600160a01b0316816001600160a01b03161461299257604051631dbfb9b360e31b815260040160405180910390fd5b60006129ab876060015188600001518960200151612c25565b90506129e68760c001518860e00151604080516020808201949094528082019290925280518083038201815260609092019052805191012090565b6001600160a01b0316816001600160a01b031614612a1757604051631dbfb9b360e31b815260040160405180910390fd5b600080612a4989608001518a60a001518b60c001518c60e001516401000003d019612a429190614250565b6000612cc4565b6020808b01518c518d8301518d51604051969850949650600095612ac195612aa8958a928a92910160609690961b6001600160601b03191686526014860194909452603485019290925260548401526074830152609482015260b40190565b6040516020818303038152906040528a60400151612e4b565b60608b01511497505050505050505092915050565b6000806000612ae6868686612ebc565b91509150612af381612ef5565b50949350505050565b6060600080856001600160a01b031685604051612b199190614263565b600060405180830381855af49150503d8060008114612b54576040519150601f19603f3d011682016040523d82523d6000602084013e612b59565b606091505b5091509150612b6a8683838761303f565b9695505050505050565b60006401000003d01980846401000003d019868709096007086401000003d019838409149392505050565b600080600080612baf86866130c0565b91509150600080612bbf8461317c565b91509150600080612bcf8561317c565b91509150600080612c03868686867f3f8731abdd661adca08a5558f0f5d272e953d363cb6f0e5d405447c01a444533612cc4565b91509150612c11828261343e565b9950995050505050505050505b9250929050565b600080612c3360028461427f565b600003612c425750601b612c46565b50601c5b60016000828670014551231950b75fc4402da1732fc9bebe19888a096040805160008152602081018083529590955260ff909316928401929092526060830152608082015260a0016020604051602081039080840390855afa158015612cb0573d6000803e3d6000fd5b5050604051601f1901519695505050505050565b600080838614198588141615612cd957600080fd5b600080858814878a141660018114612cf6578015612d7357612dee565b6401000003d019866401000003d0198b60020908915060405160208152602080820152602060408201528260608201526401000003d21960808201526401000003d01960a082015260208160c0836005600019fa612d5357600080fd5b6401000003d01981516401000003d019808e8f0960030909935050612dee565b6401000003d0198a6401000003d019038908915060405160208152602080820152602060408201528260608201526401000003d21960808201526401000003d01960a082015260208160c0836005600019fa612dce57600080fd5b6401000003d01981516401000003d0198c6401000003d019038b08099350505b50506401000003d01980896401000003d01903886401000003d01903086401000003d0198384090892506401000003d019876401000003d019036401000003d01980866401000003d019038c088409089150509550959350505050565b6000806000612e5a858561372b565b9150915060405160308152602080820152602060408201528260608201528160808201526001609082015270014551231950b75fc4402da1732fc9bebe1960b082015260208160d0836005600019fa612eb257600080fd5b5195945050505050565b6000806001600160ff1b03831681612ed960ff86901c601b613f2f565b9050612ee78782888561382b565b935093505050935093915050565b6000816004811115612f0957612f09613cbf565b03612f115750565b6001816004811115612f2557612f25613cbf565b03612f725760405162461bcd60e51b815260206004820152601860248201527f45434453413a20696e76616c6964207369676e617475726500000000000000006044820152606401611852565b6002816004811115612f8657612f86613cbf565b03612fd35760405162461bcd60e51b815260206004820152601f60248201527f45434453413a20696e76616c6964207369676e6174757265206c656e677468006044820152606401611852565b6003816004811115612fe757612fe7613cbf565b03610b4d5760405162461bcd60e51b815260206004820152602260248201527f45434453413a20696e76616c6964207369676e6174757265202773272076616c604482015261756560f01b6064820152608401611852565b606083156130ae5782516000036130a7576001600160a01b0385163b6130a75760405162461bcd60e51b815260206004820152601d60248201527f416464726573733a2063616c6c20746f206e6f6e2d636f6e74726163740000006044820152606401611852565b50816130b8565b6130b883836138ef565b949350505050565b60008060008060006130d28787613919565b9250925092506040516030815260208082015260206040820152836060820152826080820152600160908201526401000003d01960b082015260208160d0836005600019fa61312057600080fd5b80519550506040516030815260208082015282605082015260206040820152816070820152600160908201526401000003d01960b082015260208160d0836005600019fa61316d57600080fd5b80519450505050509250929050565b6000806401000003d0198384096401000003d019816401000003db190990506401000003d0198182096401000003d01982820890506401000003d019600182086401000003d0196106eb8209905060008215600181146131e15780156131ef576131fb565b6401000003db1991506131fb565b836401000003d0190391505b506401000003d019817f3f8731abdd661adca08a5558f0f5d272e953d363cb6f0e5d405447c01a4445330990506401000003d01982830992506401000003d0198182096401000003d019817f3f8731abdd661adca08a5558f0f5d272e953d363cb6f0e5d405447c01a444533096401000003d01981860894506401000003d01984860994506401000003d01983830991506401000003d019826106eb0990506401000003d0198186089450506401000003d01983860996506000806401000003d0198384096401000003d0198488096401000003d0198183099150604051602081526020808201526020604082015282606082015263400000f5600160fe1b0360808201526401000003d01960a082015260208160c0836005600019fa61332157600080fd5b6401000003d01982825109925050506401000003d0197f31fdf302724013e57ad13fb38f842afeec184f00a74789dd286729c8303c4a5982096401000003d0198283096401000003d0198682099050888114600181146133865780156133925761339a565b6001945083955061339a565b600094508295505b505050506401000003d0198a880997506401000003d019828909975080156133c3578498508197505b5050506002850660028806146133df57846401000003d0190394505b604051935060208452602080850152602060408501528060608501525050506401000003d21960808201526401000003d01960a082015260208160c0836005600019fa61342b57600080fd5b6401000003d01981518409925050915091565b6000806401000003d0198485096401000003d0198186096401000003d019807f8e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38daaaaa8c76401000003d019897f07d3d4c80bc321d5b9f315cea7fd44c5d595d2fc0bf63b92dfff1044f17c658109086401000003d01980857f534c328d23f234e6e2a413deca25caece4506144037c40314ecbd0b53d9dd262096401000003d019857f8e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38daaaaa88c0908086401000003d0197fd35771193d94918a9ca34ccbb7b640dd86cd409542f8487d9fe6b745781eb49b6401000003d019808a7fedadc6f64383dc1df7c4b2d51b54225406d36b641f5e41bbc52a56612a8c6d140986080860405160208152602080820152602060408201528160608201526401000003d21960808201526401000003d01960a082015260208160c0836005600019fa61359d57600080fd5b805191506401000003d01982840996506401000003d019807f4bda12f684bda12f684bda12f684bda12f684bda12f684bda12f684b8e38e23c6401000003d0198c7fc75e0c32d5cb7c0fa9d0a54b12a0a6d5647ab046d686da6fdffc90fc201d71a309086401000003d01980887f29a6194691f91a73715209ef6512e576722830a201be2018a765e85a9ecee931096401000003d019887f2f684bda12f684bda12f684bda12f684bda12f684bda12f684bda12f38e38d8409080892506401000003d019806401000006c4196401000003d0198c7f7a06534bb8bdb49fd5e9e6632722c2989467c1bfc8e8d978dfb425d2685c257309086401000003d01980887f6484aa716545ca2cf3a70c3fa8fe337e0a3d21162f0d6299a7bf8192bfd2a76f098708089450604051905060208152602080820152602060408201528460608201526401000003d21960808201526401000003d01960a082015260208160c0836005600019fa61370d57600080fd5b5193506401000003d019905083818389090993505050509250929050565b60008060ff8351111561373d57600080fd5b60006040516088602060005b885181101561376a5788820151848401526020928301929182019101613749565b505060898751019050603081830153600201602060005b87518110156137a25787820151848401526020928301929182019101613781565b5050608b8651885101019050855181830153508551855101608c018120915050604051818152600160208201536021602060005b87518110156137f757878201518484015260209283019291820191016137d6565b5050508451855160210182015384516022018120935083821881526002602082015384516022018120925050509250929050565b6000807f7fffffffffffffffffffffffffffffff5d576e7357a4501ddfe92f46681b20a083111561386257506000905060036138e6565b6040805160008082526020820180845289905260ff881692820192909252606081018690526080810185905260019060a0016020604051602081039080840390855afa1580156138b6573d6000803e3d6000fd5b5050604051601f1901519150506001600160a01b0381166138df576000600192509250506138e6565b9150600090505b94509492505050565b8151156138ff5781518083602001fd5b8060405162461bcd60e51b81526004016118529190613e86565b600080600060ff8451111561392d57600080fd5b60006040516088602060005b895181101561395a5789820151848401526020928301929182019101613939565b505060898851019050606081830153600201602060005b88518110156139925788820151848401526020928301929182019101613971565b5050608b8751895101019050865181830153508651865101608c018120915050604051818152600160208201536021602060005b88518110156139e757888201518484015260209283019291820191016139c6565b5050508551865160210182015385516022018120945084821881526002602082015385516022018120935083821881526003602082015385516022018120925050509250925092565b6001600160a01b0381168114610b4d57600080fd5b60008083601f840112613a5757600080fd5b50813567ffffffffffffffff811115613a6f57600080fd5b602083019150836020828501011115612c1e57600080fd5b60008060008060008060008060c0898b031215613aa357600080fd5b8835613aae81613a30565b97506020890135613abe81613a30565b96506040890135613ace81613a30565b955060608901359450608089013567ffffffffffffffff80821115613af257600080fd5b613afe8c838d01613a45565b909650945060a08b0135915080821115613b1757600080fd5b50613b248b828c01613a45565b999c989b5096995094979396929594505050565b80356001600160601b0381168114613b4f57600080fd5b919050565b600080600060608486031215613b6957600080fd5b8335613b7481613a30565b92506020840135613b8481613a30565b9150613b9260408501613b38565b90509250925092565b60006101208284031215613bae57600080fd5b50919050565b60006101008284031215613bae57600080fd5b60008060006102408486031215613bdd57600080fd5b8335613be881613a30565b9250613bf78560208601613b9b565b9150613b92856101408601613bb4565b600060208284031215613c1957600080fd5b8135613c2481613a30565b9392505050565b60006101208284031215613c3e57600080fd5b610ddb8383613b9b565b60008060408385031215613c5b57600080fd5b823591506020830135613c6d81613a30565b809150509250929050565b60008060408385031215613c8b57600080fd5b8235613c9681613a30565b91506020830135613c6d81613a30565b600060208284031215613cb857600080fd5b5035919050565b634e487b7160e01b600052602160045260246000fd5b6001600160601b038616815265ffffffffffff8516602082015263ffffffff8416604082015262ffffff8316606082015260a0810160038310613d2857634e487b7160e01b600052602160045260246000fd5b8260808301529695505050505050565b60008060006102408486031215613d4e57600080fd5b83359250613bf78560208601613b9b565b60008060208385031215613d7257600080fd5b823567ffffffffffffffff80821115613d8a57600080fd5b818501915085601f830112613d9e57600080fd5b813581811115613dad57600080fd5b8660208260051b8501011115613dc257600080fd5b60209290920196919550909350505050565b60005b83811015613def578181015183820152602001613dd7565b50506000910152565b60008151808452613e10816020860160208601613dd4565b601f01601f19169290920160200192915050565b6000602080830181845280855180835260408601915060408160051b870101925083870160005b82811015613e7957603f19888603018452613e67858351613df8565b94509285019290850190600101613e4b565b5092979650505050505050565b602081526000610ddb6020830184613df8565b60008060408385031215613eac57600080fd5b8235613eb781613a30565b9150613ec560208401613b38565b90509250929050565b6000806102208385031215613ee257600080fd5b613eec8484613b9b565b9150613ec5846101208501613bb4565b600060208284031215613f0e57600080fd5b8151613c2481613a30565b634e487b7160e01b600052601160045260246000fd5b80820180821115610dde57610dde613f19565b600060208284031215613f5457600080fd5b81518015158114613c2457600080fd5b600060208284031215613f7657600080fd5b813566ffffffffffffff81168114613c2457600080fd5b600060208284031215613f9f57600080fd5b813562ffffff81168114613c2457600080fd5b600060208284031215613fc457600080fd5b813563ffffffff81168114613c2457600080fd5b600060208284031215613fea57600080fd5b813565ffffffffffff81168114613c2457600080fd5b60006020828403121561401257600080fd5b610ddb82613b38565b634e487b7160e01b600052604160045260246000fd5b634e487b7160e01b600052603260045260246000fd5b6000808335601e1984360301811261405e57600080fd5b83018035915067ffffffffffffffff82111561407957600080fd5b602001915036819003821315612c1e57600080fd5b6000600182016140a0576140a0613f19565b5060010190565b6001600160601b038181168382160190808211156112df576112df613f19565b62ffffff8181168382160190808211156112df576112df613f19565b928352602083019190915260a01b6001600160a01b0319166040820152604c0190565b600061010080838503121561411a57600080fd5b6040519081019067ffffffffffffffff8211818310171561414b57634e487b7160e01b600052604160045260246000fd5b81604052833581526020840135602082015260408401356040820152606084013560608201526080840135608082015260a084013560a082015260c084013560c082015260e084013560e0820152809250505092915050565b65ffffffffffff8181168382160190808211156112df576112df613f19565b6001600160601b038281168282160390808211156112df576112df613f19565b60208082526030908201527f6368616e6e656c206d7573742068617665207374617465204f50454e206f722060408201526f50454e44494e475f544f5f434c4f534560801b606082015260800190565b63ffffffff8181168382160190808211156112df576112df613f19565b81810381811115610dde57610dde613f19565b60008251614275818460208701613dd4565b9190910192915050565b60008261429c57634e487b7160e01b600052601260045260246000fd5b50069056fe416464726573733a206c6f772d6c6576656c2064656c65676174652063616c6c206661696c65645fa17246d3a5d68d42baa94cde33042180b783a399c02bf63ac2076e0f708738ceeab2eef998c17fe96f30f83fbf3c55fc5047f6e40c55a0cf72d236e9d2ba72a26469706673582212202343980d92998edaee11a6676235eb75899e111708ed0a2a33545ccdcb0e050364736f6c63430008130033
    /// ```
    #[rustfmt::skip]
    #[allow(clippy::all)]
    pub static DEPLOYED_BYTECODE: alloy_sol_types::private::Bytes = alloy_sol_types::private::Bytes::from_static(
        b"`\x80`@R4\x80\x15a\0\x10W`\0\x80\xFD[P`\x046\x10a\x01\xE4W`\x005`\xE0\x1C\x80c|\x8E(\xDA\x11a\x01\x0FW\x80c\xC9f\xC4\xFE\x11a\0\xA2W\x80c\xFC\x0CTj\x11a\0qW\x80c\xFC\x0CTj\x14a\x04\xD2W\x80c\xFCU0\x9A\x14a\x05\x11W\x80c\xFC\xB7yo\x14a\x05$W\x80c\xFF\xA1\xADt\x14a\x057W`\0\x80\xFD[\x80c\xC9f\xC4\xFE\x14a\x04\x87W\x80c\xDC\x96\xFDP\x14a\x04\x90W\x80c\xDD\xAD\x19\x02\x14a\x04\x98W\x80c\xF6\x98\xDA%\x14a\x04\xC9W`\0\x80\xFD[\x80c\xAC\x96P\xD8\x11a\0\xDEW\x80c\xAC\x96P\xD8\x14a\x04;W\x80c\xB9 \xDE\xED\x14a\x04[W\x80c\xBD\xA6_E\x14a\x04aW\x80c\xBE\x9B\xAB\xDC\x14a\x04tW`\0\x80\xFD[\x80c|\x8E(\xDA\x14a\x03\xC1W\x80c\x875-e\x14a\x03\xD4W\x80c\x89\xCC\xFE\x89\x14a\x04\x10W\x80c\x8C7\x10\xC9\x14a\x04\x18W`\0\x80\xFD[\x80c)9.2\x11a\x01\x87W\x80ce\x15\x14\xBF\x11a\x01VW\x80ce\x15\x14\xBF\x14a\x02\xEFW\x80crX\x1C\xC0\x14a\x03\x02W\x80cx\xD8\x01m\x14a\x03)W\x80cz~\xBD{\x14a\x03PW`\0\x80\xFD[\x80c)9.2\x14a\x02\x83W\x80cD\xDA\xE6\xF8\x14a\x02\xA3W\x80cT\xA2\xED\xF5\x14a\x02\xCAW\x80c]/\x07\xC5\x14a\x02\xDDW`\0\x80\xFD[\x80c\x1A\x7F\xFEz\x11a\x01\xC3W\x80c\x1A\x7F\xFEz\x14a\x02$W\x80c#\xCB:\xC0\x14a\x027W\x80c$\x08l\xC2\x14a\x02JW\x80c$\x9C\xB3\xFA\x14a\x02pW`\0\x80\xFD[\x80b#\xDE)\x14a\x01\xE9W\x80c\n\xBE\xC5\x8F\x14a\x01\xFEW\x80c\x0C\xD8\x8Dr\x14a\x02\x11W[`\0\x80\xFD[a\x01\xFCa\x01\xF76`\x04a:\x87V[a\x05[V[\0[a\x01\xFCa\x02\x0C6`\x04a;TV[a\x08\x17V[a\x01\xFCa\x02\x1F6`\x04a;\xC7V[a\t\xAFV[a\x01\xFCa\x0226`\x04a<\x07V[a\n\x80V[a\x01\xFCa\x02E6`\x04a<\x07V[a\x0BPV[a\x02]a\x02X6`\x04a<+V[a\x0C\x1DV[`@Q\x90\x81R` \x01[`@Q\x80\x91\x03\x90\xF3[a\x02]a\x02~6`\x04a<HV[a\r\x8AV[a\x02\x8B`\x01\x81V[`@Q`\x01`\x01``\x1B\x03\x90\x91\x16\x81R` \x01a\x02gV[a\x02]\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81V[a\x01\xFCa\x02\xD86`\x04a<xV[a\r\xE4V[a\x02\x8Bj\x08E\x95\x16\x14\x01HJ\0\0\0\x81V[a\x01\xFCa\x02\xFD6`\x04a<xV[a\x0E\xB9V[a\x02]\x7F\xB2\x81\xFC\x8C\x12\x95M\"TM\xB4]\xE3\x15\x9A9'(\x95\xB1i\xA8R\xB3\x14\xF9\xCCv.D\xC5;\x81V[a\x02]\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81V[a\x03\xB0a\x03^6`\x04a<\xA6V[`\x06` R`\0\x90\x81R`@\x90 T`\x01`\x01``\x1B\x03\x81\x16\x90`\x01``\x1B\x81\x04e\xFF\xFF\xFF\xFF\xFF\xFF\x16\x90`\x01`\x90\x1B\x81\x04c\xFF\xFF\xFF\xFF\x16\x90`\x01`\xB0\x1B\x81\x04b\xFF\xFF\xFF\x16\x90`\x01`\xC8\x1B\x90\x04`\xFF\x16\x85V[`@Qa\x02g\x95\x94\x93\x92\x91\x90a<\xD5V[a\x01\xFCa\x03\xCF6`\x04a<\x07V[a\x0F\x89V[a\x03\xFB\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81V[`@Qc\xFF\xFF\xFF\xFF\x90\x91\x16\x81R` \x01a\x02gV[a\x01\xFCa\x10VV[a\x04+a\x04&6`\x04a=8V[a\x11oV[`@Q\x90\x15\x15\x81R` \x01a\x02gV[a\x04Na\x04I6`\x04a=_V[a\x11\xF1V[`@Qa\x02g\x91\x90a>$V[Ba\x03\xFBV[a\x01\xFCa\x04o6`\x04a<xV[a\x12\xE6V[a\x02]a\x04\x826`\x04a<xV[a\x13\xB6V[a\x02]`\x03T\x81V[a\x01\xFCa\x13\xFBV[a\x04\xBC`@Q\x80`@\x01`@R\x80`\x05\x81R` \x01d\x03\x12\xE3\x02\xE3`\xDC\x1B\x81RP\x81V[`@Qa\x02g\x91\x90a>\x86V[a\x02]`\x05T\x81V[a\x04\xF9\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81V[`@Q`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x81R` \x01a\x02gV[a\x01\xFCa\x05\x1F6`\x04a>\x99V[a\x15\tV[a\x01\xFCa\x0526`\x04a>\xCEV[a\x16\x9CV[a\x04\xBC`@Q\x80`@\x01`@R\x80`\x05\x81R` \x01d\x03\"\xE3\x02\xE3`\xDC\x1B\x81RP\x81V[3`\x01`\x01`\xA0\x1B\x03\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x16\x14a\x05\xA4W`@QcPy\xFFu`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01`\x01`\xA0\x1B\x03\x86\x160\x14a\x05\xCDW`@Qc\x178\x92!`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x82\x15a\x08\rW\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x83\x03a\x07.W`\x01`\x01``\x1B\x03\x85\x11\x15a\x06\"W`@Qc)<\xEE\xF9`\xE2\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x04\x80T`@Qc\x02&^1`\xE6\x1B\x81R\x865``\x90\x81\x1C\x93\x82\x01\x84\x90R`\x14\x88\x015\x90\x1C\x91`\0\x91`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x90c\x89\x97\x8C@\x90`$\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x06}W=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x06\xA1\x91\x90a>\xFCV[\x90P\x82`\x01`\x01`\xA0\x1B\x03\x16\x8A`\x01`\x01`\xA0\x1B\x03\x16\x03a\x06\xE9W`\x01`\x01`\xA0\x1B\x03\x81\x16\x15a\x06\xE4W`@Qc\xAC\xD5\xA8#`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x07\x1BV[\x89`\x01`\x01`\xA0\x1B\x03\x16\x81`\x01`\x01`\xA0\x1B\x03\x16\x14a\x07\x1BW`@Qc\xAC\xD5\xA8#`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x07&\x83\x83\x8Aa\x17jV[PPPa\x08\rV[\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x83\x03a\x07\xF4W\x835``\x90\x81\x1C\x90`\x14\x86\x015`\xA0\x90\x81\x1C\x91` \x88\x015\x90\x1C\x90`4\x88\x015\x90\x1C\x88\x15\x80a\x07\x99WPa\x07\x95`\x01`\x01``\x1B\x03\x80\x83\x16\x90\x85\x16a?/V[\x89\x14\x15[\x15a\x07\xB7W`@Qc\xC5.>\xFF`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01`\x01``\x1B\x03\x83\x16\x15a\x07\xD1Wa\x07\xD1\x84\x83\x85a\x17jV[`\x01`\x01``\x1B\x03\x81\x16\x15a\x07\xEBWa\x07\xEB\x82\x85\x83a\x17jV[PPPPa\x08\rV[`@Qc\r=\xCD\xE5`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[PPPPPPPPV[`\x04T\x83\x90`\x01`\xA0\x1B\x90\x04`\xFF\x16a\x08CW`@Qc\x08\xA9D\x19`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x04\x80T`@Qc\x02&^1`\xE6\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x84\x81\x16\x93\x82\x01\x93\x90\x93R3\x92\x90\x91\x16\x90c\x89\x97\x8C@\x90`$\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x08\x92W=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x08\xB6\x91\x90a>\xFCV[`\x01`\x01`\xA0\x1B\x03\x16\x14a\x08\xDDW`@Qc\xAC\xD5\xA8#`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x08\xE8\x84\x84\x84a\x17jV[`@Qc#\xB8r\xDD`\xE0\x1B\x81R3`\x04\x82\x01R0`$\x82\x01R`\x01`\x01``\x1B\x03\x83\x16`D\x82\x01R\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\x01`\x01`\xA0\x1B\x03\x16\x90c#\xB8r\xDD\x90`d\x01` `@Q\x80\x83\x03\x81`\0\x87Z\xF1\x15\x80\x15a\tcW=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\t\x87\x91\x90a?BV[\x15\x15`\x01\x14a\t\xA9W`@Qc\x02.%\x81`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[PPPPV[`\x04T\x83\x90`\x01`\xA0\x1B\x90\x04`\xFF\x16a\t\xDBW`@Qc\x08\xA9D\x19`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x04\x80T`@Qc\x02&^1`\xE6\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x84\x81\x16\x93\x82\x01\x93\x90\x93R3\x92\x90\x91\x16\x90c\x89\x97\x8C@\x90`$\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\n*W=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\nN\x91\x90a>\xFCV[`\x01`\x01`\xA0\x1B\x03\x16\x14a\nuW`@Qc\xAC\xD5\xA8#`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\t\xA9\x84\x84\x84a\x1B\x16V[`\x04T`\x01`\xA0\x1B\x90\x04`\xFF\x16a\n\xAAW`@Qc\x08\xA9D\x19`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x04\x80T`@Qc\x02&^1`\xE6\x1B\x81R3\x92\x81\x01\x92\x90\x92R`\0\x91`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x90c\x89\x97\x8C@\x90`$\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\n\xF8W=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x0B\x1C\x91\x90a>\xFCV[`\x01`\x01`\xA0\x1B\x03\x16\x14a\x0BCW`@Qc\xAC\xD5\xA8#`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x0BM3\x82a\"\x13V[PV[`\x04T`\x01`\xA0\x1B\x90\x04`\xFF\x16a\x0BzW`@Qc\x08\xA9D\x19`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x04\x80T`@Qc\x02&^1`\xE6\x1B\x81R3\x92\x81\x01\x92\x90\x92R`\0\x91`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x90c\x89\x97\x8C@\x90`$\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x0B\xC8W=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x0B\xEC\x91\x90a>\xFCV[`\x01`\x01`\xA0\x1B\x03\x16\x14a\x0C\x13W`@Qc\xAC\xD5\xA8#`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x0BM3\x82a#\x8FV[`\0\x80a\x0C.\x83a\x01\0\x015a%\x13V[\x90P`\0a\x0CB`\xC0\x85\x01`\xA0\x86\x01a?dV[f\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16`8a\x0C]`\xA0\x87\x01`\x80\x88\x01a?\x8DV[b\xFF\xFF\xFF\x16\x90\x1B`Pa\x0Cv`\x80\x88\x01``\x89\x01a?\xB2V[c\xFF\xFF\xFF\xFF\x16\x90\x1B`pa\x0C\x90``\x89\x01`@\x8A\x01a?\xD8V[e\xFF\xFF\xFF\xFF\xFF\xFF\x16\x90\x1B`\xA0a\x0C\xAC`@\x8A\x01` \x8B\x01a@\0V[`\x01`\x01``\x1B\x03\x16\x90\x1B\x17\x17\x17\x17\x90P`\0c\xFC\xB7yo`\xE0\x1B\x85`\0\x01`\0\x015\x83\x85`@Q` \x01a\r\x01\x93\x92\x91\x90\x92\x83R` \x83\x01\x91\x90\x91R``\x1B`\x01`\x01``\x1B\x03\x19\x16`@\x82\x01R`T\x01\x90V[`@\x80Q\x80\x83\x03`\x1F\x19\x01\x81R\x82\x82R\x80Q` \x91\x82\x01 `\x01`\x01`\xE0\x1B\x03\x19\x94\x90\x94\x16\x81\x84\x01R\x82\x82\x01\x93\x90\x93R\x80Q\x80\x83\x03\x82\x01\x81R``\x83\x01\x82R\x80Q\x90\x84\x01 `\x05T`\x19`\xF8\x1B`\x80\x85\x01R`\x01`\xF8\x1B`\x81\x85\x01R`\x82\x84\x01R`\xA2\x80\x84\x01\x91\x90\x91R\x81Q\x80\x84\x03\x90\x91\x01\x81R`\xC2\x90\x92\x01\x90R\x80Q\x91\x01 \x95\x94PPPPPV[`\0\x82\x81R` \x81\x81R`@\x80\x83 `\x01`\x01`\xA0\x1B\x03\x85\x16\x84R\x90\x91R\x81 T`\xFF\x16a\r\xB9W`\0a\r\xDBV[\x7F\xA2\xEFF\0\xD7B\x02-S-GG\xCB5GGFg\xD6\xF18\x04\x90%\x13\xB2\xEC\x01\xC8H\xF4\xB4[\x90P[\x92\x91PPV[`\x04T\x82\x90`\x01`\xA0\x1B\x90\x04`\xFF\x16a\x0E\x10W`@Qc\x08\xA9D\x19`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x04\x80T`@Qc\x02&^1`\xE6\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x84\x81\x16\x93\x82\x01\x93\x90\x93R3\x92\x90\x91\x16\x90c\x89\x97\x8C@\x90`$\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x0E_W=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x0E\x83\x91\x90a>\xFCV[`\x01`\x01`\xA0\x1B\x03\x16\x14a\x0E\xAAW`@Qc\xAC\xD5\xA8#`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x0E\xB4\x83\x83a\"\x13V[PPPV[`\x04T\x82\x90`\x01`\xA0\x1B\x90\x04`\xFF\x16a\x0E\xE5W`@Qc\x08\xA9D\x19`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x04\x80T`@Qc\x02&^1`\xE6\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x84\x81\x16\x93\x82\x01\x93\x90\x93R3\x92\x90\x91\x16\x90c\x89\x97\x8C@\x90`$\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x0F4W=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x0FX\x91\x90a>\xFCV[`\x01`\x01`\xA0\x1B\x03\x16\x14a\x0F\x7FW`@Qc\xAC\xD5\xA8#`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x0E\xB4\x83\x83a#\x8FV[`\x04T`\x01`\xA0\x1B\x90\x04`\xFF\x16a\x0F\xB3W`@Qc\x08\xA9D\x19`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x04\x80T`@Qc\x02&^1`\xE6\x1B\x81R3\x92\x81\x01\x92\x90\x92R`\0\x91`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x90c\x89\x97\x8C@\x90`$\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x10\x01W=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x10%\x91\x90a>\xFCV[`\x01`\x01`\xA0\x1B\x03\x16\x14a\x10LW`@Qc\xAC\xD5\xA8#`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x0BM3\x82a%\xD0V[`@\x80Q\x80\x82\x01\x82R`\x0C\x81RkHoprChannels`\xA0\x1B` \x91\x82\x01R\x81Q\x80\x83\x01\x83R`\x05\x81Rd\x03\"\xE3\x02\xE3`\xDC\x1B\x90\x82\x01R\x81Q\x7F\x8Bs\xC3\xC6\x9B\xB8\xFE=Q.\xCCL\xF7Y\xCCy#\x9F{\x17\x9B\x0F\xFA\xCA\xA9\xA7]R+9@\x0F\x91\x81\x01\x91\x90\x91R\x7F\x84\xE6\x90\x8F46\x01\xD9\xCE\x9F\xB6\r\x82P9N\xB8\xA5\x1CV\xF1\x87k\xC1\xE0\x17\xC9z\xCDeg\xF2\x91\x81\x01\x91\x90\x91R\x7F\xB4\xBC\xB1T\xE3\x86\x01\xC3\x899o\xA9\x181M\xA4-F&\xF1>\xF6\xD0\xCE\xB0~_]&\xB2\xFB\xC3``\x82\x01RF`\x80\x82\x01R0`\xA0\x82\x01R`\0\x90`\xC0\x01`@Q` \x81\x83\x03\x03\x81R\x90`@R\x80Q\x90` \x01 \x90P`\x05T\x81\x14a\x0BMW`\x05\x81\x90U`@Q\x81\x90\x7Fw\x1FR@\xAE_\xD8\xA7d\r?\xB8/\xA7\n\xAB/\xB1\xDB\xF3_.\xF4d\xF8P\x99Fqvd\xC5\x90`\0\x90\xA2PV[`@\x80Q` \x80\x82\x01\x86\x90R\x835\x82\x84\x01R\x83\x81\x015``\x83\x01Ra\x01\0\x85\x015`\x80\x83\x01R`\xC0\x80\x86\x01\x805`\xA0\x80\x86\x01\x91\x90\x91R`\xE0\x80\x89\x015\x84\x87\x01R\x86Q\x80\x87\x03\x90\x94\x01\x84R\x90\x94\x01\x90\x94R\x80Q\x91\x01 `\0\x92`\xC8\x91\x90\x91\x1C\x91a\x11\xDA\x91\x90\x86\x01a?dV[f\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x90\x81\x16\x91\x16\x11\x15\x94\x93PPPPV[``\x81g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x12\x0CWa\x12\x0Ca@\x1BV[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x12?W\x81` \x01[``\x81R` \x01\x90`\x01\x90\x03\x90\x81a\x12*W\x90P[P\x90P`\0[\x82\x81\x10\x15a\x12\xDFWa\x12\xAF0\x85\x85\x84\x81\x81\x10a\x12cWa\x12ca@1V[\x90P` \x02\x81\x01\x90a\x12u\x91\x90a@GV[\x80\x80`\x1F\x01` \x80\x91\x04\x02` \x01`@Q\x90\x81\x01`@R\x80\x93\x92\x91\x90\x81\x81R` \x01\x83\x83\x80\x82\x847`\0\x92\x01\x91\x90\x91RPa' \x92PPPV[\x82\x82\x81Q\x81\x10a\x12\xC1Wa\x12\xC1a@1V[` \x02` \x01\x01\x81\x90RP\x80\x80a\x12\xD7\x90a@\x8EV[\x91PPa\x12EV[P\x92\x91PPV[`\x04T\x82\x90`\x01`\xA0\x1B\x90\x04`\xFF\x16a\x13\x12W`@Qc\x08\xA9D\x19`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x04\x80T`@Qc\x02&^1`\xE6\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x84\x81\x16\x93\x82\x01\x93\x90\x93R3\x92\x90\x91\x16\x90c\x89\x97\x8C@\x90`$\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x13aW=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x13\x85\x91\x90a>\xFCV[`\x01`\x01`\xA0\x1B\x03\x16\x14a\x13\xACW`@Qc\xAC\xD5\xA8#`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x0E\xB4\x83\x83a%\xD0V[`@Q`\x01`\x01``\x1B\x03\x19``\x84\x81\x1B\x82\x16` \x84\x01R\x83\x90\x1B\x16`4\x82\x01R`\0\x90`H\x01`@Q` \x81\x83\x03\x03\x81R\x90`@R\x80Q\x90` \x01 \x90P\x92\x91PPV[`@\x80Q\x80\x82\x01\x82R`\n\x81Ri$7\xB89&2\xB23\xB2\xB9`\xB1\x1B` \x91\x82\x01R\x81Q\x80\x83\x01\x83R`\x05\x81Rd\x03\x12\xE3\x02\xE3`\xDC\x1B\x90\x82\x01R\x81Q\x7F\x8Bs\xC3\xC6\x9B\xB8\xFE=Q.\xCCL\xF7Y\xCCy#\x9F{\x17\x9B\x0F\xFA\xCA\xA9\xA7]R+9@\x0F\x81\x83\x01R\x7Fl\xD6\x81y\x0Cx\xC2 Q{\t\x9As\x7F\x8E\x85\xF6\x9Eyz\xBEN)\x10\xFB\x18\x9Ba\xDBK\xF2\xCD\x81\x84\x01R\x7F\x06\xC0\x15\xBD\"\xB4\xC6\x96\x90\x93<\x10X\x87\x8E\xBD\xFE\xF3\x1F\x9A\xAA\xE4\x0B\xBE\x86\xD8\xA0\x9F\xE1\xB2\x97,``\x82\x01RF`\x80\x82\x01R0`\xA0\x80\x83\x01\x91\x90\x91R\x83Q\x80\x83\x03\x90\x91\x01\x81R`\xC0\x90\x91\x01\x90\x92R\x81Q\x91\x01 `\x03T\x81\x14a\x0BMW`\x03\x81\x90U`@Q\x81\x90\x7F\xA4?\xAD\x83\x92\x0F\xD0\x94E\x85^\x85Ns\xC9\xC52\xE1t\x02\xC9\xCE\xB0\x99\x93\xA29(C\xA5\xBD\xB9\x90`\0\x90\xA2PV[`\x04T`\x01`\xA0\x1B\x90\x04`\xFF\x16a\x153W`@Qc\x08\xA9D\x19`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x04\x80T`@Qc\x02&^1`\xE6\x1B\x81R3\x92\x81\x01\x92\x90\x92R`\0\x91`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x90c\x89\x97\x8C@\x90`$\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x15\x81W=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x15\xA5\x91\x90a>\xFCV[`\x01`\x01`\xA0\x1B\x03\x16\x14a\x15\xCCW`@Qc\xAC\xD5\xA8#`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x15\xD73\x83\x83a\x17jV[`@Qc#\xB8r\xDD`\xE0\x1B\x81R3`\x04\x82\x01R0`$\x82\x01R`\x01`\x01``\x1B\x03\x82\x16`D\x82\x01R\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\x01`\x01`\xA0\x1B\x03\x16\x90c#\xB8r\xDD\x90`d\x01` `@Q\x80\x83\x03\x81`\0\x87Z\xF1\x15\x80\x15a\x16RW=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x16v\x91\x90a?BV[\x15\x15`\x01\x14a\x16\x98W`@Qc\x02.%\x81`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[PPV[`\x04T`\x01`\xA0\x1B\x90\x04`\xFF\x16a\x16\xC6W`@Qc\x08\xA9D\x19`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x04\x80T`@Qc\x02&^1`\xE6\x1B\x81R3\x92\x81\x01\x92\x90\x92R`\0\x91`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x90c\x89\x97\x8C@\x90`$\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x17\x14W=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x178\x91\x90a>\xFCV[`\x01`\x01`\xA0\x1B\x03\x16\x14a\x17_W`@Qc\xAC\xD5\xA8#`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x16\x983\x83\x83a\x1B\x16V[\x80`\x01`\x01`\x01``\x1B\x03\x82\x16\x10\x15a\x17\x96W`@Qc\xC5.>\xFF`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[j\x08E\x95\x16\x14\x01HJ\0\0\0`\x01`\x01``\x1B\x03\x82\x16\x11\x15a\x17\xCBW`@Qc)<\xEE\xF9`\xE2\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x83\x83\x80`\x01`\x01`\xA0\x1B\x03\x16\x82`\x01`\x01`\xA0\x1B\x03\x16\x03a\x17\xFFW`@QcK\xD1\xD7i`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01`\x01`\xA0\x1B\x03\x82\x16a\x18[W`@Qc\xEA\xC0\xD3\x89`\xE0\x1B\x81R` `\x04\x82\x01R`\x18`$\x82\x01R\x7Fsource must not be empty\0\0\0\0\0\0\0\0`D\x82\x01R`d\x01[`@Q\x80\x91\x03\x90\xFD[`\x01`\x01`\xA0\x1B\x03\x81\x16a\x18\xB2W`@Qc\xEA\xC0\xD3\x89`\xE0\x1B\x81R` `\x04\x82\x01R`\x1D`$\x82\x01R\x7Fdestination must not be empty\0\0\0`D\x82\x01R`d\x01a\x18RV[`\0a\x18\xBE\x87\x87a\x13\xB6V[`\0\x81\x81R`\x06` R`@\x90 \x90\x91P`\x02\x81T`\x01`\xC8\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a\x18\xEEWa\x18\xEEa<\xBFV[\x03a\x19OW`@QcI\x94c\xC1`\xE0\x1B\x81R` `\x04\x82\x01R`*`$\x82\x01R\x7Fcannot fund a channel that will `D\x82\x01Ri1\xB67\xB9\xB2\x909\xB7\xB7\xB7`\xB1\x1B`d\x82\x01R`\x84\x01a\x18RV[\x80Ta\x19e\x90\x87\x90`\x01`\x01``\x1B\x03\x16a@\xA7V[\x81T`\x01`\x01``\x1B\x03\x19\x16`\x01`\x01``\x1B\x03\x91\x90\x91\x16\x17\x81U`\0\x81T`\x01`\xC8\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a\x19\x9FWa\x19\x9Fa<\xBFV[\x03a\x1A\xAAW\x80Ta\x19\xBD\x90`\x01`\xB0\x1B\x90\x04b\xFF\xFF\xFF\x16`\x01a@\xC7V[\x81Tb\xFF\xFF\xFF\x91\x90\x91\x16`\x01`\xB0\x1B\x02m\xFF\0\0\0\0\0\0\0\xFF\xFF\xFF\xFF\xFF\xFF``\x1B\x19\x16m\xFF\xFF\xFF\xFF\0\0\0\0\xFF\xFF\xFF\xFF\xFF\xFF``\x1B\x19\x90\x91\x16\x17`\x01`\xC8\x1B\x17\x81U`@\x80Q\x7F\xDD\x90\xF98#\x035\xE5\x9D\xC9%\xC5~\xCB\x0E'\xA2\x8C-\x875n1\xF0\x0C\xD5UJ\xBDl\x1B-` \x82\x01R``\x8A\x81\x1B`\x01`\x01``\x1B\x03\x19\x90\x81\x16\x93\x83\x01\x93\x90\x93R\x89\x90\x1B\x90\x91\x16`T\x82\x01Ra\x1Ai\x90`h\x01[`@Q` \x81\x83\x03\x03\x81R\x90`@Ra'EV[\x86`\x01`\x01`\xA0\x1B\x03\x16\x88`\x01`\x01`\xA0\x1B\x03\x16\x7F\xDD\x90\xF98#\x035\xE5\x9D\xC9%\xC5~\xCB\x0E'\xA2\x8C-\x875n1\xF0\x0C\xD5UJ\xBDl\x1B-`@Q`@Q\x80\x91\x03\x90\xA3[\x80T`@Qa\x1A\xDD\x91a\x1AU\x91`\0\x80Q` aB\xC9\x839\x81Q\x91R\x91\x86\x91`\x01`\x01``\x1B\x03\x90\x91\x16\x90` \x01a@\xE3V[\x80T`@Q`\x01`\x01``\x1B\x03\x90\x91\x16\x81R\x82\x90`\0\x80Q` aB\xC9\x839\x81Q\x91R\x90` \x01`@Q\x80\x91\x03\x90\xA2PPPPPPPPV[a\x1B&`@\x83\x01` \x84\x01a@\0V[`\x01`\x01`\x01``\x1B\x03\x82\x16\x10\x15a\x1BQW`@Qc\xC5.>\xFF`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[j\x08E\x95\x16\x14\x01HJ\0\0\0`\x01`\x01``\x1B\x03\x82\x16\x11\x15a\x1B\x86W`@Qc)<\xEE\xF9`\xE2\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x82a\x01\0\x015a\x1B\x95\x81a(+V[a\x1B\xB2W`@Qc:\xE4\xEDk`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x835`\0\x90\x81R`\x06` R`@\x90 `\x01\x81T`\x01`\xC8\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a\x1B\xE1Wa\x1B\xE1a<\xBFV[\x14\x15\x80\x15a\x1C\x0CWP`\x02\x81T`\x01`\xC8\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a\x1C\tWa\x1C\ta<\xBFV[\x14\x15[\x15a\x1CtW`@QcI\x94c\xC1`\xE0\x1B\x81R` `\x04\x82\x01R`1`$\x82\x01R\x7Fspending channel must be OPEN or`D\x82\x01Rp PENDING_TO_CLOSE`x\x1B`d\x82\x01R`\x84\x01a\x18RV[a\x1C\x84`\xA0\x86\x01`\x80\x87\x01a?\x8DV[\x81T`\x01`\xB0\x1B\x90\x04b\xFF\xFF\xFF\x90\x81\x16\x91\x16\x14a\x1C\xE4W`@QcI\x94c\xC1`\xE0\x1B\x81R` `\x04\x82\x01R`\x18`$\x82\x01R\x7Fchannel epoch must match\0\0\0\0\0\0\0\0`D\x82\x01R`d\x01a\x18RV[`\0a\x1C\xF6``\x87\x01`@\x88\x01a?\xD8V[\x90P`\0a\x1D\n`\x80\x88\x01``\x89\x01a?\xB2V[\x83T\x90\x91P`\x01``\x1B\x90\x04e\xFF\xFF\xFF\xFF\xFF\xFF\x16`\x01c\xFF\xFF\xFF\xFF\x83\x16\x10\x80a\x1DBWP\x80e\xFF\xFF\xFF\xFF\xFF\xFF\x16\x83e\xFF\xFF\xFF\xFF\xFF\xFF\x16\x10[\x15a\x1D`W`@Qchn\x1E\x0F`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x1Dp`@\x89\x01` \x8A\x01a@\0V[\x84T`\x01`\x01``\x1B\x03\x91\x82\x16\x91\x16\x10\x15a\x1D\x9EW`@Qc,Q\xD8\xDB`\xE2\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0a\x1D\xA9\x89a\x0C\x1DV[\x90Pa\x1D\xB6\x81\x8A\x8Aa\x11oV[a\x1D\xD3W`@Qc\xEE\x83\\\x89`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0`@Q\x80``\x01`@R\x80\x83\x81R` \x01\x8C`\x01`\x01`\xA0\x1B\x03\x16\x81R` \x01`\x05T`@Q` \x01a\x1E\n\x91\x81R` \x01\x90V[`@\x80Q`\x1F\x19\x81\x84\x03\x01\x81R\x91\x90R\x90R\x90Pa\x1E6a\x1E06\x8B\x90\x03\x8B\x01\x8BaA\x06V[\x82a(MV[a\x1ESW`@Qc\x12\xBF\xB7\xB7`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0a\x1Eh\x83`\xC0\x8D\x015`\xE0\x8E\x015a*\xD6V[\x90P\x8A5a\x1Ev\x82\x8Ea\x13\xB6V[\x14a\x1E\x94W`@Qcf\xEE\xA9\xAB`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x1E\xA4c\xFF\xFF\xFF\xFF\x86\x16\x87aA\xA4V[\x87Te\xFF\xFF\xFF\xFF\xFF\xFF\x91\x90\x91\x16`\x01``\x1B\x02e\xFF\xFF\xFF\xFF\xFF\xFF``\x1B\x19\x90\x91\x16\x17\x87Ua\x1E\xD8`@\x8C\x01` \x8D\x01a@\0V[\x87Ta\x1E\xED\x91\x90`\x01`\x01``\x1B\x03\x16aA\xC3V[\x87T`\x01`\x01``\x1B\x03\x19\x16`\x01`\x01``\x1B\x03\x91\x90\x91\x16\x90\x81\x17\x88U`@Qa\x1FB\x91a\x1AU\x91\x7F\"\xE2\xA4\"\xA8\x86\x06V\xA3\xA3<\xFA\x1D\xAFw\x1Evy\x8C\xE5d\x97G\x95r5\x02]\xE1.\x0B$\x91\x8F5\x91` \x01a@\xE3V[\x86T`@Q`\x01`\x01``\x1B\x03\x90\x91\x16\x81R\x8B5\x90\x7F\"\xE2\xA4\"\xA8\x86\x06V\xA3\xA3<\xFA\x1D\xAFw\x1Evy\x8C\xE5d\x97G\x95r5\x02]\xE1.\x0B$\x90` \x01`@Q\x80\x91\x03\x90\xA2`\0a\x1F\x90\x8D\x83a\x13\xB6V[\x90P`\0`\x06`\0\x83\x81R` \x01\x90\x81R` \x01`\0 \x90Pa \x1C\x7Fqe\xE2\xEB\xC7\xCE5\xCC\x98\xCBvf\xF9\x94[6\x17\xF3\xF3c&\xB7m\x18\x93{\xA5\xFE\xCF\x18s\x9A\x8E`\0\x01`\0\x015\x8B`\0\x01`\x0C\x90T\x90a\x01\0\n\x90\x04e\xFF\xFF\xFF\xFF\xFF\xFF\x16`@Q` \x01a\x1AU\x93\x92\x91\x90\x92\x83R` \x83\x01\x91\x90\x91R`\xD0\x1B`\x01`\x01`\xD0\x1B\x03\x19\x16`@\x82\x01R`F\x01\x90V[\x88T`@Q`\x01``\x1B\x90\x91\x04e\xFF\xFF\xFF\xFF\xFF\xFF\x16\x81R\x8D5\x90\x7Fqe\xE2\xEB\xC7\xCE5\xCC\x98\xCBvf\xF9\x94[6\x17\xF3\xF3c&\xB7m\x18\x93{\xA5\xFE\xCF\x18s\x9A\x90` \x01`@Q\x80\x91\x03\x90\xA2`\0\x81T`\x01`\xC8\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a \x82Wa \x82a<\xBFV[\x03a!lW\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\x01`\x01`\xA0\x1B\x03\x16c\xA9\x05\x9C\xBB3\x8F`\0\x01` \x01` \x81\x01\x90a \xCD\x91\x90a@\0V[`@Q`\x01`\x01`\xE0\x1B\x03\x19`\xE0\x85\x90\x1B\x16\x81R`\x01`\x01`\xA0\x1B\x03\x90\x92\x16`\x04\x83\x01R`\x01`\x01``\x1B\x03\x16`$\x82\x01R`D\x01` `@Q\x80\x83\x03\x81`\0\x87Z\xF1\x15\x80\x15a!!W=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a!E\x91\x90a?BV[\x15\x15`\x01\x14a!gW`@Qc\x02.%\x81`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\"\x03V[a!|`@\x8E\x01` \x8F\x01a@\0V[\x81Ta!\x91\x91\x90`\x01`\x01``\x1B\x03\x16a@\xA7V[\x81T`\x01`\x01``\x1B\x03\x19\x16`\x01`\x01``\x1B\x03\x91\x90\x91\x16\x90\x81\x17\x82U`@Qa!\xD3\x91a\x1AU\x91`\0\x80Q` aB\xC9\x839\x81Q\x91R\x91\x86\x91` \x01a@\xE3V[\x80T`@Q`\x01`\x01``\x1B\x03\x90\x91\x16\x81R\x82\x90`\0\x80Q` aB\xC9\x839\x81Q\x91R\x90` \x01`@Q\x80\x91\x03\x90\xA2[PPPPPPPPPPPPPPV[`\0a\"\x1F\x82\x84a\x13\xB6V[`\0\x81\x81R`\x06` R`@\x81 \x91\x92P\x81T`\x01`\xC8\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a\"MWa\"Ma<\xBFV[\x03a\"kW`@QcI\x94c\xC1`\xE0\x1B\x81R`\x04\x01a\x18R\x90aA\xE3V[\x80T`\x01c\xFF\0\0\x01`\xB0\x1B\x03\x19\x81\x16\x82U`@\x80Q`\0\x80Q` aB\xE9\x839\x81Q\x91R` \x82\x01R\x90\x81\x01\x84\x90R`\x01`\x01``\x1B\x03\x90\x91\x16\x90a\"\xB3\x90``\x01a\x1AUV[`@Q\x83\x90`\0\x80Q` aB\xE9\x839\x81Q\x91R\x90`\0\x90\xA2\x80\x15a#\x88W`@Qc\xA9\x05\x9C\xBB`\xE0\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x85\x81\x16`\x04\x83\x01R`$\x82\x01\x83\x90R\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x16\x90c\xA9\x05\x9C\xBB\x90`D\x01[` `@Q\x80\x83\x03\x81`\0\x87Z\xF1\x15\x80\x15a#BW=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a#f\x91\x90a?BV[\x15\x15`\x01\x14a#\x88W`@Qc\x02.%\x81`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[PPPPPV[`\0a#\x9B\x83\x83a\x13\xB6V[`\0\x81\x81R`\x06` R`@\x90 \x90\x91P`\x02\x81T`\x01`\xC8\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a#\xCBWa#\xCBa<\xBFV[\x14a$(W`@QcI\x94c\xC1`\xE0\x1B\x81R` `\x04\x82\x01R`&`$\x82\x01R\x7Fchannel state must be PENDING_TO`D\x82\x01Re_CLOSE`\xD0\x1B`d\x82\x01R`\x84\x01a\x18RV[\x80Tc\xFF\xFF\xFF\xFFB\x81\x16`\x01`\x90\x1B\x90\x92\x04\x16\x10a$YW`@Qc8\xB2\x01\x95`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x80T`\x01c\xFF\0\0\x01`\xB0\x1B\x03\x19\x81\x16\x82U`@\x80Q`\0\x80Q` aB\xE9\x839\x81Q\x91R` \x82\x01R\x90\x81\x01\x84\x90R`\x01`\x01``\x1B\x03\x90\x91\x16\x90a$\xA1\x90``\x01a\x1AUV[`@Q\x83\x90`\0\x80Q` aB\xE9\x839\x81Q\x91R\x90`\0\x90\xA2\x80\x15a#\x88W`@Qc\xA9\x05\x9C\xBB`\xE0\x1B\x81R3`\x04\x82\x01R`$\x81\x01\x82\x90R\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\x01`\x01`\xA0\x1B\x03\x16\x90c\xA9\x05\x9C\xBB\x90`D\x01a##V[`\0`\x01\x81`\x1B\x7Fy\xBEf~\xF9\xDC\xBB\xACU\xA0b\x95\xCE\x87\x0B\x07\x02\x9B\xFC\xDB-\xCE(\xD9Y\xF2\x81[\x16\xF8\x17\x98p\x01EQ#\x19P\xB7_\xC4@-\xA1s/\xC9\xBE\xBE\x19\x7Fy\xBEf~\xF9\xDC\xBB\xACU\xA0b\x95\xCE\x87\x0B\x07\x02\x9B\xFC\xDB-\xCE(\xD9Y\xF2\x81[\x16\xF8\x17\x98\x87\t`@\x80Q`\0\x81R` \x81\x01\x80\x83R\x95\x90\x95R`\xFF\x90\x93\x16\x92\x84\x01\x92\x90\x92R``\x83\x01R`\x80\x82\x01R`\xA0\x01` `@Q` \x81\x03\x90\x80\x84\x03\x90\x85Z\xFA\x15\x80\x15a%\xBFW=`\0\x80>=`\0\xFD[PP`@Q`\x1F\x19\x01Q\x93\x92PPPV[`\0a%\xDC\x83\x83a\x13\xB6V[`\0\x81\x81R`\x06` R`@\x81 \x91\x92P\x81T`\x01`\xC8\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a&\nWa&\na<\xBFV[\x03a&(W`@QcI\x94c\xC1`\xE0\x1B\x81R`\x04\x01a\x18R\x90aA\xE3V[a&R\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0BaB3V[\x81T`\x01`\xC9\x1Bg\xFF\0\0\0\xFF\xFF\xFF\xFF`\x90\x1B\x19\x90\x91\x16`\xFF`\xC8\x1B\x19`\x01`\x90\x1Bc\xFF\xFF\xFF\xFF\x94\x90\x94\x16\x84\x02\x16\x17\x17\x80\x83U`@\x80Q\x7F\x07\xB5\xC9PY\x7F\xC3\xBE\xD9.*\xD3\x7F\xA8Op\x16U\xAC\xB3r\x98.Ho_\xAD6\x07\xF0J\\` \x82\x01R\x90\x81\x01\x85\x90R\x91\x90\x04`\xE0\x1B`\x01`\x01`\xE0\x1B\x03\x19\x16``\x82\x01Ra&\xD6\x90`d\x01a\x1AUV[\x80T`@Q`\x01`\x90\x1B\x90\x91\x04c\xFF\xFF\xFF\xFF\x16\x81R\x82\x90\x7F\x07\xB5\xC9PY\x7F\xC3\xBE\xD9.*\xD3\x7F\xA8Op\x16U\xAC\xB3r\x98.Ho_\xAD6\x07\xF0J\\\x90` \x01`@Q\x80\x91\x03\x90\xA2PPPPV[``a\r\xDB\x83\x83`@Q\x80``\x01`@R\x80`'\x81R` \x01aB\xA2`'\x919a*\xFCV[`\x01T`\0\x90a'\x83\x90\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x90`\x01`\xE0\x1B\x90\x04c\xFF\xFF\xFF\xFF\x16a?/V[B\x11\x15a'\x8EWP`\x01[`\x03T`\x01T\x83Q` \x80\x86\x01\x91\x90\x91 `@\x80Q\x80\x84\x01\x95\x90\x95RC`\xE0\x1B`\x01`\x01`\xE0\x1B\x03\x19\x16\x90\x85\x01R\x91\x90\x1Bc\xFF\xFF\xFF\xFF\x19\x16`D\x83\x01R``\x82\x01R`\x80\x01`@\x80Q`\x1F\x19\x81\x84\x03\x01\x81R\x91\x90R\x80Q` \x91\x82\x01 c\xFF\xFF\xFF\xFFB\x16`\x01`\xE0\x1B\x02\x91\x1C\x17`\x01U\x80\x15a\x16\x98WPP`\x01T`\x01`\x01`\xE0\x1B\x03\x81\x16`\x01`\xE0\x1B\x91\x82\x90\x04c\xFF\xFF\xFF\xFF\x16\x90\x91\x02\x17`\x02UV[`\0\x81\x15\x80a\r\xDEWPPp\x01EQ#\x19P\xB7_\xC4@-\xA1s/\xC9\xBE\xBE\x19\x11\x90V[`\0d\x01\0\0\x03\xD0\x19\x83``\x01Q\x10\x15\x80a(rWPd\x01\0\0\x03\xD0\x19\x83`@\x01Q\x10\x15[\x15a(\x90W`@Qc:\xE4\xEDk`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a(\xA2\x83`\0\x01Q\x84` \x01Qa+tV[a(\xBFW`@Qc9\"\xA5A`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0\x80a)\x11\x84` \x01Q\x85`\0\x01Q`@Q` \x01a(\xF8\x92\x91\x90``\x92\x90\x92\x1B`\x01`\x01``\x1B\x03\x19\x16\x82R`\x14\x82\x01R`4\x01\x90V[`@Q` \x81\x83\x03\x03\x81R\x90`@R\x85`@\x01Qa+\x9FV[\x91P\x91P`\0a)&\x86`@\x01Q\x84\x84a,%V[\x90Pa)a\x86`\x80\x01Q\x87`\xA0\x01Q`@\x80Q` \x80\x82\x01\x94\x90\x94R\x80\x82\x01\x92\x90\x92R\x80Q\x80\x83\x03\x82\x01\x81R``\x90\x92\x01\x90R\x80Q\x91\x01 \x90V[`\x01`\x01`\xA0\x1B\x03\x16\x81`\x01`\x01`\xA0\x1B\x03\x16\x14a)\x92W`@Qc\x1D\xBF\xB9\xB3`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0a)\xAB\x87``\x01Q\x88`\0\x01Q\x89` \x01Qa,%V[\x90Pa)\xE6\x87`\xC0\x01Q\x88`\xE0\x01Q`@\x80Q` \x80\x82\x01\x94\x90\x94R\x80\x82\x01\x92\x90\x92R\x80Q\x80\x83\x03\x82\x01\x81R``\x90\x92\x01\x90R\x80Q\x91\x01 \x90V[`\x01`\x01`\xA0\x1B\x03\x16\x81`\x01`\x01`\xA0\x1B\x03\x16\x14a*\x17W`@Qc\x1D\xBF\xB9\xB3`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0\x80a*I\x89`\x80\x01Q\x8A`\xA0\x01Q\x8B`\xC0\x01Q\x8C`\xE0\x01Qd\x01\0\0\x03\xD0\x19a*B\x91\x90aBPV[`\0a,\xC4V[` \x80\x8B\x01Q\x8CQ\x8D\x83\x01Q\x8DQ`@Q\x96\x98P\x94\x96P`\0\x95a*\xC1\x95a*\xA8\x95\x8A\x92\x8A\x92\x91\x01``\x96\x90\x96\x1B`\x01`\x01``\x1B\x03\x19\x16\x86R`\x14\x86\x01\x94\x90\x94R`4\x85\x01\x92\x90\x92R`T\x84\x01R`t\x83\x01R`\x94\x82\x01R`\xB4\x01\x90V[`@Q` \x81\x83\x03\x03\x81R\x90`@R\x8A`@\x01Qa.KV[``\x8B\x01Q\x14\x97PPPPPPPP\x92\x91PPV[`\0\x80`\0a*\xE6\x86\x86\x86a.\xBCV[\x91P\x91Pa*\xF3\x81a.\xF5V[P\x94\x93PPPPV[```\0\x80\x85`\x01`\x01`\xA0\x1B\x03\x16\x85`@Qa+\x19\x91\x90aBcV[`\0`@Q\x80\x83\x03\x81\x85Z\xF4\x91PP=\x80`\0\x81\x14a+TW`@Q\x91P`\x1F\x19`?=\x01\x16\x82\x01`@R=\x82R=`\0` \x84\x01>a+YV[``\x91P[P\x91P\x91Pa+j\x86\x83\x83\x87a0?V[\x96\x95PPPPPPV[`\0d\x01\0\0\x03\xD0\x19\x80\x84d\x01\0\0\x03\xD0\x19\x86\x87\t\t`\x07\x08d\x01\0\0\x03\xD0\x19\x83\x84\t\x14\x93\x92PPPV[`\0\x80`\0\x80a+\xAF\x86\x86a0\xC0V[\x91P\x91P`\0\x80a+\xBF\x84a1|V[\x91P\x91P`\0\x80a+\xCF\x85a1|V[\x91P\x91P`\0\x80a,\x03\x86\x86\x86\x86\x7F?\x871\xAB\xDDf\x1A\xDC\xA0\x8AUX\xF0\xF5\xD2r\xE9S\xD3c\xCBo\x0E]@TG\xC0\x1ADE3a,\xC4V[\x91P\x91Pa,\x11\x82\x82a4>V[\x99P\x99PPPPPPPPP[\x92P\x92\x90PV[`\0\x80a,3`\x02\x84aB\x7FV[`\0\x03a,BWP`\x1Ba,FV[P`\x1C[`\x01`\0\x82\x86p\x01EQ#\x19P\xB7_\xC4@-\xA1s/\xC9\xBE\xBE\x19\x88\x8A\t`@\x80Q`\0\x81R` \x81\x01\x80\x83R\x95\x90\x95R`\xFF\x90\x93\x16\x92\x84\x01\x92\x90\x92R``\x83\x01R`\x80\x82\x01R`\xA0\x01` `@Q` \x81\x03\x90\x80\x84\x03\x90\x85Z\xFA\x15\x80\x15a,\xB0W=`\0\x80>=`\0\xFD[PP`@Q`\x1F\x19\x01Q\x96\x95PPPPPPV[`\0\x80\x83\x86\x14\x19\x85\x88\x14\x16\x15a,\xD9W`\0\x80\xFD[`\0\x80\x85\x88\x14\x87\x8A\x14\x16`\x01\x81\x14a,\xF6W\x80\x15a-sWa-\xEEV[d\x01\0\0\x03\xD0\x19\x86d\x01\0\0\x03\xD0\x19\x8B`\x02\t\x08\x91P`@Q` \x81R` \x80\x82\x01R` `@\x82\x01R\x82``\x82\x01Rd\x01\0\0\x03\xD2\x19`\x80\x82\x01Rd\x01\0\0\x03\xD0\x19`\xA0\x82\x01R` \x81`\xC0\x83`\x05`\0\x19\xFAa-SW`\0\x80\xFD[d\x01\0\0\x03\xD0\x19\x81Qd\x01\0\0\x03\xD0\x19\x80\x8E\x8F\t`\x03\t\t\x93PPa-\xEEV[d\x01\0\0\x03\xD0\x19\x8Ad\x01\0\0\x03\xD0\x19\x03\x89\x08\x91P`@Q` \x81R` \x80\x82\x01R` `@\x82\x01R\x82``\x82\x01Rd\x01\0\0\x03\xD2\x19`\x80\x82\x01Rd\x01\0\0\x03\xD0\x19`\xA0\x82\x01R` \x81`\xC0\x83`\x05`\0\x19\xFAa-\xCEW`\0\x80\xFD[d\x01\0\0\x03\xD0\x19\x81Qd\x01\0\0\x03\xD0\x19\x8Cd\x01\0\0\x03\xD0\x19\x03\x8B\x08\t\x93PP[PPd\x01\0\0\x03\xD0\x19\x80\x89d\x01\0\0\x03\xD0\x19\x03\x88d\x01\0\0\x03\xD0\x19\x03\x08d\x01\0\0\x03\xD0\x19\x83\x84\t\x08\x92Pd\x01\0\0\x03\xD0\x19\x87d\x01\0\0\x03\xD0\x19\x03d\x01\0\0\x03\xD0\x19\x80\x86d\x01\0\0\x03\xD0\x19\x03\x8C\x08\x84\t\x08\x91PP\x95P\x95\x93PPPPV[`\0\x80`\0a.Z\x85\x85a7+V[\x91P\x91P`@Q`0\x81R` \x80\x82\x01R` `@\x82\x01R\x82``\x82\x01R\x81`\x80\x82\x01R`\x01`\x90\x82\x01Rp\x01EQ#\x19P\xB7_\xC4@-\xA1s/\xC9\xBE\xBE\x19`\xB0\x82\x01R` \x81`\xD0\x83`\x05`\0\x19\xFAa.\xB2W`\0\x80\xFD[Q\x95\x94PPPPPV[`\0\x80`\x01`\x01`\xFF\x1B\x03\x83\x16\x81a.\xD9`\xFF\x86\x90\x1C`\x1Ba?/V[\x90Pa.\xE7\x87\x82\x88\x85a8+V[\x93P\x93PPP\x93P\x93\x91PPV[`\0\x81`\x04\x81\x11\x15a/\tWa/\ta<\xBFV[\x03a/\x11WPV[`\x01\x81`\x04\x81\x11\x15a/%Wa/%a<\xBFV[\x03a/rW`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`\x18`$\x82\x01R\x7FECDSA: invalid signature\0\0\0\0\0\0\0\0`D\x82\x01R`d\x01a\x18RV[`\x02\x81`\x04\x81\x11\x15a/\x86Wa/\x86a<\xBFV[\x03a/\xD3W`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`\x1F`$\x82\x01R\x7FECDSA: invalid signature length\0`D\x82\x01R`d\x01a\x18RV[`\x03\x81`\x04\x81\x11\x15a/\xE7Wa/\xE7a<\xBFV[\x03a\x0BMW`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`\"`$\x82\x01R\x7FECDSA: invalid signature 's' val`D\x82\x01Raue`\xF0\x1B`d\x82\x01R`\x84\x01a\x18RV[``\x83\x15a0\xAEW\x82Q`\0\x03a0\xA7W`\x01`\x01`\xA0\x1B\x03\x85\x16;a0\xA7W`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`\x1D`$\x82\x01R\x7FAddress: call to non-contract\0\0\0`D\x82\x01R`d\x01a\x18RV[P\x81a0\xB8V[a0\xB8\x83\x83a8\xEFV[\x94\x93PPPPV[`\0\x80`\0\x80`\0a0\xD2\x87\x87a9\x19V[\x92P\x92P\x92P`@Q`0\x81R` \x80\x82\x01R` `@\x82\x01R\x83``\x82\x01R\x82`\x80\x82\x01R`\x01`\x90\x82\x01Rd\x01\0\0\x03\xD0\x19`\xB0\x82\x01R` \x81`\xD0\x83`\x05`\0\x19\xFAa1 W`\0\x80\xFD[\x80Q\x95PP`@Q`0\x81R` \x80\x82\x01R\x82`P\x82\x01R` `@\x82\x01R\x81`p\x82\x01R`\x01`\x90\x82\x01Rd\x01\0\0\x03\xD0\x19`\xB0\x82\x01R` \x81`\xD0\x83`\x05`\0\x19\xFAa1mW`\0\x80\xFD[\x80Q\x94PPPPP\x92P\x92\x90PV[`\0\x80d\x01\0\0\x03\xD0\x19\x83\x84\td\x01\0\0\x03\xD0\x19\x81d\x01\0\0\x03\xDB\x19\t\x90Pd\x01\0\0\x03\xD0\x19\x81\x82\td\x01\0\0\x03\xD0\x19\x82\x82\x08\x90Pd\x01\0\0\x03\xD0\x19`\x01\x82\x08d\x01\0\0\x03\xD0\x19a\x06\xEB\x82\t\x90P`\0\x82\x15`\x01\x81\x14a1\xE1W\x80\x15a1\xEFWa1\xFBV[d\x01\0\0\x03\xDB\x19\x91Pa1\xFBV[\x83d\x01\0\0\x03\xD0\x19\x03\x91P[Pd\x01\0\0\x03\xD0\x19\x81\x7F?\x871\xAB\xDDf\x1A\xDC\xA0\x8AUX\xF0\xF5\xD2r\xE9S\xD3c\xCBo\x0E]@TG\xC0\x1ADE3\t\x90Pd\x01\0\0\x03\xD0\x19\x82\x83\t\x92Pd\x01\0\0\x03\xD0\x19\x81\x82\td\x01\0\0\x03\xD0\x19\x81\x7F?\x871\xAB\xDDf\x1A\xDC\xA0\x8AUX\xF0\xF5\xD2r\xE9S\xD3c\xCBo\x0E]@TG\xC0\x1ADE3\td\x01\0\0\x03\xD0\x19\x81\x86\x08\x94Pd\x01\0\0\x03\xD0\x19\x84\x86\t\x94Pd\x01\0\0\x03\xD0\x19\x83\x83\t\x91Pd\x01\0\0\x03\xD0\x19\x82a\x06\xEB\t\x90Pd\x01\0\0\x03\xD0\x19\x81\x86\x08\x94PPd\x01\0\0\x03\xD0\x19\x83\x86\t\x96P`\0\x80d\x01\0\0\x03\xD0\x19\x83\x84\td\x01\0\0\x03\xD0\x19\x84\x88\td\x01\0\0\x03\xD0\x19\x81\x83\t\x91P`@Q` \x81R` \x80\x82\x01R` `@\x82\x01R\x82``\x82\x01Rc@\0\0\xF5`\x01`\xFE\x1B\x03`\x80\x82\x01Rd\x01\0\0\x03\xD0\x19`\xA0\x82\x01R` \x81`\xC0\x83`\x05`\0\x19\xFAa3!W`\0\x80\xFD[d\x01\0\0\x03\xD0\x19\x82\x82Q\t\x92PPPd\x01\0\0\x03\xD0\x19\x7F1\xFD\xF3\x02r@\x13\xE5z\xD1?\xB3\x8F\x84*\xFE\xEC\x18O\0\xA7G\x89\xDD(g)\xC80<JY\x82\td\x01\0\0\x03\xD0\x19\x82\x83\td\x01\0\0\x03\xD0\x19\x86\x82\t\x90P\x88\x81\x14`\x01\x81\x14a3\x86W\x80\x15a3\x92Wa3\x9AV[`\x01\x94P\x83\x95Pa3\x9AV[`\0\x94P\x82\x95P[PPPPd\x01\0\0\x03\xD0\x19\x8A\x88\t\x97Pd\x01\0\0\x03\xD0\x19\x82\x89\t\x97P\x80\x15a3\xC3W\x84\x98P\x81\x97P[PPP`\x02\x85\x06`\x02\x88\x06\x14a3\xDFW\x84d\x01\0\0\x03\xD0\x19\x03\x94P[`@Q\x93P` \x84R` \x80\x85\x01R` `@\x85\x01R\x80``\x85\x01RPPPd\x01\0\0\x03\xD2\x19`\x80\x82\x01Rd\x01\0\0\x03\xD0\x19`\xA0\x82\x01R` \x81`\xC0\x83`\x05`\0\x19\xFAa4+W`\0\x80\xFD[d\x01\0\0\x03\xD0\x19\x81Q\x84\t\x92PP\x91P\x91V[`\0\x80d\x01\0\0\x03\xD0\x19\x84\x85\td\x01\0\0\x03\xD0\x19\x81\x86\td\x01\0\0\x03\xD0\x19\x80\x7F\x8E8\xE3\x8E8\xE3\x8E8\xE3\x8E8\xE3\x8E8\xE3\x8E8\xE3\x8E8\xE3\x8E8\xE3\x8E8\xE3\x8D\xAA\xAA\xA8\xC7d\x01\0\0\x03\xD0\x19\x89\x7F\x07\xD3\xD4\xC8\x0B\xC3!\xD5\xB9\xF3\x15\xCE\xA7\xFDD\xC5\xD5\x95\xD2\xFC\x0B\xF6;\x92\xDF\xFF\x10D\xF1|e\x81\t\x08d\x01\0\0\x03\xD0\x19\x80\x85\x7FSL2\x8D#\xF24\xE6\xE2\xA4\x13\xDE\xCA%\xCA\xEC\xE4PaD\x03|@1N\xCB\xD0\xB5=\x9D\xD2b\td\x01\0\0\x03\xD0\x19\x85\x7F\x8E8\xE3\x8E8\xE3\x8E8\xE3\x8E8\xE3\x8E8\xE3\x8E8\xE3\x8E8\xE3\x8E8\xE3\x8E8\xE3\x8D\xAA\xAA\xA8\x8C\t\x08\x08d\x01\0\0\x03\xD0\x19\x7F\xD3Wq\x19=\x94\x91\x8A\x9C\xA3L\xCB\xB7\xB6@\xDD\x86\xCD@\x95B\xF8H}\x9F\xE6\xB7Ex\x1E\xB4\x9Bd\x01\0\0\x03\xD0\x19\x80\x8A\x7F\xED\xAD\xC6\xF6C\x83\xDC\x1D\xF7\xC4\xB2\xD5\x1BT\"T\x06\xD3kd\x1F^A\xBB\xC5*Va*\x8Cm\x14\t\x86\x08\x08`@Q` \x81R` \x80\x82\x01R` `@\x82\x01R\x81``\x82\x01Rd\x01\0\0\x03\xD2\x19`\x80\x82\x01Rd\x01\0\0\x03\xD0\x19`\xA0\x82\x01R` \x81`\xC0\x83`\x05`\0\x19\xFAa5\x9DW`\0\x80\xFD[\x80Q\x91Pd\x01\0\0\x03\xD0\x19\x82\x84\t\x96Pd\x01\0\0\x03\xD0\x19\x80\x7FK\xDA\x12\xF6\x84\xBD\xA1/hK\xDA\x12\xF6\x84\xBD\xA1/hK\xDA\x12\xF6\x84\xBD\xA1/hK\x8E8\xE2<d\x01\0\0\x03\xD0\x19\x8C\x7F\xC7^\x0C2\xD5\xCB|\x0F\xA9\xD0\xA5K\x12\xA0\xA6\xD5dz\xB0F\xD6\x86\xDAo\xDF\xFC\x90\xFC \x1Dq\xA3\t\x08d\x01\0\0\x03\xD0\x19\x80\x88\x7F)\xA6\x19F\x91\xF9\x1AsqR\t\xEFe\x12\xE5vr(0\xA2\x01\xBE \x18\xA7e\xE8Z\x9E\xCE\xE91\td\x01\0\0\x03\xD0\x19\x88\x7F/hK\xDA\x12\xF6\x84\xBD\xA1/hK\xDA\x12\xF6\x84\xBD\xA1/hK\xDA\x12\xF6\x84\xBD\xA1/8\xE3\x8D\x84\t\x08\x08\x92Pd\x01\0\0\x03\xD0\x19\x80d\x01\0\0\x06\xC4\x19d\x01\0\0\x03\xD0\x19\x8C\x7Fz\x06SK\xB8\xBD\xB4\x9F\xD5\xE9\xE6c'\"\xC2\x98\x94g\xC1\xBF\xC8\xE8\xD9x\xDF\xB4%\xD2h\\%s\t\x08d\x01\0\0\x03\xD0\x19\x80\x88\x7Fd\x84\xAAqeE\xCA,\xF3\xA7\x0C?\xA8\xFE3~\n=!\x16/\rb\x99\xA7\xBF\x81\x92\xBF\xD2\xA7o\t\x87\x08\x08\x94P`@Q\x90P` \x81R` \x80\x82\x01R` `@\x82\x01R\x84``\x82\x01Rd\x01\0\0\x03\xD2\x19`\x80\x82\x01Rd\x01\0\0\x03\xD0\x19`\xA0\x82\x01R` \x81`\xC0\x83`\x05`\0\x19\xFAa7\rW`\0\x80\xFD[Q\x93Pd\x01\0\0\x03\xD0\x19\x90P\x83\x81\x83\x89\t\t\x93PPPP\x92P\x92\x90PV[`\0\x80`\xFF\x83Q\x11\x15a7=W`\0\x80\xFD[`\0`@Q`\x88` `\0[\x88Q\x81\x10\x15a7jW\x88\x82\x01Q\x84\x84\x01R` \x92\x83\x01\x92\x91\x82\x01\x91\x01a7IV[PP`\x89\x87Q\x01\x90P`0\x81\x83\x01S`\x02\x01` `\0[\x87Q\x81\x10\x15a7\xA2W\x87\x82\x01Q\x84\x84\x01R` \x92\x83\x01\x92\x91\x82\x01\x91\x01a7\x81V[PP`\x8B\x86Q\x88Q\x01\x01\x90P\x85Q\x81\x83\x01SP\x85Q\x85Q\x01`\x8C\x01\x81 \x91PP`@Q\x81\x81R`\x01` \x82\x01S`!` `\0[\x87Q\x81\x10\x15a7\xF7W\x87\x82\x01Q\x84\x84\x01R` \x92\x83\x01\x92\x91\x82\x01\x91\x01a7\xD6V[PPP\x84Q\x85Q`!\x01\x82\x01S\x84Q`\"\x01\x81 \x93P\x83\x82\x18\x81R`\x02` \x82\x01S\x84Q`\"\x01\x81 \x92PPP\x92P\x92\x90PV[`\0\x80\x7F\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF]WnsW\xA4P\x1D\xDF\xE9/Fh\x1B \xA0\x83\x11\x15a8bWP`\0\x90P`\x03a8\xE6V[`@\x80Q`\0\x80\x82R` \x82\x01\x80\x84R\x89\x90R`\xFF\x88\x16\x92\x82\x01\x92\x90\x92R``\x81\x01\x86\x90R`\x80\x81\x01\x85\x90R`\x01\x90`\xA0\x01` `@Q` \x81\x03\x90\x80\x84\x03\x90\x85Z\xFA\x15\x80\x15a8\xB6W=`\0\x80>=`\0\xFD[PP`@Q`\x1F\x19\x01Q\x91PP`\x01`\x01`\xA0\x1B\x03\x81\x16a8\xDFW`\0`\x01\x92P\x92PPa8\xE6V[\x91P`\0\x90P[\x94P\x94\x92PPPV[\x81Q\x15a8\xFFW\x81Q\x80\x83` \x01\xFD[\x80`@QbF\x1B\xCD`\xE5\x1B\x81R`\x04\x01a\x18R\x91\x90a>\x86V[`\0\x80`\0`\xFF\x84Q\x11\x15a9-W`\0\x80\xFD[`\0`@Q`\x88` `\0[\x89Q\x81\x10\x15a9ZW\x89\x82\x01Q\x84\x84\x01R` \x92\x83\x01\x92\x91\x82\x01\x91\x01a99V[PP`\x89\x88Q\x01\x90P``\x81\x83\x01S`\x02\x01` `\0[\x88Q\x81\x10\x15a9\x92W\x88\x82\x01Q\x84\x84\x01R` \x92\x83\x01\x92\x91\x82\x01\x91\x01a9qV[PP`\x8B\x87Q\x89Q\x01\x01\x90P\x86Q\x81\x83\x01SP\x86Q\x86Q\x01`\x8C\x01\x81 \x91PP`@Q\x81\x81R`\x01` \x82\x01S`!` `\0[\x88Q\x81\x10\x15a9\xE7W\x88\x82\x01Q\x84\x84\x01R` \x92\x83\x01\x92\x91\x82\x01\x91\x01a9\xC6V[PPP\x85Q\x86Q`!\x01\x82\x01S\x85Q`\"\x01\x81 \x94P\x84\x82\x18\x81R`\x02` \x82\x01S\x85Q`\"\x01\x81 \x93P\x83\x82\x18\x81R`\x03` \x82\x01S\x85Q`\"\x01\x81 \x92PPP\x92P\x92P\x92V[`\x01`\x01`\xA0\x1B\x03\x81\x16\x81\x14a\x0BMW`\0\x80\xFD[`\0\x80\x83`\x1F\x84\x01\x12a:WW`\0\x80\xFD[P\x815g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a:oW`\0\x80\xFD[` \x83\x01\x91P\x83` \x82\x85\x01\x01\x11\x15a,\x1EW`\0\x80\xFD[`\0\x80`\0\x80`\0\x80`\0\x80`\xC0\x89\x8B\x03\x12\x15a:\xA3W`\0\x80\xFD[\x885a:\xAE\x81a:0V[\x97P` \x89\x015a:\xBE\x81a:0V[\x96P`@\x89\x015a:\xCE\x81a:0V[\x95P``\x89\x015\x94P`\x80\x89\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x80\x82\x11\x15a:\xF2W`\0\x80\xFD[a:\xFE\x8C\x83\x8D\x01a:EV[\x90\x96P\x94P`\xA0\x8B\x015\x91P\x80\x82\x11\x15a;\x17W`\0\x80\xFD[Pa;$\x8B\x82\x8C\x01a:EV[\x99\x9C\x98\x9BP\x96\x99P\x94\x97\x93\x96\x92\x95\x94PPPV[\x805`\x01`\x01``\x1B\x03\x81\x16\x81\x14a;OW`\0\x80\xFD[\x91\x90PV[`\0\x80`\0``\x84\x86\x03\x12\x15a;iW`\0\x80\xFD[\x835a;t\x81a:0V[\x92P` \x84\x015a;\x84\x81a:0V[\x91Pa;\x92`@\x85\x01a;8V[\x90P\x92P\x92P\x92V[`\0a\x01 \x82\x84\x03\x12\x15a;\xAEW`\0\x80\xFD[P\x91\x90PV[`\0a\x01\0\x82\x84\x03\x12\x15a;\xAEW`\0\x80\xFD[`\0\x80`\0a\x02@\x84\x86\x03\x12\x15a;\xDDW`\0\x80\xFD[\x835a;\xE8\x81a:0V[\x92Pa;\xF7\x85` \x86\x01a;\x9BV[\x91Pa;\x92\x85a\x01@\x86\x01a;\xB4V[`\0` \x82\x84\x03\x12\x15a<\x19W`\0\x80\xFD[\x815a<$\x81a:0V[\x93\x92PPPV[`\0a\x01 \x82\x84\x03\x12\x15a<>W`\0\x80\xFD[a\r\xDB\x83\x83a;\x9BV[`\0\x80`@\x83\x85\x03\x12\x15a<[W`\0\x80\xFD[\x825\x91P` \x83\x015a<m\x81a:0V[\x80\x91PP\x92P\x92\x90PV[`\0\x80`@\x83\x85\x03\x12\x15a<\x8BW`\0\x80\xFD[\x825a<\x96\x81a:0V[\x91P` \x83\x015a<m\x81a:0V[`\0` \x82\x84\x03\x12\x15a<\xB8W`\0\x80\xFD[P5\x91\x90PV[cNH{q`\xE0\x1B`\0R`!`\x04R`$`\0\xFD[`\x01`\x01``\x1B\x03\x86\x16\x81Re\xFF\xFF\xFF\xFF\xFF\xFF\x85\x16` \x82\x01Rc\xFF\xFF\xFF\xFF\x84\x16`@\x82\x01Rb\xFF\xFF\xFF\x83\x16``\x82\x01R`\xA0\x81\x01`\x03\x83\x10a=(WcNH{q`\xE0\x1B`\0R`!`\x04R`$`\0\xFD[\x82`\x80\x83\x01R\x96\x95PPPPPPV[`\0\x80`\0a\x02@\x84\x86\x03\x12\x15a=NW`\0\x80\xFD[\x835\x92Pa;\xF7\x85` \x86\x01a;\x9BV[`\0\x80` \x83\x85\x03\x12\x15a=rW`\0\x80\xFD[\x825g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x80\x82\x11\x15a=\x8AW`\0\x80\xFD[\x81\x85\x01\x91P\x85`\x1F\x83\x01\x12a=\x9EW`\0\x80\xFD[\x815\x81\x81\x11\x15a=\xADW`\0\x80\xFD[\x86` \x82`\x05\x1B\x85\x01\x01\x11\x15a=\xC2W`\0\x80\xFD[` \x92\x90\x92\x01\x96\x91\x95P\x90\x93PPPPV[`\0[\x83\x81\x10\x15a=\xEFW\x81\x81\x01Q\x83\x82\x01R` \x01a=\xD7V[PP`\0\x91\x01RV[`\0\x81Q\x80\x84Ra>\x10\x81` \x86\x01` \x86\x01a=\xD4V[`\x1F\x01`\x1F\x19\x16\x92\x90\x92\x01` \x01\x92\x91PPV[`\0` \x80\x83\x01\x81\x84R\x80\x85Q\x80\x83R`@\x86\x01\x91P`@\x81`\x05\x1B\x87\x01\x01\x92P\x83\x87\x01`\0[\x82\x81\x10\x15a>yW`?\x19\x88\x86\x03\x01\x84Ra>g\x85\x83Qa=\xF8V[\x94P\x92\x85\x01\x92\x90\x85\x01\x90`\x01\x01a>KV[P\x92\x97\x96PPPPPPPV[` \x81R`\0a\r\xDB` \x83\x01\x84a=\xF8V[`\0\x80`@\x83\x85\x03\x12\x15a>\xACW`\0\x80\xFD[\x825a>\xB7\x81a:0V[\x91Pa>\xC5` \x84\x01a;8V[\x90P\x92P\x92\x90PV[`\0\x80a\x02 \x83\x85\x03\x12\x15a>\xE2W`\0\x80\xFD[a>\xEC\x84\x84a;\x9BV[\x91Pa>\xC5\x84a\x01 \x85\x01a;\xB4V[`\0` \x82\x84\x03\x12\x15a?\x0EW`\0\x80\xFD[\x81Qa<$\x81a:0V[cNH{q`\xE0\x1B`\0R`\x11`\x04R`$`\0\xFD[\x80\x82\x01\x80\x82\x11\x15a\r\xDEWa\r\xDEa?\x19V[`\0` \x82\x84\x03\x12\x15a?TW`\0\x80\xFD[\x81Q\x80\x15\x15\x81\x14a<$W`\0\x80\xFD[`\0` \x82\x84\x03\x12\x15a?vW`\0\x80\xFD[\x815f\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x16\x81\x14a<$W`\0\x80\xFD[`\0` \x82\x84\x03\x12\x15a?\x9FW`\0\x80\xFD[\x815b\xFF\xFF\xFF\x81\x16\x81\x14a<$W`\0\x80\xFD[`\0` \x82\x84\x03\x12\x15a?\xC4W`\0\x80\xFD[\x815c\xFF\xFF\xFF\xFF\x81\x16\x81\x14a<$W`\0\x80\xFD[`\0` \x82\x84\x03\x12\x15a?\xEAW`\0\x80\xFD[\x815e\xFF\xFF\xFF\xFF\xFF\xFF\x81\x16\x81\x14a<$W`\0\x80\xFD[`\0` \x82\x84\x03\x12\x15a@\x12W`\0\x80\xFD[a\r\xDB\x82a;8V[cNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[cNH{q`\xE0\x1B`\0R`2`\x04R`$`\0\xFD[`\0\x80\x835`\x1E\x19\x846\x03\x01\x81\x12a@^W`\0\x80\xFD[\x83\x01\x805\x91Pg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x15a@yW`\0\x80\xFD[` \x01\x91P6\x81\x90\x03\x82\x13\x15a,\x1EW`\0\x80\xFD[`\0`\x01\x82\x01a@\xA0Wa@\xA0a?\x19V[P`\x01\x01\x90V[`\x01`\x01``\x1B\x03\x81\x81\x16\x83\x82\x16\x01\x90\x80\x82\x11\x15a\x12\xDFWa\x12\xDFa?\x19V[b\xFF\xFF\xFF\x81\x81\x16\x83\x82\x16\x01\x90\x80\x82\x11\x15a\x12\xDFWa\x12\xDFa?\x19V[\x92\x83R` \x83\x01\x91\x90\x91R`\xA0\x1B`\x01`\x01`\xA0\x1B\x03\x19\x16`@\x82\x01R`L\x01\x90V[`\0a\x01\0\x80\x83\x85\x03\x12\x15aA\x1AW`\0\x80\xFD[`@Q\x90\x81\x01\x90g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x81\x83\x10\x17\x15aAKWcNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[\x81`@R\x835\x81R` \x84\x015` \x82\x01R`@\x84\x015`@\x82\x01R``\x84\x015``\x82\x01R`\x80\x84\x015`\x80\x82\x01R`\xA0\x84\x015`\xA0\x82\x01R`\xC0\x84\x015`\xC0\x82\x01R`\xE0\x84\x015`\xE0\x82\x01R\x80\x92PPP\x92\x91PPV[e\xFF\xFF\xFF\xFF\xFF\xFF\x81\x81\x16\x83\x82\x16\x01\x90\x80\x82\x11\x15a\x12\xDFWa\x12\xDFa?\x19V[`\x01`\x01``\x1B\x03\x82\x81\x16\x82\x82\x16\x03\x90\x80\x82\x11\x15a\x12\xDFWa\x12\xDFa?\x19V[` \x80\x82R`0\x90\x82\x01R\x7Fchannel must have state OPEN or `@\x82\x01RoPENDING_TO_CLOSE`\x80\x1B``\x82\x01R`\x80\x01\x90V[c\xFF\xFF\xFF\xFF\x81\x81\x16\x83\x82\x16\x01\x90\x80\x82\x11\x15a\x12\xDFWa\x12\xDFa?\x19V[\x81\x81\x03\x81\x81\x11\x15a\r\xDEWa\r\xDEa?\x19V[`\0\x82QaBu\x81\x84` \x87\x01a=\xD4V[\x91\x90\x91\x01\x92\x91PPV[`\0\x82aB\x9CWcNH{q`\xE0\x1B`\0R`\x12`\x04R`$`\0\xFD[P\x06\x90V\xFEAddress: low-level delegate call failed_\xA1rF\xD3\xA5\xD6\x8DB\xBA\xA9L\xDE3\x04!\x80\xB7\x83\xA3\x99\xC0+\xF6:\xC2\x07n\x0Fp\x878\xCE\xEA\xB2\xEE\xF9\x98\xC1\x7F\xE9o0\xF8?\xBF<U\xFCPG\xF6\xE4\x0CU\xA0\xCFr\xD26\xE9\xD2\xBAr\xA2dipfsX\"\x12 #C\x98\r\x92\x99\x8E\xDA\xEE\x11\xA6gb5\xEBu\x89\x9E\x11\x17\x08\xED\n*3T\\\xCD\xCB\x0E\x05\x03dsolcC\0\x08\x13\x003",
    );
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct ChannelStatus(u8);
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[automatically_derived]
        impl alloy_sol_types::private::SolTypeValue<ChannelStatus> for u8 {
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
        impl ChannelStatus {
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
        impl From<u8> for ChannelStatus {
            fn from(value: u8) -> Self {
                Self::from_underlying(value)
            }
        }
        #[automatically_derived]
        impl From<ChannelStatus> for u8 {
            fn from(value: ChannelStatus) -> Self {
                value.into_underlying()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolType for ChannelStatus {
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
        impl alloy_sol_types::EventTopic for ChannelStatus {
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
    pub struct Balance(alloy::sol_types::private::primitives::aliases::U96);
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[automatically_derived]
        impl alloy_sol_types::private::SolTypeValue<Balance>
        for alloy::sol_types::private::primitives::aliases::U96 {
            #[inline]
            fn stv_to_tokens(
                &self,
            ) -> <alloy::sol_types::sol_data::Uint<
                96,
            > as alloy_sol_types::SolType>::Token<'_> {
                alloy_sol_types::private::SolTypeValue::<
                    alloy::sol_types::sol_data::Uint<96>,
                >::stv_to_tokens(self)
            }
            #[inline]
            fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
                <alloy::sol_types::sol_data::Uint<
                    96,
                > as alloy_sol_types::SolType>::tokenize(self)
                    .0
            }
            #[inline]
            fn stv_abi_encode_packed_to(
                &self,
                out: &mut alloy_sol_types::private::Vec<u8>,
            ) {
                <alloy::sol_types::sol_data::Uint<
                    96,
                > as alloy_sol_types::SolType>::abi_encode_packed_to(self, out)
            }
            #[inline]
            fn stv_abi_packed_encoded_size(&self) -> usize {
                <alloy::sol_types::sol_data::Uint<
                    96,
                > as alloy_sol_types::SolType>::abi_encoded_size(self)
            }
        }
        #[automatically_derived]
        impl Balance {
            /// The Solidity type name.
            pub const NAME: &'static str = stringify!(@ name);
            /// Convert from the underlying value type.
            #[inline]
            pub const fn from_underlying(
                value: alloy::sol_types::private::primitives::aliases::U96,
            ) -> Self {
                Self(value)
            }
            /// Return the underlying value.
            #[inline]
            pub const fn into_underlying(
                self,
            ) -> alloy::sol_types::private::primitives::aliases::U96 {
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
        impl From<alloy::sol_types::private::primitives::aliases::U96> for Balance {
            fn from(value: alloy::sol_types::private::primitives::aliases::U96) -> Self {
                Self::from_underlying(value)
            }
        }
        #[automatically_derived]
        impl From<Balance> for alloy::sol_types::private::primitives::aliases::U96 {
            fn from(value: Balance) -> Self {
                value.into_underlying()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolType for Balance {
            type RustType = alloy::sol_types::private::primitives::aliases::U96;
            type Token<'a> = <alloy::sol_types::sol_data::Uint<
                96,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SOL_NAME: &'static str = Self::NAME;
            const ENCODED_SIZE: Option<usize> = <alloy::sol_types::sol_data::Uint<
                96,
            > as alloy_sol_types::SolType>::ENCODED_SIZE;
            const PACKED_ENCODED_SIZE: Option<usize> = <alloy::sol_types::sol_data::Uint<
                96,
            > as alloy_sol_types::SolType>::PACKED_ENCODED_SIZE;
            #[inline]
            fn valid_token(token: &Self::Token<'_>) -> bool {
                Self::type_check(token).is_ok()
            }
            #[inline]
            fn type_check(token: &Self::Token<'_>) -> alloy_sol_types::Result<()> {
                <alloy::sol_types::sol_data::Uint<
                    96,
                > as alloy_sol_types::SolType>::type_check(token)
            }
            #[inline]
            fn detokenize(token: Self::Token<'_>) -> Self::RustType {
                <alloy::sol_types::sol_data::Uint<
                    96,
                > as alloy_sol_types::SolType>::detokenize(token)
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::EventTopic for Balance {
            #[inline]
            fn topic_preimage_length(rust: &Self::RustType) -> usize {
                <alloy::sol_types::sol_data::Uint<
                    96,
                > as alloy_sol_types::EventTopic>::topic_preimage_length(rust)
            }
            #[inline]
            fn encode_topic_preimage(
                rust: &Self::RustType,
                out: &mut alloy_sol_types::private::Vec<u8>,
            ) {
                <alloy::sol_types::sol_data::Uint<
                    96,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(rust, out)
            }
            #[inline]
            fn encode_topic(
                rust: &Self::RustType,
            ) -> alloy_sol_types::abi::token::WordToken {
                <alloy::sol_types::sol_data::Uint<
                    96,
                > as alloy_sol_types::EventTopic>::encode_topic(rust)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct ChannelEpoch(alloy::sol_types::private::primitives::aliases::U24);
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[automatically_derived]
        impl alloy_sol_types::private::SolTypeValue<ChannelEpoch>
        for alloy::sol_types::private::primitives::aliases::U24 {
            #[inline]
            fn stv_to_tokens(
                &self,
            ) -> <alloy::sol_types::sol_data::Uint<
                24,
            > as alloy_sol_types::SolType>::Token<'_> {
                alloy_sol_types::private::SolTypeValue::<
                    alloy::sol_types::sol_data::Uint<24>,
                >::stv_to_tokens(self)
            }
            #[inline]
            fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
                <alloy::sol_types::sol_data::Uint<
                    24,
                > as alloy_sol_types::SolType>::tokenize(self)
                    .0
            }
            #[inline]
            fn stv_abi_encode_packed_to(
                &self,
                out: &mut alloy_sol_types::private::Vec<u8>,
            ) {
                <alloy::sol_types::sol_data::Uint<
                    24,
                > as alloy_sol_types::SolType>::abi_encode_packed_to(self, out)
            }
            #[inline]
            fn stv_abi_packed_encoded_size(&self) -> usize {
                <alloy::sol_types::sol_data::Uint<
                    24,
                > as alloy_sol_types::SolType>::abi_encoded_size(self)
            }
        }
        #[automatically_derived]
        impl ChannelEpoch {
            /// The Solidity type name.
            pub const NAME: &'static str = stringify!(@ name);
            /// Convert from the underlying value type.
            #[inline]
            pub const fn from_underlying(
                value: alloy::sol_types::private::primitives::aliases::U24,
            ) -> Self {
                Self(value)
            }
            /// Return the underlying value.
            #[inline]
            pub const fn into_underlying(
                self,
            ) -> alloy::sol_types::private::primitives::aliases::U24 {
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
        impl From<alloy::sol_types::private::primitives::aliases::U24> for ChannelEpoch {
            fn from(value: alloy::sol_types::private::primitives::aliases::U24) -> Self {
                Self::from_underlying(value)
            }
        }
        #[automatically_derived]
        impl From<ChannelEpoch> for alloy::sol_types::private::primitives::aliases::U24 {
            fn from(value: ChannelEpoch) -> Self {
                value.into_underlying()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolType for ChannelEpoch {
            type RustType = alloy::sol_types::private::primitives::aliases::U24;
            type Token<'a> = <alloy::sol_types::sol_data::Uint<
                24,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SOL_NAME: &'static str = Self::NAME;
            const ENCODED_SIZE: Option<usize> = <alloy::sol_types::sol_data::Uint<
                24,
            > as alloy_sol_types::SolType>::ENCODED_SIZE;
            const PACKED_ENCODED_SIZE: Option<usize> = <alloy::sol_types::sol_data::Uint<
                24,
            > as alloy_sol_types::SolType>::PACKED_ENCODED_SIZE;
            #[inline]
            fn valid_token(token: &Self::Token<'_>) -> bool {
                Self::type_check(token).is_ok()
            }
            #[inline]
            fn type_check(token: &Self::Token<'_>) -> alloy_sol_types::Result<()> {
                <alloy::sol_types::sol_data::Uint<
                    24,
                > as alloy_sol_types::SolType>::type_check(token)
            }
            #[inline]
            fn detokenize(token: Self::Token<'_>) -> Self::RustType {
                <alloy::sol_types::sol_data::Uint<
                    24,
                > as alloy_sol_types::SolType>::detokenize(token)
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::EventTopic for ChannelEpoch {
            #[inline]
            fn topic_preimage_length(rust: &Self::RustType) -> usize {
                <alloy::sol_types::sol_data::Uint<
                    24,
                > as alloy_sol_types::EventTopic>::topic_preimage_length(rust)
            }
            #[inline]
            fn encode_topic_preimage(
                rust: &Self::RustType,
                out: &mut alloy_sol_types::private::Vec<u8>,
            ) {
                <alloy::sol_types::sol_data::Uint<
                    24,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(rust, out)
            }
            #[inline]
            fn encode_topic(
                rust: &Self::RustType,
            ) -> alloy_sol_types::abi::token::WordToken {
                <alloy::sol_types::sol_data::Uint<
                    24,
                > as alloy_sol_types::EventTopic>::encode_topic(rust)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct TicketIndex(alloy::sol_types::private::primitives::aliases::U48);
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[automatically_derived]
        impl alloy_sol_types::private::SolTypeValue<TicketIndex>
        for alloy::sol_types::private::primitives::aliases::U48 {
            #[inline]
            fn stv_to_tokens(
                &self,
            ) -> <alloy::sol_types::sol_data::Uint<
                48,
            > as alloy_sol_types::SolType>::Token<'_> {
                alloy_sol_types::private::SolTypeValue::<
                    alloy::sol_types::sol_data::Uint<48>,
                >::stv_to_tokens(self)
            }
            #[inline]
            fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
                <alloy::sol_types::sol_data::Uint<
                    48,
                > as alloy_sol_types::SolType>::tokenize(self)
                    .0
            }
            #[inline]
            fn stv_abi_encode_packed_to(
                &self,
                out: &mut alloy_sol_types::private::Vec<u8>,
            ) {
                <alloy::sol_types::sol_data::Uint<
                    48,
                > as alloy_sol_types::SolType>::abi_encode_packed_to(self, out)
            }
            #[inline]
            fn stv_abi_packed_encoded_size(&self) -> usize {
                <alloy::sol_types::sol_data::Uint<
                    48,
                > as alloy_sol_types::SolType>::abi_encoded_size(self)
            }
        }
        #[automatically_derived]
        impl TicketIndex {
            /// The Solidity type name.
            pub const NAME: &'static str = stringify!(@ name);
            /// Convert from the underlying value type.
            #[inline]
            pub const fn from_underlying(
                value: alloy::sol_types::private::primitives::aliases::U48,
            ) -> Self {
                Self(value)
            }
            /// Return the underlying value.
            #[inline]
            pub const fn into_underlying(
                self,
            ) -> alloy::sol_types::private::primitives::aliases::U48 {
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
        impl From<alloy::sol_types::private::primitives::aliases::U48> for TicketIndex {
            fn from(value: alloy::sol_types::private::primitives::aliases::U48) -> Self {
                Self::from_underlying(value)
            }
        }
        #[automatically_derived]
        impl From<TicketIndex> for alloy::sol_types::private::primitives::aliases::U48 {
            fn from(value: TicketIndex) -> Self {
                value.into_underlying()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolType for TicketIndex {
            type RustType = alloy::sol_types::private::primitives::aliases::U48;
            type Token<'a> = <alloy::sol_types::sol_data::Uint<
                48,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SOL_NAME: &'static str = Self::NAME;
            const ENCODED_SIZE: Option<usize> = <alloy::sol_types::sol_data::Uint<
                48,
            > as alloy_sol_types::SolType>::ENCODED_SIZE;
            const PACKED_ENCODED_SIZE: Option<usize> = <alloy::sol_types::sol_data::Uint<
                48,
            > as alloy_sol_types::SolType>::PACKED_ENCODED_SIZE;
            #[inline]
            fn valid_token(token: &Self::Token<'_>) -> bool {
                Self::type_check(token).is_ok()
            }
            #[inline]
            fn type_check(token: &Self::Token<'_>) -> alloy_sol_types::Result<()> {
                <alloy::sol_types::sol_data::Uint<
                    48,
                > as alloy_sol_types::SolType>::type_check(token)
            }
            #[inline]
            fn detokenize(token: Self::Token<'_>) -> Self::RustType {
                <alloy::sol_types::sol_data::Uint<
                    48,
                > as alloy_sol_types::SolType>::detokenize(token)
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::EventTopic for TicketIndex {
            #[inline]
            fn topic_preimage_length(rust: &Self::RustType) -> usize {
                <alloy::sol_types::sol_data::Uint<
                    48,
                > as alloy_sol_types::EventTopic>::topic_preimage_length(rust)
            }
            #[inline]
            fn encode_topic_preimage(
                rust: &Self::RustType,
                out: &mut alloy_sol_types::private::Vec<u8>,
            ) {
                <alloy::sol_types::sol_data::Uint<
                    48,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(rust, out)
            }
            #[inline]
            fn encode_topic(
                rust: &Self::RustType,
            ) -> alloy_sol_types::abi::token::WordToken {
                <alloy::sol_types::sol_data::Uint<
                    48,
                > as alloy_sol_types::EventTopic>::encode_topic(rust)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct TicketIndexOffset(u32);
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[automatically_derived]
        impl alloy_sol_types::private::SolTypeValue<TicketIndexOffset> for u32 {
            #[inline]
            fn stv_to_tokens(
                &self,
            ) -> <alloy::sol_types::sol_data::Uint<
                32,
            > as alloy_sol_types::SolType>::Token<'_> {
                alloy_sol_types::private::SolTypeValue::<
                    alloy::sol_types::sol_data::Uint<32>,
                >::stv_to_tokens(self)
            }
            #[inline]
            fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
                <alloy::sol_types::sol_data::Uint<
                    32,
                > as alloy_sol_types::SolType>::tokenize(self)
                    .0
            }
            #[inline]
            fn stv_abi_encode_packed_to(
                &self,
                out: &mut alloy_sol_types::private::Vec<u8>,
            ) {
                <alloy::sol_types::sol_data::Uint<
                    32,
                > as alloy_sol_types::SolType>::abi_encode_packed_to(self, out)
            }
            #[inline]
            fn stv_abi_packed_encoded_size(&self) -> usize {
                <alloy::sol_types::sol_data::Uint<
                    32,
                > as alloy_sol_types::SolType>::abi_encoded_size(self)
            }
        }
        #[automatically_derived]
        impl TicketIndexOffset {
            /// The Solidity type name.
            pub const NAME: &'static str = stringify!(@ name);
            /// Convert from the underlying value type.
            #[inline]
            pub const fn from_underlying(value: u32) -> Self {
                Self(value)
            }
            /// Return the underlying value.
            #[inline]
            pub const fn into_underlying(self) -> u32 {
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
        impl From<u32> for TicketIndexOffset {
            fn from(value: u32) -> Self {
                Self::from_underlying(value)
            }
        }
        #[automatically_derived]
        impl From<TicketIndexOffset> for u32 {
            fn from(value: TicketIndexOffset) -> Self {
                value.into_underlying()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolType for TicketIndexOffset {
            type RustType = u32;
            type Token<'a> = <alloy::sol_types::sol_data::Uint<
                32,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SOL_NAME: &'static str = Self::NAME;
            const ENCODED_SIZE: Option<usize> = <alloy::sol_types::sol_data::Uint<
                32,
            > as alloy_sol_types::SolType>::ENCODED_SIZE;
            const PACKED_ENCODED_SIZE: Option<usize> = <alloy::sol_types::sol_data::Uint<
                32,
            > as alloy_sol_types::SolType>::PACKED_ENCODED_SIZE;
            #[inline]
            fn valid_token(token: &Self::Token<'_>) -> bool {
                Self::type_check(token).is_ok()
            }
            #[inline]
            fn type_check(token: &Self::Token<'_>) -> alloy_sol_types::Result<()> {
                <alloy::sol_types::sol_data::Uint<
                    32,
                > as alloy_sol_types::SolType>::type_check(token)
            }
            #[inline]
            fn detokenize(token: Self::Token<'_>) -> Self::RustType {
                <alloy::sol_types::sol_data::Uint<
                    32,
                > as alloy_sol_types::SolType>::detokenize(token)
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::EventTopic for TicketIndexOffset {
            #[inline]
            fn topic_preimage_length(rust: &Self::RustType) -> usize {
                <alloy::sol_types::sol_data::Uint<
                    32,
                > as alloy_sol_types::EventTopic>::topic_preimage_length(rust)
            }
            #[inline]
            fn encode_topic_preimage(
                rust: &Self::RustType,
                out: &mut alloy_sol_types::private::Vec<u8>,
            ) {
                <alloy::sol_types::sol_data::Uint<
                    32,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(rust, out)
            }
            #[inline]
            fn encode_topic(
                rust: &Self::RustType,
            ) -> alloy_sol_types::abi::token::WordToken {
                <alloy::sol_types::sol_data::Uint<
                    32,
                > as alloy_sol_types::EventTopic>::encode_topic(rust)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct Timestamp(u32);
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[automatically_derived]
        impl alloy_sol_types::private::SolTypeValue<Timestamp> for u32 {
            #[inline]
            fn stv_to_tokens(
                &self,
            ) -> <alloy::sol_types::sol_data::Uint<
                32,
            > as alloy_sol_types::SolType>::Token<'_> {
                alloy_sol_types::private::SolTypeValue::<
                    alloy::sol_types::sol_data::Uint<32>,
                >::stv_to_tokens(self)
            }
            #[inline]
            fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
                <alloy::sol_types::sol_data::Uint<
                    32,
                > as alloy_sol_types::SolType>::tokenize(self)
                    .0
            }
            #[inline]
            fn stv_abi_encode_packed_to(
                &self,
                out: &mut alloy_sol_types::private::Vec<u8>,
            ) {
                <alloy::sol_types::sol_data::Uint<
                    32,
                > as alloy_sol_types::SolType>::abi_encode_packed_to(self, out)
            }
            #[inline]
            fn stv_abi_packed_encoded_size(&self) -> usize {
                <alloy::sol_types::sol_data::Uint<
                    32,
                > as alloy_sol_types::SolType>::abi_encoded_size(self)
            }
        }
        #[automatically_derived]
        impl Timestamp {
            /// The Solidity type name.
            pub const NAME: &'static str = stringify!(@ name);
            /// Convert from the underlying value type.
            #[inline]
            pub const fn from_underlying(value: u32) -> Self {
                Self(value)
            }
            /// Return the underlying value.
            #[inline]
            pub const fn into_underlying(self) -> u32 {
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
        impl From<u32> for Timestamp {
            fn from(value: u32) -> Self {
                Self::from_underlying(value)
            }
        }
        #[automatically_derived]
        impl From<Timestamp> for u32 {
            fn from(value: Timestamp) -> Self {
                value.into_underlying()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolType for Timestamp {
            type RustType = u32;
            type Token<'a> = <alloy::sol_types::sol_data::Uint<
                32,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SOL_NAME: &'static str = Self::NAME;
            const ENCODED_SIZE: Option<usize> = <alloy::sol_types::sol_data::Uint<
                32,
            > as alloy_sol_types::SolType>::ENCODED_SIZE;
            const PACKED_ENCODED_SIZE: Option<usize> = <alloy::sol_types::sol_data::Uint<
                32,
            > as alloy_sol_types::SolType>::PACKED_ENCODED_SIZE;
            #[inline]
            fn valid_token(token: &Self::Token<'_>) -> bool {
                Self::type_check(token).is_ok()
            }
            #[inline]
            fn type_check(token: &Self::Token<'_>) -> alloy_sol_types::Result<()> {
                <alloy::sol_types::sol_data::Uint<
                    32,
                > as alloy_sol_types::SolType>::type_check(token)
            }
            #[inline]
            fn detokenize(token: Self::Token<'_>) -> Self::RustType {
                <alloy::sol_types::sol_data::Uint<
                    32,
                > as alloy_sol_types::SolType>::detokenize(token)
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::EventTopic for Timestamp {
            #[inline]
            fn topic_preimage_length(rust: &Self::RustType) -> usize {
                <alloy::sol_types::sol_data::Uint<
                    32,
                > as alloy_sol_types::EventTopic>::topic_preimage_length(rust)
            }
            #[inline]
            fn encode_topic_preimage(
                rust: &Self::RustType,
                out: &mut alloy_sol_types::private::Vec<u8>,
            ) {
                <alloy::sol_types::sol_data::Uint<
                    32,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(rust, out)
            }
            #[inline]
            fn encode_topic(
                rust: &Self::RustType,
            ) -> alloy_sol_types::abi::token::WordToken {
                <alloy::sol_types::sol_data::Uint<
                    32,
                > as alloy_sol_types::EventTopic>::encode_topic(rust)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct WinProb(alloy::sol_types::private::primitives::aliases::U56);
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[automatically_derived]
        impl alloy_sol_types::private::SolTypeValue<WinProb>
        for alloy::sol_types::private::primitives::aliases::U56 {
            #[inline]
            fn stv_to_tokens(
                &self,
            ) -> <alloy::sol_types::sol_data::Uint<
                56,
            > as alloy_sol_types::SolType>::Token<'_> {
                alloy_sol_types::private::SolTypeValue::<
                    alloy::sol_types::sol_data::Uint<56>,
                >::stv_to_tokens(self)
            }
            #[inline]
            fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
                <alloy::sol_types::sol_data::Uint<
                    56,
                > as alloy_sol_types::SolType>::tokenize(self)
                    .0
            }
            #[inline]
            fn stv_abi_encode_packed_to(
                &self,
                out: &mut alloy_sol_types::private::Vec<u8>,
            ) {
                <alloy::sol_types::sol_data::Uint<
                    56,
                > as alloy_sol_types::SolType>::abi_encode_packed_to(self, out)
            }
            #[inline]
            fn stv_abi_packed_encoded_size(&self) -> usize {
                <alloy::sol_types::sol_data::Uint<
                    56,
                > as alloy_sol_types::SolType>::abi_encoded_size(self)
            }
        }
        #[automatically_derived]
        impl WinProb {
            /// The Solidity type name.
            pub const NAME: &'static str = stringify!(@ name);
            /// Convert from the underlying value type.
            #[inline]
            pub const fn from_underlying(
                value: alloy::sol_types::private::primitives::aliases::U56,
            ) -> Self {
                Self(value)
            }
            /// Return the underlying value.
            #[inline]
            pub const fn into_underlying(
                self,
            ) -> alloy::sol_types::private::primitives::aliases::U56 {
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
        impl From<alloy::sol_types::private::primitives::aliases::U56> for WinProb {
            fn from(value: alloy::sol_types::private::primitives::aliases::U56) -> Self {
                Self::from_underlying(value)
            }
        }
        #[automatically_derived]
        impl From<WinProb> for alloy::sol_types::private::primitives::aliases::U56 {
            fn from(value: WinProb) -> Self {
                value.into_underlying()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolType for WinProb {
            type RustType = alloy::sol_types::private::primitives::aliases::U56;
            type Token<'a> = <alloy::sol_types::sol_data::Uint<
                56,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SOL_NAME: &'static str = Self::NAME;
            const ENCODED_SIZE: Option<usize> = <alloy::sol_types::sol_data::Uint<
                56,
            > as alloy_sol_types::SolType>::ENCODED_SIZE;
            const PACKED_ENCODED_SIZE: Option<usize> = <alloy::sol_types::sol_data::Uint<
                56,
            > as alloy_sol_types::SolType>::PACKED_ENCODED_SIZE;
            #[inline]
            fn valid_token(token: &Self::Token<'_>) -> bool {
                Self::type_check(token).is_ok()
            }
            #[inline]
            fn type_check(token: &Self::Token<'_>) -> alloy_sol_types::Result<()> {
                <alloy::sol_types::sol_data::Uint<
                    56,
                > as alloy_sol_types::SolType>::type_check(token)
            }
            #[inline]
            fn detokenize(token: Self::Token<'_>) -> Self::RustType {
                <alloy::sol_types::sol_data::Uint<
                    56,
                > as alloy_sol_types::SolType>::detokenize(token)
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::EventTopic for WinProb {
            #[inline]
            fn topic_preimage_length(rust: &Self::RustType) -> usize {
                <alloy::sol_types::sol_data::Uint<
                    56,
                > as alloy_sol_types::EventTopic>::topic_preimage_length(rust)
            }
            #[inline]
            fn encode_topic_preimage(
                rust: &Self::RustType,
                out: &mut alloy_sol_types::private::Vec<u8>,
            ) {
                <alloy::sol_types::sol_data::Uint<
                    56,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(rust, out)
            }
            #[inline]
            fn encode_topic(
                rust: &Self::RustType,
            ) -> alloy_sol_types::abi::token::WordToken {
                <alloy::sol_types::sol_data::Uint<
                    56,
                > as alloy_sol_types::EventTopic>::encode_topic(rust)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**```solidity
struct RedeemableTicket { TicketData data; HoprCrypto.CompactSignature signature; uint256 porSecret; }
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct RedeemableTicket {
        #[allow(missing_docs)]
        pub data: <TicketData as alloy::sol_types::SolType>::RustType,
        #[allow(missing_docs)]
        pub signature: <HoprCrypto::CompactSignature as alloy::sol_types::SolType>::RustType,
        #[allow(missing_docs)]
        pub porSecret: alloy::sol_types::private::primitives::aliases::U256,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = (
            TicketData,
            HoprCrypto::CompactSignature,
            alloy::sol_types::sol_data::Uint<256>,
        );
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = (
            <TicketData as alloy::sol_types::SolType>::RustType,
            <HoprCrypto::CompactSignature as alloy::sol_types::SolType>::RustType,
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
        impl ::core::convert::From<RedeemableTicket> for UnderlyingRustTuple<'_> {
            fn from(value: RedeemableTicket) -> Self {
                (value.data, value.signature, value.porSecret)
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for RedeemableTicket {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self {
                    data: tuple.0,
                    signature: tuple.1,
                    porSecret: tuple.2,
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolValue for RedeemableTicket {
            type SolType = Self;
        }
        #[automatically_derived]
        impl alloy_sol_types::private::SolTypeValue<Self> for RedeemableTicket {
            #[inline]
            fn stv_to_tokens(&self) -> <Self as alloy_sol_types::SolType>::Token<'_> {
                (
                    <TicketData as alloy_sol_types::SolType>::tokenize(&self.data),
                    <HoprCrypto::CompactSignature as alloy_sol_types::SolType>::tokenize(
                        &self.signature,
                    ),
                    <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::SolType>::tokenize(&self.porSecret),
                )
            }
            #[inline]
            fn stv_abi_encoded_size(&self) -> usize {
                if let Some(size) = <Self as alloy_sol_types::SolType>::ENCODED_SIZE {
                    return size;
                }
                let tuple = <UnderlyingRustTuple<
                    '_,
                > as ::core::convert::From<Self>>::from(self.clone());
                <UnderlyingSolTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_encoded_size(&tuple)
            }
            #[inline]
            fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
                <Self as alloy_sol_types::SolStruct>::eip712_hash_struct(self)
            }
            #[inline]
            fn stv_abi_encode_packed_to(
                &self,
                out: &mut alloy_sol_types::private::Vec<u8>,
            ) {
                let tuple = <UnderlyingRustTuple<
                    '_,
                > as ::core::convert::From<Self>>::from(self.clone());
                <UnderlyingSolTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_encode_packed_to(&tuple, out)
            }
            #[inline]
            fn stv_abi_packed_encoded_size(&self) -> usize {
                if let Some(size) = <Self as alloy_sol_types::SolType>::PACKED_ENCODED_SIZE {
                    return size;
                }
                let tuple = <UnderlyingRustTuple<
                    '_,
                > as ::core::convert::From<Self>>::from(self.clone());
                <UnderlyingSolTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_packed_encoded_size(&tuple)
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolType for RedeemableTicket {
            type RustType = Self;
            type Token<'a> = <UnderlyingSolTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SOL_NAME: &'static str = <Self as alloy_sol_types::SolStruct>::NAME;
            const ENCODED_SIZE: Option<usize> = <UnderlyingSolTuple<
                '_,
            > as alloy_sol_types::SolType>::ENCODED_SIZE;
            const PACKED_ENCODED_SIZE: Option<usize> = <UnderlyingSolTuple<
                '_,
            > as alloy_sol_types::SolType>::PACKED_ENCODED_SIZE;
            #[inline]
            fn valid_token(token: &Self::Token<'_>) -> bool {
                <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::valid_token(token)
            }
            #[inline]
            fn detokenize(token: Self::Token<'_>) -> Self::RustType {
                let tuple = <UnderlyingSolTuple<
                    '_,
                > as alloy_sol_types::SolType>::detokenize(token);
                <Self as ::core::convert::From<UnderlyingRustTuple<'_>>>::from(tuple)
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolStruct for RedeemableTicket {
            const NAME: &'static str = "RedeemableTicket";
            #[inline]
            fn eip712_root_type() -> alloy_sol_types::private::Cow<'static, str> {
                alloy_sol_types::private::Cow::Borrowed(
                    "RedeemableTicket(TicketData data,HoprCrypto.CompactSignature signature,uint256 porSecret)",
                )
            }
            #[inline]
            fn eip712_components() -> alloy_sol_types::private::Vec<
                alloy_sol_types::private::Cow<'static, str>,
            > {
                let mut components = alloy_sol_types::private::Vec::with_capacity(2);
                components
                    .push(
                        <TicketData as alloy_sol_types::SolStruct>::eip712_root_type(),
                    );
                components
                    .extend(
                        <TicketData as alloy_sol_types::SolStruct>::eip712_components(),
                    );
                components
                    .push(
                        <HoprCrypto::CompactSignature as alloy_sol_types::SolStruct>::eip712_root_type(),
                    );
                components
                    .extend(
                        <HoprCrypto::CompactSignature as alloy_sol_types::SolStruct>::eip712_components(),
                    );
                components
            }
            #[inline]
            fn eip712_encode_data(&self) -> alloy_sol_types::private::Vec<u8> {
                [
                    <TicketData as alloy_sol_types::SolType>::eip712_data_word(
                            &self.data,
                        )
                        .0,
                    <HoprCrypto::CompactSignature as alloy_sol_types::SolType>::eip712_data_word(
                            &self.signature,
                        )
                        .0,
                    <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::SolType>::eip712_data_word(&self.porSecret)
                        .0,
                ]
                    .concat()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::EventTopic for RedeemableTicket {
            #[inline]
            fn topic_preimage_length(rust: &Self::RustType) -> usize {
                0usize
                    + <TicketData as alloy_sol_types::EventTopic>::topic_preimage_length(
                        &rust.data,
                    )
                    + <HoprCrypto::CompactSignature as alloy_sol_types::EventTopic>::topic_preimage_length(
                        &rust.signature,
                    )
                    + <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::EventTopic>::topic_preimage_length(
                        &rust.porSecret,
                    )
            }
            #[inline]
            fn encode_topic_preimage(
                rust: &Self::RustType,
                out: &mut alloy_sol_types::private::Vec<u8>,
            ) {
                out.reserve(
                    <Self as alloy_sol_types::EventTopic>::topic_preimage_length(rust),
                );
                <TicketData as alloy_sol_types::EventTopic>::encode_topic_preimage(
                    &rust.data,
                    out,
                );
                <HoprCrypto::CompactSignature as alloy_sol_types::EventTopic>::encode_topic_preimage(
                    &rust.signature,
                    out,
                );
                <alloy::sol_types::sol_data::Uint<
                    256,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(
                    &rust.porSecret,
                    out,
                );
            }
            #[inline]
            fn encode_topic(
                rust: &Self::RustType,
            ) -> alloy_sol_types::abi::token::WordToken {
                let mut out = alloy_sol_types::private::Vec::new();
                <Self as alloy_sol_types::EventTopic>::encode_topic_preimage(
                    rust,
                    &mut out,
                );
                alloy_sol_types::abi::token::WordToken(
                    alloy_sol_types::private::keccak256(out),
                )
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**```solidity
struct TicketData { bytes32 channelId; Balance amount; TicketIndex ticketIndex; TicketIndexOffset indexOffset; ChannelEpoch epoch; WinProb winProb; }
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct TicketData {
        #[allow(missing_docs)]
        pub channelId: alloy::sol_types::private::FixedBytes<32>,
        #[allow(missing_docs)]
        pub amount: <Balance as alloy::sol_types::SolType>::RustType,
        #[allow(missing_docs)]
        pub ticketIndex: <TicketIndex as alloy::sol_types::SolType>::RustType,
        #[allow(missing_docs)]
        pub indexOffset: <TicketIndexOffset as alloy::sol_types::SolType>::RustType,
        #[allow(missing_docs)]
        pub epoch: <ChannelEpoch as alloy::sol_types::SolType>::RustType,
        #[allow(missing_docs)]
        pub winProb: <WinProb as alloy::sol_types::SolType>::RustType,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = (
            alloy::sol_types::sol_data::FixedBytes<32>,
            Balance,
            TicketIndex,
            TicketIndexOffset,
            ChannelEpoch,
            WinProb,
        );
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = (
            alloy::sol_types::private::FixedBytes<32>,
            <Balance as alloy::sol_types::SolType>::RustType,
            <TicketIndex as alloy::sol_types::SolType>::RustType,
            <TicketIndexOffset as alloy::sol_types::SolType>::RustType,
            <ChannelEpoch as alloy::sol_types::SolType>::RustType,
            <WinProb as alloy::sol_types::SolType>::RustType,
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
        impl ::core::convert::From<TicketData> for UnderlyingRustTuple<'_> {
            fn from(value: TicketData) -> Self {
                (
                    value.channelId,
                    value.amount,
                    value.ticketIndex,
                    value.indexOffset,
                    value.epoch,
                    value.winProb,
                )
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for TicketData {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self {
                    channelId: tuple.0,
                    amount: tuple.1,
                    ticketIndex: tuple.2,
                    indexOffset: tuple.3,
                    epoch: tuple.4,
                    winProb: tuple.5,
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolValue for TicketData {
            type SolType = Self;
        }
        #[automatically_derived]
        impl alloy_sol_types::private::SolTypeValue<Self> for TicketData {
            #[inline]
            fn stv_to_tokens(&self) -> <Self as alloy_sol_types::SolType>::Token<'_> {
                (
                    <alloy::sol_types::sol_data::FixedBytes<
                        32,
                    > as alloy_sol_types::SolType>::tokenize(&self.channelId),
                    <Balance as alloy_sol_types::SolType>::tokenize(&self.amount),
                    <TicketIndex as alloy_sol_types::SolType>::tokenize(
                        &self.ticketIndex,
                    ),
                    <TicketIndexOffset as alloy_sol_types::SolType>::tokenize(
                        &self.indexOffset,
                    ),
                    <ChannelEpoch as alloy_sol_types::SolType>::tokenize(&self.epoch),
                    <WinProb as alloy_sol_types::SolType>::tokenize(&self.winProb),
                )
            }
            #[inline]
            fn stv_abi_encoded_size(&self) -> usize {
                if let Some(size) = <Self as alloy_sol_types::SolType>::ENCODED_SIZE {
                    return size;
                }
                let tuple = <UnderlyingRustTuple<
                    '_,
                > as ::core::convert::From<Self>>::from(self.clone());
                <UnderlyingSolTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_encoded_size(&tuple)
            }
            #[inline]
            fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
                <Self as alloy_sol_types::SolStruct>::eip712_hash_struct(self)
            }
            #[inline]
            fn stv_abi_encode_packed_to(
                &self,
                out: &mut alloy_sol_types::private::Vec<u8>,
            ) {
                let tuple = <UnderlyingRustTuple<
                    '_,
                > as ::core::convert::From<Self>>::from(self.clone());
                <UnderlyingSolTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_encode_packed_to(&tuple, out)
            }
            #[inline]
            fn stv_abi_packed_encoded_size(&self) -> usize {
                if let Some(size) = <Self as alloy_sol_types::SolType>::PACKED_ENCODED_SIZE {
                    return size;
                }
                let tuple = <UnderlyingRustTuple<
                    '_,
                > as ::core::convert::From<Self>>::from(self.clone());
                <UnderlyingSolTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_packed_encoded_size(&tuple)
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolType for TicketData {
            type RustType = Self;
            type Token<'a> = <UnderlyingSolTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SOL_NAME: &'static str = <Self as alloy_sol_types::SolStruct>::NAME;
            const ENCODED_SIZE: Option<usize> = <UnderlyingSolTuple<
                '_,
            > as alloy_sol_types::SolType>::ENCODED_SIZE;
            const PACKED_ENCODED_SIZE: Option<usize> = <UnderlyingSolTuple<
                '_,
            > as alloy_sol_types::SolType>::PACKED_ENCODED_SIZE;
            #[inline]
            fn valid_token(token: &Self::Token<'_>) -> bool {
                <UnderlyingSolTuple<'_> as alloy_sol_types::SolType>::valid_token(token)
            }
            #[inline]
            fn detokenize(token: Self::Token<'_>) -> Self::RustType {
                let tuple = <UnderlyingSolTuple<
                    '_,
                > as alloy_sol_types::SolType>::detokenize(token);
                <Self as ::core::convert::From<UnderlyingRustTuple<'_>>>::from(tuple)
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolStruct for TicketData {
            const NAME: &'static str = "TicketData";
            #[inline]
            fn eip712_root_type() -> alloy_sol_types::private::Cow<'static, str> {
                alloy_sol_types::private::Cow::Borrowed(
                    "TicketData(bytes32 channelId,uint96 amount,uint48 ticketIndex,uint32 indexOffset,uint24 epoch,uint56 winProb)",
                )
            }
            #[inline]
            fn eip712_components() -> alloy_sol_types::private::Vec<
                alloy_sol_types::private::Cow<'static, str>,
            > {
                alloy_sol_types::private::Vec::new()
            }
            #[inline]
            fn eip712_encode_type() -> alloy_sol_types::private::Cow<'static, str> {
                <Self as alloy_sol_types::SolStruct>::eip712_root_type()
            }
            #[inline]
            fn eip712_encode_data(&self) -> alloy_sol_types::private::Vec<u8> {
                [
                    <alloy::sol_types::sol_data::FixedBytes<
                        32,
                    > as alloy_sol_types::SolType>::eip712_data_word(&self.channelId)
                        .0,
                    <Balance as alloy_sol_types::SolType>::eip712_data_word(&self.amount)
                        .0,
                    <TicketIndex as alloy_sol_types::SolType>::eip712_data_word(
                            &self.ticketIndex,
                        )
                        .0,
                    <TicketIndexOffset as alloy_sol_types::SolType>::eip712_data_word(
                            &self.indexOffset,
                        )
                        .0,
                    <ChannelEpoch as alloy_sol_types::SolType>::eip712_data_word(
                            &self.epoch,
                        )
                        .0,
                    <WinProb as alloy_sol_types::SolType>::eip712_data_word(
                            &self.winProb,
                        )
                        .0,
                ]
                    .concat()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::EventTopic for TicketData {
            #[inline]
            fn topic_preimage_length(rust: &Self::RustType) -> usize {
                0usize
                    + <alloy::sol_types::sol_data::FixedBytes<
                        32,
                    > as alloy_sol_types::EventTopic>::topic_preimage_length(
                        &rust.channelId,
                    )
                    + <Balance as alloy_sol_types::EventTopic>::topic_preimage_length(
                        &rust.amount,
                    )
                    + <TicketIndex as alloy_sol_types::EventTopic>::topic_preimage_length(
                        &rust.ticketIndex,
                    )
                    + <TicketIndexOffset as alloy_sol_types::EventTopic>::topic_preimage_length(
                        &rust.indexOffset,
                    )
                    + <ChannelEpoch as alloy_sol_types::EventTopic>::topic_preimage_length(
                        &rust.epoch,
                    )
                    + <WinProb as alloy_sol_types::EventTopic>::topic_preimage_length(
                        &rust.winProb,
                    )
            }
            #[inline]
            fn encode_topic_preimage(
                rust: &Self::RustType,
                out: &mut alloy_sol_types::private::Vec<u8>,
            ) {
                out.reserve(
                    <Self as alloy_sol_types::EventTopic>::topic_preimage_length(rust),
                );
                <alloy::sol_types::sol_data::FixedBytes<
                    32,
                > as alloy_sol_types::EventTopic>::encode_topic_preimage(
                    &rust.channelId,
                    out,
                );
                <Balance as alloy_sol_types::EventTopic>::encode_topic_preimage(
                    &rust.amount,
                    out,
                );
                <TicketIndex as alloy_sol_types::EventTopic>::encode_topic_preimage(
                    &rust.ticketIndex,
                    out,
                );
                <TicketIndexOffset as alloy_sol_types::EventTopic>::encode_topic_preimage(
                    &rust.indexOffset,
                    out,
                );
                <ChannelEpoch as alloy_sol_types::EventTopic>::encode_topic_preimage(
                    &rust.epoch,
                    out,
                );
                <WinProb as alloy_sol_types::EventTopic>::encode_topic_preimage(
                    &rust.winProb,
                    out,
                );
            }
            #[inline]
            fn encode_topic(
                rust: &Self::RustType,
            ) -> alloy_sol_types::abi::token::WordToken {
                let mut out = alloy_sol_types::private::Vec::new();
                <Self as alloy_sol_types::EventTopic>::encode_topic_preimage(
                    rust,
                    &mut out,
                );
                alloy_sol_types::abi::token::WordToken(
                    alloy_sol_types::private::keccak256(out),
                )
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
    /**Custom error with signature `BalanceExceedsGlobalPerChannelAllowance()` and selector `0xa4f3bbe4`.
```solidity
error BalanceExceedsGlobalPerChannelAllowance();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct BalanceExceedsGlobalPerChannelAllowance;
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
        impl ::core::convert::From<BalanceExceedsGlobalPerChannelAllowance>
        for UnderlyingRustTuple<'_> {
            fn from(value: BalanceExceedsGlobalPerChannelAllowance) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>>
        for BalanceExceedsGlobalPerChannelAllowance {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for BalanceExceedsGlobalPerChannelAllowance {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "BalanceExceedsGlobalPerChannelAllowance()";
            const SELECTOR: [u8; 4] = [164u8, 243u8, 187u8, 228u8];
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
    /**Custom error with signature `ContractNotResponsible()` and selector `0xacd5a823`.
```solidity
error ContractNotResponsible();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct ContractNotResponsible;
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
        impl ::core::convert::From<ContractNotResponsible> for UnderlyingRustTuple<'_> {
            fn from(value: ContractNotResponsible) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for ContractNotResponsible {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for ContractNotResponsible {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "ContractNotResponsible()";
            const SELECTOR: [u8; 4] = [172u8, 213u8, 168u8, 35u8];
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
    /**Custom error with signature `InsufficientChannelBalance()` and selector `0xb147636c`.
```solidity
error InsufficientChannelBalance();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct InsufficientChannelBalance;
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
        impl ::core::convert::From<InsufficientChannelBalance>
        for UnderlyingRustTuple<'_> {
            fn from(value: InsufficientChannelBalance) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>>
        for InsufficientChannelBalance {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for InsufficientChannelBalance {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "InsufficientChannelBalance()";
            const SELECTOR: [u8; 4] = [177u8, 71u8, 99u8, 108u8];
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
    /**Custom error with signature `InvalidAggregatedTicketInterval()` and selector `0xd0dc3c1e`.
```solidity
error InvalidAggregatedTicketInterval();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct InvalidAggregatedTicketInterval;
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
        impl ::core::convert::From<InvalidAggregatedTicketInterval>
        for UnderlyingRustTuple<'_> {
            fn from(value: InvalidAggregatedTicketInterval) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>>
        for InvalidAggregatedTicketInterval {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for InvalidAggregatedTicketInterval {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "InvalidAggregatedTicketInterval()";
            const SELECTOR: [u8; 4] = [208u8, 220u8, 60u8, 30u8];
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
    /**Custom error with signature `InvalidBalance()` and selector `0xc52e3eff`.
```solidity
error InvalidBalance();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct InvalidBalance;
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
        impl ::core::convert::From<InvalidBalance> for UnderlyingRustTuple<'_> {
            fn from(value: InvalidBalance) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for InvalidBalance {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for InvalidBalance {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "InvalidBalance()";
            const SELECTOR: [u8; 4] = [197u8, 46u8, 62u8, 255u8];
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
    /**Custom error with signature `InvalidCurvePoint()` and selector `0x72454a82`.
```solidity
error InvalidCurvePoint();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct InvalidCurvePoint;
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
        impl ::core::convert::From<InvalidCurvePoint> for UnderlyingRustTuple<'_> {
            fn from(value: InvalidCurvePoint) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for InvalidCurvePoint {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for InvalidCurvePoint {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "InvalidCurvePoint()";
            const SELECTOR: [u8; 4] = [114u8, 69u8, 74u8, 130u8];
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
    /**Custom error with signature `InvalidFieldElement()` and selector `0x3ae4ed6b`.
```solidity
error InvalidFieldElement();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct InvalidFieldElement;
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
        impl ::core::convert::From<InvalidFieldElement> for UnderlyingRustTuple<'_> {
            fn from(value: InvalidFieldElement) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for InvalidFieldElement {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for InvalidFieldElement {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "InvalidFieldElement()";
            const SELECTOR: [u8; 4] = [58u8, 228u8, 237u8, 107u8];
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
    /**Custom error with signature `InvalidNoticePeriod()` and selector `0xf9ee9107`.
```solidity
error InvalidNoticePeriod();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct InvalidNoticePeriod;
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
        impl ::core::convert::From<InvalidNoticePeriod> for UnderlyingRustTuple<'_> {
            fn from(value: InvalidNoticePeriod) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for InvalidNoticePeriod {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for InvalidNoticePeriod {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "InvalidNoticePeriod()";
            const SELECTOR: [u8; 4] = [249u8, 238u8, 145u8, 7u8];
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
    /**Custom error with signature `InvalidPointWitness()` and selector `0xedfdcd98`.
```solidity
error InvalidPointWitness();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct InvalidPointWitness;
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
        impl ::core::convert::From<InvalidPointWitness> for UnderlyingRustTuple<'_> {
            fn from(value: InvalidPointWitness) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for InvalidPointWitness {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for InvalidPointWitness {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "InvalidPointWitness()";
            const SELECTOR: [u8; 4] = [237u8, 253u8, 205u8, 152u8];
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
    /**Custom error with signature `InvalidSafeAddress()` and selector `0x8e9d7c5e`.
```solidity
error InvalidSafeAddress();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct InvalidSafeAddress;
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
        impl ::core::convert::From<InvalidSafeAddress> for UnderlyingRustTuple<'_> {
            fn from(value: InvalidSafeAddress) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for InvalidSafeAddress {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for InvalidSafeAddress {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "InvalidSafeAddress()";
            const SELECTOR: [u8; 4] = [142u8, 157u8, 124u8, 94u8];
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
    /**Custom error with signature `InvalidTicketSignature()` and selector `0xcddd5356`.
```solidity
error InvalidTicketSignature();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct InvalidTicketSignature;
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
        impl ::core::convert::From<InvalidTicketSignature> for UnderlyingRustTuple<'_> {
            fn from(value: InvalidTicketSignature) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for InvalidTicketSignature {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for InvalidTicketSignature {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "InvalidTicketSignature()";
            const SELECTOR: [u8; 4] = [205u8, 221u8, 83u8, 86u8];
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
    /**Custom error with signature `InvalidTokenRecipient()` and selector `0xb9c49108`.
```solidity
error InvalidTokenRecipient();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct InvalidTokenRecipient;
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
        impl ::core::convert::From<InvalidTokenRecipient> for UnderlyingRustTuple<'_> {
            fn from(value: InvalidTokenRecipient) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for InvalidTokenRecipient {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for InvalidTokenRecipient {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "InvalidTokenRecipient()";
            const SELECTOR: [u8; 4] = [185u8, 196u8, 145u8, 8u8];
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
    /**Custom error with signature `InvalidTokensReceivedUsage()` and selector `0x69ee6f28`.
```solidity
error InvalidTokensReceivedUsage();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct InvalidTokensReceivedUsage;
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
        impl ::core::convert::From<InvalidTokensReceivedUsage>
        for UnderlyingRustTuple<'_> {
            fn from(value: InvalidTokensReceivedUsage) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>>
        for InvalidTokensReceivedUsage {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for InvalidTokensReceivedUsage {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "InvalidTokensReceivedUsage()";
            const SELECTOR: [u8; 4] = [105u8, 238u8, 111u8, 40u8];
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
    /**Custom error with signature `InvalidVRFProof()` and selector `0x95fdbdb8`.
```solidity
error InvalidVRFProof();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct InvalidVRFProof;
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
        impl ::core::convert::From<InvalidVRFProof> for UnderlyingRustTuple<'_> {
            fn from(value: InvalidVRFProof) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for InvalidVRFProof {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for InvalidVRFProof {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "InvalidVRFProof()";
            const SELECTOR: [u8; 4] = [149u8, 253u8, 189u8, 184u8];
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
    /**Custom error with signature `MultiSigUninitialized()` and selector `0x454a20c8`.
```solidity
error MultiSigUninitialized();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct MultiSigUninitialized;
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
        impl ::core::convert::From<MultiSigUninitialized> for UnderlyingRustTuple<'_> {
            fn from(value: MultiSigUninitialized) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for MultiSigUninitialized {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for MultiSigUninitialized {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "MultiSigUninitialized()";
            const SELECTOR: [u8; 4] = [69u8, 74u8, 32u8, 200u8];
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
    /**Custom error with signature `NoticePeriodNotDue()` and selector `0x7164032a`.
```solidity
error NoticePeriodNotDue();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct NoticePeriodNotDue;
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
        impl ::core::convert::From<NoticePeriodNotDue> for UnderlyingRustTuple<'_> {
            fn from(value: NoticePeriodNotDue) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for NoticePeriodNotDue {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for NoticePeriodNotDue {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "NoticePeriodNotDue()";
            const SELECTOR: [u8; 4] = [113u8, 100u8, 3u8, 42u8];
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
    /**Custom error with signature `SourceEqualsDestination()` and selector `0x97a3aed2`.
```solidity
error SourceEqualsDestination();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct SourceEqualsDestination;
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
        impl ::core::convert::From<SourceEqualsDestination> for UnderlyingRustTuple<'_> {
            fn from(value: SourceEqualsDestination) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for SourceEqualsDestination {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for SourceEqualsDestination {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "SourceEqualsDestination()";
            const SELECTOR: [u8; 4] = [151u8, 163u8, 174u8, 210u8];
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
    /**Custom error with signature `TicketIsNotAWin()` and selector `0xee835c89`.
```solidity
error TicketIsNotAWin();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct TicketIsNotAWin;
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
        impl ::core::convert::From<TicketIsNotAWin> for UnderlyingRustTuple<'_> {
            fn from(value: TicketIsNotAWin) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for TicketIsNotAWin {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for TicketIsNotAWin {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "TicketIsNotAWin()";
            const SELECTOR: [u8; 4] = [238u8, 131u8, 92u8, 137u8];
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
    /**Custom error with signature `TokenTransferFailed()` and selector `0x045c4b02`.
```solidity
error TokenTransferFailed();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct TokenTransferFailed;
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
        impl ::core::convert::From<TokenTransferFailed> for UnderlyingRustTuple<'_> {
            fn from(value: TokenTransferFailed) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for TokenTransferFailed {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for TokenTransferFailed {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "TokenTransferFailed()";
            const SELECTOR: [u8; 4] = [4u8, 92u8, 75u8, 2u8];
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
    /**Custom error with signature `WrongChannelState(string)` and selector `0x499463c1`.
```solidity
error WrongChannelState(string reason);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct WrongChannelState {
        #[allow(missing_docs)]
        pub reason: alloy::sol_types::private::String,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = (alloy::sol_types::sol_data::String,);
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = (alloy::sol_types::private::String,);
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
        impl ::core::convert::From<WrongChannelState> for UnderlyingRustTuple<'_> {
            fn from(value: WrongChannelState) -> Self {
                (value.reason,)
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for WrongChannelState {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self { reason: tuple.0 }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for WrongChannelState {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "WrongChannelState(string)";
            const SELECTOR: [u8; 4] = [73u8, 148u8, 99u8, 193u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                (
                    <alloy::sol_types::sol_data::String as alloy_sol_types::SolType>::tokenize(
                        &self.reason,
                    ),
                )
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
    /**Custom error with signature `WrongToken()` and selector `0xa0f3feea`.
```solidity
error WrongToken();
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct WrongToken;
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
        impl ::core::convert::From<WrongToken> for UnderlyingRustTuple<'_> {
            fn from(value: WrongToken) -> Self {
                ()
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for WrongToken {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for WrongToken {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "WrongToken()";
            const SELECTOR: [u8; 4] = [160u8, 243u8, 254u8, 234u8];
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
    /**Custom error with signature `ZeroAddress(string)` and selector `0xeac0d389`.
```solidity
error ZeroAddress(string reason);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct ZeroAddress {
        #[allow(missing_docs)]
        pub reason: alloy::sol_types::private::String,
    }
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        #[doc(hidden)]
        type UnderlyingSolTuple<'a> = (alloy::sol_types::sol_data::String,);
        #[doc(hidden)]
        type UnderlyingRustTuple<'a> = (alloy::sol_types::private::String,);
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
        impl ::core::convert::From<ZeroAddress> for UnderlyingRustTuple<'_> {
            fn from(value: ZeroAddress) -> Self {
                (value.reason,)
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        impl ::core::convert::From<UnderlyingRustTuple<'_>> for ZeroAddress {
            fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                Self { reason: tuple.0 }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolError for ZeroAddress {
            type Parameters<'a> = UnderlyingSolTuple<'a>;
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "ZeroAddress(string)";
            const SELECTOR: [u8; 4] = [234u8, 192u8, 211u8, 137u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                (
                    <alloy::sol_types::sol_data::String as alloy_sol_types::SolType>::tokenize(
                        &self.reason,
                    ),
                )
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
    /**Event with signature `ChannelBalanceDecreased(bytes32,uint96)` and selector `0x22e2a422a8860656a3a33cfa1daf771e76798ce5649747957235025de12e0b24`.
```solidity
event ChannelBalanceDecreased(bytes32 indexed channelId, Balance newBalance);
```*/
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    #[derive(Clone)]
    pub struct ChannelBalanceDecreased {
        #[allow(missing_docs)]
        pub channelId: alloy::sol_types::private::FixedBytes<32>,
        #[allow(missing_docs)]
        pub newBalance: <Balance as alloy::sol_types::SolType>::RustType,
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
        impl alloy_sol_types::SolEvent for ChannelBalanceDecreased {
            type DataTuple<'a> = (Balance,);
            type DataToken<'a> = <Self::DataTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type TopicList = (
                alloy_sol_types::sol_data::FixedBytes<32>,
                alloy::sol_types::sol_data::FixedBytes<32>,
            );
            const SIGNATURE: &'static str = "ChannelBalanceDecreased(bytes32,uint96)";
            const SIGNATURE_HASH: alloy_sol_types::private::B256 = alloy_sol_types::private::B256::new([
                34u8, 226u8, 164u8, 34u8, 168u8, 134u8, 6u8, 86u8, 163u8, 163u8, 60u8,
                250u8, 29u8, 175u8, 119u8, 30u8, 118u8, 121u8, 140u8, 229u8, 100u8,
                151u8, 71u8, 149u8, 114u8, 53u8, 2u8, 93u8, 225u8, 46u8, 11u8, 36u8,
            ]);
            const ANONYMOUS: bool = false;
            #[allow(unused_variables)]
            #[inline]
            fn new(
                topics: <Self::TopicList as alloy_sol_types::SolType>::RustType,
                data: <Self::DataTuple<'_> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                Self {
                    channelId: topics.1,
                    newBalance: data.0,
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
                (<Balance as alloy_sol_types::SolType>::tokenize(&self.newBalance),)
            }
            #[inline]
            fn topics(&self) -> <Self::TopicList as alloy_sol_types::SolType>::RustType {
                (Self::SIGNATURE_HASH.into(), self.channelId.clone())
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
                out[1usize] = <alloy::sol_types::sol_data::FixedBytes<
                    32,
                > as alloy_sol_types::EventTopic>::encode_topic(&self.channelId);
                Ok(())
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::private::IntoLogData for ChannelBalanceDecreased {
            fn to_log_data(&self) -> alloy_sol_types::private::LogData {
                From::from(self)
            }
            fn into_log_data(self) -> alloy_sol_types::private::LogData {
                From::from(&self)
            }
        }
        #[automatically_derived]
        impl From<&ChannelBalanceDecreased> for alloy_sol_types::private::LogData {
            #[inline]
            fn from(
                this: &ChannelBalanceDecreased,
            ) -> alloy_sol_types::private::LogData {
                alloy_sol_types::SolEvent::encode_log_data(this)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Event with signature `ChannelBalanceIncreased(bytes32,uint96)` and selector `0x5fa17246d3a5d68d42baa94cde33042180b783a399c02bf63ac2076e0f708738`.
```solidity
event ChannelBalanceIncreased(bytes32 indexed channelId, Balance newBalance);
```*/
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    #[derive(Clone)]
    pub struct ChannelBalanceIncreased {
        #[allow(missing_docs)]
        pub channelId: alloy::sol_types::private::FixedBytes<32>,
        #[allow(missing_docs)]
        pub newBalance: <Balance as alloy::sol_types::SolType>::RustType,
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
        impl alloy_sol_types::SolEvent for ChannelBalanceIncreased {
            type DataTuple<'a> = (Balance,);
            type DataToken<'a> = <Self::DataTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type TopicList = (
                alloy_sol_types::sol_data::FixedBytes<32>,
                alloy::sol_types::sol_data::FixedBytes<32>,
            );
            const SIGNATURE: &'static str = "ChannelBalanceIncreased(bytes32,uint96)";
            const SIGNATURE_HASH: alloy_sol_types::private::B256 = alloy_sol_types::private::B256::new([
                95u8, 161u8, 114u8, 70u8, 211u8, 165u8, 214u8, 141u8, 66u8, 186u8, 169u8,
                76u8, 222u8, 51u8, 4u8, 33u8, 128u8, 183u8, 131u8, 163u8, 153u8, 192u8,
                43u8, 246u8, 58u8, 194u8, 7u8, 110u8, 15u8, 112u8, 135u8, 56u8,
            ]);
            const ANONYMOUS: bool = false;
            #[allow(unused_variables)]
            #[inline]
            fn new(
                topics: <Self::TopicList as alloy_sol_types::SolType>::RustType,
                data: <Self::DataTuple<'_> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                Self {
                    channelId: topics.1,
                    newBalance: data.0,
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
                (<Balance as alloy_sol_types::SolType>::tokenize(&self.newBalance),)
            }
            #[inline]
            fn topics(&self) -> <Self::TopicList as alloy_sol_types::SolType>::RustType {
                (Self::SIGNATURE_HASH.into(), self.channelId.clone())
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
                out[1usize] = <alloy::sol_types::sol_data::FixedBytes<
                    32,
                > as alloy_sol_types::EventTopic>::encode_topic(&self.channelId);
                Ok(())
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::private::IntoLogData for ChannelBalanceIncreased {
            fn to_log_data(&self) -> alloy_sol_types::private::LogData {
                From::from(self)
            }
            fn into_log_data(self) -> alloy_sol_types::private::LogData {
                From::from(&self)
            }
        }
        #[automatically_derived]
        impl From<&ChannelBalanceIncreased> for alloy_sol_types::private::LogData {
            #[inline]
            fn from(
                this: &ChannelBalanceIncreased,
            ) -> alloy_sol_types::private::LogData {
                alloy_sol_types::SolEvent::encode_log_data(this)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Event with signature `ChannelClosed(bytes32)` and selector `0xceeab2eef998c17fe96f30f83fbf3c55fc5047f6e40c55a0cf72d236e9d2ba72`.
```solidity
event ChannelClosed(bytes32 indexed channelId);
```*/
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    #[derive(Clone)]
    pub struct ChannelClosed {
        #[allow(missing_docs)]
        pub channelId: alloy::sol_types::private::FixedBytes<32>,
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
        impl alloy_sol_types::SolEvent for ChannelClosed {
            type DataTuple<'a> = ();
            type DataToken<'a> = <Self::DataTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type TopicList = (
                alloy_sol_types::sol_data::FixedBytes<32>,
                alloy::sol_types::sol_data::FixedBytes<32>,
            );
            const SIGNATURE: &'static str = "ChannelClosed(bytes32)";
            const SIGNATURE_HASH: alloy_sol_types::private::B256 = alloy_sol_types::private::B256::new([
                206u8, 234u8, 178u8, 238u8, 249u8, 152u8, 193u8, 127u8, 233u8, 111u8,
                48u8, 248u8, 63u8, 191u8, 60u8, 85u8, 252u8, 80u8, 71u8, 246u8, 228u8,
                12u8, 85u8, 160u8, 207u8, 114u8, 210u8, 54u8, 233u8, 210u8, 186u8, 114u8,
            ]);
            const ANONYMOUS: bool = false;
            #[allow(unused_variables)]
            #[inline]
            fn new(
                topics: <Self::TopicList as alloy_sol_types::SolType>::RustType,
                data: <Self::DataTuple<'_> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                Self { channelId: topics.1 }
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
                (Self::SIGNATURE_HASH.into(), self.channelId.clone())
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
                out[1usize] = <alloy::sol_types::sol_data::FixedBytes<
                    32,
                > as alloy_sol_types::EventTopic>::encode_topic(&self.channelId);
                Ok(())
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::private::IntoLogData for ChannelClosed {
            fn to_log_data(&self) -> alloy_sol_types::private::LogData {
                From::from(self)
            }
            fn into_log_data(self) -> alloy_sol_types::private::LogData {
                From::from(&self)
            }
        }
        #[automatically_derived]
        impl From<&ChannelClosed> for alloy_sol_types::private::LogData {
            #[inline]
            fn from(this: &ChannelClosed) -> alloy_sol_types::private::LogData {
                alloy_sol_types::SolEvent::encode_log_data(this)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Event with signature `ChannelOpened(address,address)` and selector `0xdd90f938230335e59dc925c57ecb0e27a28c2d87356e31f00cd5554abd6c1b2d`.
```solidity
event ChannelOpened(address indexed source, address indexed destination);
```*/
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    #[derive(Clone)]
    pub struct ChannelOpened {
        #[allow(missing_docs)]
        pub source: alloy::sol_types::private::Address,
        #[allow(missing_docs)]
        pub destination: alloy::sol_types::private::Address,
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
        impl alloy_sol_types::SolEvent for ChannelOpened {
            type DataTuple<'a> = ();
            type DataToken<'a> = <Self::DataTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type TopicList = (
                alloy_sol_types::sol_data::FixedBytes<32>,
                alloy::sol_types::sol_data::Address,
                alloy::sol_types::sol_data::Address,
            );
            const SIGNATURE: &'static str = "ChannelOpened(address,address)";
            const SIGNATURE_HASH: alloy_sol_types::private::B256 = alloy_sol_types::private::B256::new([
                221u8, 144u8, 249u8, 56u8, 35u8, 3u8, 53u8, 229u8, 157u8, 201u8, 37u8,
                197u8, 126u8, 203u8, 14u8, 39u8, 162u8, 140u8, 45u8, 135u8, 53u8, 110u8,
                49u8, 240u8, 12u8, 213u8, 85u8, 74u8, 189u8, 108u8, 27u8, 45u8,
            ]);
            const ANONYMOUS: bool = false;
            #[allow(unused_variables)]
            #[inline]
            fn new(
                topics: <Self::TopicList as alloy_sol_types::SolType>::RustType,
                data: <Self::DataTuple<'_> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                Self {
                    source: topics.1,
                    destination: topics.2,
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
                    self.source.clone(),
                    self.destination.clone(),
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
                    &self.source,
                );
                out[2usize] = <alloy::sol_types::sol_data::Address as alloy_sol_types::EventTopic>::encode_topic(
                    &self.destination,
                );
                Ok(())
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::private::IntoLogData for ChannelOpened {
            fn to_log_data(&self) -> alloy_sol_types::private::LogData {
                From::from(self)
            }
            fn into_log_data(self) -> alloy_sol_types::private::LogData {
                From::from(&self)
            }
        }
        #[automatically_derived]
        impl From<&ChannelOpened> for alloy_sol_types::private::LogData {
            #[inline]
            fn from(this: &ChannelOpened) -> alloy_sol_types::private::LogData {
                alloy_sol_types::SolEvent::encode_log_data(this)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Event with signature `DomainSeparatorUpdated(bytes32)` and selector `0x771f5240ae5fd8a7640d3fb82fa70aab2fb1dbf35f2ef464f8509946717664c5`.
```solidity
event DomainSeparatorUpdated(bytes32 indexed domainSeparator);
```*/
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    #[derive(Clone)]
    pub struct DomainSeparatorUpdated {
        #[allow(missing_docs)]
        pub domainSeparator: alloy::sol_types::private::FixedBytes<32>,
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
        impl alloy_sol_types::SolEvent for DomainSeparatorUpdated {
            type DataTuple<'a> = ();
            type DataToken<'a> = <Self::DataTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type TopicList = (
                alloy_sol_types::sol_data::FixedBytes<32>,
                alloy::sol_types::sol_data::FixedBytes<32>,
            );
            const SIGNATURE: &'static str = "DomainSeparatorUpdated(bytes32)";
            const SIGNATURE_HASH: alloy_sol_types::private::B256 = alloy_sol_types::private::B256::new([
                119u8, 31u8, 82u8, 64u8, 174u8, 95u8, 216u8, 167u8, 100u8, 13u8, 63u8,
                184u8, 47u8, 167u8, 10u8, 171u8, 47u8, 177u8, 219u8, 243u8, 95u8, 46u8,
                244u8, 100u8, 248u8, 80u8, 153u8, 70u8, 113u8, 118u8, 100u8, 197u8,
            ]);
            const ANONYMOUS: bool = false;
            #[allow(unused_variables)]
            #[inline]
            fn new(
                topics: <Self::TopicList as alloy_sol_types::SolType>::RustType,
                data: <Self::DataTuple<'_> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                Self { domainSeparator: topics.1 }
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
                (Self::SIGNATURE_HASH.into(), self.domainSeparator.clone())
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
                out[1usize] = <alloy::sol_types::sol_data::FixedBytes<
                    32,
                > as alloy_sol_types::EventTopic>::encode_topic(&self.domainSeparator);
                Ok(())
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::private::IntoLogData for DomainSeparatorUpdated {
            fn to_log_data(&self) -> alloy_sol_types::private::LogData {
                From::from(self)
            }
            fn into_log_data(self) -> alloy_sol_types::private::LogData {
                From::from(&self)
            }
        }
        #[automatically_derived]
        impl From<&DomainSeparatorUpdated> for alloy_sol_types::private::LogData {
            #[inline]
            fn from(this: &DomainSeparatorUpdated) -> alloy_sol_types::private::LogData {
                alloy_sol_types::SolEvent::encode_log_data(this)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Event with signature `LedgerDomainSeparatorUpdated(bytes32)` and selector `0xa43fad83920fd09445855e854e73c9c532e17402c9ceb09993a2392843a5bdb9`.
```solidity
event LedgerDomainSeparatorUpdated(bytes32 indexed ledgerDomainSeparator);
```*/
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    #[derive(Clone)]
    pub struct LedgerDomainSeparatorUpdated {
        #[allow(missing_docs)]
        pub ledgerDomainSeparator: alloy::sol_types::private::FixedBytes<32>,
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
        impl alloy_sol_types::SolEvent for LedgerDomainSeparatorUpdated {
            type DataTuple<'a> = ();
            type DataToken<'a> = <Self::DataTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type TopicList = (
                alloy_sol_types::sol_data::FixedBytes<32>,
                alloy::sol_types::sol_data::FixedBytes<32>,
            );
            const SIGNATURE: &'static str = "LedgerDomainSeparatorUpdated(bytes32)";
            const SIGNATURE_HASH: alloy_sol_types::private::B256 = alloy_sol_types::private::B256::new([
                164u8, 63u8, 173u8, 131u8, 146u8, 15u8, 208u8, 148u8, 69u8, 133u8, 94u8,
                133u8, 78u8, 115u8, 201u8, 197u8, 50u8, 225u8, 116u8, 2u8, 201u8, 206u8,
                176u8, 153u8, 147u8, 162u8, 57u8, 40u8, 67u8, 165u8, 189u8, 185u8,
            ]);
            const ANONYMOUS: bool = false;
            #[allow(unused_variables)]
            #[inline]
            fn new(
                topics: <Self::TopicList as alloy_sol_types::SolType>::RustType,
                data: <Self::DataTuple<'_> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                Self {
                    ledgerDomainSeparator: topics.1,
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
                (Self::SIGNATURE_HASH.into(), self.ledgerDomainSeparator.clone())
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
                out[1usize] = <alloy::sol_types::sol_data::FixedBytes<
                    32,
                > as alloy_sol_types::EventTopic>::encode_topic(
                    &self.ledgerDomainSeparator,
                );
                Ok(())
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::private::IntoLogData for LedgerDomainSeparatorUpdated {
            fn to_log_data(&self) -> alloy_sol_types::private::LogData {
                From::from(self)
            }
            fn into_log_data(self) -> alloy_sol_types::private::LogData {
                From::from(&self)
            }
        }
        #[automatically_derived]
        impl From<&LedgerDomainSeparatorUpdated> for alloy_sol_types::private::LogData {
            #[inline]
            fn from(
                this: &LedgerDomainSeparatorUpdated,
            ) -> alloy_sol_types::private::LogData {
                alloy_sol_types::SolEvent::encode_log_data(this)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Event with signature `OutgoingChannelClosureInitiated(bytes32,uint32)` and selector `0x07b5c950597fc3bed92e2ad37fa84f701655acb372982e486f5fad3607f04a5c`.
```solidity
event OutgoingChannelClosureInitiated(bytes32 indexed channelId, Timestamp closureTime);
```*/
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    #[derive(Clone)]
    pub struct OutgoingChannelClosureInitiated {
        #[allow(missing_docs)]
        pub channelId: alloy::sol_types::private::FixedBytes<32>,
        #[allow(missing_docs)]
        pub closureTime: <Timestamp as alloy::sol_types::SolType>::RustType,
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
        impl alloy_sol_types::SolEvent for OutgoingChannelClosureInitiated {
            type DataTuple<'a> = (Timestamp,);
            type DataToken<'a> = <Self::DataTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type TopicList = (
                alloy_sol_types::sol_data::FixedBytes<32>,
                alloy::sol_types::sol_data::FixedBytes<32>,
            );
            const SIGNATURE: &'static str = "OutgoingChannelClosureInitiated(bytes32,uint32)";
            const SIGNATURE_HASH: alloy_sol_types::private::B256 = alloy_sol_types::private::B256::new([
                7u8, 181u8, 201u8, 80u8, 89u8, 127u8, 195u8, 190u8, 217u8, 46u8, 42u8,
                211u8, 127u8, 168u8, 79u8, 112u8, 22u8, 85u8, 172u8, 179u8, 114u8, 152u8,
                46u8, 72u8, 111u8, 95u8, 173u8, 54u8, 7u8, 240u8, 74u8, 92u8,
            ]);
            const ANONYMOUS: bool = false;
            #[allow(unused_variables)]
            #[inline]
            fn new(
                topics: <Self::TopicList as alloy_sol_types::SolType>::RustType,
                data: <Self::DataTuple<'_> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                Self {
                    channelId: topics.1,
                    closureTime: data.0,
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
                (<Timestamp as alloy_sol_types::SolType>::tokenize(&self.closureTime),)
            }
            #[inline]
            fn topics(&self) -> <Self::TopicList as alloy_sol_types::SolType>::RustType {
                (Self::SIGNATURE_HASH.into(), self.channelId.clone())
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
                out[1usize] = <alloy::sol_types::sol_data::FixedBytes<
                    32,
                > as alloy_sol_types::EventTopic>::encode_topic(&self.channelId);
                Ok(())
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::private::IntoLogData for OutgoingChannelClosureInitiated {
            fn to_log_data(&self) -> alloy_sol_types::private::LogData {
                From::from(self)
            }
            fn into_log_data(self) -> alloy_sol_types::private::LogData {
                From::from(&self)
            }
        }
        #[automatically_derived]
        impl From<&OutgoingChannelClosureInitiated>
        for alloy_sol_types::private::LogData {
            #[inline]
            fn from(
                this: &OutgoingChannelClosureInitiated,
            ) -> alloy_sol_types::private::LogData {
                alloy_sol_types::SolEvent::encode_log_data(this)
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Event with signature `TicketRedeemed(bytes32,uint48)` and selector `0x7165e2ebc7ce35cc98cb7666f9945b3617f3f36326b76d18937ba5fecf18739a`.
```solidity
event TicketRedeemed(bytes32 indexed channelId, TicketIndex newTicketIndex);
```*/
    #[allow(
        non_camel_case_types,
        non_snake_case,
        clippy::pub_underscore_fields,
        clippy::style
    )]
    #[derive(Clone)]
    pub struct TicketRedeemed {
        #[allow(missing_docs)]
        pub channelId: alloy::sol_types::private::FixedBytes<32>,
        #[allow(missing_docs)]
        pub newTicketIndex: <TicketIndex as alloy::sol_types::SolType>::RustType,
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
        impl alloy_sol_types::SolEvent for TicketRedeemed {
            type DataTuple<'a> = (TicketIndex,);
            type DataToken<'a> = <Self::DataTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type TopicList = (
                alloy_sol_types::sol_data::FixedBytes<32>,
                alloy::sol_types::sol_data::FixedBytes<32>,
            );
            const SIGNATURE: &'static str = "TicketRedeemed(bytes32,uint48)";
            const SIGNATURE_HASH: alloy_sol_types::private::B256 = alloy_sol_types::private::B256::new([
                113u8, 101u8, 226u8, 235u8, 199u8, 206u8, 53u8, 204u8, 152u8, 203u8,
                118u8, 102u8, 249u8, 148u8, 91u8, 54u8, 23u8, 243u8, 243u8, 99u8, 38u8,
                183u8, 109u8, 24u8, 147u8, 123u8, 165u8, 254u8, 207u8, 24u8, 115u8, 154u8,
            ]);
            const ANONYMOUS: bool = false;
            #[allow(unused_variables)]
            #[inline]
            fn new(
                topics: <Self::TopicList as alloy_sol_types::SolType>::RustType,
                data: <Self::DataTuple<'_> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                Self {
                    channelId: topics.1,
                    newTicketIndex: data.0,
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
                    <TicketIndex as alloy_sol_types::SolType>::tokenize(
                        &self.newTicketIndex,
                    ),
                )
            }
            #[inline]
            fn topics(&self) -> <Self::TopicList as alloy_sol_types::SolType>::RustType {
                (Self::SIGNATURE_HASH.into(), self.channelId.clone())
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
                out[1usize] = <alloy::sol_types::sol_data::FixedBytes<
                    32,
                > as alloy_sol_types::EventTopic>::encode_topic(&self.channelId);
                Ok(())
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::private::IntoLogData for TicketRedeemed {
            fn to_log_data(&self) -> alloy_sol_types::private::LogData {
                From::from(self)
            }
            fn into_log_data(self) -> alloy_sol_types::private::LogData {
                From::from(&self)
            }
        }
        #[automatically_derived]
        impl From<&TicketRedeemed> for alloy_sol_types::private::LogData {
            #[inline]
            fn from(this: &TicketRedeemed) -> alloy_sol_types::private::LogData {
                alloy_sol_types::SolEvent::encode_log_data(this)
            }
        }
    };
    /**Constructor`.
```solidity
constructor(address _token, Timestamp _noticePeriodChannelClosure, address _safeRegistry);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct constructorCall {
        #[allow(missing_docs)]
        pub _token: alloy::sol_types::private::Address,
        #[allow(missing_docs)]
        pub _noticePeriodChannelClosure: <Timestamp as alloy::sol_types::SolType>::RustType,
        #[allow(missing_docs)]
        pub _safeRegistry: alloy::sol_types::private::Address,
    }
    const _: () = {
        use alloy::sol_types as alloy_sol_types;
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (
                alloy::sol_types::sol_data::Address,
                Timestamp,
                alloy::sol_types::sol_data::Address,
            );
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                alloy::sol_types::private::Address,
                <Timestamp as alloy::sol_types::SolType>::RustType,
                alloy::sol_types::private::Address,
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
            impl ::core::convert::From<constructorCall> for UnderlyingRustTuple<'_> {
                fn from(value: constructorCall) -> Self {
                    (
                        value._token,
                        value._noticePeriodChannelClosure,
                        value._safeRegistry,
                    )
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for constructorCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {
                        _token: tuple.0,
                        _noticePeriodChannelClosure: tuple.1,
                        _safeRegistry: tuple.2,
                    }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolConstructor for constructorCall {
            type Parameters<'a> = (
                alloy::sol_types::sol_data::Address,
                Timestamp,
                alloy::sol_types::sol_data::Address,
            );
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
                (
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        &self._token,
                    ),
                    <Timestamp as alloy_sol_types::SolType>::tokenize(
                        &self._noticePeriodChannelClosure,
                    ),
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        &self._safeRegistry,
                    ),
                )
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `ERC777_HOOK_FUND_CHANNEL_MULTI_SIZE()` and selector `0x78d8016d`.
```solidity
function ERC777_HOOK_FUND_CHANNEL_MULTI_SIZE() external view returns (uint256);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct ERC777_HOOK_FUND_CHANNEL_MULTI_SIZECall;
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`ERC777_HOOK_FUND_CHANNEL_MULTI_SIZE()`](ERC777_HOOK_FUND_CHANNEL_MULTI_SIZECall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct ERC777_HOOK_FUND_CHANNEL_MULTI_SIZEReturn {
        #[allow(missing_docs)]
        pub _0: alloy::sol_types::private::primitives::aliases::U256,
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
            impl ::core::convert::From<ERC777_HOOK_FUND_CHANNEL_MULTI_SIZECall>
            for UnderlyingRustTuple<'_> {
                fn from(value: ERC777_HOOK_FUND_CHANNEL_MULTI_SIZECall) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for ERC777_HOOK_FUND_CHANNEL_MULTI_SIZECall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (alloy::sol_types::sol_data::Uint<256>,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
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
            impl ::core::convert::From<ERC777_HOOK_FUND_CHANNEL_MULTI_SIZEReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: ERC777_HOOK_FUND_CHANNEL_MULTI_SIZEReturn) -> Self {
                    (value._0,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for ERC777_HOOK_FUND_CHANNEL_MULTI_SIZEReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { _0: tuple.0 }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for ERC777_HOOK_FUND_CHANNEL_MULTI_SIZECall {
            type Parameters<'a> = ();
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = alloy::sol_types::private::primitives::aliases::U256;
            type ReturnTuple<'a> = (alloy::sol_types::sol_data::Uint<256>,);
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "ERC777_HOOK_FUND_CHANNEL_MULTI_SIZE()";
            const SELECTOR: [u8; 4] = [120u8, 216u8, 1u8, 109u8];
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
                    <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::SolType>::tokenize(ret),
                )
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(|r| {
                        let r: ERC777_HOOK_FUND_CHANNEL_MULTI_SIZEReturn = r.into();
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
                        let r: ERC777_HOOK_FUND_CHANNEL_MULTI_SIZEReturn = r.into();
                        r._0
                    })
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `ERC777_HOOK_FUND_CHANNEL_SIZE()` and selector `0x44dae6f8`.
```solidity
function ERC777_HOOK_FUND_CHANNEL_SIZE() external view returns (uint256);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct ERC777_HOOK_FUND_CHANNEL_SIZECall;
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`ERC777_HOOK_FUND_CHANNEL_SIZE()`](ERC777_HOOK_FUND_CHANNEL_SIZECall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct ERC777_HOOK_FUND_CHANNEL_SIZEReturn {
        #[allow(missing_docs)]
        pub _0: alloy::sol_types::private::primitives::aliases::U256,
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
            impl ::core::convert::From<ERC777_HOOK_FUND_CHANNEL_SIZECall>
            for UnderlyingRustTuple<'_> {
                fn from(value: ERC777_HOOK_FUND_CHANNEL_SIZECall) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for ERC777_HOOK_FUND_CHANNEL_SIZECall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (alloy::sol_types::sol_data::Uint<256>,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
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
            impl ::core::convert::From<ERC777_HOOK_FUND_CHANNEL_SIZEReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: ERC777_HOOK_FUND_CHANNEL_SIZEReturn) -> Self {
                    (value._0,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for ERC777_HOOK_FUND_CHANNEL_SIZEReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { _0: tuple.0 }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for ERC777_HOOK_FUND_CHANNEL_SIZECall {
            type Parameters<'a> = ();
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = alloy::sol_types::private::primitives::aliases::U256;
            type ReturnTuple<'a> = (alloy::sol_types::sol_data::Uint<256>,);
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "ERC777_HOOK_FUND_CHANNEL_SIZE()";
            const SELECTOR: [u8; 4] = [68u8, 218u8, 230u8, 248u8];
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
                    <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::SolType>::tokenize(ret),
                )
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(|r| {
                        let r: ERC777_HOOK_FUND_CHANNEL_SIZEReturn = r.into();
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
                        let r: ERC777_HOOK_FUND_CHANNEL_SIZEReturn = r.into();
                        r._0
                    })
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `LEDGER_VERSION()` and selector `0xddad1902`.
```solidity
function LEDGER_VERSION() external view returns (string memory);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct LEDGER_VERSIONCall;
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`LEDGER_VERSION()`](LEDGER_VERSIONCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct LEDGER_VERSIONReturn {
        #[allow(missing_docs)]
        pub _0: alloy::sol_types::private::String,
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
            impl ::core::convert::From<LEDGER_VERSIONCall> for UnderlyingRustTuple<'_> {
                fn from(value: LEDGER_VERSIONCall) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for LEDGER_VERSIONCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (alloy::sol_types::sol_data::String,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (alloy::sol_types::private::String,);
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
            impl ::core::convert::From<LEDGER_VERSIONReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: LEDGER_VERSIONReturn) -> Self {
                    (value._0,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for LEDGER_VERSIONReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { _0: tuple.0 }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for LEDGER_VERSIONCall {
            type Parameters<'a> = ();
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = alloy::sol_types::private::String;
            type ReturnTuple<'a> = (alloy::sol_types::sol_data::String,);
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "LEDGER_VERSION()";
            const SELECTOR: [u8; 4] = [221u8, 173u8, 25u8, 2u8];
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
                    <alloy::sol_types::sol_data::String as alloy_sol_types::SolType>::tokenize(
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
                        let r: LEDGER_VERSIONReturn = r.into();
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
                        let r: LEDGER_VERSIONReturn = r.into();
                        r._0
                    })
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `MAX_USED_BALANCE()` and selector `0x5d2f07c5`.
```solidity
function MAX_USED_BALANCE() external view returns (Balance);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct MAX_USED_BALANCECall;
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`MAX_USED_BALANCE()`](MAX_USED_BALANCECall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct MAX_USED_BALANCEReturn {
        #[allow(missing_docs)]
        pub _0: <Balance as alloy::sol_types::SolType>::RustType,
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
            impl ::core::convert::From<MAX_USED_BALANCECall>
            for UnderlyingRustTuple<'_> {
                fn from(value: MAX_USED_BALANCECall) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for MAX_USED_BALANCECall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (Balance,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                <Balance as alloy::sol_types::SolType>::RustType,
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
            impl ::core::convert::From<MAX_USED_BALANCEReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: MAX_USED_BALANCEReturn) -> Self {
                    (value._0,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for MAX_USED_BALANCEReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { _0: tuple.0 }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for MAX_USED_BALANCECall {
            type Parameters<'a> = ();
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = <Balance as alloy::sol_types::SolType>::RustType;
            type ReturnTuple<'a> = (Balance,);
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "MAX_USED_BALANCE()";
            const SELECTOR: [u8; 4] = [93u8, 47u8, 7u8, 197u8];
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
                (<Balance as alloy_sol_types::SolType>::tokenize(ret),)
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(|r| {
                        let r: MAX_USED_BALANCEReturn = r.into();
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
                        let r: MAX_USED_BALANCEReturn = r.into();
                        r._0
                    })
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `MIN_USED_BALANCE()` and selector `0x29392e32`.
```solidity
function MIN_USED_BALANCE() external view returns (Balance);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct MIN_USED_BALANCECall;
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`MIN_USED_BALANCE()`](MIN_USED_BALANCECall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct MIN_USED_BALANCEReturn {
        #[allow(missing_docs)]
        pub _0: <Balance as alloy::sol_types::SolType>::RustType,
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
            impl ::core::convert::From<MIN_USED_BALANCECall>
            for UnderlyingRustTuple<'_> {
                fn from(value: MIN_USED_BALANCECall) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for MIN_USED_BALANCECall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (Balance,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                <Balance as alloy::sol_types::SolType>::RustType,
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
            impl ::core::convert::From<MIN_USED_BALANCEReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: MIN_USED_BALANCEReturn) -> Self {
                    (value._0,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for MIN_USED_BALANCEReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { _0: tuple.0 }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for MIN_USED_BALANCECall {
            type Parameters<'a> = ();
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = <Balance as alloy::sol_types::SolType>::RustType;
            type ReturnTuple<'a> = (Balance,);
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "MIN_USED_BALANCE()";
            const SELECTOR: [u8; 4] = [41u8, 57u8, 46u8, 50u8];
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
                (<Balance as alloy_sol_types::SolType>::tokenize(ret),)
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(|r| {
                        let r: MIN_USED_BALANCEReturn = r.into();
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
                        let r: MIN_USED_BALANCEReturn = r.into();
                        r._0
                    })
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `TOKENS_RECIPIENT_INTERFACE_HASH()` and selector `0x72581cc0`.
```solidity
function TOKENS_RECIPIENT_INTERFACE_HASH() external view returns (bytes32);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct TOKENS_RECIPIENT_INTERFACE_HASHCall;
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`TOKENS_RECIPIENT_INTERFACE_HASH()`](TOKENS_RECIPIENT_INTERFACE_HASHCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct TOKENS_RECIPIENT_INTERFACE_HASHReturn {
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
            impl ::core::convert::From<TOKENS_RECIPIENT_INTERFACE_HASHCall>
            for UnderlyingRustTuple<'_> {
                fn from(value: TOKENS_RECIPIENT_INTERFACE_HASHCall) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for TOKENS_RECIPIENT_INTERFACE_HASHCall {
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
            impl ::core::convert::From<TOKENS_RECIPIENT_INTERFACE_HASHReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: TOKENS_RECIPIENT_INTERFACE_HASHReturn) -> Self {
                    (value._0,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for TOKENS_RECIPIENT_INTERFACE_HASHReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { _0: tuple.0 }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for TOKENS_RECIPIENT_INTERFACE_HASHCall {
            type Parameters<'a> = ();
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = alloy::sol_types::private::FixedBytes<32>;
            type ReturnTuple<'a> = (alloy::sol_types::sol_data::FixedBytes<32>,);
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "TOKENS_RECIPIENT_INTERFACE_HASH()";
            const SELECTOR: [u8; 4] = [114u8, 88u8, 28u8, 192u8];
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
                        let r: TOKENS_RECIPIENT_INTERFACE_HASHReturn = r.into();
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
                        let r: TOKENS_RECIPIENT_INTERFACE_HASHReturn = r.into();
                        r._0
                    })
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `VERSION()` and selector `0xffa1ad74`.
```solidity
function VERSION() external view returns (string memory);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct VERSIONCall;
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`VERSION()`](VERSIONCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct VERSIONReturn {
        #[allow(missing_docs)]
        pub _0: alloy::sol_types::private::String,
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
            impl ::core::convert::From<VERSIONCall> for UnderlyingRustTuple<'_> {
                fn from(value: VERSIONCall) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for VERSIONCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (alloy::sol_types::sol_data::String,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (alloy::sol_types::private::String,);
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
            impl ::core::convert::From<VERSIONReturn> for UnderlyingRustTuple<'_> {
                fn from(value: VERSIONReturn) -> Self {
                    (value._0,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for VERSIONReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { _0: tuple.0 }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for VERSIONCall {
            type Parameters<'a> = ();
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = alloy::sol_types::private::String;
            type ReturnTuple<'a> = (alloy::sol_types::sol_data::String,);
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "VERSION()";
            const SELECTOR: [u8; 4] = [255u8, 161u8, 173u8, 116u8];
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
                    <alloy::sol_types::sol_data::String as alloy_sol_types::SolType>::tokenize(
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
                        let r: VERSIONReturn = r.into();
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
                        let r: VERSIONReturn = r.into();
                        r._0
                    })
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `_currentBlockTimestamp()` and selector `0xb920deed`.
```solidity
function _currentBlockTimestamp() external view returns (Timestamp);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct _currentBlockTimestampCall;
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`_currentBlockTimestamp()`](_currentBlockTimestampCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct _currentBlockTimestampReturn {
        #[allow(missing_docs)]
        pub _0: <Timestamp as alloy::sol_types::SolType>::RustType,
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
            impl ::core::convert::From<_currentBlockTimestampCall>
            for UnderlyingRustTuple<'_> {
                fn from(value: _currentBlockTimestampCall) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for _currentBlockTimestampCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (Timestamp,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                <Timestamp as alloy::sol_types::SolType>::RustType,
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
            impl ::core::convert::From<_currentBlockTimestampReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: _currentBlockTimestampReturn) -> Self {
                    (value._0,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for _currentBlockTimestampReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { _0: tuple.0 }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for _currentBlockTimestampCall {
            type Parameters<'a> = ();
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = <Timestamp as alloy::sol_types::SolType>::RustType;
            type ReturnTuple<'a> = (Timestamp,);
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "_currentBlockTimestamp()";
            const SELECTOR: [u8; 4] = [185u8, 32u8, 222u8, 237u8];
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
                (<Timestamp as alloy_sol_types::SolType>::tokenize(ret),)
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(|r| {
                        let r: _currentBlockTimestampReturn = r.into();
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
                        let r: _currentBlockTimestampReturn = r.into();
                        r._0
                    })
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `_getChannelId(address,address)` and selector `0xbe9babdc`.
```solidity
function _getChannelId(address source, address destination) external pure returns (bytes32);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct _getChannelIdCall {
        #[allow(missing_docs)]
        pub source: alloy::sol_types::private::Address,
        #[allow(missing_docs)]
        pub destination: alloy::sol_types::private::Address,
    }
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`_getChannelId(address,address)`](_getChannelIdCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct _getChannelIdReturn {
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
            type UnderlyingSolTuple<'a> = (
                alloy::sol_types::sol_data::Address,
                alloy::sol_types::sol_data::Address,
            );
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                alloy::sol_types::private::Address,
                alloy::sol_types::private::Address,
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
            impl ::core::convert::From<_getChannelIdCall> for UnderlyingRustTuple<'_> {
                fn from(value: _getChannelIdCall) -> Self {
                    (value.source, value.destination)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for _getChannelIdCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {
                        source: tuple.0,
                        destination: tuple.1,
                    }
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
            impl ::core::convert::From<_getChannelIdReturn> for UnderlyingRustTuple<'_> {
                fn from(value: _getChannelIdReturn) -> Self {
                    (value._0,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for _getChannelIdReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { _0: tuple.0 }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for _getChannelIdCall {
            type Parameters<'a> = (
                alloy::sol_types::sol_data::Address,
                alloy::sol_types::sol_data::Address,
            );
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = alloy::sol_types::private::FixedBytes<32>;
            type ReturnTuple<'a> = (alloy::sol_types::sol_data::FixedBytes<32>,);
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "_getChannelId(address,address)";
            const SELECTOR: [u8; 4] = [190u8, 155u8, 171u8, 220u8];
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
                        &self.source,
                    ),
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        &self.destination,
                    ),
                )
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
                        let r: _getChannelIdReturn = r.into();
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
                        let r: _getChannelIdReturn = r.into();
                        r._0
                    })
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `_getTicketHash(((bytes32,uint96,uint48,uint32,uint24,uint56),(bytes32,bytes32),uint256))` and selector `0x24086cc2`.
```solidity
function _getTicketHash(RedeemableTicket memory redeemable) external view returns (bytes32);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct _getTicketHashCall {
        #[allow(missing_docs)]
        pub redeemable: <RedeemableTicket as alloy::sol_types::SolType>::RustType,
    }
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`_getTicketHash(((bytes32,uint96,uint48,uint32,uint24,uint56),(bytes32,bytes32),uint256))`](_getTicketHashCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct _getTicketHashReturn {
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
            type UnderlyingSolTuple<'a> = (RedeemableTicket,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                <RedeemableTicket as alloy::sol_types::SolType>::RustType,
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
            impl ::core::convert::From<_getTicketHashCall> for UnderlyingRustTuple<'_> {
                fn from(value: _getTicketHashCall) -> Self {
                    (value.redeemable,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for _getTicketHashCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { redeemable: tuple.0 }
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
            impl ::core::convert::From<_getTicketHashReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: _getTicketHashReturn) -> Self {
                    (value._0,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for _getTicketHashReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { _0: tuple.0 }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for _getTicketHashCall {
            type Parameters<'a> = (RedeemableTicket,);
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = alloy::sol_types::private::FixedBytes<32>;
            type ReturnTuple<'a> = (alloy::sol_types::sol_data::FixedBytes<32>,);
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "_getTicketHash(((bytes32,uint96,uint48,uint32,uint24,uint56),(bytes32,bytes32),uint256))";
            const SELECTOR: [u8; 4] = [36u8, 8u8, 108u8, 194u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                (
                    <RedeemableTicket as alloy_sol_types::SolType>::tokenize(
                        &self.redeemable,
                    ),
                )
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
                        let r: _getTicketHashReturn = r.into();
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
                        let r: _getTicketHashReturn = r.into();
                        r._0
                    })
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `_isWinningTicket(bytes32,((bytes32,uint96,uint48,uint32,uint24,uint56),(bytes32,bytes32),uint256),(uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256))` and selector `0x8c3710c9`.
```solidity
function _isWinningTicket(bytes32 ticketHash, RedeemableTicket memory redeemable, HoprCrypto.VRFParameters memory params) external pure returns (bool);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct _isWinningTicketCall {
        #[allow(missing_docs)]
        pub ticketHash: alloy::sol_types::private::FixedBytes<32>,
        #[allow(missing_docs)]
        pub redeemable: <RedeemableTicket as alloy::sol_types::SolType>::RustType,
        #[allow(missing_docs)]
        pub params: <HoprCrypto::VRFParameters as alloy::sol_types::SolType>::RustType,
    }
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`_isWinningTicket(bytes32,((bytes32,uint96,uint48,uint32,uint24,uint56),(bytes32,bytes32),uint256),(uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256))`](_isWinningTicketCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct _isWinningTicketReturn {
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
            type UnderlyingSolTuple<'a> = (
                alloy::sol_types::sol_data::FixedBytes<32>,
                RedeemableTicket,
                HoprCrypto::VRFParameters,
            );
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                alloy::sol_types::private::FixedBytes<32>,
                <RedeemableTicket as alloy::sol_types::SolType>::RustType,
                <HoprCrypto::VRFParameters as alloy::sol_types::SolType>::RustType,
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
            impl ::core::convert::From<_isWinningTicketCall>
            for UnderlyingRustTuple<'_> {
                fn from(value: _isWinningTicketCall) -> Self {
                    (value.ticketHash, value.redeemable, value.params)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for _isWinningTicketCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {
                        ticketHash: tuple.0,
                        redeemable: tuple.1,
                        params: tuple.2,
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
            impl ::core::convert::From<_isWinningTicketReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: _isWinningTicketReturn) -> Self {
                    (value._0,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for _isWinningTicketReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { _0: tuple.0 }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for _isWinningTicketCall {
            type Parameters<'a> = (
                alloy::sol_types::sol_data::FixedBytes<32>,
                RedeemableTicket,
                HoprCrypto::VRFParameters,
            );
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = bool;
            type ReturnTuple<'a> = (alloy::sol_types::sol_data::Bool,);
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "_isWinningTicket(bytes32,((bytes32,uint96,uint48,uint32,uint24,uint56),(bytes32,bytes32),uint256),(uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256))";
            const SELECTOR: [u8; 4] = [140u8, 55u8, 16u8, 201u8];
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
                    > as alloy_sol_types::SolType>::tokenize(&self.ticketHash),
                    <RedeemableTicket as alloy_sol_types::SolType>::tokenize(
                        &self.redeemable,
                    ),
                    <HoprCrypto::VRFParameters as alloy_sol_types::SolType>::tokenize(
                        &self.params,
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
                        let r: _isWinningTicketReturn = r.into();
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
                        let r: _isWinningTicketReturn = r.into();
                        r._0
                    })
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `canImplementInterfaceForAddress(bytes32,address)` and selector `0x249cb3fa`.
```solidity
function canImplementInterfaceForAddress(bytes32 interfaceHash, address account) external view returns (bytes32);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct canImplementInterfaceForAddressCall {
        #[allow(missing_docs)]
        pub interfaceHash: alloy::sol_types::private::FixedBytes<32>,
        #[allow(missing_docs)]
        pub account: alloy::sol_types::private::Address,
    }
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`canImplementInterfaceForAddress(bytes32,address)`](canImplementInterfaceForAddressCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct canImplementInterfaceForAddressReturn {
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
            type UnderlyingSolTuple<'a> = (
                alloy::sol_types::sol_data::FixedBytes<32>,
                alloy::sol_types::sol_data::Address,
            );
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                alloy::sol_types::private::FixedBytes<32>,
                alloy::sol_types::private::Address,
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
            impl ::core::convert::From<canImplementInterfaceForAddressCall>
            for UnderlyingRustTuple<'_> {
                fn from(value: canImplementInterfaceForAddressCall) -> Self {
                    (value.interfaceHash, value.account)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for canImplementInterfaceForAddressCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {
                        interfaceHash: tuple.0,
                        account: tuple.1,
                    }
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
            impl ::core::convert::From<canImplementInterfaceForAddressReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: canImplementInterfaceForAddressReturn) -> Self {
                    (value._0,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for canImplementInterfaceForAddressReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { _0: tuple.0 }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for canImplementInterfaceForAddressCall {
            type Parameters<'a> = (
                alloy::sol_types::sol_data::FixedBytes<32>,
                alloy::sol_types::sol_data::Address,
            );
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = alloy::sol_types::private::FixedBytes<32>;
            type ReturnTuple<'a> = (alloy::sol_types::sol_data::FixedBytes<32>,);
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "canImplementInterfaceForAddress(bytes32,address)";
            const SELECTOR: [u8; 4] = [36u8, 156u8, 179u8, 250u8];
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
                    > as alloy_sol_types::SolType>::tokenize(&self.interfaceHash),
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        &self.account,
                    ),
                )
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
                        let r: canImplementInterfaceForAddressReturn = r.into();
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
                        let r: canImplementInterfaceForAddressReturn = r.into();
                        r._0
                    })
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `channels(bytes32)` and selector `0x7a7ebd7b`.
```solidity
function channels(bytes32) external view returns (Balance balance, TicketIndex ticketIndex, Timestamp closureTime, ChannelEpoch epoch, ChannelStatus status);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct channelsCall(pub alloy::sol_types::private::FixedBytes<32>);
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`channels(bytes32)`](channelsCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct channelsReturn {
        #[allow(missing_docs)]
        pub balance: <Balance as alloy::sol_types::SolType>::RustType,
        #[allow(missing_docs)]
        pub ticketIndex: <TicketIndex as alloy::sol_types::SolType>::RustType,
        #[allow(missing_docs)]
        pub closureTime: <Timestamp as alloy::sol_types::SolType>::RustType,
        #[allow(missing_docs)]
        pub epoch: <ChannelEpoch as alloy::sol_types::SolType>::RustType,
        #[allow(missing_docs)]
        pub status: <ChannelStatus as alloy::sol_types::SolType>::RustType,
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
            impl ::core::convert::From<channelsCall> for UnderlyingRustTuple<'_> {
                fn from(value: channelsCall) -> Self {
                    (value.0,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for channelsCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self(tuple.0)
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (
                Balance,
                TicketIndex,
                Timestamp,
                ChannelEpoch,
                ChannelStatus,
            );
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                <Balance as alloy::sol_types::SolType>::RustType,
                <TicketIndex as alloy::sol_types::SolType>::RustType,
                <Timestamp as alloy::sol_types::SolType>::RustType,
                <ChannelEpoch as alloy::sol_types::SolType>::RustType,
                <ChannelStatus as alloy::sol_types::SolType>::RustType,
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
            impl ::core::convert::From<channelsReturn> for UnderlyingRustTuple<'_> {
                fn from(value: channelsReturn) -> Self {
                    (
                        value.balance,
                        value.ticketIndex,
                        value.closureTime,
                        value.epoch,
                        value.status,
                    )
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for channelsReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {
                        balance: tuple.0,
                        ticketIndex: tuple.1,
                        closureTime: tuple.2,
                        epoch: tuple.3,
                        status: tuple.4,
                    }
                }
            }
        }
        impl channelsReturn {
            fn _tokenize(
                &self,
            ) -> <channelsCall as alloy_sol_types::SolCall>::ReturnToken<'_> {
                (
                    <Balance as alloy_sol_types::SolType>::tokenize(&self.balance),
                    <TicketIndex as alloy_sol_types::SolType>::tokenize(
                        &self.ticketIndex,
                    ),
                    <Timestamp as alloy_sol_types::SolType>::tokenize(&self.closureTime),
                    <ChannelEpoch as alloy_sol_types::SolType>::tokenize(&self.epoch),
                    <ChannelStatus as alloy_sol_types::SolType>::tokenize(&self.status),
                )
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for channelsCall {
            type Parameters<'a> = (alloy::sol_types::sol_data::FixedBytes<32>,);
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = channelsReturn;
            type ReturnTuple<'a> = (
                Balance,
                TicketIndex,
                Timestamp,
                ChannelEpoch,
                ChannelStatus,
            );
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "channels(bytes32)";
            const SELECTOR: [u8; 4] = [122u8, 126u8, 189u8, 123u8];
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
                    > as alloy_sol_types::SolType>::tokenize(&self.0),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                channelsReturn::_tokenize(ret)
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
    /**Function with signature `closeIncomingChannel(address)` and selector `0x1a7ffe7a`.
```solidity
function closeIncomingChannel(address source) external;
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct closeIncomingChannelCall {
        #[allow(missing_docs)]
        pub source: alloy::sol_types::private::Address,
    }
    ///Container type for the return parameters of the [`closeIncomingChannel(address)`](closeIncomingChannelCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct closeIncomingChannelReturn {}
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
            impl ::core::convert::From<closeIncomingChannelCall>
            for UnderlyingRustTuple<'_> {
                fn from(value: closeIncomingChannelCall) -> Self {
                    (value.source,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for closeIncomingChannelCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { source: tuple.0 }
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
            impl ::core::convert::From<closeIncomingChannelReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: closeIncomingChannelReturn) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for closeIncomingChannelReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        impl closeIncomingChannelReturn {
            fn _tokenize(
                &self,
            ) -> <closeIncomingChannelCall as alloy_sol_types::SolCall>::ReturnToken<
                '_,
            > {
                ()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for closeIncomingChannelCall {
            type Parameters<'a> = (alloy::sol_types::sol_data::Address,);
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = closeIncomingChannelReturn;
            type ReturnTuple<'a> = ();
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "closeIncomingChannel(address)";
            const SELECTOR: [u8; 4] = [26u8, 127u8, 254u8, 122u8];
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
                        &self.source,
                    ),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                closeIncomingChannelReturn::_tokenize(ret)
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
    /**Function with signature `closeIncomingChannelSafe(address,address)` and selector `0x54a2edf5`.
```solidity
function closeIncomingChannelSafe(address selfAddress, address source) external;
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct closeIncomingChannelSafeCall {
        #[allow(missing_docs)]
        pub selfAddress: alloy::sol_types::private::Address,
        #[allow(missing_docs)]
        pub source: alloy::sol_types::private::Address,
    }
    ///Container type for the return parameters of the [`closeIncomingChannelSafe(address,address)`](closeIncomingChannelSafeCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct closeIncomingChannelSafeReturn {}
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
            );
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                alloy::sol_types::private::Address,
                alloy::sol_types::private::Address,
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
            impl ::core::convert::From<closeIncomingChannelSafeCall>
            for UnderlyingRustTuple<'_> {
                fn from(value: closeIncomingChannelSafeCall) -> Self {
                    (value.selfAddress, value.source)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for closeIncomingChannelSafeCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {
                        selfAddress: tuple.0,
                        source: tuple.1,
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
            impl ::core::convert::From<closeIncomingChannelSafeReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: closeIncomingChannelSafeReturn) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for closeIncomingChannelSafeReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        impl closeIncomingChannelSafeReturn {
            fn _tokenize(
                &self,
            ) -> <closeIncomingChannelSafeCall as alloy_sol_types::SolCall>::ReturnToken<
                '_,
            > {
                ()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for closeIncomingChannelSafeCall {
            type Parameters<'a> = (
                alloy::sol_types::sol_data::Address,
                alloy::sol_types::sol_data::Address,
            );
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = closeIncomingChannelSafeReturn;
            type ReturnTuple<'a> = ();
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "closeIncomingChannelSafe(address,address)";
            const SELECTOR: [u8; 4] = [84u8, 162u8, 237u8, 245u8];
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
                        &self.selfAddress,
                    ),
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        &self.source,
                    ),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                closeIncomingChannelSafeReturn::_tokenize(ret)
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
    /**Function with signature `domainSeparator()` and selector `0xf698da25`.
```solidity
function domainSeparator() external view returns (bytes32);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct domainSeparatorCall;
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`domainSeparator()`](domainSeparatorCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct domainSeparatorReturn {
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
            impl ::core::convert::From<domainSeparatorCall> for UnderlyingRustTuple<'_> {
                fn from(value: domainSeparatorCall) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for domainSeparatorCall {
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
            impl ::core::convert::From<domainSeparatorReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: domainSeparatorReturn) -> Self {
                    (value._0,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for domainSeparatorReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { _0: tuple.0 }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for domainSeparatorCall {
            type Parameters<'a> = ();
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = alloy::sol_types::private::FixedBytes<32>;
            type ReturnTuple<'a> = (alloy::sol_types::sol_data::FixedBytes<32>,);
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "domainSeparator()";
            const SELECTOR: [u8; 4] = [246u8, 152u8, 218u8, 37u8];
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
                        let r: domainSeparatorReturn = r.into();
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
                        let r: domainSeparatorReturn = r.into();
                        r._0
                    })
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `finalizeOutgoingChannelClosure(address)` and selector `0x23cb3ac0`.
```solidity
function finalizeOutgoingChannelClosure(address destination) external;
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct finalizeOutgoingChannelClosureCall {
        #[allow(missing_docs)]
        pub destination: alloy::sol_types::private::Address,
    }
    ///Container type for the return parameters of the [`finalizeOutgoingChannelClosure(address)`](finalizeOutgoingChannelClosureCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct finalizeOutgoingChannelClosureReturn {}
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
            impl ::core::convert::From<finalizeOutgoingChannelClosureCall>
            for UnderlyingRustTuple<'_> {
                fn from(value: finalizeOutgoingChannelClosureCall) -> Self {
                    (value.destination,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for finalizeOutgoingChannelClosureCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { destination: tuple.0 }
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
            impl ::core::convert::From<finalizeOutgoingChannelClosureReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: finalizeOutgoingChannelClosureReturn) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for finalizeOutgoingChannelClosureReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        impl finalizeOutgoingChannelClosureReturn {
            fn _tokenize(
                &self,
            ) -> <finalizeOutgoingChannelClosureCall as alloy_sol_types::SolCall>::ReturnToken<
                '_,
            > {
                ()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for finalizeOutgoingChannelClosureCall {
            type Parameters<'a> = (alloy::sol_types::sol_data::Address,);
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = finalizeOutgoingChannelClosureReturn;
            type ReturnTuple<'a> = ();
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "finalizeOutgoingChannelClosure(address)";
            const SELECTOR: [u8; 4] = [35u8, 203u8, 58u8, 192u8];
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
                        &self.destination,
                    ),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                finalizeOutgoingChannelClosureReturn::_tokenize(ret)
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
    /**Function with signature `finalizeOutgoingChannelClosureSafe(address,address)` and selector `0x651514bf`.
```solidity
function finalizeOutgoingChannelClosureSafe(address selfAddress, address destination) external;
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct finalizeOutgoingChannelClosureSafeCall {
        #[allow(missing_docs)]
        pub selfAddress: alloy::sol_types::private::Address,
        #[allow(missing_docs)]
        pub destination: alloy::sol_types::private::Address,
    }
    ///Container type for the return parameters of the [`finalizeOutgoingChannelClosureSafe(address,address)`](finalizeOutgoingChannelClosureSafeCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct finalizeOutgoingChannelClosureSafeReturn {}
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
            );
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                alloy::sol_types::private::Address,
                alloy::sol_types::private::Address,
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
            impl ::core::convert::From<finalizeOutgoingChannelClosureSafeCall>
            for UnderlyingRustTuple<'_> {
                fn from(value: finalizeOutgoingChannelClosureSafeCall) -> Self {
                    (value.selfAddress, value.destination)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for finalizeOutgoingChannelClosureSafeCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {
                        selfAddress: tuple.0,
                        destination: tuple.1,
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
            impl ::core::convert::From<finalizeOutgoingChannelClosureSafeReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: finalizeOutgoingChannelClosureSafeReturn) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for finalizeOutgoingChannelClosureSafeReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        impl finalizeOutgoingChannelClosureSafeReturn {
            fn _tokenize(
                &self,
            ) -> <finalizeOutgoingChannelClosureSafeCall as alloy_sol_types::SolCall>::ReturnToken<
                '_,
            > {
                ()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for finalizeOutgoingChannelClosureSafeCall {
            type Parameters<'a> = (
                alloy::sol_types::sol_data::Address,
                alloy::sol_types::sol_data::Address,
            );
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = finalizeOutgoingChannelClosureSafeReturn;
            type ReturnTuple<'a> = ();
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "finalizeOutgoingChannelClosureSafe(address,address)";
            const SELECTOR: [u8; 4] = [101u8, 21u8, 20u8, 191u8];
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
                        &self.selfAddress,
                    ),
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        &self.destination,
                    ),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                finalizeOutgoingChannelClosureSafeReturn::_tokenize(ret)
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
    /**Function with signature `fundChannel(address,uint96)` and selector `0xfc55309a`.
```solidity
function fundChannel(address account, Balance amount) external;
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct fundChannelCall {
        #[allow(missing_docs)]
        pub account: alloy::sol_types::private::Address,
        #[allow(missing_docs)]
        pub amount: <Balance as alloy::sol_types::SolType>::RustType,
    }
    ///Container type for the return parameters of the [`fundChannel(address,uint96)`](fundChannelCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct fundChannelReturn {}
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
            type UnderlyingSolTuple<'a> = (alloy::sol_types::sol_data::Address, Balance);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                alloy::sol_types::private::Address,
                <Balance as alloy::sol_types::SolType>::RustType,
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
            impl ::core::convert::From<fundChannelCall> for UnderlyingRustTuple<'_> {
                fn from(value: fundChannelCall) -> Self {
                    (value.account, value.amount)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for fundChannelCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {
                        account: tuple.0,
                        amount: tuple.1,
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
            impl ::core::convert::From<fundChannelReturn> for UnderlyingRustTuple<'_> {
                fn from(value: fundChannelReturn) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for fundChannelReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        impl fundChannelReturn {
            fn _tokenize(
                &self,
            ) -> <fundChannelCall as alloy_sol_types::SolCall>::ReturnToken<'_> {
                ()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for fundChannelCall {
            type Parameters<'a> = (alloy::sol_types::sol_data::Address, Balance);
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = fundChannelReturn;
            type ReturnTuple<'a> = ();
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "fundChannel(address,uint96)";
            const SELECTOR: [u8; 4] = [252u8, 85u8, 48u8, 154u8];
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
                        &self.account,
                    ),
                    <Balance as alloy_sol_types::SolType>::tokenize(&self.amount),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                fundChannelReturn::_tokenize(ret)
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
    /**Function with signature `fundChannelSafe(address,address,uint96)` and selector `0x0abec58f`.
```solidity
function fundChannelSafe(address selfAddress, address account, Balance amount) external;
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct fundChannelSafeCall {
        #[allow(missing_docs)]
        pub selfAddress: alloy::sol_types::private::Address,
        #[allow(missing_docs)]
        pub account: alloy::sol_types::private::Address,
        #[allow(missing_docs)]
        pub amount: <Balance as alloy::sol_types::SolType>::RustType,
    }
    ///Container type for the return parameters of the [`fundChannelSafe(address,address,uint96)`](fundChannelSafeCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct fundChannelSafeReturn {}
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
                Balance,
            );
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                alloy::sol_types::private::Address,
                alloy::sol_types::private::Address,
                <Balance as alloy::sol_types::SolType>::RustType,
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
            impl ::core::convert::From<fundChannelSafeCall> for UnderlyingRustTuple<'_> {
                fn from(value: fundChannelSafeCall) -> Self {
                    (value.selfAddress, value.account, value.amount)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for fundChannelSafeCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {
                        selfAddress: tuple.0,
                        account: tuple.1,
                        amount: tuple.2,
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
            impl ::core::convert::From<fundChannelSafeReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: fundChannelSafeReturn) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for fundChannelSafeReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        impl fundChannelSafeReturn {
            fn _tokenize(
                &self,
            ) -> <fundChannelSafeCall as alloy_sol_types::SolCall>::ReturnToken<'_> {
                ()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for fundChannelSafeCall {
            type Parameters<'a> = (
                alloy::sol_types::sol_data::Address,
                alloy::sol_types::sol_data::Address,
                Balance,
            );
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = fundChannelSafeReturn;
            type ReturnTuple<'a> = ();
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "fundChannelSafe(address,address,uint96)";
            const SELECTOR: [u8; 4] = [10u8, 190u8, 197u8, 143u8];
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
                        &self.selfAddress,
                    ),
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        &self.account,
                    ),
                    <Balance as alloy_sol_types::SolType>::tokenize(&self.amount),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                fundChannelSafeReturn::_tokenize(ret)
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
    /**Function with signature `initiateOutgoingChannelClosure(address)` and selector `0x7c8e28da`.
```solidity
function initiateOutgoingChannelClosure(address destination) external;
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct initiateOutgoingChannelClosureCall {
        #[allow(missing_docs)]
        pub destination: alloy::sol_types::private::Address,
    }
    ///Container type for the return parameters of the [`initiateOutgoingChannelClosure(address)`](initiateOutgoingChannelClosureCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct initiateOutgoingChannelClosureReturn {}
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
            impl ::core::convert::From<initiateOutgoingChannelClosureCall>
            for UnderlyingRustTuple<'_> {
                fn from(value: initiateOutgoingChannelClosureCall) -> Self {
                    (value.destination,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for initiateOutgoingChannelClosureCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { destination: tuple.0 }
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
            impl ::core::convert::From<initiateOutgoingChannelClosureReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: initiateOutgoingChannelClosureReturn) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for initiateOutgoingChannelClosureReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        impl initiateOutgoingChannelClosureReturn {
            fn _tokenize(
                &self,
            ) -> <initiateOutgoingChannelClosureCall as alloy_sol_types::SolCall>::ReturnToken<
                '_,
            > {
                ()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for initiateOutgoingChannelClosureCall {
            type Parameters<'a> = (alloy::sol_types::sol_data::Address,);
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = initiateOutgoingChannelClosureReturn;
            type ReturnTuple<'a> = ();
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "initiateOutgoingChannelClosure(address)";
            const SELECTOR: [u8; 4] = [124u8, 142u8, 40u8, 218u8];
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
                        &self.destination,
                    ),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                initiateOutgoingChannelClosureReturn::_tokenize(ret)
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
    /**Function with signature `initiateOutgoingChannelClosureSafe(address,address)` and selector `0xbda65f45`.
```solidity
function initiateOutgoingChannelClosureSafe(address selfAddress, address destination) external;
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct initiateOutgoingChannelClosureSafeCall {
        #[allow(missing_docs)]
        pub selfAddress: alloy::sol_types::private::Address,
        #[allow(missing_docs)]
        pub destination: alloy::sol_types::private::Address,
    }
    ///Container type for the return parameters of the [`initiateOutgoingChannelClosureSafe(address,address)`](initiateOutgoingChannelClosureSafeCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct initiateOutgoingChannelClosureSafeReturn {}
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
            );
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                alloy::sol_types::private::Address,
                alloy::sol_types::private::Address,
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
            impl ::core::convert::From<initiateOutgoingChannelClosureSafeCall>
            for UnderlyingRustTuple<'_> {
                fn from(value: initiateOutgoingChannelClosureSafeCall) -> Self {
                    (value.selfAddress, value.destination)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for initiateOutgoingChannelClosureSafeCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {
                        selfAddress: tuple.0,
                        destination: tuple.1,
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
            impl ::core::convert::From<initiateOutgoingChannelClosureSafeReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: initiateOutgoingChannelClosureSafeReturn) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for initiateOutgoingChannelClosureSafeReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        impl initiateOutgoingChannelClosureSafeReturn {
            fn _tokenize(
                &self,
            ) -> <initiateOutgoingChannelClosureSafeCall as alloy_sol_types::SolCall>::ReturnToken<
                '_,
            > {
                ()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for initiateOutgoingChannelClosureSafeCall {
            type Parameters<'a> = (
                alloy::sol_types::sol_data::Address,
                alloy::sol_types::sol_data::Address,
            );
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = initiateOutgoingChannelClosureSafeReturn;
            type ReturnTuple<'a> = ();
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "initiateOutgoingChannelClosureSafe(address,address)";
            const SELECTOR: [u8; 4] = [189u8, 166u8, 95u8, 69u8];
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
                        &self.selfAddress,
                    ),
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        &self.destination,
                    ),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                initiateOutgoingChannelClosureSafeReturn::_tokenize(ret)
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
    /**Function with signature `ledgerDomainSeparator()` and selector `0xc966c4fe`.
```solidity
function ledgerDomainSeparator() external view returns (bytes32);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct ledgerDomainSeparatorCall;
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`ledgerDomainSeparator()`](ledgerDomainSeparatorCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct ledgerDomainSeparatorReturn {
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
            impl ::core::convert::From<ledgerDomainSeparatorCall>
            for UnderlyingRustTuple<'_> {
                fn from(value: ledgerDomainSeparatorCall) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for ledgerDomainSeparatorCall {
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
            impl ::core::convert::From<ledgerDomainSeparatorReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: ledgerDomainSeparatorReturn) -> Self {
                    (value._0,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for ledgerDomainSeparatorReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { _0: tuple.0 }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for ledgerDomainSeparatorCall {
            type Parameters<'a> = ();
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = alloy::sol_types::private::FixedBytes<32>;
            type ReturnTuple<'a> = (alloy::sol_types::sol_data::FixedBytes<32>,);
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "ledgerDomainSeparator()";
            const SELECTOR: [u8; 4] = [201u8, 102u8, 196u8, 254u8];
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
                        let r: ledgerDomainSeparatorReturn = r.into();
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
                        let r: ledgerDomainSeparatorReturn = r.into();
                        r._0
                    })
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `multicall(bytes[])` and selector `0xac9650d8`.
```solidity
function multicall(bytes[] memory data) external returns (bytes[] memory results);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct multicallCall {
        #[allow(missing_docs)]
        pub data: alloy::sol_types::private::Vec<alloy::sol_types::private::Bytes>,
    }
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`multicall(bytes[])`](multicallCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct multicallReturn {
        #[allow(missing_docs)]
        pub results: alloy::sol_types::private::Vec<alloy::sol_types::private::Bytes>,
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
                alloy::sol_types::sol_data::Array<alloy::sol_types::sol_data::Bytes>,
            );
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                alloy::sol_types::private::Vec<alloy::sol_types::private::Bytes>,
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
            impl ::core::convert::From<multicallCall> for UnderlyingRustTuple<'_> {
                fn from(value: multicallCall) -> Self {
                    (value.data,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for multicallCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { data: tuple.0 }
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (
                alloy::sol_types::sol_data::Array<alloy::sol_types::sol_data::Bytes>,
            );
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                alloy::sol_types::private::Vec<alloy::sol_types::private::Bytes>,
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
            impl ::core::convert::From<multicallReturn> for UnderlyingRustTuple<'_> {
                fn from(value: multicallReturn) -> Self {
                    (value.results,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for multicallReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { results: tuple.0 }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for multicallCall {
            type Parameters<'a> = (
                alloy::sol_types::sol_data::Array<alloy::sol_types::sol_data::Bytes>,
            );
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = alloy::sol_types::private::Vec<
                alloy::sol_types::private::Bytes,
            >;
            type ReturnTuple<'a> = (
                alloy::sol_types::sol_data::Array<alloy::sol_types::sol_data::Bytes>,
            );
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "multicall(bytes[])";
            const SELECTOR: [u8; 4] = [172u8, 150u8, 80u8, 216u8];
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
                        alloy::sol_types::sol_data::Bytes,
                    > as alloy_sol_types::SolType>::tokenize(&self.data),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                (
                    <alloy::sol_types::sol_data::Array<
                        alloy::sol_types::sol_data::Bytes,
                    > as alloy_sol_types::SolType>::tokenize(ret),
                )
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(|r| {
                        let r: multicallReturn = r.into();
                        r.results
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
                        let r: multicallReturn = r.into();
                        r.results
                    })
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `noticePeriodChannelClosure()` and selector `0x87352d65`.
```solidity
function noticePeriodChannelClosure() external view returns (Timestamp);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct noticePeriodChannelClosureCall;
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`noticePeriodChannelClosure()`](noticePeriodChannelClosureCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct noticePeriodChannelClosureReturn {
        #[allow(missing_docs)]
        pub _0: <Timestamp as alloy::sol_types::SolType>::RustType,
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
            impl ::core::convert::From<noticePeriodChannelClosureCall>
            for UnderlyingRustTuple<'_> {
                fn from(value: noticePeriodChannelClosureCall) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for noticePeriodChannelClosureCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self
                }
            }
        }
        {
            #[doc(hidden)]
            type UnderlyingSolTuple<'a> = (Timestamp,);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                <Timestamp as alloy::sol_types::SolType>::RustType,
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
            impl ::core::convert::From<noticePeriodChannelClosureReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: noticePeriodChannelClosureReturn) -> Self {
                    (value._0,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for noticePeriodChannelClosureReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { _0: tuple.0 }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for noticePeriodChannelClosureCall {
            type Parameters<'a> = ();
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = <Timestamp as alloy::sol_types::SolType>::RustType;
            type ReturnTuple<'a> = (Timestamp,);
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "noticePeriodChannelClosure()";
            const SELECTOR: [u8; 4] = [135u8, 53u8, 45u8, 101u8];
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
                (<Timestamp as alloy_sol_types::SolType>::tokenize(ret),)
            }
            #[inline]
            fn abi_decode_returns(data: &[u8]) -> alloy_sol_types::Result<Self::Return> {
                <Self::ReturnTuple<
                    '_,
                > as alloy_sol_types::SolType>::abi_decode_sequence(data)
                    .map(|r| {
                        let r: noticePeriodChannelClosureReturn = r.into();
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
                        let r: noticePeriodChannelClosureReturn = r.into();
                        r._0
                    })
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `redeemTicket(((bytes32,uint96,uint48,uint32,uint24,uint56),(bytes32,bytes32),uint256),(uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256))` and selector `0xfcb7796f`.
```solidity
function redeemTicket(RedeemableTicket memory redeemable, HoprCrypto.VRFParameters memory params) external;
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct redeemTicketCall {
        #[allow(missing_docs)]
        pub redeemable: <RedeemableTicket as alloy::sol_types::SolType>::RustType,
        #[allow(missing_docs)]
        pub params: <HoprCrypto::VRFParameters as alloy::sol_types::SolType>::RustType,
    }
    ///Container type for the return parameters of the [`redeemTicket(((bytes32,uint96,uint48,uint32,uint24,uint56),(bytes32,bytes32),uint256),(uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256))`](redeemTicketCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct redeemTicketReturn {}
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
            type UnderlyingSolTuple<'a> = (RedeemableTicket, HoprCrypto::VRFParameters);
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                <RedeemableTicket as alloy::sol_types::SolType>::RustType,
                <HoprCrypto::VRFParameters as alloy::sol_types::SolType>::RustType,
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
            impl ::core::convert::From<redeemTicketCall> for UnderlyingRustTuple<'_> {
                fn from(value: redeemTicketCall) -> Self {
                    (value.redeemable, value.params)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for redeemTicketCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {
                        redeemable: tuple.0,
                        params: tuple.1,
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
            impl ::core::convert::From<redeemTicketReturn> for UnderlyingRustTuple<'_> {
                fn from(value: redeemTicketReturn) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for redeemTicketReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        impl redeemTicketReturn {
            fn _tokenize(
                &self,
            ) -> <redeemTicketCall as alloy_sol_types::SolCall>::ReturnToken<'_> {
                ()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for redeemTicketCall {
            type Parameters<'a> = (RedeemableTicket, HoprCrypto::VRFParameters);
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = redeemTicketReturn;
            type ReturnTuple<'a> = ();
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "redeemTicket(((bytes32,uint96,uint48,uint32,uint24,uint56),(bytes32,bytes32),uint256),(uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256))";
            const SELECTOR: [u8; 4] = [252u8, 183u8, 121u8, 111u8];
            #[inline]
            fn new<'a>(
                tuple: <Self::Parameters<'a> as alloy_sol_types::SolType>::RustType,
            ) -> Self {
                tuple.into()
            }
            #[inline]
            fn tokenize(&self) -> Self::Token<'_> {
                (
                    <RedeemableTicket as alloy_sol_types::SolType>::tokenize(
                        &self.redeemable,
                    ),
                    <HoprCrypto::VRFParameters as alloy_sol_types::SolType>::tokenize(
                        &self.params,
                    ),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                redeemTicketReturn::_tokenize(ret)
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
    /**Function with signature `redeemTicketSafe(address,((bytes32,uint96,uint48,uint32,uint24,uint56),(bytes32,bytes32),uint256),(uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256))` and selector `0x0cd88d72`.
```solidity
function redeemTicketSafe(address selfAddress, RedeemableTicket memory redeemable, HoprCrypto.VRFParameters memory params) external;
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct redeemTicketSafeCall {
        #[allow(missing_docs)]
        pub selfAddress: alloy::sol_types::private::Address,
        #[allow(missing_docs)]
        pub redeemable: <RedeemableTicket as alloy::sol_types::SolType>::RustType,
        #[allow(missing_docs)]
        pub params: <HoprCrypto::VRFParameters as alloy::sol_types::SolType>::RustType,
    }
    ///Container type for the return parameters of the [`redeemTicketSafe(address,((bytes32,uint96,uint48,uint32,uint24,uint56),(bytes32,bytes32),uint256),(uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256))`](redeemTicketSafeCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct redeemTicketSafeReturn {}
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
                RedeemableTicket,
                HoprCrypto::VRFParameters,
            );
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                alloy::sol_types::private::Address,
                <RedeemableTicket as alloy::sol_types::SolType>::RustType,
                <HoprCrypto::VRFParameters as alloy::sol_types::SolType>::RustType,
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
            impl ::core::convert::From<redeemTicketSafeCall>
            for UnderlyingRustTuple<'_> {
                fn from(value: redeemTicketSafeCall) -> Self {
                    (value.selfAddress, value.redeemable, value.params)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for redeemTicketSafeCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {
                        selfAddress: tuple.0,
                        redeemable: tuple.1,
                        params: tuple.2,
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
            impl ::core::convert::From<redeemTicketSafeReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: redeemTicketSafeReturn) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for redeemTicketSafeReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        impl redeemTicketSafeReturn {
            fn _tokenize(
                &self,
            ) -> <redeemTicketSafeCall as alloy_sol_types::SolCall>::ReturnToken<'_> {
                ()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for redeemTicketSafeCall {
            type Parameters<'a> = (
                alloy::sol_types::sol_data::Address,
                RedeemableTicket,
                HoprCrypto::VRFParameters,
            );
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = redeemTicketSafeReturn;
            type ReturnTuple<'a> = ();
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "redeemTicketSafe(address,((bytes32,uint96,uint48,uint32,uint24,uint56),(bytes32,bytes32),uint256),(uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256))";
            const SELECTOR: [u8; 4] = [12u8, 216u8, 141u8, 114u8];
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
                        &self.selfAddress,
                    ),
                    <RedeemableTicket as alloy_sol_types::SolType>::tokenize(
                        &self.redeemable,
                    ),
                    <HoprCrypto::VRFParameters as alloy_sol_types::SolType>::tokenize(
                        &self.params,
                    ),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                redeemTicketSafeReturn::_tokenize(ret)
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
    /**Function with signature `token()` and selector `0xfc0c546a`.
```solidity
function token() external view returns (address);
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct tokenCall;
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    ///Container type for the return parameters of the [`token()`](tokenCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct tokenReturn {
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
            impl ::core::convert::From<tokenCall> for UnderlyingRustTuple<'_> {
                fn from(value: tokenCall) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for tokenCall {
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
            impl ::core::convert::From<tokenReturn> for UnderlyingRustTuple<'_> {
                fn from(value: tokenReturn) -> Self {
                    (value._0,)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for tokenReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self { _0: tuple.0 }
                }
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for tokenCall {
            type Parameters<'a> = ();
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = alloy::sol_types::private::Address;
            type ReturnTuple<'a> = (alloy::sol_types::sol_data::Address,);
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "token()";
            const SELECTOR: [u8; 4] = [252u8, 12u8, 84u8, 106u8];
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
                        let r: tokenReturn = r.into();
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
                        let r: tokenReturn = r.into();
                        r._0
                    })
            }
        }
    };
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Default, Debug, PartialEq, Eq, Hash)]
    /**Function with signature `tokensReceived(address,address,address,uint256,bytes,bytes)` and selector `0x0023de29`.
```solidity
function tokensReceived(address, address from, address to, uint256 amount, bytes memory userData, bytes memory) external;
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct tokensReceivedCall {
        #[allow(missing_docs)]
        pub _0: alloy::sol_types::private::Address,
        #[allow(missing_docs)]
        pub from: alloy::sol_types::private::Address,
        #[allow(missing_docs)]
        pub to: alloy::sol_types::private::Address,
        #[allow(missing_docs)]
        pub amount: alloy::sol_types::private::primitives::aliases::U256,
        #[allow(missing_docs)]
        pub userData: alloy::sol_types::private::Bytes,
        #[allow(missing_docs)]
        pub _5: alloy::sol_types::private::Bytes,
    }
    ///Container type for the return parameters of the [`tokensReceived(address,address,address,uint256,bytes,bytes)`](tokensReceivedCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct tokensReceivedReturn {}
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
                alloy::sol_types::sol_data::Uint<256>,
                alloy::sol_types::sol_data::Bytes,
                alloy::sol_types::sol_data::Bytes,
            );
            #[doc(hidden)]
            type UnderlyingRustTuple<'a> = (
                alloy::sol_types::private::Address,
                alloy::sol_types::private::Address,
                alloy::sol_types::private::Address,
                alloy::sol_types::private::primitives::aliases::U256,
                alloy::sol_types::private::Bytes,
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
            impl ::core::convert::From<tokensReceivedCall> for UnderlyingRustTuple<'_> {
                fn from(value: tokensReceivedCall) -> Self {
                    (
                        value._0,
                        value.from,
                        value.to,
                        value.amount,
                        value.userData,
                        value._5,
                    )
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>> for tokensReceivedCall {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {
                        _0: tuple.0,
                        from: tuple.1,
                        to: tuple.2,
                        amount: tuple.3,
                        userData: tuple.4,
                        _5: tuple.5,
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
            impl ::core::convert::From<tokensReceivedReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: tokensReceivedReturn) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for tokensReceivedReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        impl tokensReceivedReturn {
            fn _tokenize(
                &self,
            ) -> <tokensReceivedCall as alloy_sol_types::SolCall>::ReturnToken<'_> {
                ()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for tokensReceivedCall {
            type Parameters<'a> = (
                alloy::sol_types::sol_data::Address,
                alloy::sol_types::sol_data::Address,
                alloy::sol_types::sol_data::Address,
                alloy::sol_types::sol_data::Uint<256>,
                alloy::sol_types::sol_data::Bytes,
                alloy::sol_types::sol_data::Bytes,
            );
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = tokensReceivedReturn;
            type ReturnTuple<'a> = ();
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "tokensReceived(address,address,address,uint256,bytes,bytes)";
            const SELECTOR: [u8; 4] = [0u8, 35u8, 222u8, 41u8];
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
                        &self._0,
                    ),
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        &self.from,
                    ),
                    <alloy::sol_types::sol_data::Address as alloy_sol_types::SolType>::tokenize(
                        &self.to,
                    ),
                    <alloy::sol_types::sol_data::Uint<
                        256,
                    > as alloy_sol_types::SolType>::tokenize(&self.amount),
                    <alloy::sol_types::sol_data::Bytes as alloy_sol_types::SolType>::tokenize(
                        &self.userData,
                    ),
                    <alloy::sol_types::sol_data::Bytes as alloy_sol_types::SolType>::tokenize(
                        &self._5,
                    ),
                )
            }
            #[inline]
            fn tokenize_returns(ret: &Self::Return) -> Self::ReturnToken<'_> {
                tokensReceivedReturn::_tokenize(ret)
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
    /**Function with signature `updateDomainSeparator()` and selector `0x89ccfe89`.
```solidity
function updateDomainSeparator() external;
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct updateDomainSeparatorCall;
    ///Container type for the return parameters of the [`updateDomainSeparator()`](updateDomainSeparatorCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct updateDomainSeparatorReturn {}
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
            impl ::core::convert::From<updateDomainSeparatorCall>
            for UnderlyingRustTuple<'_> {
                fn from(value: updateDomainSeparatorCall) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for updateDomainSeparatorCall {
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
            impl ::core::convert::From<updateDomainSeparatorReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: updateDomainSeparatorReturn) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for updateDomainSeparatorReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        impl updateDomainSeparatorReturn {
            fn _tokenize(
                &self,
            ) -> <updateDomainSeparatorCall as alloy_sol_types::SolCall>::ReturnToken<
                '_,
            > {
                ()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for updateDomainSeparatorCall {
            type Parameters<'a> = ();
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = updateDomainSeparatorReturn;
            type ReturnTuple<'a> = ();
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "updateDomainSeparator()";
            const SELECTOR: [u8; 4] = [137u8, 204u8, 254u8, 137u8];
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
                updateDomainSeparatorReturn::_tokenize(ret)
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
    /**Function with signature `updateLedgerDomainSeparator()` and selector `0xdc96fd50`.
```solidity
function updateLedgerDomainSeparator() external;
```*/
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct updateLedgerDomainSeparatorCall;
    ///Container type for the return parameters of the [`updateLedgerDomainSeparator()`](updateLedgerDomainSeparatorCall) function.
    #[allow(non_camel_case_types, non_snake_case, clippy::pub_underscore_fields)]
    #[derive(Clone)]
    pub struct updateLedgerDomainSeparatorReturn {}
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
            impl ::core::convert::From<updateLedgerDomainSeparatorCall>
            for UnderlyingRustTuple<'_> {
                fn from(value: updateLedgerDomainSeparatorCall) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for updateLedgerDomainSeparatorCall {
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
            impl ::core::convert::From<updateLedgerDomainSeparatorReturn>
            for UnderlyingRustTuple<'_> {
                fn from(value: updateLedgerDomainSeparatorReturn) -> Self {
                    ()
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            impl ::core::convert::From<UnderlyingRustTuple<'_>>
            for updateLedgerDomainSeparatorReturn {
                fn from(tuple: UnderlyingRustTuple<'_>) -> Self {
                    Self {}
                }
            }
        }
        impl updateLedgerDomainSeparatorReturn {
            fn _tokenize(
                &self,
            ) -> <updateLedgerDomainSeparatorCall as alloy_sol_types::SolCall>::ReturnToken<
                '_,
            > {
                ()
            }
        }
        #[automatically_derived]
        impl alloy_sol_types::SolCall for updateLedgerDomainSeparatorCall {
            type Parameters<'a> = ();
            type Token<'a> = <Self::Parameters<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            type Return = updateLedgerDomainSeparatorReturn;
            type ReturnTuple<'a> = ();
            type ReturnToken<'a> = <Self::ReturnTuple<
                'a,
            > as alloy_sol_types::SolType>::Token<'a>;
            const SIGNATURE: &'static str = "updateLedgerDomainSeparator()";
            const SELECTOR: [u8; 4] = [220u8, 150u8, 253u8, 80u8];
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
                updateLedgerDomainSeparatorReturn::_tokenize(ret)
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
    ///Container for all the [`HoprChannels`](self) function calls.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive()]
    pub enum HoprChannelsCalls {
        #[allow(missing_docs)]
        ERC777_HOOK_FUND_CHANNEL_MULTI_SIZE(ERC777_HOOK_FUND_CHANNEL_MULTI_SIZECall),
        #[allow(missing_docs)]
        ERC777_HOOK_FUND_CHANNEL_SIZE(ERC777_HOOK_FUND_CHANNEL_SIZECall),
        #[allow(missing_docs)]
        LEDGER_VERSION(LEDGER_VERSIONCall),
        #[allow(missing_docs)]
        MAX_USED_BALANCE(MAX_USED_BALANCECall),
        #[allow(missing_docs)]
        MIN_USED_BALANCE(MIN_USED_BALANCECall),
        #[allow(missing_docs)]
        TOKENS_RECIPIENT_INTERFACE_HASH(TOKENS_RECIPIENT_INTERFACE_HASHCall),
        #[allow(missing_docs)]
        VERSION(VERSIONCall),
        #[allow(missing_docs)]
        _currentBlockTimestamp(_currentBlockTimestampCall),
        #[allow(missing_docs)]
        _getChannelId(_getChannelIdCall),
        #[allow(missing_docs)]
        _getTicketHash(_getTicketHashCall),
        #[allow(missing_docs)]
        _isWinningTicket(_isWinningTicketCall),
        #[allow(missing_docs)]
        canImplementInterfaceForAddress(canImplementInterfaceForAddressCall),
        #[allow(missing_docs)]
        channels(channelsCall),
        #[allow(missing_docs)]
        closeIncomingChannel(closeIncomingChannelCall),
        #[allow(missing_docs)]
        closeIncomingChannelSafe(closeIncomingChannelSafeCall),
        #[allow(missing_docs)]
        domainSeparator(domainSeparatorCall),
        #[allow(missing_docs)]
        finalizeOutgoingChannelClosure(finalizeOutgoingChannelClosureCall),
        #[allow(missing_docs)]
        finalizeOutgoingChannelClosureSafe(finalizeOutgoingChannelClosureSafeCall),
        #[allow(missing_docs)]
        fundChannel(fundChannelCall),
        #[allow(missing_docs)]
        fundChannelSafe(fundChannelSafeCall),
        #[allow(missing_docs)]
        initiateOutgoingChannelClosure(initiateOutgoingChannelClosureCall),
        #[allow(missing_docs)]
        initiateOutgoingChannelClosureSafe(initiateOutgoingChannelClosureSafeCall),
        #[allow(missing_docs)]
        ledgerDomainSeparator(ledgerDomainSeparatorCall),
        #[allow(missing_docs)]
        multicall(multicallCall),
        #[allow(missing_docs)]
        noticePeriodChannelClosure(noticePeriodChannelClosureCall),
        #[allow(missing_docs)]
        redeemTicket(redeemTicketCall),
        #[allow(missing_docs)]
        redeemTicketSafe(redeemTicketSafeCall),
        #[allow(missing_docs)]
        token(tokenCall),
        #[allow(missing_docs)]
        tokensReceived(tokensReceivedCall),
        #[allow(missing_docs)]
        updateDomainSeparator(updateDomainSeparatorCall),
        #[allow(missing_docs)]
        updateLedgerDomainSeparator(updateLedgerDomainSeparatorCall),
    }
    #[automatically_derived]
    impl HoprChannelsCalls {
        /// All the selectors of this enum.
        ///
        /// Note that the selectors might not be in the same order as the variants.
        /// No guarantees are made about the order of the selectors.
        ///
        /// Prefer using `SolInterface` methods instead.
        pub const SELECTORS: &'static [[u8; 4usize]] = &[
            [0u8, 35u8, 222u8, 41u8],
            [10u8, 190u8, 197u8, 143u8],
            [12u8, 216u8, 141u8, 114u8],
            [26u8, 127u8, 254u8, 122u8],
            [35u8, 203u8, 58u8, 192u8],
            [36u8, 8u8, 108u8, 194u8],
            [36u8, 156u8, 179u8, 250u8],
            [41u8, 57u8, 46u8, 50u8],
            [68u8, 218u8, 230u8, 248u8],
            [84u8, 162u8, 237u8, 245u8],
            [93u8, 47u8, 7u8, 197u8],
            [101u8, 21u8, 20u8, 191u8],
            [114u8, 88u8, 28u8, 192u8],
            [120u8, 216u8, 1u8, 109u8],
            [122u8, 126u8, 189u8, 123u8],
            [124u8, 142u8, 40u8, 218u8],
            [135u8, 53u8, 45u8, 101u8],
            [137u8, 204u8, 254u8, 137u8],
            [140u8, 55u8, 16u8, 201u8],
            [172u8, 150u8, 80u8, 216u8],
            [185u8, 32u8, 222u8, 237u8],
            [189u8, 166u8, 95u8, 69u8],
            [190u8, 155u8, 171u8, 220u8],
            [201u8, 102u8, 196u8, 254u8],
            [220u8, 150u8, 253u8, 80u8],
            [221u8, 173u8, 25u8, 2u8],
            [246u8, 152u8, 218u8, 37u8],
            [252u8, 12u8, 84u8, 106u8],
            [252u8, 85u8, 48u8, 154u8],
            [252u8, 183u8, 121u8, 111u8],
            [255u8, 161u8, 173u8, 116u8],
        ];
    }
    #[automatically_derived]
    impl alloy_sol_types::SolInterface for HoprChannelsCalls {
        const NAME: &'static str = "HoprChannelsCalls";
        const MIN_DATA_LENGTH: usize = 0usize;
        const COUNT: usize = 31usize;
        #[inline]
        fn selector(&self) -> [u8; 4] {
            match self {
                Self::ERC777_HOOK_FUND_CHANNEL_MULTI_SIZE(_) => {
                    <ERC777_HOOK_FUND_CHANNEL_MULTI_SIZECall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::ERC777_HOOK_FUND_CHANNEL_SIZE(_) => {
                    <ERC777_HOOK_FUND_CHANNEL_SIZECall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::LEDGER_VERSION(_) => {
                    <LEDGER_VERSIONCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::MAX_USED_BALANCE(_) => {
                    <MAX_USED_BALANCECall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::MIN_USED_BALANCE(_) => {
                    <MIN_USED_BALANCECall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::TOKENS_RECIPIENT_INTERFACE_HASH(_) => {
                    <TOKENS_RECIPIENT_INTERFACE_HASHCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::VERSION(_) => <VERSIONCall as alloy_sol_types::SolCall>::SELECTOR,
                Self::_currentBlockTimestamp(_) => {
                    <_currentBlockTimestampCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::_getChannelId(_) => {
                    <_getChannelIdCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::_getTicketHash(_) => {
                    <_getTicketHashCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::_isWinningTicket(_) => {
                    <_isWinningTicketCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::canImplementInterfaceForAddress(_) => {
                    <canImplementInterfaceForAddressCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::channels(_) => <channelsCall as alloy_sol_types::SolCall>::SELECTOR,
                Self::closeIncomingChannel(_) => {
                    <closeIncomingChannelCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::closeIncomingChannelSafe(_) => {
                    <closeIncomingChannelSafeCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::domainSeparator(_) => {
                    <domainSeparatorCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::finalizeOutgoingChannelClosure(_) => {
                    <finalizeOutgoingChannelClosureCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::finalizeOutgoingChannelClosureSafe(_) => {
                    <finalizeOutgoingChannelClosureSafeCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::fundChannel(_) => {
                    <fundChannelCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::fundChannelSafe(_) => {
                    <fundChannelSafeCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::initiateOutgoingChannelClosure(_) => {
                    <initiateOutgoingChannelClosureCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::initiateOutgoingChannelClosureSafe(_) => {
                    <initiateOutgoingChannelClosureSafeCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::ledgerDomainSeparator(_) => {
                    <ledgerDomainSeparatorCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::multicall(_) => {
                    <multicallCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::noticePeriodChannelClosure(_) => {
                    <noticePeriodChannelClosureCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::redeemTicket(_) => {
                    <redeemTicketCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::redeemTicketSafe(_) => {
                    <redeemTicketSafeCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::token(_) => <tokenCall as alloy_sol_types::SolCall>::SELECTOR,
                Self::tokensReceived(_) => {
                    <tokensReceivedCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::updateDomainSeparator(_) => {
                    <updateDomainSeparatorCall as alloy_sol_types::SolCall>::SELECTOR
                }
                Self::updateLedgerDomainSeparator(_) => {
                    <updateLedgerDomainSeparatorCall as alloy_sol_types::SolCall>::SELECTOR
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
            ) -> alloy_sol_types::Result<HoprChannelsCalls>] = &[
                {
                    fn tokensReceived(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <tokensReceivedCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsCalls::tokensReceived)
                    }
                    tokensReceived
                },
                {
                    fn fundChannelSafe(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <fundChannelSafeCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsCalls::fundChannelSafe)
                    }
                    fundChannelSafe
                },
                {
                    fn redeemTicketSafe(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <redeemTicketSafeCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsCalls::redeemTicketSafe)
                    }
                    redeemTicketSafe
                },
                {
                    fn closeIncomingChannel(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <closeIncomingChannelCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsCalls::closeIncomingChannel)
                    }
                    closeIncomingChannel
                },
                {
                    fn finalizeOutgoingChannelClosure(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <finalizeOutgoingChannelClosureCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsCalls::finalizeOutgoingChannelClosure)
                    }
                    finalizeOutgoingChannelClosure
                },
                {
                    fn _getTicketHash(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <_getTicketHashCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsCalls::_getTicketHash)
                    }
                    _getTicketHash
                },
                {
                    fn canImplementInterfaceForAddress(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <canImplementInterfaceForAddressCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsCalls::canImplementInterfaceForAddress)
                    }
                    canImplementInterfaceForAddress
                },
                {
                    fn MIN_USED_BALANCE(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <MIN_USED_BALANCECall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsCalls::MIN_USED_BALANCE)
                    }
                    MIN_USED_BALANCE
                },
                {
                    fn ERC777_HOOK_FUND_CHANNEL_SIZE(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <ERC777_HOOK_FUND_CHANNEL_SIZECall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsCalls::ERC777_HOOK_FUND_CHANNEL_SIZE)
                    }
                    ERC777_HOOK_FUND_CHANNEL_SIZE
                },
                {
                    fn closeIncomingChannelSafe(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <closeIncomingChannelSafeCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsCalls::closeIncomingChannelSafe)
                    }
                    closeIncomingChannelSafe
                },
                {
                    fn MAX_USED_BALANCE(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <MAX_USED_BALANCECall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsCalls::MAX_USED_BALANCE)
                    }
                    MAX_USED_BALANCE
                },
                {
                    fn finalizeOutgoingChannelClosureSafe(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <finalizeOutgoingChannelClosureSafeCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsCalls::finalizeOutgoingChannelClosureSafe)
                    }
                    finalizeOutgoingChannelClosureSafe
                },
                {
                    fn TOKENS_RECIPIENT_INTERFACE_HASH(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <TOKENS_RECIPIENT_INTERFACE_HASHCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsCalls::TOKENS_RECIPIENT_INTERFACE_HASH)
                    }
                    TOKENS_RECIPIENT_INTERFACE_HASH
                },
                {
                    fn ERC777_HOOK_FUND_CHANNEL_MULTI_SIZE(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <ERC777_HOOK_FUND_CHANNEL_MULTI_SIZECall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsCalls::ERC777_HOOK_FUND_CHANNEL_MULTI_SIZE)
                    }
                    ERC777_HOOK_FUND_CHANNEL_MULTI_SIZE
                },
                {
                    fn channels(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <channelsCall as alloy_sol_types::SolCall>::abi_decode_raw(data)
                            .map(HoprChannelsCalls::channels)
                    }
                    channels
                },
                {
                    fn initiateOutgoingChannelClosure(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <initiateOutgoingChannelClosureCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsCalls::initiateOutgoingChannelClosure)
                    }
                    initiateOutgoingChannelClosure
                },
                {
                    fn noticePeriodChannelClosure(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <noticePeriodChannelClosureCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsCalls::noticePeriodChannelClosure)
                    }
                    noticePeriodChannelClosure
                },
                {
                    fn updateDomainSeparator(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <updateDomainSeparatorCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsCalls::updateDomainSeparator)
                    }
                    updateDomainSeparator
                },
                {
                    fn _isWinningTicket(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <_isWinningTicketCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsCalls::_isWinningTicket)
                    }
                    _isWinningTicket
                },
                {
                    fn multicall(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <multicallCall as alloy_sol_types::SolCall>::abi_decode_raw(data)
                            .map(HoprChannelsCalls::multicall)
                    }
                    multicall
                },
                {
                    fn _currentBlockTimestamp(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <_currentBlockTimestampCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsCalls::_currentBlockTimestamp)
                    }
                    _currentBlockTimestamp
                },
                {
                    fn initiateOutgoingChannelClosureSafe(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <initiateOutgoingChannelClosureSafeCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsCalls::initiateOutgoingChannelClosureSafe)
                    }
                    initiateOutgoingChannelClosureSafe
                },
                {
                    fn _getChannelId(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <_getChannelIdCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsCalls::_getChannelId)
                    }
                    _getChannelId
                },
                {
                    fn ledgerDomainSeparator(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <ledgerDomainSeparatorCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsCalls::ledgerDomainSeparator)
                    }
                    ledgerDomainSeparator
                },
                {
                    fn updateLedgerDomainSeparator(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <updateLedgerDomainSeparatorCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsCalls::updateLedgerDomainSeparator)
                    }
                    updateLedgerDomainSeparator
                },
                {
                    fn LEDGER_VERSION(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <LEDGER_VERSIONCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsCalls::LEDGER_VERSION)
                    }
                    LEDGER_VERSION
                },
                {
                    fn domainSeparator(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <domainSeparatorCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsCalls::domainSeparator)
                    }
                    domainSeparator
                },
                {
                    fn token(data: &[u8]) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <tokenCall as alloy_sol_types::SolCall>::abi_decode_raw(data)
                            .map(HoprChannelsCalls::token)
                    }
                    token
                },
                {
                    fn fundChannel(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <fundChannelCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsCalls::fundChannel)
                    }
                    fundChannel
                },
                {
                    fn redeemTicket(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <redeemTicketCall as alloy_sol_types::SolCall>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsCalls::redeemTicket)
                    }
                    redeemTicket
                },
                {
                    fn VERSION(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <VERSIONCall as alloy_sol_types::SolCall>::abi_decode_raw(data)
                            .map(HoprChannelsCalls::VERSION)
                    }
                    VERSION
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
            ) -> alloy_sol_types::Result<HoprChannelsCalls>] = &[
                {
                    fn tokensReceived(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <tokensReceivedCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::tokensReceived)
                    }
                    tokensReceived
                },
                {
                    fn fundChannelSafe(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <fundChannelSafeCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::fundChannelSafe)
                    }
                    fundChannelSafe
                },
                {
                    fn redeemTicketSafe(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <redeemTicketSafeCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::redeemTicketSafe)
                    }
                    redeemTicketSafe
                },
                {
                    fn closeIncomingChannel(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <closeIncomingChannelCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::closeIncomingChannel)
                    }
                    closeIncomingChannel
                },
                {
                    fn finalizeOutgoingChannelClosure(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <finalizeOutgoingChannelClosureCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::finalizeOutgoingChannelClosure)
                    }
                    finalizeOutgoingChannelClosure
                },
                {
                    fn _getTicketHash(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <_getTicketHashCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::_getTicketHash)
                    }
                    _getTicketHash
                },
                {
                    fn canImplementInterfaceForAddress(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <canImplementInterfaceForAddressCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::canImplementInterfaceForAddress)
                    }
                    canImplementInterfaceForAddress
                },
                {
                    fn MIN_USED_BALANCE(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <MIN_USED_BALANCECall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::MIN_USED_BALANCE)
                    }
                    MIN_USED_BALANCE
                },
                {
                    fn ERC777_HOOK_FUND_CHANNEL_SIZE(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <ERC777_HOOK_FUND_CHANNEL_SIZECall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::ERC777_HOOK_FUND_CHANNEL_SIZE)
                    }
                    ERC777_HOOK_FUND_CHANNEL_SIZE
                },
                {
                    fn closeIncomingChannelSafe(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <closeIncomingChannelSafeCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::closeIncomingChannelSafe)
                    }
                    closeIncomingChannelSafe
                },
                {
                    fn MAX_USED_BALANCE(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <MAX_USED_BALANCECall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::MAX_USED_BALANCE)
                    }
                    MAX_USED_BALANCE
                },
                {
                    fn finalizeOutgoingChannelClosureSafe(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <finalizeOutgoingChannelClosureSafeCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::finalizeOutgoingChannelClosureSafe)
                    }
                    finalizeOutgoingChannelClosureSafe
                },
                {
                    fn TOKENS_RECIPIENT_INTERFACE_HASH(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <TOKENS_RECIPIENT_INTERFACE_HASHCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::TOKENS_RECIPIENT_INTERFACE_HASH)
                    }
                    TOKENS_RECIPIENT_INTERFACE_HASH
                },
                {
                    fn ERC777_HOOK_FUND_CHANNEL_MULTI_SIZE(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <ERC777_HOOK_FUND_CHANNEL_MULTI_SIZECall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::ERC777_HOOK_FUND_CHANNEL_MULTI_SIZE)
                    }
                    ERC777_HOOK_FUND_CHANNEL_MULTI_SIZE
                },
                {
                    fn channels(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <channelsCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::channels)
                    }
                    channels
                },
                {
                    fn initiateOutgoingChannelClosure(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <initiateOutgoingChannelClosureCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::initiateOutgoingChannelClosure)
                    }
                    initiateOutgoingChannelClosure
                },
                {
                    fn noticePeriodChannelClosure(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <noticePeriodChannelClosureCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::noticePeriodChannelClosure)
                    }
                    noticePeriodChannelClosure
                },
                {
                    fn updateDomainSeparator(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <updateDomainSeparatorCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::updateDomainSeparator)
                    }
                    updateDomainSeparator
                },
                {
                    fn _isWinningTicket(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <_isWinningTicketCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::_isWinningTicket)
                    }
                    _isWinningTicket
                },
                {
                    fn multicall(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <multicallCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::multicall)
                    }
                    multicall
                },
                {
                    fn _currentBlockTimestamp(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <_currentBlockTimestampCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::_currentBlockTimestamp)
                    }
                    _currentBlockTimestamp
                },
                {
                    fn initiateOutgoingChannelClosureSafe(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <initiateOutgoingChannelClosureSafeCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::initiateOutgoingChannelClosureSafe)
                    }
                    initiateOutgoingChannelClosureSafe
                },
                {
                    fn _getChannelId(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <_getChannelIdCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::_getChannelId)
                    }
                    _getChannelId
                },
                {
                    fn ledgerDomainSeparator(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <ledgerDomainSeparatorCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::ledgerDomainSeparator)
                    }
                    ledgerDomainSeparator
                },
                {
                    fn updateLedgerDomainSeparator(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <updateLedgerDomainSeparatorCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::updateLedgerDomainSeparator)
                    }
                    updateLedgerDomainSeparator
                },
                {
                    fn LEDGER_VERSION(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <LEDGER_VERSIONCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::LEDGER_VERSION)
                    }
                    LEDGER_VERSION
                },
                {
                    fn domainSeparator(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <domainSeparatorCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::domainSeparator)
                    }
                    domainSeparator
                },
                {
                    fn token(data: &[u8]) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <tokenCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::token)
                    }
                    token
                },
                {
                    fn fundChannel(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <fundChannelCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::fundChannel)
                    }
                    fundChannel
                },
                {
                    fn redeemTicket(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <redeemTicketCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::redeemTicket)
                    }
                    redeemTicket
                },
                {
                    fn VERSION(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsCalls> {
                        <VERSIONCall as alloy_sol_types::SolCall>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsCalls::VERSION)
                    }
                    VERSION
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
                Self::ERC777_HOOK_FUND_CHANNEL_MULTI_SIZE(inner) => {
                    <ERC777_HOOK_FUND_CHANNEL_MULTI_SIZECall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::ERC777_HOOK_FUND_CHANNEL_SIZE(inner) => {
                    <ERC777_HOOK_FUND_CHANNEL_SIZECall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::LEDGER_VERSION(inner) => {
                    <LEDGER_VERSIONCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::MAX_USED_BALANCE(inner) => {
                    <MAX_USED_BALANCECall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::MIN_USED_BALANCE(inner) => {
                    <MIN_USED_BALANCECall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::TOKENS_RECIPIENT_INTERFACE_HASH(inner) => {
                    <TOKENS_RECIPIENT_INTERFACE_HASHCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::VERSION(inner) => {
                    <VERSIONCall as alloy_sol_types::SolCall>::abi_encoded_size(inner)
                }
                Self::_currentBlockTimestamp(inner) => {
                    <_currentBlockTimestampCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::_getChannelId(inner) => {
                    <_getChannelIdCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::_getTicketHash(inner) => {
                    <_getTicketHashCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::_isWinningTicket(inner) => {
                    <_isWinningTicketCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::canImplementInterfaceForAddress(inner) => {
                    <canImplementInterfaceForAddressCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::channels(inner) => {
                    <channelsCall as alloy_sol_types::SolCall>::abi_encoded_size(inner)
                }
                Self::closeIncomingChannel(inner) => {
                    <closeIncomingChannelCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::closeIncomingChannelSafe(inner) => {
                    <closeIncomingChannelSafeCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::domainSeparator(inner) => {
                    <domainSeparatorCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::finalizeOutgoingChannelClosure(inner) => {
                    <finalizeOutgoingChannelClosureCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::finalizeOutgoingChannelClosureSafe(inner) => {
                    <finalizeOutgoingChannelClosureSafeCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::fundChannel(inner) => {
                    <fundChannelCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::fundChannelSafe(inner) => {
                    <fundChannelSafeCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::initiateOutgoingChannelClosure(inner) => {
                    <initiateOutgoingChannelClosureCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::initiateOutgoingChannelClosureSafe(inner) => {
                    <initiateOutgoingChannelClosureSafeCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::ledgerDomainSeparator(inner) => {
                    <ledgerDomainSeparatorCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::multicall(inner) => {
                    <multicallCall as alloy_sol_types::SolCall>::abi_encoded_size(inner)
                }
                Self::noticePeriodChannelClosure(inner) => {
                    <noticePeriodChannelClosureCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::redeemTicket(inner) => {
                    <redeemTicketCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::redeemTicketSafe(inner) => {
                    <redeemTicketSafeCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::token(inner) => {
                    <tokenCall as alloy_sol_types::SolCall>::abi_encoded_size(inner)
                }
                Self::tokensReceived(inner) => {
                    <tokensReceivedCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::updateDomainSeparator(inner) => {
                    <updateDomainSeparatorCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
                Self::updateLedgerDomainSeparator(inner) => {
                    <updateLedgerDomainSeparatorCall as alloy_sol_types::SolCall>::abi_encoded_size(
                        inner,
                    )
                }
            }
        }
        #[inline]
        fn abi_encode_raw(&self, out: &mut alloy_sol_types::private::Vec<u8>) {
            match self {
                Self::ERC777_HOOK_FUND_CHANNEL_MULTI_SIZE(inner) => {
                    <ERC777_HOOK_FUND_CHANNEL_MULTI_SIZECall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::ERC777_HOOK_FUND_CHANNEL_SIZE(inner) => {
                    <ERC777_HOOK_FUND_CHANNEL_SIZECall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::LEDGER_VERSION(inner) => {
                    <LEDGER_VERSIONCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::MAX_USED_BALANCE(inner) => {
                    <MAX_USED_BALANCECall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::MIN_USED_BALANCE(inner) => {
                    <MIN_USED_BALANCECall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::TOKENS_RECIPIENT_INTERFACE_HASH(inner) => {
                    <TOKENS_RECIPIENT_INTERFACE_HASHCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::VERSION(inner) => {
                    <VERSIONCall as alloy_sol_types::SolCall>::abi_encode_raw(inner, out)
                }
                Self::_currentBlockTimestamp(inner) => {
                    <_currentBlockTimestampCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::_getChannelId(inner) => {
                    <_getChannelIdCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::_getTicketHash(inner) => {
                    <_getTicketHashCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::_isWinningTicket(inner) => {
                    <_isWinningTicketCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::canImplementInterfaceForAddress(inner) => {
                    <canImplementInterfaceForAddressCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::channels(inner) => {
                    <channelsCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::closeIncomingChannel(inner) => {
                    <closeIncomingChannelCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::closeIncomingChannelSafe(inner) => {
                    <closeIncomingChannelSafeCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::domainSeparator(inner) => {
                    <domainSeparatorCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::finalizeOutgoingChannelClosure(inner) => {
                    <finalizeOutgoingChannelClosureCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::finalizeOutgoingChannelClosureSafe(inner) => {
                    <finalizeOutgoingChannelClosureSafeCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::fundChannel(inner) => {
                    <fundChannelCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::fundChannelSafe(inner) => {
                    <fundChannelSafeCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::initiateOutgoingChannelClosure(inner) => {
                    <initiateOutgoingChannelClosureCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::initiateOutgoingChannelClosureSafe(inner) => {
                    <initiateOutgoingChannelClosureSafeCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::ledgerDomainSeparator(inner) => {
                    <ledgerDomainSeparatorCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::multicall(inner) => {
                    <multicallCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::noticePeriodChannelClosure(inner) => {
                    <noticePeriodChannelClosureCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::redeemTicket(inner) => {
                    <redeemTicketCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::redeemTicketSafe(inner) => {
                    <redeemTicketSafeCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::token(inner) => {
                    <tokenCall as alloy_sol_types::SolCall>::abi_encode_raw(inner, out)
                }
                Self::tokensReceived(inner) => {
                    <tokensReceivedCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::updateDomainSeparator(inner) => {
                    <updateDomainSeparatorCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::updateLedgerDomainSeparator(inner) => {
                    <updateLedgerDomainSeparatorCall as alloy_sol_types::SolCall>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
            }
        }
    }
    ///Container for all the [`HoprChannels`](self) custom errors.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Debug, PartialEq, Eq, Hash)]
    pub enum HoprChannelsErrors {
        #[allow(missing_docs)]
        AlreadyInitialized(AlreadyInitialized),
        #[allow(missing_docs)]
        BalanceExceedsGlobalPerChannelAllowance(BalanceExceedsGlobalPerChannelAllowance),
        #[allow(missing_docs)]
        ContractNotResponsible(ContractNotResponsible),
        #[allow(missing_docs)]
        InsufficientChannelBalance(InsufficientChannelBalance),
        #[allow(missing_docs)]
        InvalidAggregatedTicketInterval(InvalidAggregatedTicketInterval),
        #[allow(missing_docs)]
        InvalidBalance(InvalidBalance),
        #[allow(missing_docs)]
        InvalidCurvePoint(InvalidCurvePoint),
        #[allow(missing_docs)]
        InvalidFieldElement(InvalidFieldElement),
        #[allow(missing_docs)]
        InvalidNoticePeriod(InvalidNoticePeriod),
        #[allow(missing_docs)]
        InvalidPointWitness(InvalidPointWitness),
        #[allow(missing_docs)]
        InvalidSafeAddress(InvalidSafeAddress),
        #[allow(missing_docs)]
        InvalidTicketSignature(InvalidTicketSignature),
        #[allow(missing_docs)]
        InvalidTokenRecipient(InvalidTokenRecipient),
        #[allow(missing_docs)]
        InvalidTokensReceivedUsage(InvalidTokensReceivedUsage),
        #[allow(missing_docs)]
        InvalidVRFProof(InvalidVRFProof),
        #[allow(missing_docs)]
        MultiSigUninitialized(MultiSigUninitialized),
        #[allow(missing_docs)]
        NoticePeriodNotDue(NoticePeriodNotDue),
        #[allow(missing_docs)]
        SourceEqualsDestination(SourceEqualsDestination),
        #[allow(missing_docs)]
        TicketIsNotAWin(TicketIsNotAWin),
        #[allow(missing_docs)]
        TokenTransferFailed(TokenTransferFailed),
        #[allow(missing_docs)]
        WrongChannelState(WrongChannelState),
        #[allow(missing_docs)]
        WrongToken(WrongToken),
        #[allow(missing_docs)]
        ZeroAddress(ZeroAddress),
    }
    #[automatically_derived]
    impl HoprChannelsErrors {
        /// All the selectors of this enum.
        ///
        /// Note that the selectors might not be in the same order as the variants.
        /// No guarantees are made about the order of the selectors.
        ///
        /// Prefer using `SolInterface` methods instead.
        pub const SELECTORS: &'static [[u8; 4usize]] = &[
            [4u8, 92u8, 75u8, 2u8],
            [13u8, 193u8, 73u8, 240u8],
            [58u8, 228u8, 237u8, 107u8],
            [69u8, 74u8, 32u8, 200u8],
            [73u8, 148u8, 99u8, 193u8],
            [105u8, 238u8, 111u8, 40u8],
            [113u8, 100u8, 3u8, 42u8],
            [114u8, 69u8, 74u8, 130u8],
            [142u8, 157u8, 124u8, 94u8],
            [149u8, 253u8, 189u8, 184u8],
            [151u8, 163u8, 174u8, 210u8],
            [160u8, 243u8, 254u8, 234u8],
            [164u8, 243u8, 187u8, 228u8],
            [172u8, 213u8, 168u8, 35u8],
            [177u8, 71u8, 99u8, 108u8],
            [185u8, 196u8, 145u8, 8u8],
            [197u8, 46u8, 62u8, 255u8],
            [205u8, 221u8, 83u8, 86u8],
            [208u8, 220u8, 60u8, 30u8],
            [234u8, 192u8, 211u8, 137u8],
            [237u8, 253u8, 205u8, 152u8],
            [238u8, 131u8, 92u8, 137u8],
            [249u8, 238u8, 145u8, 7u8],
        ];
    }
    #[automatically_derived]
    impl alloy_sol_types::SolInterface for HoprChannelsErrors {
        const NAME: &'static str = "HoprChannelsErrors";
        const MIN_DATA_LENGTH: usize = 0usize;
        const COUNT: usize = 23usize;
        #[inline]
        fn selector(&self) -> [u8; 4] {
            match self {
                Self::AlreadyInitialized(_) => {
                    <AlreadyInitialized as alloy_sol_types::SolError>::SELECTOR
                }
                Self::BalanceExceedsGlobalPerChannelAllowance(_) => {
                    <BalanceExceedsGlobalPerChannelAllowance as alloy_sol_types::SolError>::SELECTOR
                }
                Self::ContractNotResponsible(_) => {
                    <ContractNotResponsible as alloy_sol_types::SolError>::SELECTOR
                }
                Self::InsufficientChannelBalance(_) => {
                    <InsufficientChannelBalance as alloy_sol_types::SolError>::SELECTOR
                }
                Self::InvalidAggregatedTicketInterval(_) => {
                    <InvalidAggregatedTicketInterval as alloy_sol_types::SolError>::SELECTOR
                }
                Self::InvalidBalance(_) => {
                    <InvalidBalance as alloy_sol_types::SolError>::SELECTOR
                }
                Self::InvalidCurvePoint(_) => {
                    <InvalidCurvePoint as alloy_sol_types::SolError>::SELECTOR
                }
                Self::InvalidFieldElement(_) => {
                    <InvalidFieldElement as alloy_sol_types::SolError>::SELECTOR
                }
                Self::InvalidNoticePeriod(_) => {
                    <InvalidNoticePeriod as alloy_sol_types::SolError>::SELECTOR
                }
                Self::InvalidPointWitness(_) => {
                    <InvalidPointWitness as alloy_sol_types::SolError>::SELECTOR
                }
                Self::InvalidSafeAddress(_) => {
                    <InvalidSafeAddress as alloy_sol_types::SolError>::SELECTOR
                }
                Self::InvalidTicketSignature(_) => {
                    <InvalidTicketSignature as alloy_sol_types::SolError>::SELECTOR
                }
                Self::InvalidTokenRecipient(_) => {
                    <InvalidTokenRecipient as alloy_sol_types::SolError>::SELECTOR
                }
                Self::InvalidTokensReceivedUsage(_) => {
                    <InvalidTokensReceivedUsage as alloy_sol_types::SolError>::SELECTOR
                }
                Self::InvalidVRFProof(_) => {
                    <InvalidVRFProof as alloy_sol_types::SolError>::SELECTOR
                }
                Self::MultiSigUninitialized(_) => {
                    <MultiSigUninitialized as alloy_sol_types::SolError>::SELECTOR
                }
                Self::NoticePeriodNotDue(_) => {
                    <NoticePeriodNotDue as alloy_sol_types::SolError>::SELECTOR
                }
                Self::SourceEqualsDestination(_) => {
                    <SourceEqualsDestination as alloy_sol_types::SolError>::SELECTOR
                }
                Self::TicketIsNotAWin(_) => {
                    <TicketIsNotAWin as alloy_sol_types::SolError>::SELECTOR
                }
                Self::TokenTransferFailed(_) => {
                    <TokenTransferFailed as alloy_sol_types::SolError>::SELECTOR
                }
                Self::WrongChannelState(_) => {
                    <WrongChannelState as alloy_sol_types::SolError>::SELECTOR
                }
                Self::WrongToken(_) => {
                    <WrongToken as alloy_sol_types::SolError>::SELECTOR
                }
                Self::ZeroAddress(_) => {
                    <ZeroAddress as alloy_sol_types::SolError>::SELECTOR
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
            ) -> alloy_sol_types::Result<HoprChannelsErrors>] = &[
                {
                    fn TokenTransferFailed(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <TokenTransferFailed as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsErrors::TokenTransferFailed)
                    }
                    TokenTransferFailed
                },
                {
                    fn AlreadyInitialized(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <AlreadyInitialized as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsErrors::AlreadyInitialized)
                    }
                    AlreadyInitialized
                },
                {
                    fn InvalidFieldElement(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <InvalidFieldElement as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsErrors::InvalidFieldElement)
                    }
                    InvalidFieldElement
                },
                {
                    fn MultiSigUninitialized(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <MultiSigUninitialized as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsErrors::MultiSigUninitialized)
                    }
                    MultiSigUninitialized
                },
                {
                    fn WrongChannelState(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <WrongChannelState as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsErrors::WrongChannelState)
                    }
                    WrongChannelState
                },
                {
                    fn InvalidTokensReceivedUsage(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <InvalidTokensReceivedUsage as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsErrors::InvalidTokensReceivedUsage)
                    }
                    InvalidTokensReceivedUsage
                },
                {
                    fn NoticePeriodNotDue(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <NoticePeriodNotDue as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsErrors::NoticePeriodNotDue)
                    }
                    NoticePeriodNotDue
                },
                {
                    fn InvalidCurvePoint(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <InvalidCurvePoint as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsErrors::InvalidCurvePoint)
                    }
                    InvalidCurvePoint
                },
                {
                    fn InvalidSafeAddress(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <InvalidSafeAddress as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsErrors::InvalidSafeAddress)
                    }
                    InvalidSafeAddress
                },
                {
                    fn InvalidVRFProof(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <InvalidVRFProof as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsErrors::InvalidVRFProof)
                    }
                    InvalidVRFProof
                },
                {
                    fn SourceEqualsDestination(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <SourceEqualsDestination as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsErrors::SourceEqualsDestination)
                    }
                    SourceEqualsDestination
                },
                {
                    fn WrongToken(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <WrongToken as alloy_sol_types::SolError>::abi_decode_raw(data)
                            .map(HoprChannelsErrors::WrongToken)
                    }
                    WrongToken
                },
                {
                    fn BalanceExceedsGlobalPerChannelAllowance(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <BalanceExceedsGlobalPerChannelAllowance as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(
                                HoprChannelsErrors::BalanceExceedsGlobalPerChannelAllowance,
                            )
                    }
                    BalanceExceedsGlobalPerChannelAllowance
                },
                {
                    fn ContractNotResponsible(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <ContractNotResponsible as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsErrors::ContractNotResponsible)
                    }
                    ContractNotResponsible
                },
                {
                    fn InsufficientChannelBalance(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <InsufficientChannelBalance as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsErrors::InsufficientChannelBalance)
                    }
                    InsufficientChannelBalance
                },
                {
                    fn InvalidTokenRecipient(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <InvalidTokenRecipient as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsErrors::InvalidTokenRecipient)
                    }
                    InvalidTokenRecipient
                },
                {
                    fn InvalidBalance(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <InvalidBalance as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsErrors::InvalidBalance)
                    }
                    InvalidBalance
                },
                {
                    fn InvalidTicketSignature(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <InvalidTicketSignature as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsErrors::InvalidTicketSignature)
                    }
                    InvalidTicketSignature
                },
                {
                    fn InvalidAggregatedTicketInterval(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <InvalidAggregatedTicketInterval as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsErrors::InvalidAggregatedTicketInterval)
                    }
                    InvalidAggregatedTicketInterval
                },
                {
                    fn ZeroAddress(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <ZeroAddress as alloy_sol_types::SolError>::abi_decode_raw(data)
                            .map(HoprChannelsErrors::ZeroAddress)
                    }
                    ZeroAddress
                },
                {
                    fn InvalidPointWitness(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <InvalidPointWitness as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsErrors::InvalidPointWitness)
                    }
                    InvalidPointWitness
                },
                {
                    fn TicketIsNotAWin(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <TicketIsNotAWin as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsErrors::TicketIsNotAWin)
                    }
                    TicketIsNotAWin
                },
                {
                    fn InvalidNoticePeriod(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <InvalidNoticePeriod as alloy_sol_types::SolError>::abi_decode_raw(
                                data,
                            )
                            .map(HoprChannelsErrors::InvalidNoticePeriod)
                    }
                    InvalidNoticePeriod
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
            ) -> alloy_sol_types::Result<HoprChannelsErrors>] = &[
                {
                    fn TokenTransferFailed(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <TokenTransferFailed as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsErrors::TokenTransferFailed)
                    }
                    TokenTransferFailed
                },
                {
                    fn AlreadyInitialized(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <AlreadyInitialized as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsErrors::AlreadyInitialized)
                    }
                    AlreadyInitialized
                },
                {
                    fn InvalidFieldElement(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <InvalidFieldElement as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsErrors::InvalidFieldElement)
                    }
                    InvalidFieldElement
                },
                {
                    fn MultiSigUninitialized(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <MultiSigUninitialized as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsErrors::MultiSigUninitialized)
                    }
                    MultiSigUninitialized
                },
                {
                    fn WrongChannelState(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <WrongChannelState as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsErrors::WrongChannelState)
                    }
                    WrongChannelState
                },
                {
                    fn InvalidTokensReceivedUsage(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <InvalidTokensReceivedUsage as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsErrors::InvalidTokensReceivedUsage)
                    }
                    InvalidTokensReceivedUsage
                },
                {
                    fn NoticePeriodNotDue(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <NoticePeriodNotDue as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsErrors::NoticePeriodNotDue)
                    }
                    NoticePeriodNotDue
                },
                {
                    fn InvalidCurvePoint(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <InvalidCurvePoint as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsErrors::InvalidCurvePoint)
                    }
                    InvalidCurvePoint
                },
                {
                    fn InvalidSafeAddress(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <InvalidSafeAddress as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsErrors::InvalidSafeAddress)
                    }
                    InvalidSafeAddress
                },
                {
                    fn InvalidVRFProof(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <InvalidVRFProof as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsErrors::InvalidVRFProof)
                    }
                    InvalidVRFProof
                },
                {
                    fn SourceEqualsDestination(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <SourceEqualsDestination as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsErrors::SourceEqualsDestination)
                    }
                    SourceEqualsDestination
                },
                {
                    fn WrongToken(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <WrongToken as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsErrors::WrongToken)
                    }
                    WrongToken
                },
                {
                    fn BalanceExceedsGlobalPerChannelAllowance(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <BalanceExceedsGlobalPerChannelAllowance as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(
                                HoprChannelsErrors::BalanceExceedsGlobalPerChannelAllowance,
                            )
                    }
                    BalanceExceedsGlobalPerChannelAllowance
                },
                {
                    fn ContractNotResponsible(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <ContractNotResponsible as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsErrors::ContractNotResponsible)
                    }
                    ContractNotResponsible
                },
                {
                    fn InsufficientChannelBalance(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <InsufficientChannelBalance as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsErrors::InsufficientChannelBalance)
                    }
                    InsufficientChannelBalance
                },
                {
                    fn InvalidTokenRecipient(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <InvalidTokenRecipient as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsErrors::InvalidTokenRecipient)
                    }
                    InvalidTokenRecipient
                },
                {
                    fn InvalidBalance(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <InvalidBalance as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsErrors::InvalidBalance)
                    }
                    InvalidBalance
                },
                {
                    fn InvalidTicketSignature(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <InvalidTicketSignature as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsErrors::InvalidTicketSignature)
                    }
                    InvalidTicketSignature
                },
                {
                    fn InvalidAggregatedTicketInterval(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <InvalidAggregatedTicketInterval as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsErrors::InvalidAggregatedTicketInterval)
                    }
                    InvalidAggregatedTicketInterval
                },
                {
                    fn ZeroAddress(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <ZeroAddress as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsErrors::ZeroAddress)
                    }
                    ZeroAddress
                },
                {
                    fn InvalidPointWitness(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <InvalidPointWitness as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsErrors::InvalidPointWitness)
                    }
                    InvalidPointWitness
                },
                {
                    fn TicketIsNotAWin(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <TicketIsNotAWin as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsErrors::TicketIsNotAWin)
                    }
                    TicketIsNotAWin
                },
                {
                    fn InvalidNoticePeriod(
                        data: &[u8],
                    ) -> alloy_sol_types::Result<HoprChannelsErrors> {
                        <InvalidNoticePeriod as alloy_sol_types::SolError>::abi_decode_raw_validate(
                                data,
                            )
                            .map(HoprChannelsErrors::InvalidNoticePeriod)
                    }
                    InvalidNoticePeriod
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
                Self::AlreadyInitialized(inner) => {
                    <AlreadyInitialized as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::BalanceExceedsGlobalPerChannelAllowance(inner) => {
                    <BalanceExceedsGlobalPerChannelAllowance as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::ContractNotResponsible(inner) => {
                    <ContractNotResponsible as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::InsufficientChannelBalance(inner) => {
                    <InsufficientChannelBalance as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::InvalidAggregatedTicketInterval(inner) => {
                    <InvalidAggregatedTicketInterval as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::InvalidBalance(inner) => {
                    <InvalidBalance as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::InvalidCurvePoint(inner) => {
                    <InvalidCurvePoint as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::InvalidFieldElement(inner) => {
                    <InvalidFieldElement as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::InvalidNoticePeriod(inner) => {
                    <InvalidNoticePeriod as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::InvalidPointWitness(inner) => {
                    <InvalidPointWitness as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::InvalidSafeAddress(inner) => {
                    <InvalidSafeAddress as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::InvalidTicketSignature(inner) => {
                    <InvalidTicketSignature as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::InvalidTokenRecipient(inner) => {
                    <InvalidTokenRecipient as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::InvalidTokensReceivedUsage(inner) => {
                    <InvalidTokensReceivedUsage as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::InvalidVRFProof(inner) => {
                    <InvalidVRFProof as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::MultiSigUninitialized(inner) => {
                    <MultiSigUninitialized as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::NoticePeriodNotDue(inner) => {
                    <NoticePeriodNotDue as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::SourceEqualsDestination(inner) => {
                    <SourceEqualsDestination as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::TicketIsNotAWin(inner) => {
                    <TicketIsNotAWin as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::TokenTransferFailed(inner) => {
                    <TokenTransferFailed as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::WrongChannelState(inner) => {
                    <WrongChannelState as alloy_sol_types::SolError>::abi_encoded_size(
                        inner,
                    )
                }
                Self::WrongToken(inner) => {
                    <WrongToken as alloy_sol_types::SolError>::abi_encoded_size(inner)
                }
                Self::ZeroAddress(inner) => {
                    <ZeroAddress as alloy_sol_types::SolError>::abi_encoded_size(inner)
                }
            }
        }
        #[inline]
        fn abi_encode_raw(&self, out: &mut alloy_sol_types::private::Vec<u8>) {
            match self {
                Self::AlreadyInitialized(inner) => {
                    <AlreadyInitialized as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::BalanceExceedsGlobalPerChannelAllowance(inner) => {
                    <BalanceExceedsGlobalPerChannelAllowance as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::ContractNotResponsible(inner) => {
                    <ContractNotResponsible as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::InsufficientChannelBalance(inner) => {
                    <InsufficientChannelBalance as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::InvalidAggregatedTicketInterval(inner) => {
                    <InvalidAggregatedTicketInterval as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::InvalidBalance(inner) => {
                    <InvalidBalance as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::InvalidCurvePoint(inner) => {
                    <InvalidCurvePoint as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::InvalidFieldElement(inner) => {
                    <InvalidFieldElement as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::InvalidNoticePeriod(inner) => {
                    <InvalidNoticePeriod as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::InvalidPointWitness(inner) => {
                    <InvalidPointWitness as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::InvalidSafeAddress(inner) => {
                    <InvalidSafeAddress as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::InvalidTicketSignature(inner) => {
                    <InvalidTicketSignature as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::InvalidTokenRecipient(inner) => {
                    <InvalidTokenRecipient as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::InvalidTokensReceivedUsage(inner) => {
                    <InvalidTokensReceivedUsage as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::InvalidVRFProof(inner) => {
                    <InvalidVRFProof as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::MultiSigUninitialized(inner) => {
                    <MultiSigUninitialized as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::NoticePeriodNotDue(inner) => {
                    <NoticePeriodNotDue as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::SourceEqualsDestination(inner) => {
                    <SourceEqualsDestination as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::TicketIsNotAWin(inner) => {
                    <TicketIsNotAWin as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::TokenTransferFailed(inner) => {
                    <TokenTransferFailed as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::WrongChannelState(inner) => {
                    <WrongChannelState as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
                Self::WrongToken(inner) => {
                    <WrongToken as alloy_sol_types::SolError>::abi_encode_raw(inner, out)
                }
                Self::ZeroAddress(inner) => {
                    <ZeroAddress as alloy_sol_types::SolError>::abi_encode_raw(
                        inner,
                        out,
                    )
                }
            }
        }
    }
    ///Container for all the [`HoprChannels`](self) events.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Debug, PartialEq, Eq, Hash)]
    pub enum HoprChannelsEvents {
        #[allow(missing_docs)]
        ChannelBalanceDecreased(ChannelBalanceDecreased),
        #[allow(missing_docs)]
        ChannelBalanceIncreased(ChannelBalanceIncreased),
        #[allow(missing_docs)]
        ChannelClosed(ChannelClosed),
        #[allow(missing_docs)]
        ChannelOpened(ChannelOpened),
        #[allow(missing_docs)]
        DomainSeparatorUpdated(DomainSeparatorUpdated),
        #[allow(missing_docs)]
        LedgerDomainSeparatorUpdated(LedgerDomainSeparatorUpdated),
        #[allow(missing_docs)]
        OutgoingChannelClosureInitiated(OutgoingChannelClosureInitiated),
        #[allow(missing_docs)]
        TicketRedeemed(TicketRedeemed),
    }
    #[automatically_derived]
    impl HoprChannelsEvents {
        /// All the selectors of this enum.
        ///
        /// Note that the selectors might not be in the same order as the variants.
        /// No guarantees are made about the order of the selectors.
        ///
        /// Prefer using `SolInterface` methods instead.
        pub const SELECTORS: &'static [[u8; 32usize]] = &[
            [
                7u8, 181u8, 201u8, 80u8, 89u8, 127u8, 195u8, 190u8, 217u8, 46u8, 42u8,
                211u8, 127u8, 168u8, 79u8, 112u8, 22u8, 85u8, 172u8, 179u8, 114u8, 152u8,
                46u8, 72u8, 111u8, 95u8, 173u8, 54u8, 7u8, 240u8, 74u8, 92u8,
            ],
            [
                34u8, 226u8, 164u8, 34u8, 168u8, 134u8, 6u8, 86u8, 163u8, 163u8, 60u8,
                250u8, 29u8, 175u8, 119u8, 30u8, 118u8, 121u8, 140u8, 229u8, 100u8,
                151u8, 71u8, 149u8, 114u8, 53u8, 2u8, 93u8, 225u8, 46u8, 11u8, 36u8,
            ],
            [
                95u8, 161u8, 114u8, 70u8, 211u8, 165u8, 214u8, 141u8, 66u8, 186u8, 169u8,
                76u8, 222u8, 51u8, 4u8, 33u8, 128u8, 183u8, 131u8, 163u8, 153u8, 192u8,
                43u8, 246u8, 58u8, 194u8, 7u8, 110u8, 15u8, 112u8, 135u8, 56u8,
            ],
            [
                113u8, 101u8, 226u8, 235u8, 199u8, 206u8, 53u8, 204u8, 152u8, 203u8,
                118u8, 102u8, 249u8, 148u8, 91u8, 54u8, 23u8, 243u8, 243u8, 99u8, 38u8,
                183u8, 109u8, 24u8, 147u8, 123u8, 165u8, 254u8, 207u8, 24u8, 115u8, 154u8,
            ],
            [
                119u8, 31u8, 82u8, 64u8, 174u8, 95u8, 216u8, 167u8, 100u8, 13u8, 63u8,
                184u8, 47u8, 167u8, 10u8, 171u8, 47u8, 177u8, 219u8, 243u8, 95u8, 46u8,
                244u8, 100u8, 248u8, 80u8, 153u8, 70u8, 113u8, 118u8, 100u8, 197u8,
            ],
            [
                164u8, 63u8, 173u8, 131u8, 146u8, 15u8, 208u8, 148u8, 69u8, 133u8, 94u8,
                133u8, 78u8, 115u8, 201u8, 197u8, 50u8, 225u8, 116u8, 2u8, 201u8, 206u8,
                176u8, 153u8, 147u8, 162u8, 57u8, 40u8, 67u8, 165u8, 189u8, 185u8,
            ],
            [
                206u8, 234u8, 178u8, 238u8, 249u8, 152u8, 193u8, 127u8, 233u8, 111u8,
                48u8, 248u8, 63u8, 191u8, 60u8, 85u8, 252u8, 80u8, 71u8, 246u8, 228u8,
                12u8, 85u8, 160u8, 207u8, 114u8, 210u8, 54u8, 233u8, 210u8, 186u8, 114u8,
            ],
            [
                221u8, 144u8, 249u8, 56u8, 35u8, 3u8, 53u8, 229u8, 157u8, 201u8, 37u8,
                197u8, 126u8, 203u8, 14u8, 39u8, 162u8, 140u8, 45u8, 135u8, 53u8, 110u8,
                49u8, 240u8, 12u8, 213u8, 85u8, 74u8, 189u8, 108u8, 27u8, 45u8,
            ],
        ];
    }
    #[automatically_derived]
    impl alloy_sol_types::SolEventInterface for HoprChannelsEvents {
        const NAME: &'static str = "HoprChannelsEvents";
        const COUNT: usize = 8usize;
        fn decode_raw_log(
            topics: &[alloy_sol_types::Word],
            data: &[u8],
        ) -> alloy_sol_types::Result<Self> {
            match topics.first().copied() {
                Some(
                    <ChannelBalanceDecreased as alloy_sol_types::SolEvent>::SIGNATURE_HASH,
                ) => {
                    <ChannelBalanceDecreased as alloy_sol_types::SolEvent>::decode_raw_log(
                            topics,
                            data,
                        )
                        .map(Self::ChannelBalanceDecreased)
                }
                Some(
                    <ChannelBalanceIncreased as alloy_sol_types::SolEvent>::SIGNATURE_HASH,
                ) => {
                    <ChannelBalanceIncreased as alloy_sol_types::SolEvent>::decode_raw_log(
                            topics,
                            data,
                        )
                        .map(Self::ChannelBalanceIncreased)
                }
                Some(<ChannelClosed as alloy_sol_types::SolEvent>::SIGNATURE_HASH) => {
                    <ChannelClosed as alloy_sol_types::SolEvent>::decode_raw_log(
                            topics,
                            data,
                        )
                        .map(Self::ChannelClosed)
                }
                Some(<ChannelOpened as alloy_sol_types::SolEvent>::SIGNATURE_HASH) => {
                    <ChannelOpened as alloy_sol_types::SolEvent>::decode_raw_log(
                            topics,
                            data,
                        )
                        .map(Self::ChannelOpened)
                }
                Some(
                    <DomainSeparatorUpdated as alloy_sol_types::SolEvent>::SIGNATURE_HASH,
                ) => {
                    <DomainSeparatorUpdated as alloy_sol_types::SolEvent>::decode_raw_log(
                            topics,
                            data,
                        )
                        .map(Self::DomainSeparatorUpdated)
                }
                Some(
                    <LedgerDomainSeparatorUpdated as alloy_sol_types::SolEvent>::SIGNATURE_HASH,
                ) => {
                    <LedgerDomainSeparatorUpdated as alloy_sol_types::SolEvent>::decode_raw_log(
                            topics,
                            data,
                        )
                        .map(Self::LedgerDomainSeparatorUpdated)
                }
                Some(
                    <OutgoingChannelClosureInitiated as alloy_sol_types::SolEvent>::SIGNATURE_HASH,
                ) => {
                    <OutgoingChannelClosureInitiated as alloy_sol_types::SolEvent>::decode_raw_log(
                            topics,
                            data,
                        )
                        .map(Self::OutgoingChannelClosureInitiated)
                }
                Some(<TicketRedeemed as alloy_sol_types::SolEvent>::SIGNATURE_HASH) => {
                    <TicketRedeemed as alloy_sol_types::SolEvent>::decode_raw_log(
                            topics,
                            data,
                        )
                        .map(Self::TicketRedeemed)
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
    impl alloy_sol_types::private::IntoLogData for HoprChannelsEvents {
        fn to_log_data(&self) -> alloy_sol_types::private::LogData {
            match self {
                Self::ChannelBalanceDecreased(inner) => {
                    alloy_sol_types::private::IntoLogData::to_log_data(inner)
                }
                Self::ChannelBalanceIncreased(inner) => {
                    alloy_sol_types::private::IntoLogData::to_log_data(inner)
                }
                Self::ChannelClosed(inner) => {
                    alloy_sol_types::private::IntoLogData::to_log_data(inner)
                }
                Self::ChannelOpened(inner) => {
                    alloy_sol_types::private::IntoLogData::to_log_data(inner)
                }
                Self::DomainSeparatorUpdated(inner) => {
                    alloy_sol_types::private::IntoLogData::to_log_data(inner)
                }
                Self::LedgerDomainSeparatorUpdated(inner) => {
                    alloy_sol_types::private::IntoLogData::to_log_data(inner)
                }
                Self::OutgoingChannelClosureInitiated(inner) => {
                    alloy_sol_types::private::IntoLogData::to_log_data(inner)
                }
                Self::TicketRedeemed(inner) => {
                    alloy_sol_types::private::IntoLogData::to_log_data(inner)
                }
            }
        }
        fn into_log_data(self) -> alloy_sol_types::private::LogData {
            match self {
                Self::ChannelBalanceDecreased(inner) => {
                    alloy_sol_types::private::IntoLogData::into_log_data(inner)
                }
                Self::ChannelBalanceIncreased(inner) => {
                    alloy_sol_types::private::IntoLogData::into_log_data(inner)
                }
                Self::ChannelClosed(inner) => {
                    alloy_sol_types::private::IntoLogData::into_log_data(inner)
                }
                Self::ChannelOpened(inner) => {
                    alloy_sol_types::private::IntoLogData::into_log_data(inner)
                }
                Self::DomainSeparatorUpdated(inner) => {
                    alloy_sol_types::private::IntoLogData::into_log_data(inner)
                }
                Self::LedgerDomainSeparatorUpdated(inner) => {
                    alloy_sol_types::private::IntoLogData::into_log_data(inner)
                }
                Self::OutgoingChannelClosureInitiated(inner) => {
                    alloy_sol_types::private::IntoLogData::into_log_data(inner)
                }
                Self::TicketRedeemed(inner) => {
                    alloy_sol_types::private::IntoLogData::into_log_data(inner)
                }
            }
        }
    }
    use alloy::contract as alloy_contract;
    /**Creates a new wrapper around an on-chain [`HoprChannels`](self) contract instance.

See the [wrapper's documentation](`HoprChannelsInstance`) for more details.*/
    #[inline]
    pub const fn new<
        P: alloy_contract::private::Provider<N>,
        N: alloy_contract::private::Network,
    >(
        address: alloy_sol_types::private::Address,
        provider: P,
    ) -> HoprChannelsInstance<P, N> {
        HoprChannelsInstance::<P, N>::new(address, provider)
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
        _token: alloy::sol_types::private::Address,
        _noticePeriodChannelClosure: <Timestamp as alloy::sol_types::SolType>::RustType,
        _safeRegistry: alloy::sol_types::private::Address,
    ) -> impl ::core::future::Future<
        Output = alloy_contract::Result<HoprChannelsInstance<P, N>>,
    > {
        HoprChannelsInstance::<
            P,
            N,
        >::deploy(provider, _token, _noticePeriodChannelClosure, _safeRegistry)
    }
    /**Creates a `RawCallBuilder` for deploying this contract using the given `provider`
and constructor arguments, if any.

This is a simple wrapper around creating a `RawCallBuilder` with the data set to
the bytecode concatenated with the constructor's ABI-encoded arguments.*/
    #[inline]
    pub fn deploy_builder<
        P: alloy_contract::private::Provider<N>,
        N: alloy_contract::private::Network,
    >(
        provider: P,
        _token: alloy::sol_types::private::Address,
        _noticePeriodChannelClosure: <Timestamp as alloy::sol_types::SolType>::RustType,
        _safeRegistry: alloy::sol_types::private::Address,
    ) -> alloy_contract::RawCallBuilder<P, N> {
        HoprChannelsInstance::<
            P,
            N,
        >::deploy_builder(provider, _token, _noticePeriodChannelClosure, _safeRegistry)
    }
    /**A [`HoprChannels`](self) instance.

Contains type-safe methods for interacting with an on-chain instance of the
[`HoprChannels`](self) contract located at a given `address`, using a given
provider `P`.

If the contract bytecode is available (see the [`sol!`](alloy_sol_types::sol!)
documentation on how to provide it), the `deploy` and `deploy_builder` methods can
be used to deploy a new instance of the contract.

See the [module-level documentation](self) for all the available methods.*/
    #[derive(Clone)]
    pub struct HoprChannelsInstance<P, N = alloy_contract::private::Ethereum> {
        address: alloy_sol_types::private::Address,
        provider: P,
        _network: ::core::marker::PhantomData<N>,
    }
    #[automatically_derived]
    impl<P, N> ::core::fmt::Debug for HoprChannelsInstance<P, N> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple("HoprChannelsInstance").field(&self.address).finish()
        }
    }
    /// Instantiation and getters/setters.
    #[automatically_derived]
    impl<
        P: alloy_contract::private::Provider<N>,
        N: alloy_contract::private::Network,
    > HoprChannelsInstance<P, N> {
        /**Creates a new wrapper around an on-chain [`HoprChannels`](self) contract instance.

See the [wrapper's documentation](`HoprChannelsInstance`) for more details.*/
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
            _token: alloy::sol_types::private::Address,
            _noticePeriodChannelClosure: <Timestamp as alloy::sol_types::SolType>::RustType,
            _safeRegistry: alloy::sol_types::private::Address,
        ) -> alloy_contract::Result<HoprChannelsInstance<P, N>> {
            let call_builder = Self::deploy_builder(
                provider,
                _token,
                _noticePeriodChannelClosure,
                _safeRegistry,
            );
            let contract_address = call_builder.deploy().await?;
            Ok(Self::new(contract_address, call_builder.provider))
        }
        /**Creates a `RawCallBuilder` for deploying this contract using the given `provider`
and constructor arguments, if any.

This is a simple wrapper around creating a `RawCallBuilder` with the data set to
the bytecode concatenated with the constructor's ABI-encoded arguments.*/
        #[inline]
        pub fn deploy_builder(
            provider: P,
            _token: alloy::sol_types::private::Address,
            _noticePeriodChannelClosure: <Timestamp as alloy::sol_types::SolType>::RustType,
            _safeRegistry: alloy::sol_types::private::Address,
        ) -> alloy_contract::RawCallBuilder<P, N> {
            alloy_contract::RawCallBuilder::new_raw_deploy(
                provider,
                [
                    &BYTECODE[..],
                    &alloy_sol_types::SolConstructor::abi_encode(
                        &constructorCall {
                            _token,
                            _noticePeriodChannelClosure,
                            _safeRegistry,
                        },
                    )[..],
                ]
                    .concat()
                    .into(),
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
    impl<P: ::core::clone::Clone, N> HoprChannelsInstance<&P, N> {
        /// Clones the provider and returns a new instance with the cloned provider.
        #[inline]
        pub fn with_cloned_provider(self) -> HoprChannelsInstance<P, N> {
            HoprChannelsInstance {
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
    > HoprChannelsInstance<P, N> {
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
        ///Creates a new call builder for the [`ERC777_HOOK_FUND_CHANNEL_MULTI_SIZE`] function.
        pub fn ERC777_HOOK_FUND_CHANNEL_MULTI_SIZE(
            &self,
        ) -> alloy_contract::SolCallBuilder<
            &P,
            ERC777_HOOK_FUND_CHANNEL_MULTI_SIZECall,
            N,
        > {
            self.call_builder(&ERC777_HOOK_FUND_CHANNEL_MULTI_SIZECall)
        }
        ///Creates a new call builder for the [`ERC777_HOOK_FUND_CHANNEL_SIZE`] function.
        pub fn ERC777_HOOK_FUND_CHANNEL_SIZE(
            &self,
        ) -> alloy_contract::SolCallBuilder<&P, ERC777_HOOK_FUND_CHANNEL_SIZECall, N> {
            self.call_builder(&ERC777_HOOK_FUND_CHANNEL_SIZECall)
        }
        ///Creates a new call builder for the [`LEDGER_VERSION`] function.
        pub fn LEDGER_VERSION(
            &self,
        ) -> alloy_contract::SolCallBuilder<&P, LEDGER_VERSIONCall, N> {
            self.call_builder(&LEDGER_VERSIONCall)
        }
        ///Creates a new call builder for the [`MAX_USED_BALANCE`] function.
        pub fn MAX_USED_BALANCE(
            &self,
        ) -> alloy_contract::SolCallBuilder<&P, MAX_USED_BALANCECall, N> {
            self.call_builder(&MAX_USED_BALANCECall)
        }
        ///Creates a new call builder for the [`MIN_USED_BALANCE`] function.
        pub fn MIN_USED_BALANCE(
            &self,
        ) -> alloy_contract::SolCallBuilder<&P, MIN_USED_BALANCECall, N> {
            self.call_builder(&MIN_USED_BALANCECall)
        }
        ///Creates a new call builder for the [`TOKENS_RECIPIENT_INTERFACE_HASH`] function.
        pub fn TOKENS_RECIPIENT_INTERFACE_HASH(
            &self,
        ) -> alloy_contract::SolCallBuilder<&P, TOKENS_RECIPIENT_INTERFACE_HASHCall, N> {
            self.call_builder(&TOKENS_RECIPIENT_INTERFACE_HASHCall)
        }
        ///Creates a new call builder for the [`VERSION`] function.
        pub fn VERSION(&self) -> alloy_contract::SolCallBuilder<&P, VERSIONCall, N> {
            self.call_builder(&VERSIONCall)
        }
        ///Creates a new call builder for the [`_currentBlockTimestamp`] function.
        pub fn _currentBlockTimestamp(
            &self,
        ) -> alloy_contract::SolCallBuilder<&P, _currentBlockTimestampCall, N> {
            self.call_builder(&_currentBlockTimestampCall)
        }
        ///Creates a new call builder for the [`_getChannelId`] function.
        pub fn _getChannelId(
            &self,
            source: alloy::sol_types::private::Address,
            destination: alloy::sol_types::private::Address,
        ) -> alloy_contract::SolCallBuilder<&P, _getChannelIdCall, N> {
            self.call_builder(
                &_getChannelIdCall {
                    source,
                    destination,
                },
            )
        }
        ///Creates a new call builder for the [`_getTicketHash`] function.
        pub fn _getTicketHash(
            &self,
            redeemable: <RedeemableTicket as alloy::sol_types::SolType>::RustType,
        ) -> alloy_contract::SolCallBuilder<&P, _getTicketHashCall, N> {
            self.call_builder(&_getTicketHashCall { redeemable })
        }
        ///Creates a new call builder for the [`_isWinningTicket`] function.
        pub fn _isWinningTicket(
            &self,
            ticketHash: alloy::sol_types::private::FixedBytes<32>,
            redeemable: <RedeemableTicket as alloy::sol_types::SolType>::RustType,
            params: <HoprCrypto::VRFParameters as alloy::sol_types::SolType>::RustType,
        ) -> alloy_contract::SolCallBuilder<&P, _isWinningTicketCall, N> {
            self.call_builder(
                &_isWinningTicketCall {
                    ticketHash,
                    redeemable,
                    params,
                },
            )
        }
        ///Creates a new call builder for the [`canImplementInterfaceForAddress`] function.
        pub fn canImplementInterfaceForAddress(
            &self,
            interfaceHash: alloy::sol_types::private::FixedBytes<32>,
            account: alloy::sol_types::private::Address,
        ) -> alloy_contract::SolCallBuilder<&P, canImplementInterfaceForAddressCall, N> {
            self.call_builder(
                &canImplementInterfaceForAddressCall {
                    interfaceHash,
                    account,
                },
            )
        }
        ///Creates a new call builder for the [`channels`] function.
        pub fn channels(
            &self,
            _0: alloy::sol_types::private::FixedBytes<32>,
        ) -> alloy_contract::SolCallBuilder<&P, channelsCall, N> {
            self.call_builder(&channelsCall(_0))
        }
        ///Creates a new call builder for the [`closeIncomingChannel`] function.
        pub fn closeIncomingChannel(
            &self,
            source: alloy::sol_types::private::Address,
        ) -> alloy_contract::SolCallBuilder<&P, closeIncomingChannelCall, N> {
            self.call_builder(&closeIncomingChannelCall { source })
        }
        ///Creates a new call builder for the [`closeIncomingChannelSafe`] function.
        pub fn closeIncomingChannelSafe(
            &self,
            selfAddress: alloy::sol_types::private::Address,
            source: alloy::sol_types::private::Address,
        ) -> alloy_contract::SolCallBuilder<&P, closeIncomingChannelSafeCall, N> {
            self.call_builder(
                &closeIncomingChannelSafeCall {
                    selfAddress,
                    source,
                },
            )
        }
        ///Creates a new call builder for the [`domainSeparator`] function.
        pub fn domainSeparator(
            &self,
        ) -> alloy_contract::SolCallBuilder<&P, domainSeparatorCall, N> {
            self.call_builder(&domainSeparatorCall)
        }
        ///Creates a new call builder for the [`finalizeOutgoingChannelClosure`] function.
        pub fn finalizeOutgoingChannelClosure(
            &self,
            destination: alloy::sol_types::private::Address,
        ) -> alloy_contract::SolCallBuilder<&P, finalizeOutgoingChannelClosureCall, N> {
            self.call_builder(
                &finalizeOutgoingChannelClosureCall {
                    destination,
                },
            )
        }
        ///Creates a new call builder for the [`finalizeOutgoingChannelClosureSafe`] function.
        pub fn finalizeOutgoingChannelClosureSafe(
            &self,
            selfAddress: alloy::sol_types::private::Address,
            destination: alloy::sol_types::private::Address,
        ) -> alloy_contract::SolCallBuilder<
            &P,
            finalizeOutgoingChannelClosureSafeCall,
            N,
        > {
            self.call_builder(
                &finalizeOutgoingChannelClosureSafeCall {
                    selfAddress,
                    destination,
                },
            )
        }
        ///Creates a new call builder for the [`fundChannel`] function.
        pub fn fundChannel(
            &self,
            account: alloy::sol_types::private::Address,
            amount: <Balance as alloy::sol_types::SolType>::RustType,
        ) -> alloy_contract::SolCallBuilder<&P, fundChannelCall, N> {
            self.call_builder(&fundChannelCall { account, amount })
        }
        ///Creates a new call builder for the [`fundChannelSafe`] function.
        pub fn fundChannelSafe(
            &self,
            selfAddress: alloy::sol_types::private::Address,
            account: alloy::sol_types::private::Address,
            amount: <Balance as alloy::sol_types::SolType>::RustType,
        ) -> alloy_contract::SolCallBuilder<&P, fundChannelSafeCall, N> {
            self.call_builder(
                &fundChannelSafeCall {
                    selfAddress,
                    account,
                    amount,
                },
            )
        }
        ///Creates a new call builder for the [`initiateOutgoingChannelClosure`] function.
        pub fn initiateOutgoingChannelClosure(
            &self,
            destination: alloy::sol_types::private::Address,
        ) -> alloy_contract::SolCallBuilder<&P, initiateOutgoingChannelClosureCall, N> {
            self.call_builder(
                &initiateOutgoingChannelClosureCall {
                    destination,
                },
            )
        }
        ///Creates a new call builder for the [`initiateOutgoingChannelClosureSafe`] function.
        pub fn initiateOutgoingChannelClosureSafe(
            &self,
            selfAddress: alloy::sol_types::private::Address,
            destination: alloy::sol_types::private::Address,
        ) -> alloy_contract::SolCallBuilder<
            &P,
            initiateOutgoingChannelClosureSafeCall,
            N,
        > {
            self.call_builder(
                &initiateOutgoingChannelClosureSafeCall {
                    selfAddress,
                    destination,
                },
            )
        }
        ///Creates a new call builder for the [`ledgerDomainSeparator`] function.
        pub fn ledgerDomainSeparator(
            &self,
        ) -> alloy_contract::SolCallBuilder<&P, ledgerDomainSeparatorCall, N> {
            self.call_builder(&ledgerDomainSeparatorCall)
        }
        ///Creates a new call builder for the [`multicall`] function.
        pub fn multicall(
            &self,
            data: alloy::sol_types::private::Vec<alloy::sol_types::private::Bytes>,
        ) -> alloy_contract::SolCallBuilder<&P, multicallCall, N> {
            self.call_builder(&multicallCall { data })
        }
        ///Creates a new call builder for the [`noticePeriodChannelClosure`] function.
        pub fn noticePeriodChannelClosure(
            &self,
        ) -> alloy_contract::SolCallBuilder<&P, noticePeriodChannelClosureCall, N> {
            self.call_builder(&noticePeriodChannelClosureCall)
        }
        ///Creates a new call builder for the [`redeemTicket`] function.
        pub fn redeemTicket(
            &self,
            redeemable: <RedeemableTicket as alloy::sol_types::SolType>::RustType,
            params: <HoprCrypto::VRFParameters as alloy::sol_types::SolType>::RustType,
        ) -> alloy_contract::SolCallBuilder<&P, redeemTicketCall, N> {
            self.call_builder(
                &redeemTicketCall {
                    redeemable,
                    params,
                },
            )
        }
        ///Creates a new call builder for the [`redeemTicketSafe`] function.
        pub fn redeemTicketSafe(
            &self,
            selfAddress: alloy::sol_types::private::Address,
            redeemable: <RedeemableTicket as alloy::sol_types::SolType>::RustType,
            params: <HoprCrypto::VRFParameters as alloy::sol_types::SolType>::RustType,
        ) -> alloy_contract::SolCallBuilder<&P, redeemTicketSafeCall, N> {
            self.call_builder(
                &redeemTicketSafeCall {
                    selfAddress,
                    redeemable,
                    params,
                },
            )
        }
        ///Creates a new call builder for the [`token`] function.
        pub fn token(&self) -> alloy_contract::SolCallBuilder<&P, tokenCall, N> {
            self.call_builder(&tokenCall)
        }
        ///Creates a new call builder for the [`tokensReceived`] function.
        pub fn tokensReceived(
            &self,
            _0: alloy::sol_types::private::Address,
            from: alloy::sol_types::private::Address,
            to: alloy::sol_types::private::Address,
            amount: alloy::sol_types::private::primitives::aliases::U256,
            userData: alloy::sol_types::private::Bytes,
            _5: alloy::sol_types::private::Bytes,
        ) -> alloy_contract::SolCallBuilder<&P, tokensReceivedCall, N> {
            self.call_builder(
                &tokensReceivedCall {
                    _0,
                    from,
                    to,
                    amount,
                    userData,
                    _5,
                },
            )
        }
        ///Creates a new call builder for the [`updateDomainSeparator`] function.
        pub fn updateDomainSeparator(
            &self,
        ) -> alloy_contract::SolCallBuilder<&P, updateDomainSeparatorCall, N> {
            self.call_builder(&updateDomainSeparatorCall)
        }
        ///Creates a new call builder for the [`updateLedgerDomainSeparator`] function.
        pub fn updateLedgerDomainSeparator(
            &self,
        ) -> alloy_contract::SolCallBuilder<&P, updateLedgerDomainSeparatorCall, N> {
            self.call_builder(&updateLedgerDomainSeparatorCall)
        }
    }
    /// Event filters.
    #[automatically_derived]
    impl<
        P: alloy_contract::private::Provider<N>,
        N: alloy_contract::private::Network,
    > HoprChannelsInstance<P, N> {
        /// Creates a new event filter using this contract instance's provider and address.
        ///
        /// Note that the type can be any event, not just those defined in this contract.
        /// Prefer using the other methods for building type-safe event filters.
        pub fn event_filter<E: alloy_sol_types::SolEvent>(
            &self,
        ) -> alloy_contract::Event<&P, E, N> {
            alloy_contract::Event::new_sol(&self.provider, &self.address)
        }
        ///Creates a new event filter for the [`ChannelBalanceDecreased`] event.
        pub fn ChannelBalanceDecreased_filter(
            &self,
        ) -> alloy_contract::Event<&P, ChannelBalanceDecreased, N> {
            self.event_filter::<ChannelBalanceDecreased>()
        }
        ///Creates a new event filter for the [`ChannelBalanceIncreased`] event.
        pub fn ChannelBalanceIncreased_filter(
            &self,
        ) -> alloy_contract::Event<&P, ChannelBalanceIncreased, N> {
            self.event_filter::<ChannelBalanceIncreased>()
        }
        ///Creates a new event filter for the [`ChannelClosed`] event.
        pub fn ChannelClosed_filter(
            &self,
        ) -> alloy_contract::Event<&P, ChannelClosed, N> {
            self.event_filter::<ChannelClosed>()
        }
        ///Creates a new event filter for the [`ChannelOpened`] event.
        pub fn ChannelOpened_filter(
            &self,
        ) -> alloy_contract::Event<&P, ChannelOpened, N> {
            self.event_filter::<ChannelOpened>()
        }
        ///Creates a new event filter for the [`DomainSeparatorUpdated`] event.
        pub fn DomainSeparatorUpdated_filter(
            &self,
        ) -> alloy_contract::Event<&P, DomainSeparatorUpdated, N> {
            self.event_filter::<DomainSeparatorUpdated>()
        }
        ///Creates a new event filter for the [`LedgerDomainSeparatorUpdated`] event.
        pub fn LedgerDomainSeparatorUpdated_filter(
            &self,
        ) -> alloy_contract::Event<&P, LedgerDomainSeparatorUpdated, N> {
            self.event_filter::<LedgerDomainSeparatorUpdated>()
        }
        ///Creates a new event filter for the [`OutgoingChannelClosureInitiated`] event.
        pub fn OutgoingChannelClosureInitiated_filter(
            &self,
        ) -> alloy_contract::Event<&P, OutgoingChannelClosureInitiated, N> {
            self.event_filter::<OutgoingChannelClosureInitiated>()
        }
        ///Creates a new event filter for the [`TicketRedeemed`] event.
        pub fn TicketRedeemed_filter(
            &self,
        ) -> alloy_contract::Event<&P, TicketRedeemed, N> {
            self.event_filter::<TicketRedeemed>()
        }
    }
}
