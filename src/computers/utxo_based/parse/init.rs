use std::thread;

use chrono::{Days, NaiveDate};

use crate::{
    computers::utxo_based::export::UtxoDatasets,
    structs::{DateMap, HeightDatasets},
};

use super::{DateDataVec, TxidIndexToBlockPath, TxidToTxData, TxoutIndexToTxoutValue};

pub struct InitiatedParsers {
    pub date_data_vec: DateDataVec,
    pub txid_index_to_block_path: TxidIndexToBlockPath,
    pub txid_to_tx_data: TxidToTxData,
    pub txout_index_to_txout_value: TxoutIndexToTxoutValue,
    pub txout_index_counter: usize,
    pub iter_height: usize,
}

impl InitiatedParsers {
    pub fn init(
        datasets: &UtxoDatasets,
        height_to_date: &[NaiveDate],
        date_to_first_block: &DateMap<usize>,
    ) -> color_eyre::Result<Self> {
        if datasets.get_min_last_height().unwrap_or(0) == 0 {
            println!("New dataset present, starting over...");

            Ok(Self {
                date_data_vec: DateDataVec::new(),
                txid_index_to_block_path: TxidIndexToBlockPath::new(),
                txid_to_tx_data: TxidToTxData::new(),
                txout_index_to_txout_value: TxoutIndexToTxoutValue::new(),
                txout_index_counter: 0,
                iter_height: 0,
            })
        } else {
            let txid_index_to_block_path_handle = thread::spawn(TxidIndexToBlockPath::import);

            let txid_to_tx_data_handle = thread::spawn(TxidToTxData::import);

            let txout_index_to_txout_value_handle = thread::spawn(TxoutIndexToTxoutValue::import);

            let mut date_data_vec = DateDataVec::import(height_to_date)?;

            let mut txid_index_to_block_path = txid_index_to_block_path_handle.join().unwrap()?;

            println!(
                "txid_index_to_block_path len {}",
                txid_index_to_block_path.len()
            );

            let mut txid_to_tx_data = txid_to_tx_data_handle.join().unwrap()?;

            println!("txid_to_tx_data len {}", txid_to_tx_data.len());

            let mut txout_index_to_txout_value =
                txout_index_to_txout_value_handle.join().unwrap()?;

            println!(
                "txout_index_to_txout_value len {}",
                txout_index_to_txout_value.len()
            );

            let mut txout_index_counter = txout_index_to_txout_value
                .keys()
                .max()
                .map(|index| *index + 1)
                .unwrap_or(0)
                .to_owned();

            let snapshot_start_height = date_data_vec
                .iter()
                .map(|date_data| date_data.date)
                .max()
                .and_then(|date| date.checked_add_days(Days::new(1)))
                .and_then(|date| {
                    let min_last_height = datasets.get_min_last_height();

                    date_to_first_block.get(&date).map(|snapshot_start_height| {

                        if min_last_height.unwrap_or(0) < snapshot_start_height - 1 {
                            println!("snapshot_start_height {snapshot_start_height} > last_saved_height {min_last_height:?}");
                            println!("Starting over...");

                            date_data_vec.clear();

                            txid_index_to_block_path.clear();

                            txid_to_tx_data.clear();

                            txout_index_to_txout_value.clear();

                            txout_index_counter = 0;

                            return 0;
                        }

                        snapshot_start_height
                    })
                });

            let iter_height = snapshot_start_height.unwrap_or(0);

            Ok(Self {
                date_data_vec,
                txid_index_to_block_path,
                txid_to_tx_data,
                txout_index_to_txout_value,
                txout_index_counter,
                iter_height,
            })
        }
    }
}