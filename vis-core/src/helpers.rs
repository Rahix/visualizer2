use std::time;

pub fn time(start: time::Instant) -> f32 {
    let elapsed = time::Instant::now() - start;

    elapsed.as_secs() as f32 + elapsed.subsec_nanos() as f32 * 1e-9
}
