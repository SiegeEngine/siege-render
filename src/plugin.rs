
use dacite::core::{CommandBuffer, Extent2D};
use renderer::{Params, Stats};
use errors::*;

/// This is a trait for Plugins to the Renderer.
pub trait Plugin {
    /// Record geometry-pass objects. Z-buffer is active.
    ///
    /// Fragment shader output is interpreted as a Luminance value, where
    /// 1.0 is the current white level.  Luminance values >1.0 are permitted
    /// (these will bloom, and/or be levelled by tonemapping).
    fn record_geometry(&self, command_buffer: CommandBuffer);

    /// Record transparent-pass objects. Z-buffer is read-only.
    ///
    /// Fragment shader output is interpreted as a Luminance value, where
    /// 1.0 is the current white level.  Luminance values >1.0 are permitted
    /// (these will bloom, and/or be levelled by tonemapping).
    ///
    /// Fragment shader output should be alpha blended on top of current scene.
    fn record_transparent(&self, command_buffer: CommandBuffer);

    /// Record UI layer. Depth buffer is not active (you will have to handle
    /// UI depth yourself).
    ///
    /// Output should be in sRGB if renderer.ui_needs_gamma() is true, otherwise
    /// it should be in sRGB linear (without the gamma function applied, which
    /// means you need to un-gamma your sRGB colors).
    ///
    /// Fragment shader output should be alpha blended on top of current scene.
    /// You should use pre-multiplied alpha (since alpha blending is subtly
    /// different between sRGB and linear, and it could be either case).
    fn record_ui(&self, command_buffer: CommandBuffer);

    /// This callback gives your plugin a chance to update itself, based on
    /// changed parameters or stats.  It also allows your plugin to change
    /// any of the render parameters.
    ///
    /// Return true if you need to re-record your command buffers.  Otherwise
    /// return false.
    fn update(&mut self, params: &mut Params, stats: &Stats) -> Result<bool>;

    /// This callback is called whenever the window size changes. The window
    /// size is passed in as `extent`. Your command buffers will always be
    /// re-recorded on window resize, so no return value is required.
    fn rebuild(&mut self, extent: Extent2D) -> Result<()>;
}
