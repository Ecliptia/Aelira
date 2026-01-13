use std::io::{Read, Seek};
use symphonia::core::io::{MediaSourceStream, ReadOnlySource};
use symphonia::core::probe::Hint;
use symphonia::core::formats::{FormatReader};
use symphonia::core::codecs::{Decoder, DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error;
use symphonia::core::audio::AudioBufferRef;
use crate::playback::codecs::map_mime_to_hint;

pub struct AudioDecoder {

    pub reader: Box<dyn FormatReader>,

    pub decoder: Box<dyn Decoder>,

    pub track_id: u32,

}



#[allow(dead_code)]

impl AudioDecoder {

    pub fn new<R: Read + Seek + Send + Sync + 'static>(source: R, mime: Option<&str>) -> Result<Self, Error> {

        let source = ReadOnlySource::new(source);

        let mss = MediaSourceStream::new(Box::new(source), Default::default());

        let mut hint = Hint::new();

        if let Some(m) = mime {

            hint = map_mime_to_hint(m);

        }



        let probed = symphonia::default::get_probe()

            .format(&hint, mss, &Default::default(), &Default::default())?;

        

        let reader = probed.format;

        let track = reader.tracks()

            .iter()

            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)

            .ok_or(Error::Unsupported("No supported audio track found"))?;



        let decoder = symphonia::default::get_codecs()

            .make(&track.codec_params, &DecoderOptions::default())?;



        let track_id = track.id;



        Ok(Self {

            reader,

            decoder,

            track_id,

        })

    }



    pub fn next_packet(&mut self) -> Result<AudioBufferRef<'_>, Error> {

        loop {

            let packet = self.reader.next_packet()?;

            if packet.track_id() != self.track_id {

                continue;

            }

            return self.decoder.decode(&packet);

        }

    }

}
