use crate::{
    datasets::AnyDataset,
    structs::{AnyHeightMap, BiMap},
};

use super::ProcessedBlockData;

pub struct CoinblocksDataset {
    pub destroyed: BiMap<f64>,
}

impl CoinblocksDataset {
    pub fn import(parent_path: &str) -> color_eyre::Result<Self> {
        let folder_path = format!("{parent_path}/coinblocks");
        let f = |s: &str| format!("{folder_path}/{s}");

        Ok(Self {
            destroyed: BiMap::new_on_disk_bin(&f("destroyed")),
        })
    }
}

impl AnyDataset for CoinblocksDataset {
    fn insert_block_data(
        &self,
        &ProcessedBlockData {
            height,
            coinblocks_destroyed_vec,
            is_date_last_block,
            date,
            ..
        }: &ProcessedBlockData,
    ) {
        self.destroyed
            .height
            .insert(height, *coinblocks_destroyed_vec.last().unwrap());

        if is_date_last_block {
            self.destroyed
                .date
                .insert(date, coinblocks_destroyed_vec.iter().sum())
        }
    }

    fn to_any_height_map_vec(&self) -> Vec<&(dyn AnyHeightMap + Send + Sync)> {
        vec![&self.destroyed.height]
    }

    fn to_any_date_map_vec(&self) -> Vec<&(dyn crate::structs::AnyDateMap + Send + Sync)> {
        vec![&self.destroyed.date]
    }
}