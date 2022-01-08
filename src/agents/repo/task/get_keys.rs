use crate::{agents::repo, objects::JsError};
use uuid::Uuid;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::IdbCursor;
use yew_agent::HandlerId;

#[derive(Debug)]
pub struct Task {
    stage: Stage,
    handler_id: HandlerId,
    kind: Kind,
    request: Option<web_sys::IdbRequest>,
    transaction: Option<web_sys::IdbTransaction>,
    keys: Vec<JsValue>,
}

#[derive(Debug)]
enum Stage {
    Init,
    WaitingForReadRequest,
    WaitingForTransaction,
    TransactionCompleted,
}

#[derive(Debug)]
pub enum Kind {
    ItemYearMonth(Uuid),
}

impl Kind {
    fn table_name(&self) -> &str {
        match &self {
            Self::ItemYearMonth(_) => "items",
        }
    }

    fn index_name(&self) -> Option<&str> {
        match &self {
            Self::ItemYearMonth(_) => Some("channel_id_year_month"),
        }
    }

    fn key_range(&self) -> Result<web_sys::IdbKeyRange, JsError> {
        match &self {
            Kind::ItemYearMonth(channel_id) => web_sys::IdbKeyRange::bound(
                &serde_wasm_bindgen::to_value(&vec![&*channel_id.to_string(), ""])?,
                &serde_wasm_bindgen::to_value(&vec![&*channel_id.to_string(), "9999-99"])?,
            )
            .map_err(Into::into),
        }
    }

    fn cursor_direction(&self) -> web_sys::IdbCursorDirection {
        match &self {
            Kind::ItemYearMonth(_) => web_sys::IdbCursorDirection::Prevunique,
        }
    }
}

impl Task {
    pub fn new(handler_id: HandlerId, kind: Kind) -> Self {
        Self {
            stage: Stage::Init,
            kind,
            handler_id,
            request: None,
            transaction: None,
            keys: Vec::new(),
        }
    }

    pub fn transaction_complete(&mut self) {
        self.stage = Stage::TransactionCompleted;
    }
}

impl super::TaskProcessor<Task> for super::super::Repo {
    fn process(&mut self, task: &mut Task) -> Result<bool, JsError> {
        match task.stage {
            Stage::Init => {
                let db = self.db.as_ref().ok_or("db not set")?;
                let trans = db.transaction_with_str_sequence_and_mode(
                    &serde_wasm_bindgen::to_value(&vec![task.kind.table_name()])?,
                    web_sys::IdbTransactionMode::Readonly,
                )?;
                let os = trans.object_store(task.kind.table_name())?;

                trans.set_onabort(Some(self.idb_closure_trans_abort.as_ref().unchecked_ref()));
                trans.set_onerror(Some(self.idb_closure_trans_error.as_ref().unchecked_ref()));
                trans.set_oncomplete(Some(
                    self.idb_closure_trans_complete.as_ref().unchecked_ref(),
                ));
                task.transaction = Some(trans);

                let key_range = task.kind.key_range()?;
                let request = match task.kind.index_name() {
                    Some(index_name) => {
                        os.index(index_name)?.open_cursor_with_range_and_direction(
                            &key_range,
                            task.kind.cursor_direction(),
                        )?
                    }
                    None => os.open_cursor_with_range_and_direction(
                        &key_range,
                        task.kind.cursor_direction(),
                    )?,
                };
                request.set_onsuccess(Some(self.idb_closure_success.as_ref().unchecked_ref()));
                request.set_onerror(Some(self.idb_closure_error.as_ref().unchecked_ref()));
                task.request = Some(request);

                task.stage = Stage::WaitingForReadRequest;
                Ok(false)
            }
            Stage::WaitingForReadRequest => {
                let request = task.request.as_ref().ok_or("request not set")?;

                match request.ready_state() {
                    web_sys::IdbRequestReadyState::Done => {
                        super::request_ok(request)?;

                        let result = request.result()?;

                        if result.is_null() {
                            task.stage = Stage::WaitingForTransaction;
                        } else {
                            let cursor: IdbCursor = request.result()?.dyn_into()?;

                            task.keys.push(cursor.key()?);
                            cursor.continue_()?;
                            task.stage = Stage::WaitingForReadRequest;
                        }

                        Ok(false)
                    }
                    _ => Ok(false),
                }
            }
            Stage::WaitingForTransaction => Ok(false),
            Stage::TransactionCompleted => {
                match &task.kind {
                    Kind::ItemYearMonth(_) => self.link.respond(
                        task.handler_id,
                        repo::Response::YearMonthKeys(
                            task.keys
                                .iter()
                                .map(|i| {
                                    serde_wasm_bindgen::from_value(i.clone()).map_err(|e| {
                                        JsError::from_str(&*format!(
                                            "error converting result: {:?}",
                                            e
                                        ))
                                    })
                                })
                                .collect::<Result<Vec<(String, String)>, JsError>>()?
                                .iter()
                                .map(|(_, key)| key.clone())
                                .collect(),
                        ),
                    ),
                }

                Ok(true)
            }
        }
    }
}
