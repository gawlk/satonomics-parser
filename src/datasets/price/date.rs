use std::collections::HashMap;

use chrono::NaiveDate;
use color_eyre::eyre::Error;

use crate::{
    datasets::{AnyDataset, GenericDataset, MinInitialState},
    parse::{AnyDateMap, AnyExportableMap, DateMap, WNaiveDate},
    price::Kraken,
};

pub struct DateDataset {
    min_initial_state: MinInitialState,

    kraken_daily: Option<HashMap<String, f32>>,

    pub closes: DateMap<f32>,
}

impl DateDataset {
    pub fn import(parent_path: &str) -> color_eyre::Result<Self> {
        let name = "close";

        let s = Self {
            min_initial_state: MinInitialState::default(),

            kraken_daily: None,

            closes: DateMap::new_in_memory_json(&format!("{parent_path}/{name}")),
        };

        s.min_initial_state.compute_from_dataset(&s);

        Ok(s)
    }

    pub fn get(&mut self, date: NaiveDate) -> color_eyre::Result<f32> {
        if self.closes.is_date_safe(date) {
            Ok(self
                .closes
                .inner
                .lock()
                .as_ref()
                .unwrap()
                .get(&WNaiveDate::wrap(date))
                .unwrap()
                .to_owned())
        } else {
            let price = self.get_from_daily_kraken(&date.to_string())?;

            self.closes.insert(date, price);

            Ok(price)
        }
    }

    fn get_from_daily_kraken(&mut self, date: &str) -> color_eyre::Result<f32> {
        if self.kraken_daily.is_none() {
            self.kraken_daily.replace(Kraken::fetch_daily_prices()?);
        }

        self.kraken_daily
            .as_ref()
            .unwrap()
            .get(date)
            .cloned()
            .ok_or(Error::msg("Couldn't find date in daily kraken"))
    }
}

impl GenericDataset for DateDataset {}

impl AnyDataset for DateDataset {
    fn get_min_initial_state(&self) -> &MinInitialState {
        &self.min_initial_state
    }

    fn to_any_inserted_date_map_vec(&self) -> Vec<&(dyn AnyDateMap + Send + Sync)> {
        vec![&self.closes]
    }

    fn to_any_exported_date_map_vec(&self) -> Vec<&(dyn AnyExportableMap + Send + Sync)> {
        vec![&self.closes]
    }
}
