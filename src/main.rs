#![macro_use]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

use core::future::Future;
use drogue_device::*;
use embassy::time::{with_timeout, Duration, TimeoutError, Timer};

#[embassy::main]
async fn main(spawner: embassy::executor::Spawner) {
    env_logger::builder()
    .filter_level(log::LevelFilter::Info)
    .format_timestamp_nanos()
    .init();

    static ADD: ActorContext<Counter> = ActorContext::new();
    static SUB: ActorContext<Counter> = ActorContext::new();

    let sub = ADD.mount(spawner, Counter("add", "add", None));
    SUB.mount(spawner, Counter("sub", "sub", Some(sub)));
    return 0;
}

pub struct Counter(&'static str, &'static str, Option<Address<Counter>>);

pub enum Message {
    Int(i32),
    Register(Address<Counter>),
}

impl Actor for Counter {
    type Message<'a> = Message;

    type OnMountFuture<'m, M>
    where
        M: 'm, = impl Future<Output = ()> + 'm;
    fn on_mount<'m, M>(
        &'m mut self,
        me: Address<Self>,
        inbox: &'m mut M,
    ) -> Self::OnMountFuture<'m, M>
    where
        M: Inbox<Self> + 'm,
    {
        async move {
            if let Some(sub) = self.2 {
                sub.notify(Message::Register(me)).unwrap();
            }
            log::info!("[{}] started!", self.0);

            let mut add: Option<Address<Counter>> = None;

            loop {
                match with_timeout(Duration::from_secs(2), inbox.next()).await {
                    Ok(r) => match r {
                        Some(mut m) => match *m.message() {
                            Message::Int(message) => {
                                log::info!("[{}] got message with value: {}", self.0, message);
                                if let Some(add) = add {
                                    Timer::after(Duration::from_secs(1)).await;
                                    add.notify(Message::Int(message + 2)).unwrap();
                                } 
                                if let Some(sub) = self.2 {
                                    if message != 5 {
                                        Timer::after(Duration::from_secs(1)).await;
                                        sub.notify(Message::Int(message - 1)).unwrap();
                                    }
                                }
                            },
                            Message::Register(p) => {
                                add.replace(p);
                                p.notify(Message::Int(0)).unwrap();
                            }
                        },
                        _ => {}
                    },
                    Err(TimeoutError) => {
                        log::info!("Timeout");
                        break;
                    }
                }
            }
        }
    }
}