use crate::{
    datasets::{AnyDataset, MinInitialState, ProcessedBlockData},
    parse::{AnyBiMap, BiMap},
    states::InputState,
};

pub struct InputSubDataset {
    min_initial_state: MinInitialState,

    pub count: BiMap<f32>,
    pub volume: BiMap<f32>,
}

impl InputSubDataset {
    pub fn import(parent_path: &str) -> color_eyre::Result<Self> {
        let f = |s: &str| format!("{parent_path}/{s}");

        let mut s = Self {
            min_initial_state: MinInitialState::default(),

            count: BiMap::new_bin(&f("input_count")),
            volume: BiMap::new_bin(&f("input_volume")),
        };

        s.min_initial_state
            .consume(MinInitialState::compute_from_dataset(&s));

        Ok(s)
    }

    pub fn insert(
        &self,
        &ProcessedBlockData {
            height,
            date,
            is_date_last_block,
            date_blocks_range,
            ..
        }: &ProcessedBlockData,
        state: &InputState,
    ) {
        let count = self.count.height.insert(height, state.count);

        self.volume.height.insert(height, state.volume);

        if is_date_last_block {
            self.count.date.insert(date, count);

            self.volume.date_insert_sum_range(date, date_blocks_range);
        }
    }
}

impl AnyDataset for InputSubDataset {
    fn get_min_initial_state(&self) -> &MinInitialState {
        &self.min_initial_state
    }

    fn to_any_bi_map_vec(&self) -> Vec<&(dyn AnyBiMap + Send + Sync)> {
        vec![&self.count, &self.volume]
    }
}
