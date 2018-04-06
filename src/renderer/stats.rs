
use std::time::{Duration, Instant};
use renderer::Timings;

#[derive(Debug, Clone, Copy)]
pub struct Stats {
    pub last_updated: Instant,
    /// How long each frame has lasted for, in seconds, averaged
    pub frametime_60: f32,
    pub frametime_600: f32,
    pub frametime_6000: f32,
    /// How long was spent waiting for the GPU, in seconds, averaged
    pub outerrendertime_60: f32,
    pub outerrendertime_600: f32,
    pub outerrendertime_6000: f32,
    /// How long does the GPU report rendering, in seconds, averaged
    pub rendertime_60: f32,
    pub rendertime_600: f32,
    pub rendertime_6000: f32,
    /// Timings for each render pass
    pub full_60: f32,
    pub geometry_60: f32,
    pub shading_60: f32,
    pub transparent_60: f32,
    pub blur1_60: f32,
    pub blur2_60: f32,
    pub post_60: f32,
    pub ui_60: f32
}

impl Default for Stats {
    fn default() -> Stats {
        Stats {
            last_updated: Instant::now(),
            frametime_60: 0.0,
            frametime_600: 0.0,
            frametime_6000: 0.0,
            outerrendertime_60: 0.0,
            outerrendertime_600: 0.0,
            outerrendertime_6000: 0.0,
            rendertime_60: 0.0,
            rendertime_600: 0.0,
            rendertime_6000: 0.0,
            full_60: 0.0,
            geometry_60: 0.0,
            shading_60: 0.0,
            transparent_60: 0.0,
            blur1_60: 0.0,
            blur2_60: 0.0,
            post_60: 0.0,
            ui_60: 0.0,
        }
    }
}

impl Stats {
    pub fn update_60(&mut self, frametime: &Duration, outertime: &Duration, innertime: &Duration,
                     timings: &Timings)
    {
        self.frametime_60 = duration_to_seconds(frametime) / 60.0;
        self.outerrendertime_60 = duration_to_seconds(outertime) / 60.0;
        self.rendertime_60 = duration_to_seconds(innertime) / 60.0;

        self.full_60 = duration_to_seconds(&timings.full) / 60.0;
        self.geometry_60 = duration_to_seconds(&timings.geometry) / 60.0;
        self.shading_60 = duration_to_seconds(&timings.shading) / 60.0;
        self.transparent_60 = duration_to_seconds(&timings.transparent) / 60.0;
        self.blur1_60 = duration_to_seconds(&timings.blur1) / 60.0;
        self.blur2_60 = duration_to_seconds(&timings.blur2) / 60.0;
        self.post_60 = duration_to_seconds(&timings.post) / 60.0;
        self.ui_60 = duration_to_seconds(&timings.ui) / 60.0;

        self.last_updated = Instant::now();
    }

    pub fn update_600(&mut self, frametime: &Duration, outertime: &Duration, innertime: &Duration)
    {
        self.frametime_600 = duration_to_seconds(frametime) / 600.0;
        self.outerrendertime_600 = duration_to_seconds(outertime) / 600.0;
        self.rendertime_600 = duration_to_seconds(innertime) / 600.0;
        self.last_updated = Instant::now();
    }

    pub fn update_6000(&mut self, frametime: &Duration, outertime: &Duration, innertime: &Duration)
    {
        self.frametime_6000 = duration_to_seconds(frametime) / 6000.0;
        self.outerrendertime_6000 = duration_to_seconds(outertime) / 6000.0;
        self.rendertime_6000 = duration_to_seconds(innertime) / 6000.0;
        self.last_updated = Instant::now();
    }
}

fn duration_to_seconds(duration: &Duration) -> f32
{
    duration.as_secs() as f32 +
        duration.subsec_nanos() as f32 * 0.000_000_001
}
