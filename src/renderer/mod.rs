#[derive(Debug, Deserialize, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum VulkanLogLevel {
    Error,
    Warning,
    PerformanceWarning,
    Information,
    Debug,
}

#[repr(u32)]
#[derive(Debug, Deserialize, Clone, Copy)]
pub enum Tonemapper {
    Clamp = 0,
    Reinhard = 1,
    Exposure = 2,
    HybridLogGamma = 3,
    Falsecolor = 4,
}
