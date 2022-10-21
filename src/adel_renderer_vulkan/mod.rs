// Simple offset_of macro akin to C++ offsetof, taken from Ash source code
#[macro_export]
macro_rules! offset_of {
    ($base:path, $field:ident) => {{
        #[allow(unused_unsafe)]
        unsafe {
            let b: $base = std::mem::zeroed();
            (&b.$field as *const _ as isize) - (&b as *const _ as isize)
        }
    }};
}

pub mod utility;
mod renderer;

pub mod definitions;
pub use definitions::*;
pub use renderer::*;
