use super::{Sink, Status};

pub(super) fn has_permission() -> bool {
    true
}
pub(super) fn request_permission() -> bool {
    false
}
pub(super) fn open_permission_settings() -> Result<(), String> {
    Err("Global keyboard input supports macOS and Windows only".to_owned())
}
pub(super) fn listen(_sink: &Sink) -> Result<(), Status> {
    Err(Status::Failed(
        "Global keyboard input supports macOS and Windows only".to_owned(),
    ))
}
