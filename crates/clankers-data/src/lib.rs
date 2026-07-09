//! # MCAP logging, replay, and inspection
//!
//! Read, write, inspect, and compare robot logs in [MCAP](https://mcap.dev) format.
//! Works with the `clankers` CLI (`clankers inspect`, `clankers replay`, `clankers compare`)
//! and with [`ReplayContext`](https://docs.rs/clankers-testing/latest/clankers_testing/struct.ReplayContext.html) in tests.
//!
//! ## Inspect a log
//!
//! ```no_run
//! use clankers_data::{format_inspect_report, McapLog};
//!
//! let log = McapLog::open("sample_data/camera_log.mcap")?;
//! println!("{}", format_inspect_report(log.report()));
//! # Ok::<(), clankers_core::RobotError>(())
//! ```
//!
//! ## Replay messages
//!
//! ```no_run
//! # #[tokio::main]
//! # async fn main() -> clankers_core::RobotResult<()> {
//! use clankers_data::Replay;
//!
//! let replay = Replay::from_mcap("sample_data/camera_log.mcap")?;
//! let result = replay.run(|_msg| async { Ok(()) }).await?;
//! println!("handled {} messages", result.summary.input_messages);
//! # Ok(())
//! # }
//! ```
//!
//! ## Key types
//!
//! | Type | Role |
//! |------|------|
//! | [`McapLog`] | Open a log and produce an [`InspectReport`] |
//! | [`Replay`] | Stream messages with optional per-message handler |
//! | [`ReplaySummary`] | Counts, duration, topics seen |
//! | [`CompareReport`] | Diff two logs (used by `clankers compare`) |
//! | [`McapWriter`] | Write MCAP files from nodes |

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
