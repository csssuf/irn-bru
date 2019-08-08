use actix_web::{http, server,  App, HttpResponse, Json, State};
use serde::{Deserialize, Serialize};

use std::cell::RefCell;
use std::error::Error;

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
    Health(ApiHealth),
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

#[derive(Clone, Debug, Serialize)]
struct ApiHealth {
    slots: Vec<String>,
    temp: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct DropRequest {
    slot: usize,
}

pub(crate) fn drop(state: State<ApiState>, body: Json<DropRequest>) -> HttpResponse {
    match drop_impl(state, body) {
        Ok(response) => response,
        Err(e) => {
            let error = format!("error: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::Error(ApiError { error, error_code: 500 }))
        }
    }
}

fn drop_impl(state: State<ApiState>, body: Json<DropRequest>) -> Result<HttpResponse, Box<dyn Error>> {
    let mut machine = state.machine.borrow_mut();

    if body.slot == 0 || body.slot > machine.slots() {
        return Ok(HttpResponse::BadRequest()
            .json(ApiResponse::Error(ApiError {
                error: "Bad slot number".to_owned(),
                error_code: 400,
            }
        )));
    }

    if machine.drop(body.slot - 1)? {
        let message = format!("Dropped drink from slot {}", body.slot);
        Ok(HttpResponse::Ok().json(ApiResponse::Success(ApiSuccess { message })))
    } else {
        let error = format!("Slot {} disabled", body.slot);
        Ok(HttpResponse::ServiceUnavailable().json(ApiResponse::Error(ApiError { error, error_code: 503 })))
    }
}

pub(crate) fn health(state: State<ApiState>) -> HttpResponse {
    match health_impl(state) {
        Ok(response) => response,
        Err(e) => {
            let error = format!("error: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::Error(ApiError { error, error_code: 500 }))
        }
    }
}

fn health_impl(state: State<ApiState>) -> Result<HttpResponse, Box<dyn Error>> {
    let machine = state.machine.borrow();

    let temp = machine.get_temperature()?;
    let bus_ids = machine.get_bus_ids();
    let slots = machine.get_active()?
        .iter()
        .zip(bus_ids.iter())
        .enumerate()
        .map(|(idx, (status, bus_id))| {
            if *status {
                format!("Slot {} ({}) is active", idx + 1, bus_id)
            } else {
                format!("Slot {} ({}) is disabled", idx + 1, bus_id)
            }
        })
        .collect();
    Ok(HttpResponse::Ok().json(ApiResponse::Health(ApiHealth { slots, temp })))
}
