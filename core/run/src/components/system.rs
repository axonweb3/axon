use std::panic::PanicInfo;

use backtrace::Backtrace;

use protocol::tokio;
#[cfg(unix)]
use protocol::tokio::signal::unix as os_impl;

pub(crate) async fn set_ctrl_c_handle() {
    let ctrl_c_handler = tokio::spawn(async {
        #[cfg(windows)]
        let _ = tokio::signal::ctrl_c().await;
        #[cfg(unix)]
        {
            let mut sigtun_int = os_impl::signal(os_impl::SignalKind::interrupt()).unwrap();
            let mut sigtun_term = os_impl::signal(os_impl::SignalKind::terminate()).unwrap();
            tokio::select! {
                _ = sigtun_int.recv() => {}
                _ = sigtun_term.recv() => {}
            };
        }
    });

    // register channel of panic
    let (panic_sender, mut panic_receiver) = tokio::sync::mpsc::channel::<()>(1);

    std::panic::set_hook(Box::new(move |info: &PanicInfo| {
        let panic_sender = panic_sender.clone();
        panic_log(info);
        panic_sender.try_send(()).expect("panic_receiver is droped");
    }));

    tokio::select! {
        _ = ctrl_c_handler => { log::info!("ctrl + c is pressed, quit.") },
        _ = panic_receiver.recv() => { log::info!("child thread panic, quit.") },
    };
}

fn panic_log(info: &PanicInfo) {
    let backtrace = Backtrace::new();
    let thread = std::thread::current();
    let name = thread.name().unwrap_or("unnamed");
    let location = info.location().unwrap(); // The current implementation always returns Some
    let msg = match info.payload().downcast_ref::<&'static str>() {
        Some(s) => *s,
        None => match info.payload().downcast_ref::<String>() {
            Some(s) => &**s,
            None => "Box<Any>",
        },
    };
    log::error!(
        target: "panic", "thread '{}' panicked at '{}': {}:{} {:?}",
        name,
        msg,
        location.file(),
        location.line(),
        backtrace,
    );
}
