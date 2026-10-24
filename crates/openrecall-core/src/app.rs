use crate::{
    AppResult, Clock, RandomSource,
    contracts::{Bootstrap, Request, Response},
    storage::{SCHEMA_VERSION, Store},
};
use std::{path::Path, sync::Arc};
pub struct App {
    pub(crate) store: Store,
    #[allow(dead_code)] // Used by scheduling/session tasks, injected from the first migration.
    pub(crate) clock: Arc<dyn Clock>,
    #[allow(dead_code)]
    pub(crate) random: Box<dyn RandomSource>,
}
impl App {
    pub fn open(
        path: &Path,
        clock: Arc<dyn Clock>,
        random: Box<dyn RandomSource>,
    ) -> AppResult<Self> {
        Ok(Self {
            store: Store::open(path)?,
            clock,
            random,
        })
    }
    pub fn execute(&mut self, request: Request) -> AppResult<Response> {
        self.store.begin()?;
        let result = self.dispatch(request);
        self.store.finish(result)
    }
    pub fn execute_json(&mut self, request: serde_json::Value) -> AppResult<serde_json::Value> {
        let request = serde_json::from_value(request)?;
        Ok(serde_json::to_value(self.execute(request)?)?)
    }
    fn dispatch(&mut self, request: Request) -> AppResult<Response> {
        match request {
            Request::Bootstrap {} => Ok(Response::Bootstrap(Bootstrap {
                settings: self.store.settings()?,
                schema_version: SCHEMA_VERSION,
            })),
        }
    }
}
