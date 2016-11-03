// Copyright 2015, 2016 Ethcore (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use io::IoChannel;
use client::{BlockChainClient, MiningBlockChainClient, Client, ClientConfig, BlockID};
use ethereum;
use block::IsBlock;
use tests::helpers::*;
use types::filter::Filter;
use util::*;
use devtools::*;
use miner::Miner;
use rlp::{Rlp, View};
use spec::Spec;
use views::BlockView;
use util::stats::Histogram;

#[test]
fn imports_from_empty() {
	let dir = RandomTempPath::new();
	let spec = get_test_spec();
	let db_config = DatabaseConfig::with_columns(::db::NUM_COLUMNS);

	let client = Client::new(
		ClientConfig::default(),
		&spec,
		dir.as_path(),
		Arc::new(Miner::with_spec(&spec)),
		IoChannel::disconnected(),
		&db_config
	).unwrap();
	client.import_verified_blocks();
	client.flush_queue();
}

#[test]
fn should_return_registrar() {
	let dir = RandomTempPath::new();
	let spec = ethereum::new_morden();
	let db_config = DatabaseConfig::with_columns(::db::NUM_COLUMNS);

	let client = Client::new(
		ClientConfig::default(),
		&spec,
		dir.as_path(),
		Arc::new(Miner::with_spec(&spec)),
		IoChannel::disconnected(),
		&db_config
	).unwrap();
	let params = client.additional_params();
	let address = params.get("registrar").unwrap();

	assert_eq!(address.len(), 40);
	assert!(U256::from_str(address).is_ok());
}

#[test]
fn returns_state_root_basic() {
	let client_result = generate_dummy_client(6);
	let client = client_result.reference();
	let test_spec = get_test_spec();
	let genesis_header = test_spec.genesis_header();

	assert!(client.state_data(genesis_header.state_root()).is_some());
}

#[test]
fn imports_good_block() {
	let dir = RandomTempPath::new();
	let spec = get_test_spec();
	let db_config = DatabaseConfig::with_columns(::db::NUM_COLUMNS);

	let client = Client::new(
		ClientConfig::default(),
		&spec,
		dir.as_path(),
		Arc::new(Miner::with_spec(&spec)),
		IoChannel::disconnected(),
		&db_config
	).unwrap();
	let good_block = get_good_dummy_block();
	if let Err(_) = client.import_block(good_block) {
		panic!("error importing block being good by definition");
	}
	client.flush_queue();
	client.import_verified_blocks();

	let block = client.block_header(BlockID::Number(1)).unwrap();
	assert!(!block.is_empty());
}

#[test]
fn query_none_block() {
	let dir = RandomTempPath::new();
	let spec = get_test_spec();
	let db_config = DatabaseConfig::with_columns(::db::NUM_COLUMNS);

	let client = Client::new(
		ClientConfig::default(),
		&spec,
		dir.as_path(),
		Arc::new(Miner::with_spec(&spec)),
		IoChannel::disconnected(),
		&db_config
	).unwrap();
    let non_existant = client.block_header(BlockID::Number(188));
	assert!(non_existant.is_none());
}

#[test]
fn query_bad_block() {
	let client_result = get_test_client_with_blocks(vec![get_bad_state_dummy_block()]);
	let client = client_result.reference();
	let bad_block:Option<Bytes> = client.block_header(BlockID::Number(1));

	assert!(bad_block.is_none());
}

#[test]
fn returns_chain_info() {
	let dummy_block = get_good_dummy_block();
	let client_result = get_test_client_with_blocks(vec![dummy_block.clone()]);
	let client = client_result.reference();
	let block = BlockView::new(&dummy_block);
	let info = client.chain_info();
	assert_eq!(info.best_block_hash, block.header().hash());
}

#[test]
fn returns_logs() {
	let dummy_block = get_good_dummy_block();
	let client_result = get_test_client_with_blocks(vec![dummy_block.clone()]);
	let client = client_result.reference();
	let logs = client.logs(Filter {
		from_block: BlockID::Earliest,
		to_block: BlockID::Latest,
		address: None,
		topics: vec![],
		limit: None,
	});
	assert_eq!(logs.len(), 0);
}

#[test]
fn returns_logs_with_limit() {
	let dummy_block = get_good_dummy_block();
	let client_result = get_test_client_with_blocks(vec![dummy_block.clone()]);
	let client = client_result.reference();
	let logs = client.logs(Filter {
		from_block: BlockID::Earliest,
		to_block: BlockID::Latest,
		address: None,
		topics: vec![],
		limit: Some(2),
	});
	assert_eq!(logs.len(), 0);
}

#[test]
fn returns_block_body() {
	let dummy_block = get_good_dummy_block();
	let client_result = get_test_client_with_blocks(vec![dummy_block.clone()]);
	let client = client_result.reference();
	let block = BlockView::new(&dummy_block);
	let body = client.block_body(BlockID::Hash(block.header().hash())).unwrap();
	let body = Rlp::new(&body);
	assert_eq!(body.item_count(), 2);
	assert_eq!(body.at(0).as_raw()[..], block.rlp().at(1).as_raw()[..]);
	assert_eq!(body.at(1).as_raw()[..], block.rlp().at(2).as_raw()[..]);
}

#[test]
fn imports_block_sequence() {
	let client_result = generate_dummy_client(6);
	let client = client_result.reference();
	let block = client.block_header(BlockID::Number(5)).unwrap();

	assert!(!block.is_empty());
}

#[test]
fn can_collect_garbage() {
	let client_result = generate_dummy_client(100);
	let client = client_result.reference();
	client.tick();
	assert!(client.blockchain_cache_info().blocks < 100 * 1024);
}


#[test]
fn can_generate_gas_price_median() {
	let client_result = generate_dummy_client_with_data(3, 1, &vec_into![1, 2, 3]);
	let client = client_result.reference();
	assert_eq!(Some(U256::from(2)), client.gas_price_median(3));

	let client_result = generate_dummy_client_with_data(4, 1, &vec_into![1, 4, 3, 2]);
	let client = client_result.reference();
	assert_eq!(Some(U256::from(3)), client.gas_price_median(4));
}

#[test]
fn can_generate_gas_price_histogram() {
	let client_result = generate_dummy_client_with_data(20, 1, &vec_into![6354,8593,6065,4842,7845,7002,689,4958,4250,6098,5804,4320,643,8895,2296,8589,7145,2000,2512,1408]);
	let client = client_result.reference();

	let hist = client.gas_price_histogram(20, 5).unwrap();
	let correct_hist = Histogram { bucket_bounds: vec_into![643, 2294, 3945, 5596, 7247, 8898], counts: vec![4,2,4,6,4] };
	assert_eq!(hist, correct_hist);
}

#[test]
fn empty_gas_price_histogram() {
	let client_result = generate_dummy_client_with_data(20, 0, &vec_into![]);
	let client = client_result.reference();

	assert!(client.gas_price_histogram(20, 5).is_none());
}


#[test]
fn can_handle_long_fork() {
	let client_result = generate_dummy_client(1200);
	let client = client_result.reference();
	for _ in 0..20 {
		client.import_verified_blocks();
	}
	assert_eq!(1200, client.chain_info().best_block_number);

	push_blocks_to_client(client, 45, 1201, 800);
	push_blocks_to_client(client, 49, 1201, 800);
	push_blocks_to_client(client, 53, 1201, 600);

	for _ in 0..400 {
		client.import_verified_blocks();
	}
	assert_eq!(2000, client.chain_info().best_block_number);
}

#[test]
fn can_mine() {
	let dummy_blocks = get_good_dummy_block_seq(2);
	let client_result = get_test_client_with_blocks(vec![dummy_blocks[0].clone()]);
	let client = client_result.reference();

	let b = client.prepare_open_block(Address::default(), (3141562.into(), 31415620.into()), vec![]).close();

	assert_eq!(*b.block().header().parent_hash(), BlockView::new(&dummy_blocks[0]).header_view().sha3());
}

#[test]
fn change_history_size() {
	let dir = RandomTempPath::new();
	let test_spec = Spec::new_null();
	let mut config = ClientConfig::default();
	let db_config = DatabaseConfig::with_columns(::db::NUM_COLUMNS);
	config.history = 2;
	let address = Address::random();
	{
		let client = Client::new(ClientConfig::default(), &test_spec, dir.as_path(), Arc::new(Miner::with_spec(&test_spec)), IoChannel::disconnected(), &db_config).unwrap();
		for _ in 0..20 {
			let mut b = client.prepare_open_block(Address::default(), (3141562.into(), 31415620.into()), vec![]);
			b.block_mut().fields_mut().state.add_balance(&address, &5.into());
			b.block_mut().fields_mut().state.commit().unwrap();
			let b = b.close_and_lock().seal(&*test_spec.engine, vec![]).unwrap();
			client.import_sealed_block(b).unwrap(); // account change is in the journal overlay
		}
	}
	let mut config = ClientConfig::default();
	config.history = 10;
	let client = Client::new(config, &test_spec, dir.as_path(), Arc::new(Miner::with_spec(&test_spec)), IoChannel::disconnected(), &db_config).unwrap();
	assert_eq!(client.state().balance(&address), 100.into());
}
