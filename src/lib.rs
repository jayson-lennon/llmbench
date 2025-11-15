#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::must_use_candidate)]
#![allow(non_snake_case)]
#![allow(clippy::needless_update)]
#![allow(clippy::len_without_is_empty)]

pub mod feat;

pub mod all_bench_results;
pub mod bench_loader;
pub mod bench_result;
pub mod completion;
pub mod evaluator;
pub mod init;
pub mod models;
pub mod promptrequest;
pub mod result_writer;
