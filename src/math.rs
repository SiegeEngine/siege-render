pub use self::inner::*;

#[cfg(feature = "use-siege-math")]
mod inner {
    pub use siege_math::{Mat4, Point3, Vec4};
}

#[cfg(feature = "use-cgmath")]
mod inner {
    pub use cgmath::Matrix4 as Mat4;
    pub use cgmath::Point3;
    pub use cgmath::Vector4 as Vec4;
}

#[cfg(feature = "use-nalgebra")]
mod inner {
    pub use nalgebra::Matrix4 as Mat4;
    pub use nalgebra::Point3;
    pub use nalgebra::Vector4 as Vec4;
}
