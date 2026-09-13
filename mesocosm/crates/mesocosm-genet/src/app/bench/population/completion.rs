// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Explicit, private-device completion diagnostic. These are serialized CPU
//! wall durations, not GPU timestamps or native presentation measurements.
use super::{
    model::{Config, Workload},
    renderer::{RenderStats, Renderer},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    path::PathBuf,
    time::{Duration, Instant},
};

const MAX_SAMPLES: usize = 120;
const MAX_CASES: usize = 32;
const WARMUPS: usize = 3;
const TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    width: u32,
    height: u32,
    samples: usize,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    name: String,
    config: Config,
}

impl Manifest {
    fn validate(&self) -> Result<(), String> {
        if self.width == 0
            || self.height == 0
            || self.width > 8192
            || self.height > 8192
            || u64::from(self.width) * u64::from(self.height) > 16_777_216
        {
            return Err(
                "viewport must be nonempty, at most 8192 per side and 16777216 pixels".into(),
            );
        }
        if !(1..=MAX_SAMPLES).contains(&self.samples) {
            return Err("samples must be 1..120".into());
        }
        if !(1..=MAX_CASES).contains(&self.cases.len()) {
            return Err("cases must be 1..32".into());
        }
        let mut names = BTreeSet::new();
        for case in &self.cases {
            if case.name.trim().is_empty() || case.name.len() > 128 || !names.insert(&case.name) {
                return Err("case names must be unique, nonempty, at most 128 bytes".into());
            }
        }
        Ok(())
    }
}

#[derive(Serialize)]
struct Sample {
    yaw_degrees: f32,
    cpu_render_submit_us: f64,
    completion_wait_us: f64,
    serialized_total_us: f64,
    stats: RenderStats,
}

#[derive(Debug, Serialize)]
struct Summary {
    median: f64,
    p95: f64,
    max: f64,
}

fn summarize(mut values: Vec<f64>) -> Summary {
    assert!(!values.is_empty());
    values.sort_by(f64::total_cmp);
    let n = values.len();
    Summary {
        median: if n % 2 == 0 {
            (values[n / 2 - 1] + values[n / 2]) / 2.
        } else {
            values[n / 2]
        },
        p95: values[(95 * n).div_ceil(100) - 1],
        max: values[n - 1],
    }
}

#[derive(Serialize)]
struct CaseReceipt {
    name: String,
    config: Config,
    workload_digest: String,
    workload_preparation_us: u64,
    renderer_setup_us: f64,
    cold_frame: Sample,
    warmup_frames: Vec<Sample>,
    samples: Vec<Sample>,
    cpu_render_submit_us: Summary,
    completion_wait_us: Summary,
    serialized_total_us: Summary,
}

fn micros(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000_000.
}

fn drain(device: &wgpu::Device) -> Result<(), String> {
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(TIMEOUT),
        })
        .map_err(|e| format!("bounded GPU completion failed: {e:?}"))?;
    Ok(())
}

fn measure(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    renderer: &mut Renderer,
    workload: &Workload,
    yaw_degrees: f32,
) -> Result<Sample, String> {
    // Prior work is drained outside this sample's measured interval.
    drain(device)?;
    let started = Instant::now();
    let stats = renderer.render(device, queue, workload, yaw_degrees.to_radians(), [1.; 3])?;
    let submitted = Instant::now();
    drain(device)?;
    let completed = Instant::now();
    Ok(Sample {
        yaw_degrees,
        cpu_render_submit_us: micros(submitted.duration_since(started)),
        completion_wait_us: micros(completed.duration_since(submitted)),
        serialized_total_us: micros(completed.duration_since(started)),
        stats,
    })
}

fn run() -> Result<(), String> {
    let input = PathBuf::from(
        std::env::var_os("MESOCOSM_POPULATION_COMPLETION_INPUT")
            .ok_or("set MESOCOSM_POPULATION_COMPLETION_INPUT to an explicit manifest")?,
    );
    let output = PathBuf::from(
        std::env::var_os("MESOCOSM_POPULATION_COMPLETION_OUTPUT")
            .ok_or("set MESOCOSM_POPULATION_COMPLETION_OUTPUT to an explicit receipt path")?,
    );
    if input == output {
        return Err("input and output must differ".into());
    }
    if std::fs::metadata(&input).map_err(|e| e.to_string())?.len() > 1_048_576 {
        return Err("manifest exceeds 1 MiB".into());
    }
    let manifest: Manifest =
        serde_json::from_slice(&std::fs::read(&input).map_err(|e| e.to_string())?)
            .map_err(|e| format!("invalid completion manifest: {e}"))?;
    manifest.validate()?;
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&Default::default()))
        .map_err(|e| format!("completion adapter unavailable: {e:?}"))?;
    let info = adapter.get_info();
    let supported_features = format!("{:?}", adapter.features());
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: wgpu::Features::empty(),
        ..Default::default()
    }))
    .map_err(|e| format!("completion device unavailable: {e:?}"))?;
    let mut cases = Vec::with_capacity(manifest.cases.len());
    for case in manifest.cases {
        let workload = Workload::new(case.config)?;
        drain(&device)?;
        let setup = Instant::now();
        let mut renderer = Renderer::new(
            &device,
            &queue,
            (manifest.width, manifest.height),
            &workload,
        )?;
        let renderer_setup_us = micros(setup.elapsed());
        let cold_frame = measure(&device, &queue, &mut renderer, &workload, 0.)?;
        let mut warmup_frames = Vec::with_capacity(WARMUPS);
        for i in 0..WARMUPS {
            warmup_frames.push(measure(
                &device,
                &queue,
                &mut renderer,
                &workload,
                (i + 1) as f32 * 15.,
            )?);
        }
        let mut samples = Vec::with_capacity(manifest.samples);
        for i in 0..manifest.samples {
            samples.push(measure(
                &device,
                &queue,
                &mut renderer,
                &workload,
                ((i + WARMUPS + 1) % 24) as f32 * 15.,
            )?);
        }
        cases.push(CaseReceipt {
            name: case.name,
            config: workload.config.clone(),
            workload_digest: workload.digest.clone(),
            workload_preparation_us: workload.preparation_us,
            renderer_setup_us,
            cold_frame,
            warmup_frames,
            cpu_render_submit_us: summarize(
                samples.iter().map(|s| s.cpu_render_submit_us).collect(),
            ),
            completion_wait_us: summarize(samples.iter().map(|s| s.completion_wait_us).collect()),
            serialized_total_us: summarize(samples.iter().map(|s| s.serialized_total_us).collect()),
            samples,
        });
    }
    let receipt = serde_json::json!({
        "schema": 1,
        "measurement": "private-device serialized CPU render-to-completion wall microseconds",
        "adapter": { "name": info.name, "vendor": info.vendor, "device": info.device,
            "device_type": format!("{:?}", info.device_type), "backend": format!("{:?}", info.backend),
            "driver": info.driver, "driver_info": info.driver_info,
            "supported_features": supported_features, "enabled_features": format!("{:?}", device.features()) },
        "width": manifest.width, "height": manifest.height,
        "limits": { "max_samples": MAX_SAMPLES, "max_cases": MAX_CASES,
            "completion_timeout_seconds": TIMEOUT.as_secs(), "warmup_frames": WARMUPS },
        "limitations": [
            "Not GPU timestamps, isolated GPU execution time, fill rate, or native presentation latency.",
            "Render/submit includes CPU instance building, validation, uploads and command submission; GPU may overlap that interval.",
            "Completion wait includes host synchronization overhead; pre-drain is outside each sample.",
            "Cases run in manifest order on one private device; adapter driver caches and thermal state carry between cases.",
            "Cold frame means first render of a fresh renderer; renderer setup and workload preparation are separate.",
            "Three warmups precede resident steady samples; no readback, document, acquire, or presentation work is included.",
            "Opaque early depth rejection prevents attributing increased overlap directly to fragment work."
        ],
        "cases": cases,
    });
    std::fs::write(
        &output,
        serde_json::to_vec_pretty(&receipt).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("write completion receipt {}: {e}", output.display()))?;
    println!("population completion receipt: {}", output.display());
    Ok(())
}

#[test]
#[ignore = "explicit private GPU completion diagnostic; requires INPUT and OUTPUT manifest environment paths"]
fn population_completion_receipt() {
    run().expect("population completion diagnostic");
}

#[test]
fn percentile_summary_is_nearest_rank_and_even_median() {
    let summary = summarize((1..=100).rev().map(f64::from).collect());
    assert_eq!(
        (summary.median, summary.p95, summary.max),
        (50.5, 95., 100.)
    );
    let singleton = summarize(vec![7.]);
    assert_eq!(
        (singleton.median, singleton.p95, singleton.max),
        (7., 7., 7.)
    );
}

#[test]
fn manifest_refuses_unbounded_work_and_duplicate_case_names() {
    let mut manifest = Manifest {
        width: 128,
        height: 128,
        samples: 1,
        cases: vec![Case {
            name: "dense".into(),
            config: Config::default(),
        }],
    };
    assert!(manifest.validate().is_ok());
    manifest.samples = 121;
    assert!(manifest.validate().is_err());
    manifest.samples = 1;
    manifest.width = 8192;
    manifest.height = 8192;
    assert!(manifest.validate().is_err());
    manifest.width = 128;
    manifest.height = 128;
    manifest.cases.push(Case {
        name: "dense".into(),
        config: Config::default(),
    });
    assert!(manifest.validate().is_err());
}
