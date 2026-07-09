//! # clankeRS procedural macros
//!
//! Attribute macros that reduce boilerplate for robot nodes and replay tests.
//!
//! ## [`node`] — robot entry point
//!
//! Expands an `async fn` into a `main` that loads `clankeRS.toml`, initializes
//! tracing, and runs the Tokio runtime:
//!
//! ```ignore
//! use clankers::prelude::*;
//!
//! #[clankers::node]
//! async fn main(ctx: RobotContext) -> RobotResult<()> {
//!     Ok(())
//! }
//! ```
//!
//! ## [`replay_test`] — MCAP fixture test
//!
//! Wraps an async test body with a [`clankers::testing::ReplayContext`](https://docs.rs/clankers/latest/clankers/testing/struct.ReplayContext.html):
//!
//! ```ignore
//! use clankers::prelude::*;
//!
//! #[clankers::replay_test("tests/fixtures/camera_log.mcap")]
//! async fn replays_cleanly(ctx: ReplayContext) -> RobotResult<()> {
//!     let result = ctx.run_replay(|_msg| async { Ok(()) }).await?;
//!     assert_no_panics(&result)?;
//!     Ok(())
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, LitStr};

/// Marks an async function as a clankeRS robot node entry point.
///
/// Generates `fn main()` that loads [`RobotContext`](https://docs.rs/clankers/latest/clankers/struct.RobotContext.html) from
/// the working directory and runs the async body on a Tokio runtime.
#[proc_macro_attribute]
pub fn node(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let body = &input.block;

    let expanded = quote! {
        fn main() {
            clankers::runtime::init_tracing();
            let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
            rt.block_on(async {
                let ctx = clankers::RobotContext::from_work_dir(".")
                    .expect("load clankeRS.toml");
                run_node(ctx).await.expect("node failed");
            });
        }

        async fn run_node(ctx: clankers::RobotContext) -> clankers::RobotResult<()> #body
    };

    TokenStream::from(expanded)
}

/// Marks a test function as a replay-based test using an MCAP fixture path.
///
/// The attribute argument is the path to the `.mcap` file (relative to the crate root).
#[proc_macro_attribute]
pub fn replay_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mcap_path = parse_macro_input!(attr as LitStr);
    let path_val = mcap_path.value();

    let input = parse_macro_input!(item as ItemFn);
    let body = &input.block;

    let expanded = quote! {
        #[tokio::test]
        async fn replay_test_impl() {
            let ctx = clankers::testing::ReplayContext::new(#path_val);
            async fn run(ctx: clankers::testing::ReplayContext) -> clankers::RobotResult<()> #body
            run(ctx).await.expect("replay test failed");
        }
    };

    TokenStream::from(expanded)
}
