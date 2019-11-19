
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Mesh(::siege_mesh::Error),
    Ddsfile(::ddsfile::Error),
    Fmt(::std::fmt::Error),
    Io(std::io::Error),
    Addr(::std::net::AddrParseError),
    WinitCreation(::winit::CreationError),
    Dacite(::dacite::core::Error),
    DaciteEarly(::dacite::core::EarlyInstanceError),
    DaciteWinit(::dacite_winit::Error),
    General(String),
    MissingExtensions(String),
    NoSuitableDevice,
    DeviceNotSuitable(String),
    OutOfGraphicsMemory,
    MemoryNotHostWritable,
    NoSuitableSurfaceFormat,
    WrongVertexType,
    UnsupportedFormat,
    SwapchainTimeout,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Mesh(ref e) => write!(f, "{}", e),
            Error::Ddsfile(ref e) => write!(f, "{}", e),
            Error::Fmt(ref e) => write!(f, "{}", e),
            Error::Io(ref e) => write!(f, "I/O error: {}", e),
            Error::Addr(ref e) => write!(f, "{}", e),
            Error::WinitCreation(ref e) => write!(f, "{}", e),
            Error::Dacite(ref e) => write!(f, "{}", e),
            Error::DaciteEarly(ref e) => write!(f, "{}", e),
            Error::DaciteWinit(ref e) => write!(f, "{}", e),
            Error::General(ref s) => write!(f, "General Error: '{}'", s),
            Error::MissingExtensions(ref s) => write!(f, "Vulkan Extensions Missing: '{}'", s),
            Error::NoSuitableDevice => write!(f, "No Suitable Graphics Device Found"),
            Error::DeviceNotSuitable(ref s) => write!(f, "Device not suitable: '{}'", s),
            Error::OutOfGraphicsMemory => write!(f, "Out of graphics memory (or memory type requested does not exist)"),
            Error::MemoryNotHostWritable => write!(f, "Device memory is not host writable"),
            Error::NoSuitableSurfaceFormat => write!(f, "No Suitable Surface Format Found"),
            Error::WrongVertexType => write!(f, "Mesh has wrong vertex type"),
            Error::UnsupportedFormat => write!(f, "Unsupported or indeterminate file format"),
            Error::SwapchainTimeout => write!(f, "Swapchain acquire timed out (perhaps took longer than 4 seconds)"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::Io(ref e) => Some(e),
            Error::Mesh(ref e) => Some(e),
            Error::Ddsfile(ref e) => Some(e),
            Error::Fmt(ref e) => Some(e),
            Error::Addr(ref e) => Some(e),
            Error::WinitCreation(ref e) => Some(e),
            Error::Dacite(ref e) => Some(e),
            Error::DaciteEarly(ref e) => Some(e),
            Error::DaciteWinit(ref e) => Some(e),
            _ => None
        }
    }
}

impl From<siege_mesh::Error> for Error {
    fn from(e: siege_mesh::Error) -> Error {
        Error::Mesh(e)
    }
}

impl From<ddsfile::Error> for Error {
    fn from(e: ddsfile::Error) -> Error {
        Error::Ddsfile(e)
    }
}

impl From<std::fmt::Error> for Error {
    fn from(e: std::fmt::Error) -> Error {
        Error::Fmt(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::Io(e)
    }
}

impl From<std::net::AddrParseError> for Error {
    fn from(e: std::net::AddrParseError) -> Error {
        Error::Addr(e)
    }
}

impl From<winit::CreationError> for Error {
    fn from(e: winit::CreationError) -> Error {
        Error::WinitCreation(e)
    }
}

impl From<dacite::core::Error> for Error {
    fn from(e: dacite::core::Error) -> Error {
        Error::Dacite(e)
    }
}

impl From<dacite::core::EarlyInstanceError> for Error {
    fn from(e: dacite::core::EarlyInstanceError) -> Error {
        Error::DaciteEarly(e)
    }
}

impl From<dacite_winit::Error> for Error {
    fn from(e: dacite_winit::Error) -> Error {
        Error::DaciteWinit(e)
    }
}
