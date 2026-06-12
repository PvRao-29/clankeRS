//! MCAP data logging, replay, and inspection for clankeRS.

pub mod compare;
pub mod inspect;
pub mod log;
pub mod replay;
pub mod sample;
pub mod writer;

pub use compare::{compare_logs, format_compare_report, CompareReport};
pub use inspect::{format_inspect_report, InspectReport, McapLog};
pub use log::McapRecord;
pub use replay::{Replay, ReplayResult, ReplaySummary};
pub use writer::McapWriter;
