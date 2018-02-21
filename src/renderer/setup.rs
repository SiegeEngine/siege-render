
use errors::*;
use winit::Window;
use dacite_winit::WindowExt;
use dacite::core::{InstanceExtensions, Instance};

pub fn compute_instance_extensions(window: &Window) -> Result<InstanceExtensions>
{

    let available_extensions = Instance::get_instance_extension_properties(None)?;

    let required_extensions = window.get_required_extensions()?;

    let missing_extensions = required_extensions.difference(&available_extensions);

    if missing_extensions.is_empty() {
        Ok(required_extensions.to_extensions())
    } else {
        let mut s = String::new();
        for (name, spec_version) in missing_extensions.properties() {
            s.push_str(&*format!("Extension {} (revision {})", name, spec_version));
        }
        Err(ErrorKind::MissingExtensions(s).into())
    }
}
