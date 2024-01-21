use super::RawAddress;

pub struct PartialTxoutData {
    pub raw_address: RawAddress,
    pub sats: u64,
    pub address_index_opt: Option<u32>,
}

impl PartialTxoutData {
    pub fn new(raw_address: RawAddress, sats: u64, address_index_opt: Option<u32>) -> Self {
        Self {
            raw_address,
            sats,
            address_index_opt,
        }
    }
}