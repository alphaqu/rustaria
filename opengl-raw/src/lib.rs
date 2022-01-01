pub mod gll;

pub static mut ERROR_HANDLING: bool = true;

pub fn set_errors(errors: bool) {
    unsafe  {
        ERROR_HANDLING = errors;
    }
}