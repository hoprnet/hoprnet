macro_rules! inherit_signer {
    ($struct_name:ident.$attr:ident) => {
        impl<D> k256::ecdsa::signature::DigestSigner<D, k256::ecdsa::Signature> for $struct_name
        where
            D: digest::FixedOutput<OutputSize = k256::elliptic_curve::consts::U32>
                + Clone
                + Default
                + digest::Reset
                + digest::Update
                + digest::HashMarker,
        {
            fn try_sign_digest(
                &self,
                digest: D,
            ) -> Result<k256::ecdsa::Signature, k256::ecdsa::Error> {
                self.$attr.try_sign_digest(digest)
            }
        }

        impl<D> k256::ecdsa::signature::DigestSigner<D, k256::ecdsa::recoverable::Signature>
            for $struct_name
        where
            D: digest::FixedOutput<OutputSize = k256::elliptic_curve::consts::U32>
                + Clone
                + Default
                + digest::Reset
                + digest::Update
                + digest::HashMarker,
        {
            fn try_sign_digest(
                &self,
                digest: D,
            ) -> Result<k256::ecdsa::recoverable::Signature, k256::ecdsa::Error> {
                self.$attr.try_sign_digest(digest)
            }
        }
    };
}

macro_rules! inherit_verifier {
    ($struct_name:ident.$attr:ident) => {
        impl $struct_name {
            /// Get the sec1 representation of the public key
            pub fn to_bytes(&self) -> [u8; 33] {
                let mut data = [0u8; 33];
                let generic_array = self.$attr.to_bytes();
                data.copy_from_slice(&generic_array);
                data
            }
        }

        impl<D> k256::ecdsa::signature::DigestVerifier<D, k256::ecdsa::Signature> for $struct_name
        where
            D: digest::Digest + digest::FixedOutput<OutputSize = k256::elliptic_curve::consts::U32>,
        {
            fn verify_digest(
                &self,
                digest: D,
                signature: &k256::ecdsa::Signature,
            ) -> Result<(), k256::ecdsa::Error> {
                self.$attr.verify_digest(digest, signature)
            }
        }

        impl<D> k256::ecdsa::signature::DigestVerifier<D, k256::ecdsa::recoverable::Signature>
            for $struct_name
        where
            D: digest::Digest + digest::FixedOutput<OutputSize = k256::elliptic_curve::consts::U32>,
        {
            fn verify_digest(
                &self,
                digest: D,
                signature: &k256::ecdsa::recoverable::Signature,
            ) -> Result<(), k256::ecdsa::Error> {
                self.$attr.verify_digest(digest, signature)
            }
        }
    };
}

macro_rules! params {
    (
        $(#[$outer:meta])*
        $name:ident{
            bip32: $bip32:expr,
            bip49: $bip49:expr,
            bip84: $bip84:expr,
            bip32_pub: $bip32pub:expr,
            bip49_pub: $bip49pub:expr,
            bip84_pub: $bip84pub:expr
        }
    ) => {
        $(#[$outer])*
        #[derive(Debug, Clone)]
        pub struct $name;

        impl crate::enc::NetworkParams for $name {
            const PRIV_VERSION: u32 = $bip32;
            const BIP49_PRIV_VERSION: u32 = $bip49;
            const BIP84_PRIV_VERSION: u32 = $bip84;
            const PUB_VERSION: u32 = $bip32pub;
            const BIP49_PUB_VERSION: u32 = $bip49pub;
            const BIP84_PUB_VERSION: u32 = $bip84pub;
        }
    }
}
