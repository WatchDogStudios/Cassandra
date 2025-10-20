use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogEvent {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub component: Option<String>,
    pub tenant_id: Option<String>,
    pub project_id: Option<String>,
    pub metadata: serde_json::Value,
}

impl LogEvent {
    pub fn new(level: LogLevel, message: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
            timestamp: Utc::now(),
            component: None,
            tenant_id: None,
            project_id: None,
            metadata: serde_json::Value::Null,
        }
    }

    pub fn with_component(mut self, component: impl Into<String>) -> Self {
        self.component = Some(component.into());
        self
    }

    pub fn with_tenant(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }

    pub fn with_project(mut self, project_id: impl Into<String>) -> Self {
        self.project_id = Some(project_id.into());
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

pub trait LogSink: Send + Sync {
    fn on_event(&self, event: &LogEvent);
}

#[derive(Clone, Default)]
pub struct LogPipeline {
    sinks: Arc<RwLock<Vec<Arc<dyn LogSink>>>>,
}

impl LogPipeline {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_sink(&self, sink: Arc<dyn LogSink>) {
        self.sinks
            .write()
            .expect("log sinks lock poisoned")
            .push(sink);
    }

    pub fn emit(&self, event: LogEvent) {
        let sinks = self
            .sinks
            .read()
            .expect("log sinks lock poisoned")
            .clone();
        for sink in sinks {
            sink.on_event(&event);
        }
    }
}

#[derive(Clone, Default)]
pub struct InMemoryLogSink {
    events: Arc<RwLock<Vec<LogEvent>>>,
}

impl InMemoryLogSink {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn snapshot(&self) -> Vec<LogEvent> {
        self.events
            .read()
            .expect("log sink lock poisoned")
            .clone()
    }
}

impl LogSink for InMemoryLogSink {
    fn on_event(&self, event: &LogEvent) {
        self.events
            .write()
            .expect("log sink lock poisoned")
            .push(event.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn pipeline_forwards_events_to_sink() {
        let pipeline = LogPipeline::new();
        let sink = Arc::new(InMemoryLogSink::new());
        pipeline.register_sink(sink.clone());

        pipeline.emit(LogEvent::new(LogLevel::Info, "upload started"));

        let events = sink.snapshot();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].message, "upload started");
    }
}
