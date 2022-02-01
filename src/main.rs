#![no_main]

#![feature(type_alias_impl_trait)]

use embassy::executor::{Executor, Spawner};
use embassy::time::{Duration, Timer};
use embassy::util::Forever;
use log::*;

#[embassy::task]
async fn run(mut val: i32) {
    loop {
        info!("tick from runner");
        info!("val is {}", val);
        val = val + 1;
        Timer::after(Duration::from_secs(1)).await;
    }
}

static EXECUTOR: Forever<Executor> = Forever::new();
#[embassy::main]
async fn main(spawner: Spawner) {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .format_timestamp_nanos()
        .init();
    info!("Going to spawn first actor");
    let x = 2;
    let y = 8;
    spawner.spawn(run(x)).unwrap();

    let executor = EXECUTOR.put(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(run(x)).unwrap()
        spawner.spawn(run(y)).unwrap();
    });

//    let y = 10;
//    spawner.spawn(run(y)).unwrap();
}


