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
pub unsafe fn as_bytes<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts(
        (p as *const T) as *const u8,
        std::mem::size_of::<T>()
    )
}

pub fn print_type_of<T>(_: &T) {
    log::info!("T is of Type {:?}", std::any::type_name::<T>());
}
use nalgebra;
pub fn print_row_ordered_matrix(mat4: nalgebra::Matrix4::<f32>) {
    for i in mat4.iter() {

    }
}