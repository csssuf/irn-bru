use actix_web::{http, server,  App, HttpResponse, Json, State};
use serde::{Deserialize, Serialize};

use std::cell::RefCell;

use crate::machine::Machine;

pub(crate) struct ApiState {
    machine: RefCell<Machine>,
}

impl ApiState {
    pub(crate) fn from_machine(machine: Machine) -> ApiState {
        ApiState { machine: RefCell::new(machine) }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
enum ApiResponse {
    Error(ApiError),
    Success(ApiSuccess),
}

#[derive(Clone, Debug, Serialize)]
struct ApiSuccess {
    message: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiError {
    error: String,
    error_code: u16,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct DropRequest {
    slot: usize,
}

pub(crate) fn drop(state: State<ApiState>, body: Json<DropRequest>) -> HttpResponse {
    let mut machine = state.machine.borrow_mut();

    if body.slot == 0 || body.slot > machine.slots() {
        return HttpResponse::BadRequest()
            .json(ApiResponse::Error(ApiError {
                error: "Bad slot number".to_owned(),
                error_code: 400,
            }
        ));
    }

    match machine.drop(body.slot - 1) {
        Ok(true) => {
            let message = format!("Dropped drink from slot {}", body.slot);
            HttpResponse::Ok().json(ApiResponse::Success(ApiSuccess { message }))
        }
        Ok(false) => {
            let error = format!("Slot {} disabled", body.slot);
            HttpResponse::ServiceUnavailable().json(ApiResponse::Error(ApiError { error, error_code: 503 }))
        },
        Err(e) => {
            let error = format!("error: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::Error(ApiError { error, error_code: 500 }))
        }
    }
}
