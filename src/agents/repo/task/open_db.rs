use crate::{agents::repo::Message, objects::JsError};
use std::collections::HashMap;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{IdbDatabase, IdbIndexParameters};

#[derive(Debug)]
pub struct Task {
    stage: Stage,
    closures: Vec<Closure<dyn Fn(web_sys::Event)>>,
    request: Option<web_sys::IdbOpenDbRequest>,
}

#[derive(Debug)]
pub enum Stage {
    Init,
    WaitingForRequest,
    UpdateRequested,
    Updated,
    Finalize,
}

impl Task {
    pub fn new() -> Self {
        Self {
            stage: Stage::Init,
            closures: Vec::new(),
            request: None,
        }
    }

    pub fn get_stage(&self) -> &Stage {
        &self.stage
    }

    pub fn store_closure(&mut self, closure: Closure<dyn Fn(web_sys::Event)>) {
        self.closures.push(closure);
    }

    pub fn set_request(&mut self, request: web_sys::IdbOpenDbRequest) {
        self.request = Some(request);
        self.stage = Stage::WaitingForRequest;
    }

    pub fn set_update_requested(&mut self) {
        self.stage = Stage::UpdateRequested;
    }

    pub fn set_updated(&mut self) {
        self.stage = Stage::Updated;
    }

    pub fn request_completed(&mut self) {
        self.stage = Stage::Finalize;
    }
}

impl super::TaskProcessor<Task> for super::super::Repo {
    fn process(&mut self, task: &mut Task) -> Result<bool, JsError> {
        match task.get_stage() {
            Stage::Init => {
                let window: web_sys::Window = web_sys::window().ok_or("could not get window")?;
                let idb_factory: web_sys::IdbFactory =
                    window.indexed_db()?.ok_or("could not get indexed db")?;
                let idb_open_request: web_sys::IdbOpenDbRequest =
                    idb_factory.open_with_u32("podcast-player", 1)?;
                let callback_update = self.link.callback(Message::OpenDbUpdate);
                let callback_success = self.link.callback(Message::OpenDbResult);
                let callback_error = self.link.callback(Message::OpenDbResult);
                let closure_update =
                    Closure::wrap(
                        Box::new(move |event: web_sys::Event| callback_update.emit(event))
                            as Box<dyn Fn(_)>,
                    );
                idb_open_request.set_onupgradeneeded(Some(closure_update.as_ref().unchecked_ref()));
                let closure_success = Closure::wrap(Box::new(move |event: web_sys::Event| {
                    callback_success.emit(Ok(event))
                }) as Box<dyn Fn(_)>);
                let closure_error = Closure::wrap(Box::new(move |event: web_sys::Event| {
                    callback_error.emit(Err(event))
                }) as Box<dyn Fn(_)>);
                idb_open_request.set_onsuccess(Some(closure_success.as_ref().unchecked_ref()));
                idb_open_request.set_onerror(Some(closure_error.as_ref().unchecked_ref()));

                task.store_closure(closure_update);
                task.store_closure(closure_success);
                task.store_closure(closure_error);
                task.set_request(idb_open_request);

                Ok(false)
            }
            Stage::WaitingForRequest => Ok(false),
            Stage::UpdateRequested => {
                let idb_db = IdbDatabase::from(
                    task.request
                        .as_ref()
                        .ok_or("could not get reference")?
                        .result()?,
                );
                let object_stores = vec![
                    "channels",
                    "feeds",
                    "items",
                    "enclosures",
                    "images",
                    "images-meta",
                    "configuration",
                ];
                let mut indices = HashMap::new();
                indices.insert(
                    "items",
                    vec![
                        (
                            "channel_id_year_month",
                            vec!["val.channel_id", "keys.year_month"],
                        ),
                        ("download_required", vec!["keys.download_required"]),
                        ("download_ok", vec!["keys.download_ok"]),
                    ],
                );

                for object_store in object_stores {
                    match idb_db.create_object_store(object_store) {
                        Ok(os) => {
                            if indices.contains_key(object_store) {
                                for (name, key_paths) in &indices[object_store] {
                                    match os.create_index_with_str_sequence_and_optional_parameters(
                                        name,
                                        &serde_wasm_bindgen::to_value(key_paths)?,
                                        &IdbIndexParameters::new(),
                                    ) {
                                        Ok(_) => log::info!("created index {}", name),
                                        Err(e) => log::error!("failed to create index: {:?}", e),
                                    };
                                }
                            }
                        }
                        Err(e) => {
                            log::error!(
                                "failed to create object store \"{}\": {:?}",
                                object_store,
                                e
                            );
                        }
                    }
                }

                task.set_updated();
                Ok(false)
            }
            Stage::Updated => Ok(false),
            Stage::Finalize => {
                self.db = Some(
                    task.request
                        .as_ref()
                        .ok_or("could not get reference")?
                        .result()?
                        .into(),
                );

                Ok(true)
            }
        }
    }
}
