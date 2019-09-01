#![cfg_attr(not(feature = "std"), no_std)]
// HACK: This sequence needs to be repeated in each project.
//       See https://github.com/rust-lang/cargo/issues/5034
// For clippy lints see: https://rust-lang.github.io/rust-clippy/master
// For rustc lints see: https://doc.rust-lang.org/rustc/lints/index.html
#![warn(
    // Enable sets of warnings
    clippy::all,
    clippy::pedantic,
    // TODO: clippy::cargo,
    rust_2018_idioms,
    future_incompatible,
    unused,

    // Additional unused warnings (not included in `unused`)
    unused_lifetimes,
    unused_qualifications,
    unused_results,

    // Additional misc. warnings
    anonymous_parameters,
    deprecated_in_future,
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    keyword_idents,
    macro_use_extern_crate,
    // TODO: missing_docs,
    missing_doc_code_examples,
    private_doc_tests,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    // TODO: unreachable_pub,
    unsafe_code,
    variant_size_differences
)]
#![cfg_attr(feature = "std", warn(
    // TODO: missing_debug_implementations,
))]
// TODO: We do a lot of casting all over the place, mostly to/from usize.
#![allow(clippy::cast_possible_truncation)]

mod channel;
mod constraint;
mod fft;
// TODO pub mod fibonacci;
mod geometric_series;
mod hash;
mod hashable;
mod masked_keccak;
mod merkle_tree;
mod polynomial;
mod proof_params;
mod utils;
// TODO mod verifier;

pub use channel::{ProverChannel, VerifierChannel};
pub use proof_params::ProofParams;
// TODO pub use verifier::check_proof;

use merkle_tree::Proof as MerkleProof;

// In no std mode, substitute no_std_compat
#[cfg(not(feature = "std"))]
#[cfg_attr(feature = "std", macro_use)]
extern crate no_std_compat as std;

// Conditionally include MmapVec. If the feature is disabled substitute Vec
// instead.
#[cfg(feature = "mmap")]
mod mmap_vec;
#[cfg(not(feature = "mmap"))]
mod mmap_vec {
    pub use std::vec::Vec as MmapVec;
}

// Prover functionality is only available if the feature is set. Currently
// requires std. TODO: Make it work without std.
//
// Optional prover functionality. Note that prover requires std.
// TODO #[cfg(feature = "prover")]
// TODO pub mod pedersen_merkle;
// TODO #[cfg(feature = "prover")]
// TODO mod proofs;
#[cfg(feature = "prover")]
mod trace_table;

// Exports for prover
#[cfg(feature = "prover")]
pub use merkle_tree::Tree as MerkleTree;
// TODO #[cfg(feature = "prover")]
// TODO pub use proofs::stark_proof;
#[cfg(feature = "prover")]
pub use trace_table::TraceTable;

// Exports for benchmarking
// TODO: Avoid publicly exposing.
pub use fft::fft_cofactor_bit_reversed;
