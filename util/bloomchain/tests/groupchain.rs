/*******************************************************************************
 * Copyright (c) 2015-2018 Parity Technologies (UK) Ltd.
 * Copyright (c) 2018-2019 Aion foundation.
 *
 *     This file is part of the aion network project.
 *
 *     The aion network project is free software: you can redistribute it
 *     and/or modify it under the terms of the GNU General Public License
 *     as published by the Free Software Foundation, either version 3 of
 *     the License, or any later version.
 *
 *     The aion network project is distributed in the hope that it will
 *     be useful, but WITHOUT ANY WARRANTY; without even the implied
 *     warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
 *     See the GNU General Public License for more details.
 *
 *     You should have received a copy of the GNU General Public License
 *     along with the aion network project source files.
 *     If not, see <https://www.gnu.org/licenses/>.
 *
 ******************************************************************************/

extern crate bloomchain;
extern crate rustc_hex;

mod util;

use crate::bloomchain::{Bloom, Config};
use crate::bloomchain::group::BloomGroupChain;
use util::{BloomGroupMemoryDatabase, FromHex, for_each_bloom, generate_n_random_blooms};

#[test]
fn simple_bloom_group_search() {
    let config = Config::default();
    let mut db = BloomGroupMemoryDatabase::default();
    let bloom = Bloom::from_hex("00000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002020000000000000000000000000000000000000000000008000000001000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");

    let modified_blooms = {
        let chain = BloomGroupChain::new(config, &db);
        let block_number = 23;
        chain.insert(block_number, bloom.clone())
    };

    // number of modified blooms should always be equal number of levels
    assert_eq!(modified_blooms.len(), config.levels);
    db.insert_blooms(modified_blooms);

    let chain = BloomGroupChain::new(config, &db);
    assert_eq!(chain.with_bloom(&(0..100), &bloom), vec![23]);
    assert_eq!(chain.with_bloom(&(0..22), &bloom), vec![]);
    assert_eq!(chain.with_bloom(&(23..23), &bloom), vec![23]);
    assert_eq!(chain.with_bloom(&(24..100), &bloom), vec![]);
}

#[test]
fn partly_matching_bloom_group_searach() {
    let config = Config::default();
    let mut db = BloomGroupMemoryDatabase::default();
    let bloom0 = Bloom::from_hex("10100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
    let bloom1 = Bloom::from_hex("11000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
    let bloom2 = Bloom::from_hex("10000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");

    let modified_blooms_0 = {
        let chain = BloomGroupChain::new(config, &db);
        let block_number = 14;
        chain.insert(block_number, bloom0)
    };

    db.insert_blooms(modified_blooms_0);

    let modified_blooms_1 = {
        let chain = BloomGroupChain::new(config, &db);
        let block_number = 15;
        chain.insert(block_number, bloom1)
    };

    db.insert_blooms(modified_blooms_1);

    let chain = BloomGroupChain::new(config, &db);
    assert_eq!(chain.with_bloom(&(0..100), &bloom2), vec![14, 15]);
}

#[test]
fn bloom_group_replace() {
    let config = Config::default();
    let mut db = BloomGroupMemoryDatabase::default();
    let bloom0 = Bloom::from_hex("10000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
    let bloom1 = Bloom::from_hex("01000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
    let bloom2 = Bloom::from_hex("00100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
    let bloom3 = Bloom::from_hex("00010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
    let bloom4 = Bloom::from_hex("00001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
    let bloom5 = Bloom::from_hex("00000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");

    let modified_blooms_0 = {
        let chain = BloomGroupChain::new(config, &db);
        let block_number = 14;
        chain.insert(block_number, bloom0.clone())
    };

    db.insert_blooms(modified_blooms_0);

    let modified_blooms_1 = {
        let chain = BloomGroupChain::new(config, &db);
        let block_number = 15;
        chain.insert(block_number, bloom1.clone())
    };

    db.insert_blooms(modified_blooms_1);

    let modified_blooms_2 = {
        let chain = BloomGroupChain::new(config, &db);
        let block_number = 16;
        chain.insert(block_number, bloom2.clone())
    };

    db.insert_blooms(modified_blooms_2);

    let modified_blooms_3 = {
        let chain = BloomGroupChain::new(config, &db);
        let block_number = 17;
        chain.insert(block_number, bloom3.clone())
    };

    db.insert_blooms(modified_blooms_3);

    let reset_modified_blooms = {
        let chain = BloomGroupChain::new(config, &db);
        chain.replace(&(15..17), vec![bloom4.clone(), bloom5.clone()])
    };

    db.insert_blooms(reset_modified_blooms);

    let chain = BloomGroupChain::new(config, &db);
    assert_eq!(chain.with_bloom(&(0..100), &bloom0), vec![14]);
    assert_eq!(chain.with_bloom(&(0..100), &bloom1), vec![]);
    assert_eq!(chain.with_bloom(&(0..100), &bloom2), vec![]);
    assert_eq!(chain.with_bloom(&(0..100), &bloom3), vec![]);
    assert_eq!(chain.with_bloom(&(0..100), &bloom4), vec![15]);
    assert_eq!(chain.with_bloom(&(0..100), &bloom5), vec![16]);
}

#[test]
fn file_test_bloom_group_search() {
    let config = Config::default();
    let mut db = BloomGroupMemoryDatabase::default();
    let blooms_file = include_bytes!("data/blooms.txt");

    for_each_bloom(blooms_file, |block_number, bloom| {
        let modified_blooms = {
            let chain = BloomGroupChain::new(config, &db);
            chain.insert(block_number, bloom)
        };

        // number of modified blooms should always be equal number of levels
        assert_eq!(modified_blooms.len(), config.levels);
        db.insert_blooms(modified_blooms);
    });

    for_each_bloom(blooms_file, |block_number, bloom| {
        let chain = BloomGroupChain::new(config, &db);
        let blocks = chain.with_bloom(&(block_number..block_number), &bloom);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0], block_number);
    });
}

#[test]
fn random_bloom_group_replacement() {
    let insertions = 10_000;

    let config = Config::default();
    let mut db = BloomGroupMemoryDatabase::default();
    let blooms = generate_n_random_blooms(insertions);

    for (i, bloom) in blooms.iter().enumerate() {
        let modified_blooms = {
            let chain = BloomGroupChain::new(config, &db);
            chain.replace(&(i..i), vec![bloom.clone()])
        };

        db.insert_blooms(modified_blooms);
    }

    for (i, bloom) in blooms.iter().enumerate() {
        let chain = BloomGroupChain::new(config, &db);
        let blocks = chain.with_bloom(&(i..i), bloom);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0], i);
    }
}
