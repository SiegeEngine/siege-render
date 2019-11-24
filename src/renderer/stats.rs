
use std::time::{Instant, Duration};
use dacite::core::QueryResult;
use crate::renderer::{Timestamp, TS_QUERY_COUNT};

#[derive(Debug, Clone)]
pub struct Timings {
    pub frame: f32,
    pub cpu: f32,
    pub render: f32,
    pub geometry: f32,
    pub shading: f32,
    pub transparent: f32,
    pub blur1: f32,
    pub blur2: f32,
    pub post: f32,
    pub ui: f32,
}
impl Timings {
    pub fn new() -> Timings {
        Timings {
            frame: 0.0,
            cpu: 0.0,
            render: 0.0,
            geometry: 0.0,
            shading: 0.0,
            transparent: 0.0,
            blur1: 0.0,
            blur2: 0.0,
            post: 0.0,
            ui: 0.0,
        }
    }

    pub fn one(
        frame_duration: &Duration,
        query_results: &[QueryResult; TS_QUERY_COUNT as usize],
        cputime_ms: f32,
        timestamp_period: f32)
        -> Timings
    {
        let qr: Vec<u32> = query_results.iter().map(|r| {
            match r {
                &QueryResult::U32(u) => u,
                &QueryResult::U64(u) => u as u32
            }
        }).collect();

        let to_ms = |start_index, end_index| {
            ((qr[end_index as usize].saturating_sub(qr[start_index as usize]))
             as f32 * timestamp_period) * 0.000_001
        };

        Timings {
            frame: duration_to_milliseconds(frame_duration),
            cpu: cputime_ms,
            render: to_ms(Timestamp::FullStart, Timestamp::FullEnd),
            geometry: to_ms(Timestamp::GeometryStart, Timestamp::GeometryEnd),
            shading: to_ms(Timestamp::ShadingStart, Timestamp::ShadingEnd),
            transparent: to_ms(Timestamp::TransparentStart, Timestamp::TransparentEnd),
            blur1: to_ms(Timestamp::Blur1Start, Timestamp::Blur1End),
            blur2: to_ms(Timestamp::Blur2Start, Timestamp::Blur2End),
            post: to_ms(Timestamp::PostStart, Timestamp::PostEnd),
            ui: to_ms(Timestamp::UiStart, Timestamp::UiEnd),
        }
    }

    pub fn accumulate(&mut self, other: &Timings) {
        self.frame += other.frame;
        self.cpu += other.cpu;
        self.render += other.render;
        self.geometry += other.geometry;
        self.shading += other.shading;
        self.transparent += other.transparent;
        self.blur1 += other.blur1;
        self.blur2 += other.blur2;
        self.post += other.post;
        self.ui += other.ui;
    }
}

#[derive(Debug, Clone)]
pub struct Stats {
    pub last_updated: Instant,

    pub timings_60: Timings,
    pub timings_600: Timings,
}

impl Default for Stats {
    fn default() -> Stats {
        Stats {
            last_updated: Instant::now(),

            timings_60: Timings::new(),
            timings_600: Timings::new(),
        }
    }
}

impl Stats {
    pub fn update_60(&mut self, timings: Timings)
    {
        self.timings_60 = timings;
        self.last_updated = Instant::now();
    }

    pub fn update_600(&mut self, timings: Timings)
    {
        self.timings_600 = timings;
        self.last_updated = Instant::now();
    }
}

fn duration_to_milliseconds(duration: &Duration) -> f32
{
    duration.as_secs() as f32 * 1000.0 +
        duration.subsec_nanos() as f32 * 0.000_001
}
