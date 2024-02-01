use std::time::Instant;

use chrono::{offset::Local, Datelike};

use crate::{
    bitcoin::{BitcoinDB, NUMBER_OF_UNSAFE_BLOCKS},
    databases::Databases,
    datasets::{AllDatasets, AnyDatasets, ProcessedDateData},
    export_all::{export_all, ExportedData},
    min_height::find_first_unsafe_height,
    parse_block::{parse_block, ParseData},
    states::States,
    structs::DateData,
    utils::timestamp_to_naive_date,
};

pub fn iter_blocks(bitcoin_db: &BitcoinDB, block_count: usize) -> color_eyre::Result<()> {
    println!("{:?} - Starting aged", Local::now());

    let mut datasets = AllDatasets::import()?;
    let address_datasets_is_empty = datasets.address.is_empty();
    let min_initial_unsafe_address_date = datasets.address.get_min_initial_first_unsafe_date();
    let min_initial_unsafe_address_height = datasets.address.get_min_initial_first_unsafe_height();

    println!("{:?} - Imported datasets", Local::now());

    let mut databases = Databases::default();

    println!("{:?} - Imported databases", Local::now());

    let mut states = States::import().unwrap_or_default();

    println!("{:?} - Imported states", Local::now());

    let mut height = find_first_unsafe_height(&mut states, &databases, &datasets);

    println!("{:?} - Starting parsing at height: {height}", Local::now());

    let mut block_iter = bitcoin_db.iter_block(height, block_count);

    let mut next_block_opt = None;
    let mut blocks_loop_date = None;

    'parsing: loop {
        let time = Instant::now();

        'days: loop {
            let mut blocks_loop_i = 0;

            if next_block_opt.is_some() {
                blocks_loop_date.take();
            }

            let mut coinbase_vec = vec![];
            let mut coinblocks_destroyed_vec = vec![];
            let mut coindays_destroyed_vec = vec![];
            let mut fees_vec = vec![];

            'blocks: loop {
                let current_block_opt = next_block_opt.take().or_else(|| block_iter.next());

                next_block_opt = block_iter.next();

                if let Some(current_block) = current_block_opt {
                    let timestamp = current_block.header.time;

                    let current_block_date = timestamp_to_naive_date(timestamp);
                    let current_block_height = height + blocks_loop_i;

                    let next_block_date = next_block_opt
                        .as_ref()
                        .map(|next_block| timestamp_to_naive_date(next_block.header.time));

                    // Always run for the first block of the loop
                    if blocks_loop_date.is_none() {
                        blocks_loop_date.replace(current_block_date);

                        if states
                            .date_data_vec
                            .last()
                            .map(|date_data| *date_data.date < current_block_date)
                            .unwrap_or(true)
                        {
                            states
                                .date_data_vec
                                .push(DateData::new(current_block_date, vec![]));
                        }

                        println!(
                            "{:?} - Processing {current_block_date} (height: {height})...",
                            Local::now()
                        );
                    }

                    let blocks_loop_date = blocks_loop_date.unwrap();

                    if current_block_date > blocks_loop_date {
                        panic!("current block should always have the same date as the current blocks loop");
                    }

                    let is_date_last_block = next_block_date
                        // Do NOT change `blocks_loop_date` to `current_block_date` !!!
                        .map_or(true, |next_block_date| blocks_loop_date < next_block_date);

                    let compute_addresses = !address_datasets_is_empty
                        && (min_initial_unsafe_address_date
                            .map_or(true, |min_date| current_block_date >= min_date)
                            || min_initial_unsafe_address_height
                                .map_or(true, |min_height| current_block_height >= min_height));

                    parse_block(ParseData {
                        bitcoin_db,
                        block: current_block,
                        block_index: blocks_loop_i,
                        coinbase_vec: &mut coinbase_vec,
                        coinblocks_destroyed_vec: &mut coinblocks_destroyed_vec,
                        coindays_destroyed_vec: &mut coindays_destroyed_vec,
                        compute_addresses,
                        databases: &mut databases,
                        datasets: &mut datasets,
                        date: current_block_date,
                        fees_vec: &mut fees_vec,
                        height: current_block_height,
                        is_date_last_block,
                        states: &mut states,
                        timestamp,
                    });

                    blocks_loop_i += 1;

                    if is_date_last_block {
                        datasets.insert_date_data(ProcessedDateData {
                            block_count,
                            first_height: height,
                            height: current_block_height,
                            date: blocks_loop_date,
                        });

                        height += blocks_loop_i;

                        if next_block_date
                            .map_or(true, |next_block_date| next_block_date.day() == 1)
                            || height > (block_count - (NUMBER_OF_UNSAFE_BLOCKS * 10))
                        {
                            break 'days;
                        } else {
                            break 'blocks;
                        }
                    }
                } else {
                    break 'parsing;
                }
            }
        }

        let last_height = height - 1;

        println!(
            "Parsing month took {} seconds (last height: {last_height})\n",
            time.elapsed().as_secs_f32(),
        );

        export_all(ExportedData {
            block_count,
            databases: &mut databases,
            datasets: &datasets,
            date: blocks_loop_date.unwrap(),
            height: last_height,
            states: &states,
        })?;
    }

    datasets.export()?;

    Ok(())
}
