use std::time::Instant;

use encryptdrive_desktop::preview::{DatasetSize, ExplorerState, PreviewModel};

fn main() {
    for size in [
        DatasetSize::OneHundred,
        DatasetSize::OneThousand,
        DatasetSize::TenThousand,
    ] {
        let started = Instant::now();
        let model = PreviewModel::fixture(size);
        let construction = started.elapsed();

        let mut state = ExplorerState::new();
        let projection_started = Instant::now();
        let visible = state.visible_ids(&model);
        let cold_projection = projection_started.elapsed();

        let warm_started = Instant::now();
        for _ in 0..1_000 {
            std::hint::black_box(state.visible_ids(&model));
        }
        let warm_total = warm_started.elapsed();
        println!(
            "entries={} fixture_ms={:.3} cold_projection_ms={:.3} cached_1000_ms={:.3}",
            visible.len(),
            construction.as_secs_f64() * 1_000.0,
            cold_projection.as_secs_f64() * 1_000.0,
            warm_total.as_secs_f64() * 1_000.0,
        );
    }
}
