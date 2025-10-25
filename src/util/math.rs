/// Rounds a float to the specified number of decimal places.
///
/// # Examples
/// ```
/// assert_eq!(round_to(3.14159, 2), 3.14);
/// assert_eq!(round_to(-1.9876, 1), -2.0);
/// ```
pub fn round_to(value: f32, places: u32) -> f32 {
    let factor = 10f32.powi(places as i32);
    (value * factor).round() / factor
}
