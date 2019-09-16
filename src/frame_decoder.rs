use super::frame;
use crate::block;
use crate::decoding::scratch::DecoderScratch;
use std::io::Read;

pub struct FrameDecoder {
    pub frame: frame::Frame,
    decoder_scratch: DecoderScratch,
    frame_finished: bool,
    block_counter: usize,
}

pub enum BlockDecodingStrategy {
    All,
    UptoBlocks(usize),
    UptoBytes(usize),
}

impl FrameDecoder {
    pub fn new(source: &mut Read) -> Result<FrameDecoder, String> {
        let frame = frame::read_frame_header(source)?;
        let window_size = frame.header.window_size()?;
        frame.check_valid()?;
        Ok(FrameDecoder {
            frame: frame,
            frame_finished: false,
            block_counter: 0,
            decoder_scratch: DecoderScratch::new(window_size as usize),
        })
    }

    pub fn is_finished(&self) -> bool {
        self.frame_finished
    }

    pub fn blocks_decoded(&self) -> usize {
        self.block_counter
    }

    pub fn decode_blocks(
        &mut self,
        source: &mut Read,
        strat: BlockDecodingStrategy,
    ) -> Result<bool, crate::errors::FrameDecoderError> {
        let mut block_dec = block::block_decoder::new();

        let buffer_size_before = self.decoder_scratch.buffer.len();
        let block_counter_before = self.block_counter;
        loop {
            if crate::VERBOSE {
                println!("################");
                println!("Next Block: {}", self.block_counter);
                println!("################");
            }
            let block_header = match block_dec.read_block_header(source) {
                Ok(h) => h,
                Err(m) => return Err(crate::errors::FrameDecoderError::FailedToReadBlockHeader(m)),
            };
            if crate::VERBOSE {
                println!("");
                println!(
                    "Found {} block with size: {}, which will be of size: {}",
                    block_header.block_type,
                    block_header.content_size,
                    block_header.decompressed_size
                );
            }

            match block_dec.decode_block_content(&block_header, &mut self.decoder_scratch, source) {
                Ok(h) => h,
                Err(m) => return Err(crate::errors::FrameDecoderError::FailedToReadBlockBody(m)),
            };

            self.block_counter += 1;

            if crate::VERBOSE {
                println!("Output: {}", self.decoder_scratch.buffer.len());
            }

            if block_header.last_block {
                self.frame_finished = true;
                //TODO flush buffer
                if self.frame.header.descriptor.content_checksum_flag() {
                    //TODO checksum
                }
                break;
            }

            match strat {
                BlockDecodingStrategy::All => { /* keep going */ }
                BlockDecodingStrategy::UptoBlocks(n) => {
                    if self.block_counter - block_counter_before >= n {
                        break;
                    }
                }
                BlockDecodingStrategy::UptoBytes(n) => {
                    if self.decoder_scratch.buffer.len() - buffer_size_before >= n {
                        break;
                    }
                }
            }
        }

        Ok(self.frame_finished)
    }

    pub fn collect(&mut self) -> Option<Vec<u8>> {
        self.decoder_scratch.buffer.drain_to_window_size()
    }

    pub fn collect_to_writer(&mut self, w: &mut std::io::Write) -> Result<usize, std::io::Error> {
        self.decoder_scratch.buffer.drain_to_window_size_writer(w)
    }

    pub fn drain_buffer(&mut self) -> Vec<u8> {
        self.decoder_scratch.buffer.drain()
    }

    pub fn drain_buffer_to_writer(
        &mut self,
        w: &mut std::io::Write,
    ) -> Result<usize, std::io::Error> {
        self.decoder_scratch.buffer.drain_to_writer(w)
    }

    pub fn can_collect(&self) -> usize {
        match self.decoder_scratch.buffer.can_drain_to_window_size() {
            Some(x) => x,
            None => 0,
        }
    }

    pub fn can_drain(&self) -> usize {
        self.decoder_scratch.buffer.can_drain()
    }
}

impl std::io::Read for FrameDecoder {
    fn read(&mut self, target: &mut [u8]) -> std::result::Result<usize, std::io::Error> {
        if self.frame_finished {
            Ok(self.decoder_scratch.buffer.read_all(target))
        } else {
            self.decoder_scratch.buffer.read(target)
        }
    }
}
