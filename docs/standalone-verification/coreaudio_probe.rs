//! A silence-only lifecycle probe for an explicitly selected virtual CoreAudio
//! output. Compile with the vendored CPAL revision used by the app. This probe
//! never opens an input or chooses a default device. GPL-3.0-or-later.
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use std::time::{Duration, Instant};

struct CallbackOwner {
    dropped: Arc<AtomicBool>,
    calls: Arc<AtomicUsize>,
}
impl CallbackOwner {
    fn called(&self) {
        self.calls.fetch_add(1, Ordering::Relaxed);
    }
}
impl Drop for CallbackOwner {
    fn drop(&mut self) {
        self.dropped.store(true, Ordering::Release);
    }
}

fn verify() -> Result<(), Box<dyn std::error::Error>> {
    if !cfg!(target_os = "macos") {
        return Err("This probe requires macOS.".into());
    }
    let name = std::env::args()
        .nth(1)
        .ok_or("Pass an exact, previously verified virtual output name.")?;
    if ![
        "Immersed",
        "Steam Streaming Speakers",
        "Virtual Desktop Speakers",
    ]
    .contains(&name.as_str())
    {
        return Err("The selected name is outside this probe's virtual-only allowlist.".into());
    }
    let mut matching = cpal::default_host()
        .output_devices()?
        .filter(|device| device.name().ok().as_deref() == Some(name.as_str()));
    let device = matching.next().ok_or("Virtual output is absent.")?;
    if matching.next().is_some() {
        return Err("Virtual output name is ambiguous.".into());
    }
    let supported = device.default_output_config()?;
    if supported.sample_format() != cpal::SampleFormat::F32 {
        return Err("Probe requires a native float32 virtual output.".into());
    }
    let config = supported.config(); // Keep the virtual endpoint's existing rate and period.
    let mut worst_drop_us = 0;
    let mut callbacks = 0;
    for _ in 0..12 {
        let dropped = Arc::new(AtomicBool::new(false));
        let calls = Arc::new(AtomicUsize::new(0));
        let failed = Arc::new(AtomicBool::new(false));
        let owner = CallbackOwner {
            dropped: dropped.clone(),
            calls: calls.clone(),
        };
        let error_flag = failed.clone();
        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _| {
                owner.called();
                data.fill(0.0);
            },
            move |_| {
                error_flag.store(true, Ordering::Relaxed);
            },
            None,
        )?;
        stream.play()?;
        let deadline = Instant::now() + Duration::from_secs(2);
        while calls.load(Ordering::Relaxed) < 4
            && !failed.load(Ordering::Relaxed)
            && Instant::now() < deadline
        {
            std::thread::sleep(Duration::from_millis(5));
        }
        if failed.load(Ordering::Relaxed) || calls.load(Ordering::Relaxed) < 4 {
            return Err("Virtual stream did not run cleanly.".into());
        }
        let closing = Instant::now();
        drop(stream);
        worst_drop_us = worst_drop_us.max(closing.elapsed().as_micros());
        if !dropped.load(Ordering::Acquire) {
            return Err("Callback owner remained alive after stream destruction.".into());
        }
        let final_calls = calls.load(Ordering::Relaxed);
        std::thread::sleep(Duration::from_millis(20));
        if calls.load(Ordering::Relaxed) != final_calls {
            return Err("Callback continued after stream destruction.".into());
        }
        callbacks += final_calls;
    }
    println!(
        "cycles=12 callbacks={callbacks} worst_drop_us={worst_drop_us} retained_owners=0 callbacks_after_drop=0 rate={}",
        config.sample_rate.0
    );
    Ok(())
}
fn main() {
    if let Err(error) = verify() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
