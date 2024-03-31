use crate::{
    bitcoin::{sats_to_btc, ONE_YEAR_IN_BLOCK_TIME},
    datasets::AnyDataset,
    parse::{AnyDateMap, AnyExportableMap, AnyHeightMap, BiMap, DateMap},
    utils::{ONE_MONTH_IN_DAYS, ONE_WEEK_IN_DAYS, ONE_YEAR_IN_DAYS},
};

use super::{ExportData, GenericDataset, MinInitialState, ProcessedBlockData, ProcessedDateData};

pub struct MiningDataset {
    min_initial_state: MinInitialState,

    pub blocks_mined: DateMap<usize>,
    pub coinbase: BiMap<f32>,
    pub fees: BiMap<f32>,

    pub subsidy: BiMap<f32>,
    pub subsidy_in_dollars: BiMap<f32>,
    pub annualized_issuance: BiMap<f32>,
    pub yearly_inflation_rate: BiMap<f32>,
    pub last_subsidy: DateMap<f32>,
    pub last_subsidy_in_dollars: DateMap<f32>,
    pub blocks_mined_1w_sma: DateMap<f32>,
    pub blocks_mined_1m_sma: DateMap<f32>,
}

impl MiningDataset {
    pub fn import(parent_path: &str) -> color_eyre::Result<Self> {
        let f = |s: &str| format!("{parent_path}/{s}");

        let s = Self {
            min_initial_state: MinInitialState::default(),

            blocks_mined: DateMap::new_on_disk_bin(&f("blocks_mined")),
            coinbase: BiMap::new_on_disk_bin(&f("coinbase")),
            fees: BiMap::new_on_disk_bin(&f("fees")),

            subsidy: BiMap::new_on_disk_bin(&f("subsidy")),
            subsidy_in_dollars: BiMap::new_on_disk_bin(&f("subsidy_in_dollars")),
            last_subsidy: DateMap::new_on_disk_bin(&f("last_subsidy")),
            last_subsidy_in_dollars: DateMap::new_on_disk_bin(&f("last_subsidy_in_dollars")),
            annualized_issuance: BiMap::new_on_disk_bin(&f("annualized_issuance")),
            yearly_inflation_rate: BiMap::new_on_disk_bin(&f("yearly_inflation_rate")),
            blocks_mined_1w_sma: DateMap::new_on_disk_bin(&f("blocks_mined_7d_sma")),
            blocks_mined_1m_sma: DateMap::new_on_disk_bin(&f("blocks_mined_1m_sma")),
        };

        s.min_initial_state.compute_from_dataset(&s);

        Ok(s)
    }
}

impl GenericDataset for MiningDataset {
    fn insert_date_data(
        &self,
        &ProcessedDateData {
            date,
            first_height,
            height,
            ..
        }: &ProcessedDateData,
    ) {
        self.blocks_mined.insert(date, height + 1 - first_height);
    }

    fn insert_block_data(
        &self,
        &ProcessedBlockData {
            height,
            coinbase,
            fees,
            ..
        }: &ProcessedBlockData,
    ) {
        self.coinbase.insert(height, sats_to_btc(coinbase));

        self.fees.insert(height, sats_to_btc(fees.iter().sum()));
    }
}

impl AnyDataset for MiningDataset {
    fn to_any_inserted_date_map_vec(&self) -> Vec<&(dyn AnyDateMap + Send + Sync)> {
        vec![&self.blocks_mined]
    }

    fn to_any_inserted_height_map_vec(&self) -> Vec<&(dyn AnyHeightMap + Send + Sync)> {
        vec![&self.coinbase.height, &self.fees.height]
    }

    fn compute(
        &mut self,
        &ExportData {
            circulating_supply,
            last_height_to_date,
            sum_heights_to_date,
            price,
            ..
        }: &ExportData,
    ) {
        self.coinbase.compute_date(sum_heights_to_date);
        self.fees.compute_date(sum_heights_to_date);

        self.subsidy.set_height_then_compute_date(
            self.coinbase.height.subtract(&self.fees.height),
            last_height_to_date,
        );

        self.subsidy_in_dollars.set_height_then_compute_date(
            self.subsidy.height.multiply(&price.height),
            last_height_to_date,
        );

        self.annualized_issuance
            .set_height(self.subsidy.height.last_x_sum(ONE_YEAR_IN_BLOCK_TIME));
        self.annualized_issuance
            .set_date(self.subsidy.date.last_x_sum(ONE_YEAR_IN_DAYS));

        self.yearly_inflation_rate.set_height(
            self.annualized_issuance
                .height
                .divide(&circulating_supply.height),
        );
        self.yearly_inflation_rate.set_date(
            self.annualized_issuance
                .date
                .divide(&circulating_supply.date),
        );

        self.last_subsidy.compute_from_height_map(
            self.subsidy.height.inner.lock().as_ref().unwrap(),
            last_height_to_date,
        );
        self.last_subsidy_in_dollars.compute_from_height_map(
            self.subsidy_in_dollars
                .height
                .inner
                .lock()
                .as_ref()
                .unwrap(),
            last_height_to_date,
        );

        self.blocks_mined_1w_sma
            .set_inner(self.blocks_mined.simple_moving_average(ONE_WEEK_IN_DAYS));
        self.blocks_mined_1m_sma
            .set_inner(self.blocks_mined.simple_moving_average(ONE_MONTH_IN_DAYS));
    }

    fn to_any_exported_bi_map_vec(&self) -> Vec<&(dyn AnyExportableMap + Send + Sync)> {
        vec![
            &self.coinbase,
            &self.fees,
            &self.subsidy,
            &self.subsidy_in_dollars,
            &self.annualized_issuance,
            &self.yearly_inflation_rate,
        ]
    }

    fn to_any_exported_date_map_vec(&self) -> Vec<&(dyn AnyExportableMap + Send + Sync)> {
        vec![
            &self.last_subsidy,
            &self.last_subsidy_in_dollars,
            &self.blocks_mined,
            &self.blocks_mined_1w_sma,
            &self.blocks_mined_1m_sma,
        ]
    }

    fn get_min_initial_state(&self) -> &MinInitialState {
        &self.min_initial_state
    }
}
