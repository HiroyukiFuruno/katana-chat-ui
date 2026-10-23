use std::{
    env,
    fs::{File, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use kcu_host_support::{ManagedHostCommand, ManagedHostUpdate};
use serde_json::{Map, Value};

const TRACE_PATH_ENV: &str = "KCU_TRACE_PATH";

#[derive(Clone)]
pub struct TraceWriter {
    file: Arc<Mutex<File>>,
    failure: Arc<Mutex<Option<String>>>,
    reported: Arc<AtomicBool>,
}

impl TraceWriter {
    pub fn from_environment() -> Result<Option<Self>, String> {
        let Some(path) = env::var_os(TRACE_PATH_ENV) else {
            return Ok(None);
        };
        let path = PathBuf::from(path);
        if !path.is_absolute() {
            return Err(format!("{TRACE_PATH_ENV} must be an absolute path"));
        }
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|error| format!("failed to create trace: {error}"))?;
        Ok(Some(Self {
            file: Arc::new(Mutex::new(file)),
            failure: Arc::new(Mutex::new(None)),
            reported: Arc::new(AtomicBool::new(false)),
        }))
    }

    pub fn command(&self, host: &str, seq: u64, command: &ManagedHostCommand) {
        let mut event = Self::event(host, "command");
        event.insert("seq".to_string(), Value::from(seq));
        event.insert("command".to_string(), Value::from(format!("{command:?}")));
        self.write(Value::Object(event));
    }

    pub fn update(&self, host: &str, update: &ManagedHostUpdate) {
        let messages = update
            .surface
            .message_list
            .messages
            .iter()
            .map(|message| Value::from(message.id))
            .collect::<Vec<_>>();
        let mut event = Self::event(host, "update");
        event.insert("ack_seq".to_string(), Value::from(update.ack_seq));
        event.insert(
            "draft".to_string(),
            Value::from(update.surface.composer.text.as_str()),
        );
        event.insert("message_ids".to_string(), Value::Array(messages));
        event.insert(
            "error".to_string(),
            update.error.as_deref().map_or(Value::Null, Value::from),
        );
        event.insert(
            "effect".to_string(),
            update
                .effect
                .as_ref()
                .map_or(Value::Null, |effect| Value::from(format!("{effect:?}"))),
        );
        self.write(Value::Object(event));
    }

    fn event(host: &str, kind: &str) -> Map<String, Value> {
        let mut event = Map::new();
        event.insert("host".to_string(), Value::from(host));
        event.insert("kind".to_string(), Value::from(kind));
        event
    }

    fn write(&self, value: Value) {
        if self.failed() {
            return;
        }
        if let Err(error) = self.write_line(&value) {
            self.fail(error);
        }
    }

    fn write_line(&self, value: &Value) -> Result<(), String> {
        let mut file = self
            .file
            .lock()
            .map_err(|_| "trace writer lock poisoned".to_string())?;
        serde_json::to_writer(&mut *file, value)
            .map_err(|error| format!("failed to encode trace: {error}"))?;
        file.write_all(b"\n")
            .and_then(|_| file.flush())
            .map_err(|error| format!("failed to write trace: {error}"))
    }

    fn failed(&self) -> bool {
        match self.failure.lock() {
            Ok(failure) => failure.is_some(),
            Err(_) => {
                self.fail("trace failure state lock poisoned".to_string());
                true
            }
        }
    }

    fn fail(&self, error: String) {
        if let Ok(mut failure) = self.failure.lock()
            && failure.is_none()
        {
            *failure = Some(error.clone());
        }
        if !self.reported.swap(true, Ordering::AcqRel) {
            eprintln!("KCU_TRACE_ERROR: {error}");
        }
    }
}

#[cfg(test)]
mod tests;
