use utils_types::traits::BinarySerializable;

pub struct Packet {

}

impl BinarySerializable for Packet {
    const SIZE: usize = 0;

    fn deserialize(data: &[u8]) -> utils_types::errors::Result<Self> {
        todo!()
    }

    fn serialize(&self) -> Box<[u8]> {
        todo!()
    }
}