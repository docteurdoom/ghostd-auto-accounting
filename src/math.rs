pub fn precise(input: f64) -> f64 {
    let zeros = 100000000.0;
    let precise = ((input * zeros).round())/zeros;
    trace!("Precision set from {} to {}.", input, precise);
    return precise;
}