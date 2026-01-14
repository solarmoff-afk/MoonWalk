// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

#[cfg(feature = "video")]
use std::path::PathBuf;

#[cfg(feature = "video")]
use std::thread;

#[cfg(feature = "video")]
use std::sync::mpsc::{channel, Sender};

#[cfg(feature = "video")]
use video_rs::{Encoder, Location, Time};

#[cfg(feature = "video")]
use video_rs::encode::Settings;

#[cfg(feature = "video")]
use ndarray::Array3;

#[cfg(feature = "video")]
use image::buffer::ConvertBuffer;

#[cfg(feature = "video")]
use crate::MoonWalkError;

#[cfg(feature = "video")]
use crate::MoonWalk;

#[cfg(feature = "video")]
use crate::public::{VideoFormat, VideoPreset};

#[cfg(feature = "video")]
enum VideoMessage {
    Frame {
        pixels: image::RgbaImage,
        frame_idx: usize,
    },

    Finish,
}

#[cfg(feature = "video")]
pub struct MoonVideo {
    tx: Sender<VideoMessage>,
    width: u32,
    height: u32,
    frame_count: usize,
    worker_handle: Option<thread::JoinHandle<Result<(), String>>>,
}

#[cfg(feature = "video")]
impl MoonVideo {
    pub fn new(
        width: u32,
        height: u32,
        fps: usize,
        path: &str,
        format: VideoFormat,
        _preset: VideoPreset,
    ) -> Result<Self, MoonWalkError> {
        match format {
            VideoFormat::Mp4 | VideoFormat::Mov | VideoFormat::Mkv => {},
            
            _ => return Err(MoonWalkError::IOError(
                "GIF/RAW formats are currently not supported by video-rs 0.10 backend".to_string())
            ),
        }

        video_rs::init().map_err(|e| MoonWalkError::IOError(e.to_string()))?;

        let (tx, rx) = channel::<VideoMessage>();
        let path_owned = path.to_string();

        let w = width as usize;
        let h = height as usize;

        let handle = thread::spawn(move || -> Result<(), String> {
            let destination: Location = PathBuf::from(path_owned).into();
            let settings = Settings::preset_h264_yuv420p(w, h, false);

            let mut encoder = Encoder::new(&destination, settings)
                .map_err(|e| e.to_string())?;
            
            while let Ok(msg) = rx.recv() {
                match msg {
                    VideoMessage::Frame { pixels, frame_idx } => {
                        let rgb_image: image::RgbImage = pixels.convert();
                        let raw = rgb_image.into_raw();
                        
                        let frame_array = Array3::from_shape_vec((h, w, 3), raw)
                            .map_err(|e| e.to_string())?;

                        let time_sec = frame_idx as f32 / fps as f32;
                        let position = Time::from_secs(time_sec);

                        encoder.encode(&frame_array, position).map_err(|e| e.to_string())?;
                    },

                    VideoMessage::Finish => break,
                }
            }
            
            encoder.finish().map_err(|e| e.to_string())?;
            Ok(())
        });

        Ok(Self {
            tx,
            width,
            height,
            frame_count: 0,
            worker_handle: Some(handle),
        })
    }

    pub fn add_frame(&mut self, mw: &MoonWalk, texture_id: u32) -> Result<(), MoonWalkError> {
        let texture = mw.renderer.state.textures.get(&texture_id)
            .ok_or(MoonWalkError::IOError("Texture not found".to_string()))?;

        let rgba_image = texture.download(&mw.renderer.context)?;

        if rgba_image.width() != self.width || rgba_image.height() != self.height {
            return Err(MoonWalkError::IOError("Frame size mismatch".to_string()));
        }

        self.tx.send(VideoMessage::Frame { 
            pixels: rgba_image, 
            frame_idx: self.frame_count 
        }).map_err(|_| MoonWalkError::IOError("Encoder thread died".to_string()))?;

        self.frame_count += 1;
        Ok(())
    }

    pub fn finish(mut self) -> Result<(), MoonWalkError> {
        let _ = self.tx.send(VideoMessage::Finish);
        
        if let Some(handle) = self.worker_handle.take() {
            match handle.join() {
                Ok(res) => res.map_err(|e| MoonWalkError::IOError(e)),
                Err(_) => Err(MoonWalkError::IOError("Encoder thread panicked".to_string())),
            }
        } else {
            Ok(())
        }
    }
}

#[cfg(not(feature = "video"))]
pub struct MoonVideo {}
