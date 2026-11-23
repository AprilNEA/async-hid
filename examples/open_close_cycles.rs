//! Opens and drops one device repeatedly, then waits, so a memory tool can
//! check that nothing an open allocates outlives its handle.
//!
//! On macOS, count what survives with `heap <pid> -s` while the example waits.
//! The objects an open creates (`HIDElement`, `HIDLibElement`, ...) should not
//! grow with the cycle count.
//!
//! Usage: `cargo run --example open_close_cycles -- <vid> <pid> <usage_page> [cycles]`
//! (ids in hex, e.g. `046d b019 1 100`)

use std::time::Duration;

use async_hid::{HidBackend, HidResult};
use futures_lite::stream::StreamExt;

fn hex(arg: Option<String>, name: &str) -> u16 {
    let arg = arg.unwrap_or_else(|| panic!("missing {name}"));
    u16::from_str_radix(arg.trim_start_matches("0x"), 16).unwrap_or_else(|_| panic!("{name} must be hex"))
}

#[pollster::main]
async fn main() -> HidResult<()> {
    let mut args = std::env::args().skip(1);
    let vendor_id = hex(args.next(), "vid");
    let product_id = hex(args.next(), "pid");
    let usage_page = hex(args.next(), "usage_page");
    let cycles: usize = args
        .next()
        .map_or(100, |n| n.parse().expect("cycles must be a number"));

    let backend = HidBackend::default();
    let mut opened = 0;
    for _ in 0..cycles {
        let device = backend
            .enumerate()
            .await?
            .find(|info| info.vendor_id == vendor_id && info.product_id == product_id && info.usage_page == usage_page)
            .await
            .expect("Could not find device");
        let (reader, writer) = device.open().await?;
        drop((reader, writer));
        opened += 1;
    }

    println!("opened and dropped {opened} times; pid {} — waiting 10 min", std::process::id());
    std::thread::sleep(Duration::from_secs(600));
    Ok(())
}
