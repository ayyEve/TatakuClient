use symphonia::core::{probe::Hint, audio::{AudioBufferRef, Signal}, io::{MediaSourceStream, MediaSource}};

use std::io::Cursor;
use std::sync::Arc;

use crate::errors::{TatakuError, TatakuResult};

#[derive(Clone)]
pub struct Sound {
    // Samples split by channel
    pub samples: Arc<Vec<Vec<f32>>>,
    pub sample_rate: u32,
    pub channels: usize,
}

impl Sound {
    // todo: make not crash
    pub fn load(path: impl AsRef<str>) -> TatakuResult<Self> {
        let file = std::fs::File::open(path.as_ref()).expect(&format!("Failed to open file. {:?}", path.as_ref()));
        Sound::decode(file)
    }

    pub fn load_raw(bytes: Vec<u8>) -> TatakuResult<Self> {
        Sound::decode(Cursor::new(bytes))
    }

    fn decode(source: impl MediaSource + 'static) -> TatakuResult<Self> {
        let source = MediaSourceStream::new(
            Box::new(source),
            Default::default()
        );

        let probe_maybe = symphonia::default::get_probe().format(
            &Hint::new().with_extension("mp3"),
            source,
            &Default::default(),
            &Default::default(),
        );
        match probe_maybe {
            Ok(probe) => {

                let mut reader = probe.format;

                let track = reader.default_track().unwrap();
                let track_id = track.id;

                // println!("Sample Rate Track: {:?}", track.codec_params.sample_format);
                // println!("Codec Type: {}", track.codec_params.codec);

                let mut decoder = symphonia::default::get_codecs().make(&track.codec_params, &Default::default()).expect("Failed to get codecs and produce decoder.");

                let mut samples = Vec::new();
                let mut sample_rate = 0;
                let mut channels = 0;

                while let Ok(packet) = reader.next_packet() {
                    if packet.track_id() != track_id { continue; }

                    match decoder.decode(&packet) {
                        Ok(AudioBufferRef::F32(f)) => {
                            if sample_rate == 0 { sample_rate = f.spec().rate; println!("sample rate f32: {}", sample_rate)}
                            if channels == 0 { 
                                channels = f.spec().channels.count();

                                // Populate channels vec
                                for _ in 0..channels {
                                    samples.push(Vec::new());
                                }
                            }

                            for chan in 0..channels {
                                samples[chan].extend(f.chan(chan));
                            }
                        }

                        Ok(AudioBufferRef::S32(f)) => {
                            if sample_rate == 0 { sample_rate = f.spec().rate; println!("sample rate s32: {}", sample_rate)}
                            if channels == 0 {
                                channels = f.spec().channels.count();

                                // Populate channels vec
                                for _ in 0..channels {
                                    samples.push(Vec::new());
                                }
                            }

                            for chan in 0..channels {
                                samples[chan].extend(f.chan(chan).iter().map(|&x| x as f32 / i32::MAX as f32));
                            }
                        }

                        Err(e) => {
                            println!("Error when decoding sound: {:?}", e);
                        }
                    }
                }

                assert_eq!(channels, samples.len(), "Amount of channels in sample list and from metadata do not match.");

                Ok(Self {
                    samples: Arc::new(samples),
                    sample_rate,
                    channels
                })
            }
            Err(e) => Err(TatakuError::Audio(e.into()))
        }
    }
}
