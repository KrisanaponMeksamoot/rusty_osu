use std::{
    fs::File,
    path::Path,
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use symphonia::{
    core::{audio::SampleBuffer, codecs::DecoderOptions, io::MediaSourceStream, probe::Hint},
    default::{get_codecs, get_probe},
};

#[derive(PartialEq)]
pub enum PlayerState {
    Stopped,
    Playing,
    Paused,
    Loading,
}

pub struct AudioPlayer {
    sample_data: Arc<Mutex<Vec<f32>>>,
    index: Arc<Mutex<usize>>,
    state: Arc<Mutex<PlayerState>>,
    start_time: Arc<Mutex<Option<Instant>>>,
    stream: Option<cpal::Stream>,
}

impl AudioPlayer {
    pub fn new_async(path: &Path) -> (Self, thread::JoinHandle<()>) {
        let sample_data = Arc::new(Mutex::new(Vec::new()));
        let index = Arc::new(Mutex::new(0));
        let state = Arc::new(Mutex::new(PlayerState::Loading));
        let start_time = Arc::new(Mutex::new(None));

        let data_ptr = sample_data.clone();
        let state_ptr = state.clone();
        let path = path.to_path_buf();

        let handle = thread::spawn(move || {
            *state_ptr.lock().unwrap() = PlayerState::Stopped;
            Self::decode_mp3(&path, &data_ptr);
        });

        (
            Self {
                sample_data,
                index,
                state,
                start_time,
                stream: None,
            },
            handle,
        )
    }

    fn decode_mp3(path: &Path, samples: &Arc<Mutex<Vec<f32>>>) {
        let file = File::open(path).expect("Cannot open file");
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let probed = get_probe()
            .format(&Hint::new(), mss, &Default::default(), &Default::default())
            .expect("Unsupported format");
        let mut format = probed.format;
        let track = format.default_track().expect("No track");
        let track_id = track.id;
        let track_codec_params = &track.codec_params;
        let mut decoder = get_codecs()
            .make(track_codec_params, &DecoderOptions::default())
            .expect("Decoder failed");

        while let Ok(packet) = format.next_packet() {
            if packet.track_id() != track_id {
                continue;
            }
            let decoded = decoder.decode(&packet).expect("Decode failed");
            let spec = *decoded.spec();
            let mut buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
            buf.copy_interleaved_ref(decoded);
            samples.lock().unwrap().extend_from_slice(buf.samples());
        }
    }

    pub fn start(&mut self) {
        if self.stream.is_some() {
            return;
        }

        let data_ptr = self.sample_data.clone();
        let index_ptr = self.index.clone();
        let state_ptr = self.state.clone();

        let host = cpal::default_host();
        let device = host.default_output_device().expect("No output");
        let config = device.default_output_config().unwrap();

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device
                .build_output_stream(
                    &config.into(),
                    move |output: &mut [f32], _| {
                        let mut state = state_ptr.lock().unwrap();
                        if *state != PlayerState::Playing {
                            for s in output.iter_mut() {
                                *s = 0.0;
                            }
                            return;
                        }

                        let data = data_ptr.lock().unwrap();
                        let mut idx = index_ptr.lock().unwrap();
                        for sample in output.iter_mut() {
                            if *idx < data.len() {
                                *sample = data[*idx];
                                *idx += 1;
                            } else {
                                *sample = 0.0;
                                *state = PlayerState::Stopped;
                            }
                        }
                    },
                    |err| eprintln!("Stream error: {}", err),
                    None,
                )
                .unwrap(),
            _ => panic!("Unsupported format"),
        };

        stream.play().unwrap();
        self.stream = Some(stream);
    }

    pub fn play(&mut self) {
        if *self.state.lock().unwrap() != PlayerState::Playing {
            *self.state.lock().unwrap() = PlayerState::Playing;
            *self.start_time.lock().unwrap() = Some(Instant::now());
        }
    }

    pub fn pause(&mut self) {
        if *self.state.lock().unwrap() == PlayerState::Playing {
            *self.state.lock().unwrap() = PlayerState::Paused;
        }
    }

    pub fn stop(&mut self) {
        *self.state.lock().unwrap() = PlayerState::Stopped;
        *self.index.lock().unwrap() = 0;
        *self.start_time.lock().unwrap() = None;
    }

    pub fn get_time_ms(&self) -> f32 {
        if *self.state.lock().unwrap() != PlayerState::Playing {
            return 0.0;
        }
        self.start_time
            .lock()
            .unwrap()
            .map(|t| t.elapsed().as_secs_f32() * 1000.0)
            .unwrap_or(0.0)
    }

    pub fn is_playing(&self) -> bool {
        *self.state.lock().unwrap() == PlayerState::Playing
    }

    pub fn is_loaded(&self) -> bool {
        *self.state.lock().unwrap() != PlayerState::Loading
    }
}
