#[cfg(test)]
pub mod dummy_rng {
    use elliptic_curve::rand_core::{CryptoRng, Error, RngCore};
    use hex_literal::hex;

    const RNG_BYTES: [u8; 512] = hex!("cc6cb43c4928eea3c31e0d3bfcf8563f85d4bcc771e8efc4792fe3422a09f08a
    36dd22e648fce34edcd20439d9075075073f6da33d344430a45e7e2dfd297890
    7975caa9619afec8b43b3da891ec2369710a61d9630fbdfcd5509da466139c5f7
    a3c91f01fd6fac3665ad229def29873a2b0498bfefadbcb95f946bbea2a3f7657
    701dceffd55c5ce032ae663e1298e041c6b4350ef0e4ec921bedb0c5982709dc2
    b5939053317e14a63ed2f4dccea56145b256667fee63fdc037a7540bd6e16c238
    ff5d9ccfd9acfc9f91755b123b81106a1e3ec6bcc569063cdc78bda0e780aea6d
    06b20c784a295e700f429ee37508a62d98ab3634cfd6ba1c60e40d5c822d8cade
    c591ada0091ea9eae7422980e3defc89ca13ca2bc0de0c8397c5f9abb7b51e373
    7764cab0cfb2faf11e898de2fcc0e1df8fd96a2b1208111420e3aab3953329247
    aeb5416751b120466f41d8e5c094a4cdf6afee1143f42dba102529a0ebac44ced
    199341cc319b533429858c4ac159f7057aad9c2c9211b82c8c227439ec16a4883
    f50c24ee05a3f938e617fb40b7e56dff9a0536b9b7a3b70c607e76086ee61bd05
    d626878acfd5d7ca093d75fd152a00de1ebcd06788b9f6bfa2b289799b75b31c5
    bb8cfb2c005c7de64fb7c8f08613fafe824f1cbebd869aae560299d771f2b896b
    26fcf9a70b0ea3066531ac1a9190b52eb12cc10997aca62d7ce");

    /// Dummy RNG that cyclically outputs the same set of random bytes
    #[derive(Clone, Copy, Debug, Default)]
    pub(crate) struct DummyFixedRng {
        ptr: usize
    }

    impl DummyFixedRng {
        pub fn new() -> Self {
            DummyFixedRng { ptr: 0 }
        }

        fn read_raw_byte(&mut self) -> u8 {
            if self.ptr >= RNG_BYTES.len() {
                self.ptr = 0
            } else if self.ptr != 0 {
                self.ptr += 1;
            }

            RNG_BYTES[self.ptr]
        }

        fn read_bytes(&mut self, data: &mut [u8]) {
            for i in 0..data.len() {
                data[i] = self.read_raw_byte()
            }
        }
    }

    impl RngCore for DummyFixedRng {
        fn next_u32(&mut self) -> u32 {
            let mut data = [0u8; 4];
            self.read_bytes(&mut data);
            u32::from_ne_bytes(data)
        }

        fn next_u64(&mut self) -> u64 {
            let mut data = [0u8; 8];
            self.read_bytes(&mut data);
            u64::from_ne_bytes(data)
        }

        fn fill_bytes(&mut self, dest: &mut [u8]) {
            self.read_bytes(dest)
        }

        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
            self.fill_bytes(dest);
            Ok(())
        }
    }

    impl CryptoRng for DummyFixedRng {}
}