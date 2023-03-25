use nalgebra;
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
    ::std::slice::from_raw_parts((p as *const T) as *const u8, std::mem::size_of::<T>())
}

pub fn print_type_of<T>(_: &T) {
    log::info!("T is of Type {:?}", std::any::type_name::<T>());
}
pub fn print_column_order_matrix_row_ordered(mat4: &nalgebra::Matrix4<f32>) {
    let mut mat_arr: [[f32; 4]; 4] = [[0.0; 4]; 4];
    for (position, value) in mat4.iter().enumerate() {
        let row_value = position % 4;
        let coloumn_value = position / 4;
        mat_arr[row_value][coloumn_value] = *value;
    }
    // TODO: Ugly fix later
    let mut mat_output = String::new();
    for row in mat_arr.iter() {
        mat_output.push_str("\n[");
        for col in row.iter() {
            let col_str = col.to_owned().to_string();
            mat_output.push_str(&col_str);
            mat_output.push_str(" ");
        }
        mat_output.push_str("]");
    }
    log::info!("Row Ordered Matrix {}", mat_output);
}

pub unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
}
