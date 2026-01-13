use tokio::io::{AsyncRead, AsyncReadExt};
use tokio_util::codec::FramedRead;
use futures_util::StreamExt;
use crate::playback::demuxers::webm::WebmOpusDemuxer;
use crate::playback::decoder::symphonia::AudioDecoder;
use audiopus::{coder::Encoder as OpusEncoder, Application, SampleRate, Channels};
use symphonia::core::audio::Signal;
use std::io::Cursor;

pub enum AudioPipeline<R: AsyncRead + Unpin + Send> {
    WebmOpus(FramedRead<R, WebmOpusDemuxer>),
    Pcm(PcmToOpusStream),
}

pub struct PcmToOpusStream {
    decoder: AudioDecoder,
    encoder: OpusEncoder,
    pcm_buffer: Vec<f32>,
}

impl PcmToOpusStream {
    pub fn new(data: Vec<u8>, mime: Option<&str>) -> Option<Self> {
        let cursor = Cursor::new(data);
        let decoder = AudioDecoder::new(cursor, mime).ok()?;
        let encoder = OpusEncoder::new(SampleRate::Hz48000, Channels::Stereo, Application::Audio).ok()?;
        
        Some(Self {
            decoder,
            encoder,
            pcm_buffer: Vec::new(),
        })
    }

    pub fn next_packet(&mut self) -> Option<Result<Vec<u8>, std::io::Error>> {
        loop {
            match self.decoder.next_packet() {
                Ok(audio_buf) => {
                    use symphonia::core::audio::AudioBufferRef;
                    
                    let mut samples = Vec::new();
                    match audio_buf {
                        AudioBufferRef::F32(buf) => {
                            for i in 0..buf.frames() {
                                samples.push(buf.chan(0)[i]);
                                samples.push(buf.chan(1 % buf.spec().channels.count())[i]);
                            }
                        },
                        AudioBufferRef::S16(buf) => {
                            for i in 0..buf.frames() {
                                samples.push(buf.chan(0)[i] as f32 / 32768.0);
                                samples.push(buf.chan(1 % buf.spec().channels.count())[i] as f32 / 32768.0);
                            }
                        },
                        _ => {
                            continue;
                        }
                    }

                    self.pcm_buffer.extend(samples);

                    if self.pcm_buffer.len() >= 1920 {
                        let frame: Vec<f32> = self.pcm_buffer.drain(0..1920).collect();
                        let mut output = vec![0u8; 4000];
                        match self.encoder.encode_float(&frame, &mut output) {
                            Ok(len) => {
                                output.truncate(len);
                                return Some(Ok(output));
                            },
                            Err(e) => return Some(Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Opus error: {:?}", e)))),
                        }
                    }
                },
                Err(symphonia::core::errors::Error::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    return None;
                },
                Err(e) => return Some(Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Symphonia error: {}", e)))),
            }
        }
    }
}

pub struct AudioProcessor<R: AsyncRead + Unpin + Send> {
    pipeline: AudioPipeline<R>,
}

impl<R: AsyncRead + Unpin + Send + 'static> AudioProcessor<R> {
    pub async fn new(mut source: R, format: &str) -> Self {
        if format == "webm/opus" {
            let stream = FramedRead::new(source, WebmOpusDemuxer::new());
            return Self {
                pipeline: AudioPipeline::WebmOpus(stream),
            };
        }

        let mut buffer = Vec::new();
        let _ = source.read_to_end(&mut buffer).await;
        
        if let Some(pcm_stream) = PcmToOpusStream::new(buffer, Some(format)) {
            Self {
                pipeline: AudioPipeline::Pcm(pcm_stream),
            }
        } else {
            let stream = FramedRead::new(source, WebmOpusDemuxer::new());
            return Self {
                pipeline: AudioPipeline::WebmOpus(stream),
            };
        }
    }

    pub async fn next_packet(&mut self) -> Option<Result<Vec<u8>, std::io::Error>> {
        match &mut self.pipeline {
            AudioPipeline::WebmOpus(stream) => {
                stream.next().await.map(|res| res.map(|b| b.to_vec()))
            },
            AudioPipeline::Pcm(stream) => {
                stream.next_packet()
            }
        }
    }
}