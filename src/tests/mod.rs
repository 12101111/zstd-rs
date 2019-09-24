#[cfg(test)]
#[test]
fn test_frame_header_reading() {
    use crate::frame;
    use std::fs;

    let mut content = fs::File::open("./test_img.zst").unwrap();
    let (frame, _) = frame::read_frame_header(&mut content).unwrap();
    frame.check_valid().unwrap();
}

#[test]
fn test_block_header_reading() {
    use crate::decoding;
    use crate::frame;
    use std::fs;

    let mut content = fs::File::open("/home/moritz/rust/zstd-rs/test_img.zst").unwrap();
    let (frame, _) = frame::read_frame_header(&mut content).unwrap();
    frame.check_valid().unwrap();

    let mut block_dec = decoding::block_decoder::new();
    let block_header = block_dec.read_block_header(&mut content).unwrap();
    let _ = block_header; //TODO validate blockheader in a smart way
}

#[test]
fn test_frame_decoder() {
    use crate::frame_decoder;
    use std::fs;

    let mut content = fs::File::open("/home/moritz/rust/zstd-rs/test_img.zst").unwrap();

    struct NullWriter(());
    impl std::io::Write for NullWriter {
        fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
            Ok(buf.len())
        }
        fn flush(&mut self) -> Result<(), std::io::Error> {
            Ok(())
        }
    }
    let mut _null_target = NullWriter(());

    let mut frame_dec = frame_decoder::FrameDecoder::new();
    frame_dec.reset(&mut content).unwrap();
    frame_dec
        .decode_blocks(&mut content, frame_decoder::BlockDecodingStrategy::All)
        .unwrap();
}

#[test]
fn test_decode_from_to() {
    use crate::frame_decoder;
    use std::fs::File;
    use std::io::Read;
    let f = File::open("./decodecorpus_files/z000088.zst").unwrap();
    let mut frame_dec = frame_decoder::FrameDecoder::new();

    
    let content: Vec<u8> = f.bytes().map(|x| x.unwrap()).collect();

    let mut target = Vec::with_capacity(1024*1024);
    target.resize(1024*1024, 0u8);
    
    let source1 = &content[..50*1024];
    let (read1,written1) = frame_dec.decode_from_to(source1, target.as_mut_slice()).unwrap();

    let source2 = &content[read1..];
    let (read2,written2) = frame_dec.decode_from_to(source2, &mut target[written1..]).unwrap();

    let read = read1+read2;
    let written = written1+written2;

    let result = &target.as_slice()[..written];

    if read != content.len() {
        panic!(format!("Byte counter: {} was wrong. Should be: {}", read, content.len()));
    }

    let original_f = File::open("./decodecorpus_files/z000088").unwrap();
    let original: Vec<u8> = original_f.bytes().map(|x| x.unwrap()).collect();

    if original.len() != result.len() {
        panic!(
            "Result has wrong length: {}, should be: {}",
            result.len(),
            original.len()
        );
    }

    let mut counter = 0;
    let min = if original.len() < result.len() {
        original.len()
    } else {
        result.len()
    };
    for idx in 0..min {
        if original[idx] != result[idx] {
            counter += 1;
            //println!(
            //    "Original {:3} not equal to result {:3} at byte: {}",
            //    original[idx], result[idx], idx,
            //);
        }
    }
    if counter > 0 {
        panic!("Result differs in at least {} bytes from original", counter);
    }
}

#[test]
fn test_specific_file() {
    use crate::frame_decoder;
    use std::fs;
    use std::io::Read;

    let mut content = fs::File::open("./decodecorpus_files/z000088.zst").unwrap();

    struct NullWriter(());
    impl std::io::Write for NullWriter {
        fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
            Ok(buf.len())
        }
        fn flush(&mut self) -> Result<(), std::io::Error> {
            Ok(())
        }
    }
    let mut _null_target = NullWriter(());

    let mut frame_dec = frame_decoder::FrameDecoder::new();
    frame_dec.reset(&mut content).unwrap();
    frame_dec
        .decode_blocks(&mut content, frame_decoder::BlockDecodingStrategy::All)
        .unwrap();
    let result = frame_dec.drain_buffer();

    let original_f = fs::File::open("./decodecorpus_files/z000088").unwrap();
    let original: Vec<u8> = original_f.bytes().map(|x| x.unwrap()).collect();

    println!("Results for file:");

    if original.len() != result.len() {
        println!(
            "Result has wrong length: {}, should be: {}",
            result.len(),
            original.len()
        );
    }

    let mut counter = 0;
    let min = if original.len() < result.len() {
        original.len()
    } else {
        result.len()
    };
    for idx in 0..min {
        if original[idx] != result[idx] {
            counter += 1;
            //println!(
            //    "Original {:3} not equal to result {:3} at byte: {}",
            //    original[idx], result[idx], idx,
            //);
        }
    }
    if counter > 0 {
        println!("Result differs in at least {} bytes from original", counter);
    }
}

pub mod decode_corpus;
pub mod fuzz_regressions;
