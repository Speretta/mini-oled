pub mod canvas;
pub mod properties;
pub mod sh1106;

#[macro_export]
macro_rules! fast_mul {
    ($value:expr, $right:expr) => {{
        let value_u32 = ($value) as u32;
        if $right > 0 && ($right & ($right - 1)) == 0 {
            value_u32 << $right.trailing_zeros()
        } else {
            value_u32 * $right
        }
    }};
}



