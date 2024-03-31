use crate::{
    datasets::{AnyDataset, ExportData, MinInitialState, ProcessedBlockData},
    parse::{AnyExportableMap, AnyHeightMap, BiMap},
};

/// TODO: Fix fees not taken into account ?
pub struct RealizedSubDataset {
    min_initial_state: MinInitialState,

    realized_profit: BiMap<f32>,
    realized_loss: BiMap<f32>,
}

impl RealizedSubDataset {
    pub fn import(parent_path: &str) -> color_eyre::Result<Self> {
        let f = |s: &str| format!("{parent_path}/{s}");

        let s = Self {
            min_initial_state: MinInitialState::default(),

            realized_profit: BiMap::new_on_disk_bin(&f("realized_profit")),
            realized_loss: BiMap::new_on_disk_bin(&f("realized_loss")),
        };

        s.min_initial_state.compute_from_dataset(&s);

        Ok(s)
    }

    pub fn insert(
        &self,
        &ProcessedBlockData { height, .. }: &ProcessedBlockData,
        height_state: &RealizedState,
    ) {
        self.realized_profit
            .height
            .insert(height, height_state.realized_profit);

        self.realized_loss
            .height
            .insert(height, height_state.realized_loss);
    }
}

impl AnyDataset for RealizedSubDataset {
    fn compute(
        &mut self,
        &ExportData {
            sum_heights_to_date,
            ..
        }: &ExportData,
    ) {
        self.realized_loss.compute_date(sum_heights_to_date);
        self.realized_profit.compute_date(sum_heights_to_date);
    }

    fn get_min_initial_state(&self) -> &MinInitialState {
        &self.min_initial_state
    }

    fn to_any_inserted_height_map_vec(&self) -> Vec<&(dyn AnyHeightMap + Send + Sync)> {
        vec![&self.realized_loss.height, &self.realized_profit.height]
    }

    fn to_any_exported_bi_map_vec(&self) -> Vec<&(dyn AnyExportableMap + Send + Sync)> {
        vec![&self.realized_loss, &self.realized_profit]
    }
}

// ---
// STATE
// ---

#[derive(Debug, Default)]
pub struct RealizedState {
    pub realized_profit: f32,
    pub realized_loss: f32,
}

impl RealizedState {
    pub fn iterate(&mut self, realized_profit: f32, realized_loss: f32) {
        self.realized_profit += realized_profit;
        self.realized_loss += realized_loss;
    }
}
