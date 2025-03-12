pub trait ToHexStr: AsRef<[u8]> {
    fn to_hex(&self) -> String {
        hex::encode(self)
    }
}

impl ToHexStr for Vec<u8> {
    fn to_hex(&self) -> String {
        hex::encode(self)
    }
}

impl<const N: usize> ToHexStr for [u8; N] {
    fn to_hex(&self) -> String {
        hex::encode(self)
    }
}
