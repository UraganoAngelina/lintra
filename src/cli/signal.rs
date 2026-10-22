//! Signal handling belongs exclusively to the executable, never the engine.
use lintra::CancellationToken;

/// The same callback is used by the OS adapter and portable tests. It only sets
/// the cooperative flag: no I/O, process exit or report mutation occurs here.
fn cancellation_handler(token: CancellationToken) -> impl FnMut() + Send + 'static {
    move || token.cancel()
}
/// Register once per validate process, before input/output processing starts.
/// Installation failure is technical; validation must not silently ignore Ctrl+C.
pub fn install() -> Result<CancellationToken, ctrlc::Error> {
    let token = CancellationToken::default();
    ctrlc::set_handler(cancellation_handler(token.clone()))?;
    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn callback_cancels_shared_token_idempotently_and_keeps_other_runs_independent() {
        let token = CancellationToken::default();
        let other = CancellationToken::default();
        let mut callback = cancellation_handler(token.clone());
        assert!(!token.is_cancelled());
        callback();
        callback();
        assert!(token.is_cancelled());
        assert!(!other.is_cancelled());
    }
}
