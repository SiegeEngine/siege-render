
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
    }
}
