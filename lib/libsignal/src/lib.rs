// Mostly stolen from https://stackoverflow.com/a/77591939

/// Waits for a signal that requests a graceful shutdown, like SIGTERM or SIGINT.
#[cfg(unix)]
async fn wait_for_signal_impl() {
    use tokio::signal::unix::{SignalKind, signal};
    use tracing::Level;

    // Infos here:
    // https://www.gnu.org/software/libc/manual/html_node/Termination-Signals.html
    let mut signal_terminate = signal(SignalKind::terminate()).unwrap();
    let mut signal_interrupt = signal(SignalKind::interrupt()).unwrap();

    tokio::select! {
        _ = signal_terminate.recv() => {
            let span = tracing::span!(Level::INFO, "signal");
            let guard = span.enter();
            tracing::warn!("Received SIGTERM.");
            drop(guard);
        },
        _ = signal_interrupt.recv() => {
            let span = tracing::span!(Level::INFO, "signal");
            let guard = span.enter();
            tracing::warn!("Received SIGINT.");
            drop(guard);
        },
    };
}

/// Waits for a signal that requests a graceful shutdown, Ctrl-C (SIGINT).
#[cfg(windows)]
async fn wait_for_signal_impl() {
    use tokio::signal::windows;
    use tracing::Level;

    // https://learn.microsoft.com/en-us/windows/console/handlerroutine
    let mut signal_c = windows::ctrl_c().unwrap();
    let mut signal_break = windows::ctrl_break().unwrap();
    let mut signal_close = windows::ctrl_close().unwrap();
    let mut signal_shutdown = windows::ctrl_shutdown().unwrap();

    tokio::select! {
        _ = signal_c.recv() => {
            let span = tracing::span!(Level::INFO, "signal");
            let guard = span.enter();
            tracing::warn!("Received CTRL_C.");
            drop(guard);
        }
        _ = signal_break.recv() => {
            let span = tracing::span!(Level::INFO, "signal");
            let guard = span.enter();
            tracing::warn!("Received CTRL_BREAK.");
            drop(guard);
        }
        _ = signal_close.recv() => {
            let span = tracing::span!(Level::INFO, "signal");
            let guard = span.enter();
            tracing::warn!("Received CTRL_CLOSE.");
            drop(guard);
        }
        _ = signal_shutdown.recv() => {
            let span = tracing::span!(Level::INFO, "signal");
            let guard = span.enter();
            tracing::warn!("Received CTRL_SHUTDOWN.");
            drop(guard);
        }
    };
}

pub async fn wait_for_signal() {
    wait_for_signal_impl().await
}
