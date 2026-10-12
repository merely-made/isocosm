// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Clone the present renderer's existing handles for the lit body tenant.

pub fn from_renderer(renderer: &netrender::Renderer) -> tenant::HostDevice {
    let held = &renderer.wgpu_device.core;
    tenant::HostDevice {
        instance: held.instance.clone(),
        adapter: held.adapter.clone(),
        device: held.device.clone(),
        queue: held.queue.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires the independently owned actual GPU gate"]
    fn tenant_borrows_all_four_retained_present_handles_on_the_existing_pin() {
        let core = genet_render_host::RenderCore::boot(netrender::NetrenderOptions::default())
            .expect("this gate requires an actual supported GPU; boot refusal is not a pass");
        let host = from_renderer(core.renderer());
        let held = &core.renderer().wgpu_device.core;
        assert_eq!(host.instance, held.instance, "tenant instance belongs to the present core");
        assert_eq!(host.adapter, held.adapter, "tenant adapter belongs to the present core");
        assert_eq!(&host.device, core.device(), "tenant device belongs to the present core");
        assert_eq!(&host.queue, core.queue(), "tenant queue belongs to the present core");
    }
}
