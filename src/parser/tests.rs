use super::{
    application_extension, comment_extension, graphic_block, graphic_control_extension, image_data,
    image_descriptor, parse_gif, plain_text_block, version, Block, GIFVersion,
    GraphicControlExtension, ImageData, ImageDescriptor, SubBlocks, GIF,
};
use nom::{Context::Code, Err::Error, ErrorKind};

#[test]
fn should_parse_version_89a() {
    assert_eq!(version(&b"89a"[..]), Ok((&b""[..], GIFVersion::GIF89a)));
}

#[test]
fn should_parse_version_87a() {
    assert_eq!(version(&b"87a"[..]), Ok((&b""[..], GIFVersion::GIF87a)));
}

#[test]
fn should_fail_on_unknown_version() {
    assert_eq!(
        version(&b"12a"[..]),
        Err(Error(Code(&b"12a"[..], ErrorKind::Alt)))
    );
}

#[test]
fn should_parse_graphic_control_extension() {
    assert_eq!(
        graphic_control_extension(&[0x21, 0xf9, 0x04, 0x01, 0x64, 0x00, 0x02, 0x00][..]),
        Ok((
            &b""[..],
            GraphicControlExtension {
                byte_size: 4,
                packed_field: 0x01,
                delay_time: 100,
                transparent_color_index: 2,
            }
        ))
    );
}

#[test]
fn should_read_image_descriptor() {
    assert_eq!(
        image_descriptor(&[0x2c, 0x01, 0x00, 0x02, 0x00, 0x05, 0x00, 0x06, 0x00, 0x81][..]),
        Ok((
            &b""[..],
            ImageDescriptor {
                left: 1,
                top: 2,
                width: 5,
                height: 6,
                packed_field: 0x81,
            }
        ))
    );
}

#[test]
fn should_parse_image_data() {
    let data = [1, 2, 255, 255, 3, 255, 255, 255, 0];
    assert_eq!(
        image_data(&data[..]),
        Ok((
            &b""[..],
            ImageData {
                lzw_minimum_code_size: 1,
                data: SubBlocks(&data[1..])
            }
        ))
    );

    let data = [3, 2, 255, 255, 3, 255, 255, 255, 0, 1, 2, 3];
    assert_eq!(
        image_data(&data[..]),
        Ok((
            &[1, 2, 3][..],
            ImageData {
                lzw_minimum_code_size: 3,
                data: SubBlocks(&data[1..9])
            }
        ))
    );
}

#[test]
fn should_parse_graphic_block() {
    let data = [
        // graphic control extension
        0x21, 0xf9, 0x04, // byte size
        0x00, // packed field
        0x00, 0x00, // delay time
        0x00, // transparent color index
        0x00, // block terminator
        // image descriptor
        0x2c, 0x00, 0x00, // left
        0x00, 0x00, // top
        0x0a, 0x00, // width
        0x0a, 0x00, // height
        0x00, // packed field
        // no local color table
        // image data
        0x02, // LZW minimum code size
        0x16, 0x8c, 0x2d, 0x99, 0x87, 0x2a, 0x1c, 0xdc, 0x33, 0xa0, 0x02, 0x75, 0xec, 0x95, 0xfa,
        0xa8, 0xde, 0x60, 0x8c, 0x04, 0x91, 0x4c, 0x01, 0x00,
    ];
    assert_eq!(
        graphic_block(&data[..]),
        Ok((
            &[][..],
            Block::GraphicBlock {
                graphic_control_extension: Some(GraphicControlExtension {
                    byte_size: 4,
                    packed_field: 0,
                    delay_time: 0,
                    transparent_color_index: 0,
                }),
                image_descriptor: ImageDescriptor {
                    left: 0,
                    top: 0,
                    width: 10,
                    height: 10,
                    packed_field: 0,
                },
                local_color_table: None,
                image_data: ImageData {
                    lzw_minimum_code_size: 2,
                    data: SubBlocks(
                        &[
                            0x16, 0x8c, 0x2d, 0x99, 0x87, 0x2a, 0x1c, 0xdc, 0x33, 0xa0, 0x02, 0x75,
                            0xec, 0x95, 0xfa, 0xa8, 0xde, 0x60, 0x8c, 0x04, 0x91, 0x4c, 0x01, 0x00,
                        ][..]
                    )
                }
            }
        ))
    );
}

#[test]
fn should_parse_plain_text_block() {
    let data = [
        0x21, 0x01, 0x0C, 0x00, 0x00, 0x00, 0x00, 0x64, 0x00, 0x64, 0x00, 0x14, 0x14, 0x01, 0x00,
        0x0B, 0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x00,
    ];
    assert_eq!(
        plain_text_block(&data[..]),
        Ok((
            &[][..],
            Block::TextBlock {
                graphic_control_extension: None,
                text: SubBlocks(&data[2..])
            }
        ))
    );
}

#[test]
fn should_parse_application_extension() {
    let data = [
        0x21, 0xFF, 0x0B, 0x4E, 0x45, 0x54, 0x53, 0x43, 0x41, 0x50, 0x45, 0x32, 0x2E, 0x30, 0x03,
        0x01, 0x05, 0x00, 0x00,
    ];
    assert_eq!(
        application_extension(&data[..]),
        Ok((&[][..], Block::ApplicationExtension(SubBlocks(&data[2..]))))
    );
}

#[test]
fn should_parse_comment_extension() {
    let data = [
        0x21, 0xFE, 0x09, 0x62, 0x6C, 0x75, 0x65, 0x62, 0x65, 0x72, 0x72, 0x79, 0x00,
    ];
    assert_eq!(
        comment_extension(&data[..]),
        Ok((&[][..], Block::CommentExtension(SubBlocks(&data[2..]))))
    );
}

#[test]
fn should_parse_gif() {
    let gif_data = include_bytes!("../../fixtures/sample_1.gif");
    assert_eq!(
        parse_gif(gif_data),
        Ok(GIF {
            version: GIFVersion::GIF89a,
            width: 10,
            height: 10,
            global_color_table: Some(
                &[0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0xff, 0x00, 0x00, 0x00][..]
            ),
            data: vec![Block::GraphicBlock {
                graphic_control_extension: Some(GraphicControlExtension {
                    byte_size: 4,
                    packed_field: 0,
                    delay_time: 0,
                    transparent_color_index: 0
                }),
                image_descriptor: ImageDescriptor {
                    left: 0,
                    top: 0,
                    width: 10,
                    height: 10,
                    packed_field: 0,
                },
                local_color_table: None,
                image_data: ImageData {
                    lzw_minimum_code_size: 2,
                    data: SubBlocks(
                        &[
                            0x16, 0x8C, 0x2D, 0x99, 0x87, 0x2A, 0x1C, 0xDC, 0x33, 0xA0, 0x02, 0x75,
                            0xEC, 0x95, 0xFA, 0xA8, 0xDE, 0x60, 0x8C, 0x04, 0x91, 0x4C, 0x01, 0x00
                        ][..]
                    )
                },
            }],
        })
    );

    let gif_data = include_bytes!("../../fixtures/giflib-logo.gif");
    assert_eq!(
        parse_gif(gif_data),
        Ok(GIF {
            version: GIFVersion::GIF89a,
            width: 50,
            height: 50,
            global_color_table: Some(
                &[
                    0xBA, 0x0D, 0x03, 0xBD, 0x0C, 0x03, 0xBA, 0x0F, 0x05, 0xAD, 0x0C, 0x13, 0xA9,
                    0x0D, 0x14, 0xBA, 0x1C, 0x12, 0xBA, 0x1E, 0x14, 0xAB, 0x1E, 0x22, 0xC1, 0x0C,
                    0x03, 0xD1, 0x07, 0x01, 0xD8, 0x07, 0x01, 0xDA, 0x07, 0x01, 0xD8, 0x07, 0x02,
                    0xD0, 0x08, 0x01, 0xD1, 0x08, 0x01, 0xD1, 0x09, 0x01, 0xD2, 0x09, 0x01, 0xD0,
                    0x08, 0x02, 0xD0, 0x09, 0x02, 0xD1, 0x09, 0x02, 0xC1, 0x07, 0x11, 0xC0, 0x08,
                    0x11, 0xC1, 0x09, 0x12, 0xD1, 0x19, 0x11, 0xD2, 0x18, 0x11, 0xD1, 0x18, 0x12,
                    0xD0, 0x19, 0x12, 0xD1, 0x19, 0x12, 0xFF, 0x00, 0x00, 0x14, 0x0D, 0xA9, 0x13,
                    0x0C, 0xAD, 0x03, 0x0D, 0xBA, 0x05, 0x0F, 0xBA, 0x03, 0x0C, 0xBD, 0x12, 0x1C,
                    0xBA, 0x14, 0x1E, 0xBA, 0x22, 0x1E, 0xAB, 0x03, 0x0C, 0xC1, 0x11, 0x07, 0xC1,
                    0x11, 0x08, 0xC0, 0x12, 0x09, 0xC1, 0x01, 0x07, 0xD1, 0x01, 0x08, 0xD0, 0x01,
                    0x08, 0xD1, 0x01, 0x09, 0xD1, 0x02, 0x08, 0xD0, 0x02, 0x09, 0xD0, 0x02, 0x09,
                    0xD1, 0x01, 0x09, 0xD2, 0x01, 0x07, 0xD8, 0x02, 0x07, 0xD8, 0x01, 0x07, 0xDA,
                    0x11, 0x19, 0xD1, 0x12, 0x19, 0xD0, 0x12, 0x18, 0xD1, 0x12, 0x19, 0xD1, 0x11,
                    0x18, 0xD2, 0x00, 0x00, 0xFF, 0xBA, 0xA4, 0x9A, 0x9A, 0xA4, 0xBA, 0xBA, 0xB3,
                    0xA9, 0xB8, 0xB3, 0xAB, 0xAB, 0xB3, 0xB8, 0xA9, 0xB3, 0xBA, 0xBA, 0xC4, 0xBA,
                    0xD0, 0xC7, 0xC0, 0xD0, 0xC7, 0xC1, 0xD0, 0xC8, 0xC1, 0xD1, 0xC8, 0xC1, 0xD1,
                    0xC9, 0xC2, 0xC0, 0xC7, 0xD0, 0xC1, 0xC7, 0xD0, 0xC1, 0xC8, 0xD0, 0xC1, 0xC8,
                    0xD1, 0xC2, 0xC9, 0xD1, 0xD0, 0xD7, 0xD0, 0xD1, 0xD8, 0xD0, 0xD0, 0xD8, 0xD1,
                    0xD1, 0xD8, 0xD1, 0xD2, 0xD9, 0xD1, 0xD1, 0xD9, 0xD2, 0xFF, 0xFF, 0xFF, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
                ][..]
            ),
            data: vec![Block::GraphicBlock {
                graphic_control_extension: Some(GraphicControlExtension {
                    byte_size: 4,
                    packed_field: 0,
                    delay_time: 0,
                    transparent_color_index: 0
                }),
                image_descriptor: ImageDescriptor {
                    left: 0,
                    top: 0,
                    width: 50,
                    height: 50,
                    packed_field: 0,
                },
                local_color_table: None,
                image_data: ImageData {
                    lzw_minimum_code_size: 7,
                    data: SubBlocks(
                        &[
                            0xFE, 0x80, 0x08, 0x0C, 0x83, 0x01, 0x01, 0x83, 0x0C, 0x85, 0x87, 0x89,
                            0x84, 0x86, 0x83, 0x03, 0x1E, 0x31, 0x32, 0x31, 0x21, 0x21, 0x31, 0x96,
                            0x1F, 0x1F, 0x33, 0x91, 0x98, 0x32, 0x9D, 0x98, 0x33, 0x9D, 0x25, 0x0C,
                            0x1C, 0xA4, 0x12, 0x12, 0xA4, 0x1C, 0xA6, 0xA8, 0xAA, 0xA5, 0xA7, 0xA4,
                            0x16, 0x28, 0x39, 0xB2, 0x2E, 0x2E, 0xB2, 0x39, 0xB4, 0xB6, 0x2E, 0x32,
                            0xB9, 0xB5, 0xB2, 0x32, 0xA3, 0xAD, 0xAB, 0xAE, 0xA9, 0xC3, 0xAC, 0x1C,
                            0xB0, 0xBC, 0xC9, 0xB3, 0xBD, 0xB7, 0xCC, 0xBF, 0xC2, 0xD0, 0xC1, 0xD2,
                            0xC7, 0xB1, 0xCB, 0xCA, 0xCD, 0xD7, 0x32, 0x01, 0xA6, 0x12, 0x00, 0x00,
                            0xDC, 0xDE, 0xE0, 0xDF, 0xA6, 0xE1, 0xA6, 0x04, 0x1E, 0xB4, 0x2E, 0x98,
                            0xE9, 0xEB, 0xB4, 0xED, 0xEA, 0x1F, 0xB4, 0x32, 0x1F, 0xDB, 0xE4, 0xE3,
                            0xDD, 0xF7, 0xE5, 0xDD, 0xF5, 0x12, 0xE7, 0xEC, 0xF1, 0xEE, 0x00, 0xC2,
                            0xFB, 0xC7, 0x0E, 0x18, 0xB1, 0x68, 0x07, 0xA7, 0x59, 0x00, 0x71, 0x0D,
                            0x97, 0xB5, 0x87, 0x39, 0x9E, 0x4D, 0x33, 0x46, 0x71, 0x18, 0x85, 0x6A,
                            0xD8, 0x20, 0x3A, 0xCC, 0x18, 0xD1, 0x60, 0x45, 0x84, 0xC6, 0x2E, 0x36,
                            0x64, 0xB6, 0x71, 0xA3, 0x36, 0x71, 0x28, 0xED, 0xA5, 0x94, 0xF0, 0x88,
                            0x60, 0x40, 0x97, 0x03, 0xDD, 0xF1, 0xD3, 0x47, 0x33, 0xDF, 0xBD, 0x01,
                            0x1D, 0x60, 0xBE, 0xDB, 0x09, 0x70, 0x9E, 0xC7, 0x62, 0x40, 0x11, 0x22,
                            0xD3, 0x48, 0xB2, 0xA8, 0x2D, 0x89, 0x09, 0x93, 0x7E, 0x7C, 0x85, 0xB1,
                            0xA4, 0x51, 0x88, 0x48, 0x97, 0x2A, 0x1D, 0x36, 0x94, 0xA3, 0xD3, 0x6C,
                            0x33, 0x6D, 0x72, 0x13, 0x50, 0x00, 0x83, 0x57, 0x03, 0x06, 0xBC, 0x62,
                            0x38, 0x40, 0xC2, 0x86, 0xD9, 0x11, 0x23, 0xCC, 0xDA, 0x10, 0x21, 0xF0,
                            0x43, 0x08, 0x9D, 0xFE, 0x59, 0x57, 0x16, 0xD0, 0x51, 0x24, 0xC8, 0x10,
                            0x1E, 0x3C, 0x86, 0x08, 0x09, 0xD2, 0xC3, 0x47, 0x92, 0x23, 0x49, 0x7E,
                            0xFC, 0x48, 0x42, 0x78, 0x87, 0x88, 0x14, 0x2F, 0x13, 0xAB, 0xFB, 0x89,
                            0x50, 0xC3, 0x90, 0x28, 0x90, 0x9B, 0x40, 0x81, 0x1C, 0x65, 0xC9, 0x12,
                            0xCA, 0x96, 0x29, 0x23, 0xA9, 0x31, 0xF2, 0x28, 0xE3, 0x69, 0x1A, 0x84,
                            0x60, 0xBE, 0x0C, 0x39, 0x73, 0x69, 0xD2, 0x51, 0x8E, 0x70, 0x26, 0x6A,
                            0x2B, 0xC6, 0xE7, 0x54, 0x09, 0x50, 0x85, 0x1E, 0x4D, 0xFB, 0x34, 0x65,
                            0xD5, 0x9D, 0x65, 0xC1, 0x88, 0x4B, 0x4E, 0x00, 0x37, 0x03, 0x3C, 0xA0,
                            0x2C, 0x69, 0x02, 0x04, 0x88, 0x65, 0x28, 0xC5, 0x85, 0x2F, 0x49, 0xBE,
                            0x04, 0xCA, 0x8F, 0x11, 0x3A, 0x05, 0x96, 0xE0, 0x8D, 0xCF, 0x54, 0x02,
                            0xE0, 0xC2, 0x91, 0x1B, 0xB7, 0x5C, 0xDC, 0xF2, 0x72, 0x20, 0xD9, 0x9F,
                            0x47, 0x2F, 0x08, 0x72, 0x18, 0x06, 0xD1, 0xB6, 0xD3, 0x57, 0x46, 0x8D,
                            0x9B, 0xB5, 0x2C, 0xD7, 0xE5, 0x51, 0x61, 0x08, 0x52, 0x7B, 0x7D, 0xFD,
                            0x24, 0x36, 0x72, 0xE7, 0x80, 0x3F, 0xD1, 0x3C, 0x7D, 0xF5, 0xA6, 0xD9,
                            0x07, 0xD9, 0x11, 0xF9, 0xE9, 0xF6, 0xD4, 0x7E, 0x03, 0x58, 0xA0, 0x20,
                            0x01, 0x04, 0x28, 0x68, 0x01, 0x83, 0x0E, 0x1E, 0x10, 0x1C, 0x77, 0xE0,
                            0x35, 0xC7, 0xDC, 0x77, 0xC7, 0xF9, 0x40, 0x82, 0x09, 0x28, 0x98, 0xE0,
                            0x41, 0x07, 0x28, 0x84, 0xD8, 0x01, 0x88, 0x21, 0x7A, 0xE0, 0x41, 0x88,
                            0x28, 0x8C, 0x88, 0xE2, 0x87, 0x28, 0x80, 0x60, 0x02, 0x09, 0x3E, 0x28,
                            0xD7, 0x1D, 0x85, 0xC7, 0xCD, 0x08, 0x45, 0x0F, 0x07, 0x50, 0x20, 0xC0,
                            0x83, 0x0D, 0x2E, 0xD8, 0xA3, 0x05, 0x03, 0xC4, 0xA0, 0xDF, 0x0D, 0x4A,
                            0xD4, 0x17, 0x60, 0x80, 0x42, 0x60, 0xFE, 0x80, 0xCA, 0x04, 0x41, 0x91,
                            0xC2, 0xC0, 0x2E, 0xEE, 0xE5, 0x40, 0xA4, 0x91, 0xA8, 0x21, 0xA9, 0x41,
                            0x7C, 0x4E, 0x0A, 0x19, 0x65, 0x0D, 0x47, 0x50, 0x59, 0x9F, 0x10, 0x57,
                            0xF6, 0x87, 0x0A, 0x03, 0x6F, 0x29, 0xD6, 0x4E, 0x0A, 0x23, 0xFC, 0x90,
                            0xDD, 0x85, 0xCC, 0x69, 0x77, 0x1C, 0x0F, 0x06, 0x24, 0xA0, 0xD2, 0x9C,
                            0x12, 0x04, 0x50, 0x66, 0x4C, 0x31, 0xA1, 0xA9, 0x26, 0x8D, 0x7C, 0xBA,
                            0xD9, 0x1C, 0x9C, 0x72, 0x56, 0x47, 0x67, 0x00, 0x5A, 0x5A, 0xC5, 0x0C,
                            0x97, 0x5E, 0xAA, 0x97, 0x24, 0x96, 0x1C, 0x3C, 0xA9, 0x9F, 0x0D, 0x5D,
                            0x02, 0x58, 0x25, 0x6A, 0x60, 0x32, 0xEA, 0xE8, 0x96, 0x49, 0x50, 0xC6,
                            0x04, 0x13, 0x94, 0x09, 0xD7, 0x29, 0xA5, 0x61, 0x4E, 0x35, 0xE6, 0x9D,
                            0x3C, 0xA5, 0x23, 0xC2, 0x0E, 0x84, 0x05, 0x36, 0x18, 0x60, 0x3F, 0xF4,
                            0x30, 0x84, 0x5D, 0x78, 0x0D, 0x21, 0xAB, 0x0E, 0x05, 0x04, 0x5A, 0x13,
                            0x37, 0x76, 0x8E, 0xE7, 0x8E, 0x08, 0x6A, 0x89, 0x90, 0x56, 0x0D, 0x36,
                            0x90, 0x70, 0x80, 0x58, 0x60, 0x61, 0xA0, 0x01, 0x06, 0x05, 0xF8, 0x46,
                            0xA7, 0x3E, 0x84, 0xEA, 0x77, 0x95, 0x2C, 0x20, 0x50, 0xC0, 0xA8, 0x54,
                            0x0C, 0x14, 0xFA, 0xAC, 0xA1, 0xB6, 0xA0, 0x20, 0xAD, 0x98, 0xDC, 0x36,
                            0x0A, 0x25, 0xB6, 0x51, 0x6A, 0x3B, 0x6D, 0x93, 0x8D, 0xB6, 0x65, 0xAE,
                            0xAE, 0x2E, 0x78, 0x30, 0x00, 0x03, 0xA6, 0x08, 0x70, 0x8F, 0xBB, 0x2B,
                            0x31, 0x4B, 0xEA, 0xB9, 0x66, 0x0A, 0xD4, 0x01, 0x01, 0xF1, 0x6A, 0x35,
                            0xA8, 0xB5, 0x07, 0x5E, 0x8B, 0x82, 0x05, 0xE3, 0x22, 0x54, 0xAD, 0xB3,
                            0x07, 0xFE, 0x1B, 0xF0, 0x34, 0x97, 0x82, 0xAB, 0xB0, 0xC1, 0xDD, 0x52,
                            0x0B, 0x10, 0x0C, 0xA5, 0x9A, 0x79, 0xE7, 0xBD, 0xF9, 0x45, 0x82, 0x43,
                            0x5D, 0x00, 0x1F, 0xC0, 0x50, 0x2F, 0x4C, 0x21, 0xD8, 0x8B, 0xEF, 0xB2,
                            0xFA, 0x0A, 0x5A, 0xE7, 0xB7, 0xD7, 0xFA, 0x0B, 0x70, 0xC3, 0xE4, 0x26,
                            0x5C, 0x72, 0xC1, 0x27, 0x8B, 0xDA, 0xAD, 0xCA, 0x24, 0xF7, 0xCB, 0x0C,
                            0xC3, 0x2E, 0xD7, 0xCC, 0x40, 0x09, 0x9D, 0xCC, 0xF3, 0x41, 0xCE, 0x9C,
                            0x84, 0xB2, 0xB3, 0x24, 0x9C, 0x58, 0xA2, 0xAE, 0x22, 0x8D, 0x20, 0x52,
                            0xF4, 0x22, 0x46, 0x1F, 0x82, 0x40, 0x20, 0x0
                        ][..]
                    )
                },
            }],
        })
    );
}
