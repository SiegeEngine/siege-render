
use crate::error::Error;
use dacite::core::PhysicalDevice;
use dacite::khr_surface::SurfaceKhr;

#[derive(Debug)]
pub struct QueueIndices {
    pub transfer_family: u32,
    pub transfer_index: u32,
    pub graphics_family: u32,
    pub graphics_index: u32,
    pub present_family: u32,
    pub present_index: u32,
}

impl QueueIndices {

    pub fn new(physical_device: &PhysicalDevice,
               surface: &SurfaceKhr)
               -> Result<QueueIndices, Error>
    {
        use std::collections::HashSet;
        use dacite::core::QueueFlags;

        let queue_family_properties: Vec<_> = physical_device.get_queue_family_properties();

        // The family used must support the feature in question.  For each feature,
        // we build a set of all families that would work.
        let mut p_set: HashSet<usize> = HashSet::new();
        let mut g_set: HashSet<usize> = HashSet::new();
        let mut t_set: HashSet<usize> = HashSet::new();
        for (i,qfp) in queue_family_properties.iter().enumerate() {
            if qfp.queue_count < 1 { continue; }
            if physical_device.get_surface_support_khr(i as u32, surface) == Ok(true) {
                p_set.insert(i);
            }
            if qfp.queue_flags.contains(QueueFlags::GRAPHICS) {
                g_set.insert(i);
            }
            if qfp.queue_flags.contains(QueueFlags::TRANSFER) {
                t_set.insert(i);
            }
        }
        if p_set.len() < 1 {
            return Err(Error::DeviceNotSuitable(
                "No suitable presentation queue found".to_owned()))
        }
        if g_set.len() < 1 {
            return Err(Error::DeviceNotSuitable(
                "No suitable graphics queue found".to_owned()))
        }
        if t_set.len() < 1 {
            return Err(Error::DeviceNotSuitable(
                "No suitable transfer queue found".to_owned()))
        }

        let (transfer_family, transfer_index) = {
            // We prefer to have transfer queue separate, esp from graphics queue
            let t_no_g_set: HashSet<usize> = t_set.difference(&g_set).map(|x|*x).collect();
            let t_no_gp_set: HashSet<usize> = t_no_g_set.difference(&p_set).map(|x|*x).collect();
            let (transfer_family, transfer_index) = if t_no_gp_set.len() > 0 {
                (*t_no_gp_set.iter().next().unwrap(), 0)
            } else if t_no_g_set.len() > 0 {
                (*t_no_g_set.iter().next().unwrap(), 0)
            } else {
                let t_no_p_set: HashSet<usize> = t_set.difference(&p_set).map(|x|*x).collect();
                if t_no_p_set.len() > 0 {
                    (*t_no_p_set.iter().next().unwrap(), 0)
                } else {
                    (*t_set.iter().next().unwrap(), 0)
                }
            };
            (transfer_family, transfer_index)
        };

        let (graphics_family, graphics_index) = {
            // We prefer to have graphics different from transfer
            let mut g_t_set: HashSet<usize> = g_set.clone();
            g_t_set.remove(&transfer_family);
            // We prefer to have graphics and present on the same queue
            let gp_set: HashSet<usize> = p_set.intersection(&g_set).map(|x|*x).collect();
            // Lets try for both of the above
            let gp_t_set: HashSet<usize> = g_t_set.intersection(&gp_set).map(|x|*x).collect();

            let (graphics_family, graphics_index) = if gp_t_set.len() > 0 {
                (*gp_t_set.iter().next().unwrap(), 0)
            } else if g_t_set.len() > 0 {
                (*g_t_set.iter().next().unwrap(), 0)
            } else {
                // At this point, we already know we are stuck with the transfer family.
                // No sense trying to prefer a presentation set queue, we only have one left.
                let qc = queue_family_properties[transfer_family].queue_count;
                if qc >=3 {
                    (transfer_family, 1)
                } else if qc >= 2 {
                    (transfer_family, 0)
                } else {
                    return Err(Error::DeviceNotSuitable(
                        "Not enough queues for two graphics queues".to_owned()))
                }
            };
            (graphics_family, graphics_index)
        };

        let (present_family, present_index) = {
            if p_set.contains(&graphics_family) {
                (graphics_family, graphics_index)
            } else {
                let mut p_no_t_set: HashSet<usize> = p_set.clone();
                p_no_t_set.remove(&transfer_family);
                if p_no_t_set.len() > 1 {
                    (*p_no_t_set.iter().next().unwrap(), 0)
                } else {
                    // At this point, we already know we are stuck with the transfer family.
                    let qc = queue_family_properties[transfer_family].queue_count;
                    if qc >= 2 {
                        (transfer_family, 1)
                    } else {
                        (transfer_family, 0)
                    }
                }
            }
        };

        Ok(QueueIndices {
            transfer_family: transfer_family as u32,
            transfer_index: transfer_index as u32,
            graphics_family: graphics_family as u32,
            graphics_index: graphics_index as u32,
            present_family: present_family as u32,
            present_index: present_index as u32,
        })
    }
}
