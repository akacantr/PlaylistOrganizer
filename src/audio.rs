use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug, Clone, Default)]
pub struct PlayingTrackInfo {
    pub url: String,
    pub title: String,
    pub artist: String,
    #[allow(dead_code)]
    pub image_url: String,
}

struct AudioInner {
    stream_handle: Option<OutputStreamHandle>,
    sink: Option<Sink>,
    current_track: Option<PlayingTrackInfo>,
    is_paused: bool,
    volume: f32,
    play_start: Option<Instant>,
    cached_bytes: Option<(String, Vec<u8>)>,
}

pub struct AudioPlayer {
    _stream: Option<OutputStream>,
    inner: Arc<Mutex<AudioInner>>,
    client: reqwest::Client,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let (stream, stream_handle) = match OutputStream::try_default() {
            Ok((s, h)) => (Some(s), Some(h)),
            Err(e) => {
                eprintln!("Ses aygıtı başlatılamadı (başlatma devam ediyor): {}", e);
                (None, None)
            }
        };

        let sink = stream_handle.as_ref().and_then(|h| Sink::try_new(h).ok());

        Self {
            _stream: stream,
            inner: Arc::new(Mutex::new(AudioInner {
                stream_handle,
                sink,
                current_track: None,
                is_paused: false,
                volume: 0.8,
                play_start: None,
                cached_bytes: None,
            })),
            client: reqwest::Client::new(),
        }
    }

    pub fn ensure_stream(&mut self) {
        if self._stream.is_none() {
            if let Ok((stream, handle)) = OutputStream::try_default() {
                if let Ok(mut lock) = self.inner.lock() {
                    lock.stream_handle = Some(handle);
                }
                self._stream = Some(stream);
            }
        }
    }

    pub fn play(&self, url: &str, title: &str, artist: &str, image_url: &str) {
        let track_info = PlayingTrackInfo {
            url: url.to_string(),
            title: title.to_string(),
            artist: artist.to_string(),
            image_url: image_url.to_string(),
        };

        // Check if we already have the bytes cached in memory for instant replay
        let mut cached_to_play: Option<Vec<u8>> = None;
        if let Ok(mut lock) = self.inner.lock() {
            if let Some(ref sink) = lock.sink {
                sink.stop();
            }
            lock.current_track = Some(track_info);
            lock.is_paused = false;
            lock.play_start = None;

            if let Some((ref cached_url, ref bytes)) = lock.cached_bytes {
                if cached_url == url {
                    cached_to_play = Some(bytes.clone());
                }
            }
        }

        if let Some(bytes) = cached_to_play {
            if let Ok(mut lock) = self.inner.lock() {
                if let Some(ref handle) = lock.stream_handle {
                    if let Ok(new_sink) = Sink::try_new(handle) {
                        new_sink.set_volume(lock.volume);
                        if let Ok(source) = Decoder::new(Cursor::new(bytes)) {
                            new_sink.append(source);
                            new_sink.play();
                            lock.sink = Some(new_sink);
                            lock.play_start = Some(Instant::now());
                            lock.is_paused = false;
                            return;
                        }
                    }
                }
            }
        }

        let url_str = url.to_string();
        let client = self.client.clone();
        let inner_clone = self.inner.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();

            if let Ok(rt) = rt {
                let bytes_res =
                    rt.block_on(async { client.get(&url_str).send().await?.bytes().await });

                if let Ok(bytes) = bytes_res {
                    let bytes_vec = bytes.to_vec();
                    if let Ok(mut lock) = inner_clone.lock() {
                        if lock.current_track.as_ref().map(|t| t.url.as_str()) != Some(&url_str) {
                            return;
                        }

                        lock.cached_bytes = Some((url_str.clone(), bytes_vec.clone()));

                        if let Some(ref handle) = lock.stream_handle {
                            if let Ok(new_sink) = Sink::try_new(handle) {
                                new_sink.set_volume(lock.volume);
                                if let Ok(source) = Decoder::new(Cursor::new(bytes_vec)) {
                                    new_sink.append(source);
                                    new_sink.play();
                                    lock.sink = Some(new_sink);
                                    lock.play_start = Some(Instant::now());
                                    lock.is_paused = false;
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn pause(&self) {
        if let Ok(mut lock) = self.inner.lock() {
            if let Some(ref sink) = lock.sink {
                sink.pause();
                lock.is_paused = true;
            }
        }
    }

    pub fn resume(&self) {
        if let Ok(mut lock) = self.inner.lock() {
            if let Some(ref sink) = lock.sink {
                sink.play();
                lock.is_paused = false;
            }
        }
    }

    pub fn toggle(&self) {
        if self.is_playing() {
            self.pause();
        } else {
            self.resume();
        }
    }

    pub fn stop(&self) {
        if let Ok(mut lock) = self.inner.lock() {
            if let Some(ref sink) = lock.sink {
                sink.stop();
            }
            lock.current_track = None;
            lock.is_paused = false;
            lock.play_start = None;
        }
    }

    pub fn set_volume(&self, vol: f32) {
        let clamped = vol.clamp(0.0, 1.0);
        if let Ok(mut lock) = self.inner.lock() {
            lock.volume = clamped;
            if let Some(ref sink) = lock.sink {
                sink.set_volume(clamped);
            }
        }
    }

    pub fn volume(&self) -> f32 {
        self.inner.lock().map(|l| l.volume).unwrap_or(0.8)
    }

    pub fn is_playing(&self) -> bool {
        if let Ok(lock) = self.inner.lock() {
            if lock.is_paused {
                return false;
            }
            if let Some(ref sink) = lock.sink {
                return !sink.empty();
            }
        }
        false
    }

    #[allow(dead_code)]
    pub fn is_paused(&self) -> bool {
        self.inner.lock().map(|l| l.is_paused).unwrap_or(false)
    }

    pub fn is_exhausted(&self) -> bool {
        if let Ok(lock) = self.inner.lock() {
            if lock.current_track.is_some() && lock.play_start.is_some() {
                if let Some(ref sink) = lock.sink {
                    return sink.empty();
                }
            }
        }
        false
    }

    pub fn replay(&self) {
        if let Some(track) = self.current_track_info() {
            self.play(&track.url, &track.title, &track.artist, &track.image_url);
        }
    }

    pub fn current_url(&self) -> Option<String> {
        self.inner
            .lock()
            .ok()?
            .current_track
            .as_ref()
            .map(|t| t.url.clone())
    }

    pub fn current_track_info(&self) -> Option<PlayingTrackInfo> {
        self.inner.lock().ok()?.current_track.clone()
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}
