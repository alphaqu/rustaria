pub mod gll;

pub static mut ERROR_HANDLING: bool = true;

fn main() {
}

pub fn set_errors(errors: bool) {
    unsafe  {
        ERROR_HANDLING = errors;
    }
}