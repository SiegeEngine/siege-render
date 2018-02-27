
error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links {
        Mesh(::siege_mesh::Error, ::siege_mesh::ErrorKind);
        Ddsfile(::ddsfile::Error, ::ddsfile::ErrorKind);
    }

    foreign_links {
        Fmt(::std::fmt::Error);
        Io(::std::io::Error);
        Addr(::std::net::AddrParseError);
        WinitCreation(::winit::CreationError);
        Dacite(::dacite::core::Error);
        DaciteEarly(::dacite::core::EarlyInstanceError);
        DaciteWinit(::dacite_winit::Error);
    }

    errors {
        General(s: String) {
            description("General Error"),
            display("General Error: '{}'", s),
        }
        MissingExtensions(s: String) {
            description("Vulkan Extensions Missing"),
            display("Vulkan Extensions Missing: '{}'", s),
        }
        NoSuitableDevice {
            description("No Suitable Graphics Device Found"),
        }
        DeviceNotSuitable(s: String) {
            description("Device not suitable"),
            display("Device not suitable: '{}'", s),
        }
        OutOfGraphicsMemory {
            description("Out of graphics memory (or memory type requested does not exist)"),
        }
        MemoryNotHostWritable {
            description("Device memory is not host writable"),
        }
        NoSuitableSurfaceFormat {
            description("No Suitable Surface Format Found"),
        }
        WrongVertexType {
            description("Mesh has wrong vertex type"),
        }
        UnsupportedFormat {
            description("Unsupported or indeterminate file format"),
        }
        SwapchainTimeout {
            description("Swapchain acquire timed out (perhaps took longer than 4 seconds)"),
        }
    }
}
