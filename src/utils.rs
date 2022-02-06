use crate::objects::JsError;
use web_sys::ConnectionType;

pub fn get_connection_type() -> Result<ConnectionType, JsError> {
    let net_info = web_sys::window()
        .ok_or("could not obtain window")?
        .navigator()
        .connection()?;
    match net_info.is_falsy() {
        true => Ok(ConnectionType::Unknown),
        false => Ok(net_info.type_()),
    }
}
