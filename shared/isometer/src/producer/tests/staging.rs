// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;

struct Solid {
    blue: bool,
    live: bool,
    supported: bool,
    draws: u64,
    accepted: Option<SlabCamera>,
    staged: Option<SlabCamera>,
}
impl Solid {
    fn camera(&self) -> SlabCamera {
        SlabCamera {
            projection: Default::default(),
            centre: [u8::from(self.blue) as f32, 0.0, 0.0],
            forward: [0.0, 0.0, -1.0],
            half_height: 1.0,
            aspect: 1.0,
            depth: 4.0,
            cutaway: None,
        }
    }
    fn draw(&mut self, request: &FrameRequest<'_>) -> wgpu::TextureView {
        self.draws += 1;
        let (_, view) = crate::scene::target(
            request.device,
            request.size[0],
            request.size[1],
            crate::scene::CAPTURE_FORMAT,
            "transactional specimen",
        );
        let mut encoder = request.device.create_command_encoder(&Default::default());
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(if self.blue {
                            wgpu::Color::BLUE
                        } else {
                            wgpu::Color::RED
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
        }
        request.queue.submit([encoder.finish()]);
        view
    }
}
impl SceneSource for Solid {
    fn inputs(&mut self, request: &FrameRequest<'_>) -> Result<SceneSignature, String> {
        Ok(SceneSignature {
            size: request.size,
            camera: Some(self.camera()),
            host: vec![u64::from(self.blue)],
            ..Default::default()
        })
    }
    fn frame(&mut self, request: &FrameRequest<'_>) -> Result<Option<wgpu::TextureView>, String> {
        self.accepted = Some(self.camera());
        Ok(Some(self.draw(request)))
    }
    fn supports_native_staging(&self) -> bool {
        self.supported
    }
    fn stage_frame(
        &mut self,
        request: &FrameRequest<'_>,
    ) -> Result<Option<wgpu::TextureView>, String> {
        self.staged = Some(self.camera());
        Ok(Some(self.draw(request)))
    }
    fn publish_staged(&mut self) {
        if let Some(camera) = self.staged.take() {
            self.accepted = Some(camera);
        }
    }
    fn discard_staged(&mut self) {
        self.staged = None;
    }
    fn staged_is_live(&self) -> bool {
        self.live
    }
    fn presented_camera(&self) -> Option<SlabCamera> {
        self.accepted
    }
}

#[test]
#[ignore = "explicit R3 actual-device staged upscale isolation"]
fn staged_upscale_isolates_the_accepted_image_and_banks_no_skip_state_before_publication() {
    let (device, queue) = device().expect("R3 control requires an actual GPU adapter");
    let mut producer = SceneProducer::new(Solid {
        blue: false,
        live: true,
        supported: true,
        draws: 0,
        accepted: None,
        staged: None,
    });
    let request = scaled(&device, &queue, [8, 8], 2);
    assert!(producer.render_scene_staged(&request).unwrap().is_some());
    assert_eq!(producer.renders(), 0);
    assert!(producer.presented_camera().is_none());
    producer.publish_staged();
    let accepted = producer.display.as_ref().unwrap().0.clone();
    let pixels = read_back(&device, &queue, &accepted, [8, 8]);
    assert_eq!(pixel(&pixels, 8, 4, 4), [255, 0, 0, 255]);
    let signature = producer.signature().clone();
    let camera = producer.presented_camera();
    producer.source_mut().blue = true;
    assert!(producer.render_scene_staged(&request).unwrap().is_some());
    assert_eq!(producer.renders(), 1);
    assert_eq!(producer.signature(), &signature);
    assert_eq!(producer.presented_camera(), camera);
    assert_eq!(
        read_back(&device, &queue, &accepted, [8, 8]),
        pixels,
        "B never writes the accepted upscale target"
    );
    producer.discard_staged();
    assert_eq!(producer.renders(), 1);
    assert_eq!(producer.signature(), &signature);
    assert!(producer.render_scene_staged(&request).unwrap().is_some());
    producer.publish_staged();
    assert_eq!(producer.renders(), 2);
    assert_eq!(
        pixel(
            &read_back(
                &device,
                &queue,
                &producer.display.as_ref().unwrap().0,
                [8, 8]
            ),
            8,
            4,
            4
        ),
        [0, 0, 255, 255]
    );
    let draws = producer.draws;
    assert!(producer.render_scene_staged(&request).unwrap().is_none());
    assert_eq!(producer.draws, draws);
    producer.publish_staged();
    assert_eq!(producer.renders(), 2);

    producer.source_mut().supported = false;
    assert!(
        producer
            .render_scene_staged(&request)
            .unwrap_err()
            .contains("does not support")
    );
    assert_eq!(
        producer.draws, draws,
        "unsupported sources are never invoked transactionally"
    );
    producer.source_mut().supported = true;
    producer.source_mut().blue = false;
    producer.render_scene_staged(&request).unwrap();
    producer.source_mut().live = false;
    producer.publish_staged();
    assert_eq!(
        producer.renders(),
        2,
        "revoked candidate cannot advance the generation"
    );
}
