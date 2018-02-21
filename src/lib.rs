
#![recursion_limit = "1024"]

// serialization
#[macro_use]
extern crate serde_derive;
extern crate serde;

// errors
#[macro_use]
extern crate error_chain;

// logging
#[macro_use]
extern crate log;

// graphics
extern crate vks;
extern crate dacite;
extern crate dacite_winit;
extern crate winit;
extern crate siege_mesh;

// files
extern crate ddsfile;
extern crate zstd;

// win32
#[cfg(windows)] extern crate user32;
#[cfg(windows)] extern crate winapi;

// math
extern crate siege_math;

// time
extern crate chrono;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
