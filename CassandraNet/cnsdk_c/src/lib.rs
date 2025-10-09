//! C ABI for CassandraNet (stub implementation)
use cncore::init_tracing;
use cncore::platform::{PlatformError, PlatformServices, TaskRequest};
use libc::c_char;
use serde_json::Value;
use std::ffi::{CStr, CString};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use uuid::Uuid;

static INIT: AtomicBool = AtomicBool::new(false);

const ERR_INVALID: i32 = -1;
const ERR_UNAUTHORIZED: i32 = -2;
const ERR_INTERNAL: i32 = -3;

#[repr(C)]
pub struct cass_session {
    _private: *mut std::ffi::c_void,
}

#[repr(C)]
pub struct cass_config {
    pub api_key: *const c_char,
    pub gateway_url: *const c_char,
}

#[repr(C)]
pub struct cass_uuid {
    pub bytes: [u8; 16],
}

fn platform() -> Arc<PlatformServices> {
    PlatformServices::global().unwrap_or_else(PlatformServices::init_global)
}

fn uuid_to_c(id: Uuid) -> cass_uuid {
    cass_uuid {
        bytes: *id.as_bytes(),
    }
}

fn uuid_from_c(ptr: *const cass_uuid) -> Option<Uuid> {
    let data = unsafe { ptr.as_ref()? };
    Some(Uuid::from_bytes(data.bytes))
}

fn from_c_str(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let s = unsafe { CStr::from_ptr(ptr) };
    s.to_str().ok().map(|s| s.to_owned())
}

fn set_c_string(out: *mut *mut c_char, value: String) -> i32 {
    if out.is_null() {
        return ERR_INVALID;
    }
    match CString::new(value) {
        Ok(cstr) => {
            unsafe {
                *out = cstr.into_raw();
            }
            0
        }
        Err(_) => ERR_INVALID,
    }
}

fn map_error(err: PlatformError) -> i32 {
    match err {
        PlatformError::InvalidInput(_) => ERR_INVALID,
        PlatformError::Unauthorized | PlatformError::Forbidden => ERR_UNAUTHORIZED,
        PlatformError::NotFound(_) => ERR_INVALID,
        PlatformError::Conflict(_) => ERR_INVALID,
        PlatformError::Internal(_) => ERR_INTERNAL,
    }
}

#[no_mangle]
pub extern "C" fn cass_string_free(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(ptr);
    }
}

#[no_mangle]
pub extern "C" fn cass_init(cfg: *const cass_config) -> i32 {
    if cfg.is_null() {
        return ERR_INVALID;
    }
    if INIT.swap(true, Ordering::SeqCst) {
        return 0;
    }
    init_tracing();
    let _ = platform();
    0
}

#[no_mangle]
pub extern "C" fn cass_authenticate(api_key: *const c_char) -> i32 {
    let api_key = match from_c_str(api_key) {
        Some(k) => k,
        None => return ERR_INVALID,
    };
    match platform().auth().authenticate_api_key(&api_key) {
        Ok(_) => 0,
        Err(err) => map_error(err),
    }
}

#[no_mangle]
pub extern "C" fn cass_send_metric(name: *const c_char, value: f64) -> i32 {
    if name.is_null() {
        return ERR_INVALID;
    }
    let _name = unsafe { CStr::from_ptr(name) };
    let _value = value;
    0
}

#[no_mangle]
pub extern "C" fn cass_get_server_session(out_session: *mut cass_session) -> i32 {
    if out_session.is_null() {
        return ERR_INVALID;
    }
    unsafe {
        *out_session = cass_session {
            _private: std::ptr::null_mut(),
        };
    }
    0
}

#[no_mangle]
pub extern "C" fn cass_shutdown() {
    INIT.store(false, Ordering::SeqCst);
}

#[no_mangle]
pub extern "C" fn cass_create_tenant(name: *const c_char, out_id: *mut cass_uuid) -> i32 {
    let name = match from_c_str(name) {
        Some(n) => n,
        None => return ERR_INVALID,
    };
    if out_id.is_null() {
        return ERR_INVALID;
    }
    match platform().provisioning().create_tenant(name) {
        Ok(tenant) => {
            unsafe {
                *out_id = uuid_to_c(tenant.id);
            }
            0
        }
        Err(err) => map_error(err),
    }
}

#[no_mangle]
pub extern "C" fn cass_create_project(
    tenant_id: *const cass_uuid,
    name: *const c_char,
    out_id: *mut cass_uuid,
) -> i32 {
    let tenant = match uuid_from_c(tenant_id) {
        Some(id) => id,
        None => return ERR_INVALID,
    };
    let name = match from_c_str(name) {
        Some(n) => n,
        None => return ERR_INVALID,
    };
    if out_id.is_null() {
        return ERR_INVALID;
    }
    match platform().provisioning().create_project(tenant, name) {
        Ok(project) => {
            unsafe {
                *out_id = uuid_to_c(project.id);
            }
            0
        }
        Err(err) => map_error(err),
    }
}

#[no_mangle]
pub extern "C" fn cass_register_agent(
    tenant_id: *const cass_uuid,
    project_id: *const cass_uuid,
    hostname: *const c_char,
    out_agent_id: *mut cass_uuid,
    out_api_key: *mut *mut c_char,
) -> i32 {
    let tenant = match uuid_from_c(tenant_id) {
        Some(id) => id,
        None => return ERR_INVALID,
    };
    let project = match uuid_from_c(project_id) {
        Some(id) => id,
        None => return ERR_INVALID,
    };
    let hostname = match from_c_str(hostname) {
        Some(h) => h,
        None => return ERR_INVALID,
    };
    if out_agent_id.is_null() || out_api_key.is_null() {
        return ERR_INVALID;
    }
    match platform()
        .provisioning()
        .register_agent(tenant, project, hostname)
    {
        Ok(provisioned) => {
            unsafe {
                *out_agent_id = uuid_to_c(provisioned.agent.id);
            }
            set_c_string(out_api_key, provisioned.api_key.value)
        }
        Err(err) => map_error(err),
    }
}

#[no_mangle]
pub extern "C" fn cass_issue_agent_token(
    agent_id: *const cass_uuid,
    out_token: *mut *mut c_char,
) -> i32 {
    let agent = match uuid_from_c(agent_id) {
        Some(id) => id,
        None => return ERR_INVALID,
    };
    if out_token.is_null() {
        return ERR_INVALID;
    }
    match platform().provisioning().issue_agent_token(agent) {
        Ok(token) => set_c_string(out_token, token.token),
        Err(err) => map_error(err),
    }
}

#[no_mangle]
pub extern "C" fn cass_schedule_task(
    tenant_id: *const cass_uuid,
    kind: *const c_char,
    payload_json: *const c_char,
    out_task_id: *mut cass_uuid,
) -> i32 {
    let tenant = match uuid_from_c(tenant_id) {
        Some(id) => id,
        None => return ERR_INVALID,
    };
    let kind = match from_c_str(kind) {
        Some(k) => k,
        None => return ERR_INVALID,
    };
    if out_task_id.is_null() {
        return ERR_INVALID;
    }
    let payload = if payload_json.is_null() {
        Value::Null
    } else {
        match from_c_str(payload_json) {
            Some(raw) => serde_json::from_str(&raw).unwrap_or(Value::Null),
            None => Value::Null,
        }
    };
    let request = TaskRequest {
        tenant_id: tenant,
        kind,
        payload,
    };
    match platform().orchestration().schedule_task(request) {
        Ok(task) => {
            unsafe {
                *out_task_id = uuid_to_c(task.id);
            }
            0
        }
        Err(err) => map_error(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn ffi_provisioning_flow() {
        let cfg = cass_config {
            api_key: ptr::null(),
            gateway_url: ptr::null(),
        };
        assert_eq!(cass_init(&cfg), 0);

        let tenant_name = CString::new("Tenant").unwrap();
        let mut tenant_id = cass_uuid { bytes: [0; 16] };
        assert_eq!(cass_create_tenant(tenant_name.as_ptr(), &mut tenant_id), 0);

        let project_name = CString::new("Project").unwrap();
        let mut project_id = cass_uuid { bytes: [0; 16] };
        assert_eq!(
            cass_create_project(&tenant_id, project_name.as_ptr(), &mut project_id),
            0
        );

        let hostname = CString::new("agent-local").unwrap();
        let mut agent_id = cass_uuid { bytes: [0; 16] };
        let mut api_key_ptr: *mut c_char = ptr::null_mut();
        assert_eq!(
            cass_register_agent(
                &tenant_id,
                &project_id,
                hostname.as_ptr(),
                &mut agent_id,
                &mut api_key_ptr,
            ),
            0
        );
        assert!(!api_key_ptr.is_null());
        cass_string_free(api_key_ptr);

        let mut token_ptr: *mut c_char = ptr::null_mut();
        assert_eq!(cass_issue_agent_token(&agent_id, &mut token_ptr), 0);
        assert!(!token_ptr.is_null());
        cass_string_free(token_ptr);

        let mut task_id = cass_uuid { bytes: [0; 16] };
        let task_kind = CString::new("heartbeat").unwrap();
        let payload = CString::new("{\"ok\":true}").unwrap();
        assert_eq!(
            cass_schedule_task(
                &tenant_id,
                task_kind.as_ptr(),
                payload.as_ptr(),
                &mut task_id
            ),
            0
        );
        cass_shutdown();
    }
}
