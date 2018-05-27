use ash::extensions::DebugReport;
use ash::version::{EntryV1_0, InstanceV1_0};
use ash::vk::types::{DebugReportCallbackEXT, StructureType};
use config::Config;
use errors::*;
use libc::c_void;
use renderer::VulkanLogLevel;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;

pub fn setup_debug_report<E: EntryV1_0, I: InstanceV1_0>(entry: &E, config: &Config, instance: &I)
                                                         -> Result<DebugReportCallbackEXT>
{
    use ash::vk::types::{DebugReportCallbackCreateInfoEXT, DEBUG_REPORT_DEBUG_BIT_EXT,
                         DEBUG_REPORT_ERROR_BIT_EXT, DEBUG_REPORT_INFORMATION_BIT_EXT,
                         DEBUG_REPORT_PERFORMANCE_WARNING_BIT_EXT, DEBUG_REPORT_WARNING_BIT_EXT};

    let debug_report = DebugReport::new(entry, instance)?;

    let flags = {
        let mut flags = DEBUG_REPORT_ERROR_BIT_EXT;
        if config.vulkan_log_level >= VulkanLogLevel::Warning {
            flags |= DEBUG_REPORT_WARNING_BIT_EXT;
        }
        if config.vulkan_log_level >= VulkanLogLevel::PerformanceWarning {
            flags |= DEBUG_REPORT_PERFORMANCE_WARNING_BIT_EXT;
        }
        if config.vulkan_log_level >= VulkanLogLevel::Information {
            flags |= DEBUG_REPORT_INFORMATION_BIT_EXT;
        }
        if config.vulkan_log_level >= VulkanLogLevel::Debug {
            flags |= DEBUG_REPORT_DEBUG_BIT_EXT;
        }
        flags
    };

    let create_info = DebugReportCallbackCreateInfoEXT {
        s_type: StructureType::DebugReportCallbackCreateInfoExt,
        p_next: ptr::null(),
        flags: flags,
        pfn_callback: callback,
        p_user_data: ptr::null_mut(),
    };

    Ok(unsafe {
        debug_report.create_debug_report_callback_ext(
            &create_info,
            None, // allocation callbacks
        )
    }?)
}

use ash::vk::types::{DebugReportFlagsEXT, DebugReportObjectTypeEXT};

unsafe extern "system" fn callback(
    flags: DebugReportFlagsEXT,
    _object_type: DebugReportObjectTypeEXT,
    _object: u64,
    _location: usize,
    message_code: i32,
    _layer_prefix: *const c_char,
    message: *const c_char,
    _user_data: *mut c_void,
) -> u32 {
    use ash::vk::types::{DEBUG_REPORT_DEBUG_BIT_EXT, DEBUG_REPORT_ERROR_BIT_EXT,
                         DEBUG_REPORT_INFORMATION_BIT_EXT,
                         DEBUG_REPORT_PERFORMANCE_WARNING_BIT_EXT, DEBUG_REPORT_WARNING_BIT_EXT};

    if *message != 0 {
        let cstr = CStr::from_ptr(message);
        let s = cstr.to_string_lossy();

        if flags.intersects(DEBUG_REPORT_ERROR_BIT_EXT) {
            error!("\r\n  vk[{}]: {}", message_code, s);
        } else if flags.intersects(DEBUG_REPORT_WARNING_BIT_EXT) {
            warn!("\r\n  vk[{}]: {}", message_code, s);
        } else if flags.intersects(DEBUG_REPORT_PERFORMANCE_WARNING_BIT_EXT) {
            warn!("\r\n  vk[{}]: {}", message_code, s);
        } else if flags.intersects(DEBUG_REPORT_INFORMATION_BIT_EXT) {
            info!("\r\n  vk[{}]: {}", message_code, s);
        } else if flags.intersects(DEBUG_REPORT_DEBUG_BIT_EXT) {
            debug!("\r\n  vk[{}]: {}", message_code, s);
        }
    }

    0
}
