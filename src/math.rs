pub use self::inner::*;

#[cfg(feature = "use-siege-math")]
mod inner {
    pub use siege_math::{Point3, Vec4};
}

#[cfg(feature = "use-cgmath")]
mod inner {
    pub use cgmath::Point3;
    pub use cgmath::Vector4 as Vec4;
}

#[cfg(feature = "use-nalgebra")]
mod inner {
    pub use nalgebra::Point3;
    pub use nalgebra::Vector4 as Vec4;
}
