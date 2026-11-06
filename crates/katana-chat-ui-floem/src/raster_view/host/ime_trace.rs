use super::RasterChatView;
use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

fn ime_state_record(
    source: &str,
    sequence: u64,
    observed_at: SystemTime,
    marked: bool,
) -> Result<String, &'static str> {
    let seconds = observed_at
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "before-unix-epoch")?
        .as_secs_f64();
    Ok(format!(
        "KCU_IME_STATE {{\"kind\":\"ime-state\",\"source\":\"{source}\",\"seq\":{sequence},\"observed_at\":{seconds:.6},\"marked\":{marked}}}"
    ))
}

fn ime_state_output(source: &str, sequence: u64, observed_at: SystemTime, marked: bool) -> String {
    match ime_state_record(source, sequence, observed_at, marked) {
        Ok(record) => record,
        Err(error) => format!("KCU_IME_STATE_ERROR source={source} clock={error}"),
    }
}

impl<OnAction, OnDraft> RasterChatView<OnAction, OnDraft> {
    pub(super) fn trace_ime_state(&mut self, marked: bool) {
        if env::var("KCU_IME_STATE_TRACE").as_deref() != Ok("1") {
            return;
        }
        let source = "floem-kuc-runtime-frame";
        self.ime_trace_seq = self.ime_trace_seq.saturating_add(1);
        eprintln!(
            "{}",
            ime_state_output(source, self.ime_trace_seq, SystemTime::now(), marked)
        );
    }
}

#[cfg(test)]
mod ime_trace_tests {
    use super::{ime_state_output, ime_state_record};
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn formats_typed_state_record() -> Result<(), &'static str> {
        let record = ime_state_record(
            "floem-kuc-runtime-frame",
            7,
            UNIX_EPOCH + Duration::from_secs_f64(12.5),
            false,
        )?;
        assert!(record.contains("\"source\":\"floem-kuc-runtime-frame\""));
        assert!(record.contains("\"observed_at\":12.500000"));
        assert!(record.ends_with("\"marked\":false}"));
        Ok(())
    }

    #[test]
    fn rejects_pre_epoch_clock_without_state_record() {
        assert_eq!(
            ime_state_record(
                "floem-kuc-runtime-frame",
                8,
                UNIX_EPOCH - Duration::from_secs(1),
                true,
            ),
            Err("before-unix-epoch")
        );
        assert_eq!(
            ime_state_output(
                "floem-kuc-runtime-frame",
                8,
                UNIX_EPOCH - Duration::from_secs(1),
                true,
            ),
            "KCU_IME_STATE_ERROR source=floem-kuc-runtime-frame clock=before-unix-epoch"
        );
    }
}
