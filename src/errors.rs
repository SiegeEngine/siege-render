error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links {
    }

    foreign_links {
        FfiNul(::std::ffi::NulError);
        AshDeviceError(::ash::DeviceError);
        Utf8Error(::std::str::Utf8Error);
    }

    errors {
        General(s: String) {
            description("General Error"),
            display("General Error: '{}'", s),
        }
        GeneralStatic(s: &'static str) {
            description("General Error"),
            display("General Error: '{}'", s),
        }
        MissingExtension(s: String) {
            description("Vulkan Extension Missing"),
            display("Vulkan Extension Missing: '{}'", s),
        }
        AshInstanceError(inner: ::ash::InstanceError) {
            description("Ash Instance Error"),
            display("Ash Instance Error: '{}'", inner),
        }
        AshLoadingError(inner: ::ash::LoadingError) {
            description("Ash Loading Error"),
            display("Ash Loading Error: '{:?}'", inner),
        }
        VkNotReady {
            description("Vulkan: Not Ready"),
        }
        VkTimeout {
            description("Vulkan: Timeout"),
        }
        VkEventSet {
            description("Vulkan: Event Set"),
        }
        VkEventReset {
            description("Vulkan: Event Reset"),
        }
        VkIncomplete {
            description("Vulkan: Incomplete"),
        }
        VkOutOfHostMemory {
            description("Vulkan: Out of Host Memory"),
        }
        VkOutOfDeviceMemory {
            description("Vulkan: Out of Device Memory"),
        }
        VkInitializationFailed {
            description("Vulkan: Initialization Failed"),
        }
        VkDeviceLost {
            description("Vulkan: Device Lost"),
        }
        VkMemoryMapFailed {
            description("Vulkan: Memory Map Failed"),
        }
        VkLayerNotPresent {
            description("Vulkan: Layer Not Present"),
        }
        VkExtensionNotPresent {
            description("Vulkan: Extension Not Present"),
        }
        VkFeatureNotPresent {
            description("Vulkan: Feature Not Present"),
        }
        VkIncompatibleDriver {
            description("Vulkan: Incompatible Driver"),
        }
        VkTooManyObjects {
            description("Vulkan: Too Many Objects"),
        }
        VkFormatNotSupported {
            description("Vulkan: Format Not Supported"),
        }
        VkFragmentedPool {
            description("Vulkan: Fragmented Pool"),
        }
        VkSurfaceLostKhr {
            description("Vulkan: Surface Lost (khr)"),
        }
        VkNativeWindowInUseKhr {
            description("Vulkan: Window in use (khr)"),
        }
        VkSuboptimalKhr {
            description("Vulkan: Suboptimal (khr)"),
        }
        VkErrorOutOfDateKhr {
            description("Vulkan: Error out of date (khr)"),
        }
        VkIncompatibleDisplayKhr {
            description("Vulkan: Incompatible Display (khr)"),
        }
        VkValidationFailedExt {
            description("Vulkan: Validation Failed"),
        }
        DeviceNotSuitable(s: String) {
            description("Device not suitable"),
            display("Device not suitable: '{}'", s),
        }
        NoSuitableDevice {
            description("No Suitable Graphics Device Found"),
        }
        MemoryNotHostWritable {
            description("Device memory is not host writable"),
        }
        OutOfGraphicsMemory {
            description("Out of graphics memory (or memory type requested does not exist)"),
        }
        NoSuitableSurfaceFormat {
            description("No Suitable Surface Format Found"),
        }
    }
}

impl From<::ash::InstanceError> for Error {
    fn from(i: ::ash::InstanceError) -> Error {
        Error::from_kind(ErrorKind::AshInstanceError(i))
    }
}

impl From<::ash::LoadingError> for Error {
    fn from(i: ::ash::LoadingError) -> Error {
        Error::from_kind(ErrorKind::AshLoadingError(i))
    }
}

impl From<::ash::vk::types::Result> for Error {
    fn from(r: ::ash::vk::types::Result) -> Error {
        use ash::vk::types::Result as R;
        Error::from_kind(match r {
            R::Success => ErrorKind::General("Vulkan: Success".to_owned()),
            R::NotReady => ErrorKind::VkNotReady,
            R::Timeout => ErrorKind::VkTimeout,
            R::EventSet => ErrorKind::VkEventSet,
            R::EventReset => ErrorKind::VkEventReset,
            R::Incomplete => ErrorKind::VkIncomplete,
            R::ErrorOutOfHostMemory => ErrorKind::VkOutOfHostMemory,
            R::ErrorOutOfDeviceMemory => ErrorKind::VkOutOfDeviceMemory,
            R::ErrorInitializationFailed => ErrorKind::VkInitializationFailed,
            R::ErrorDeviceLost => ErrorKind::VkDeviceLost,
            R::ErrorMemoryMapFailed => ErrorKind::VkMemoryMapFailed,
            R::ErrorLayerNotPresent => ErrorKind::VkLayerNotPresent,
            R::ErrorExtensionNotPresent => ErrorKind::VkExtensionNotPresent,
            R::ErrorFeatureNotPresent => ErrorKind::VkFeatureNotPresent,
            R::ErrorIncompatibleDriver => ErrorKind::VkIncompatibleDriver,
            R::ErrorTooManyObjects => ErrorKind::VkTooManyObjects,
            R::ErrorFormatNotSupported => ErrorKind::VkFormatNotSupported,
            R::ErrorFragmentedPool => ErrorKind::VkFragmentedPool,
            R::ErrorSurfaceLostKhr => ErrorKind::VkSurfaceLostKhr,
            R::ErrorNativeWindowInUseKhr => ErrorKind::VkNativeWindowInUseKhr,
            R::SuboptimalKhr => ErrorKind::VkSuboptimalKhr,
            R::ErrorOutOfDateKhr => ErrorKind::VkErrorOutOfDateKhr,
            R::ErrorIncompatibleDisplayKhr => ErrorKind::VkIncompatibleDisplayKhr,
            R::ErrorValidationFailedExt => ErrorKind::VkValidationFailedExt,
        })
    }
}

impl From<Vec<&'static str>> for Error {
    fn from(v: Vec<&'static str>) -> Error {
        Error::from_kind(ErrorKind::General(v.join("\n")))
    }
}
