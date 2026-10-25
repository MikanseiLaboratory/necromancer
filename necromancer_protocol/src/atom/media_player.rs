//! # Media player; 6/19 atoms
//!
//! ## Unimplemented atoms (13)
//!
//! FourCC | Atom name | Length
//! ------ | --------- | ------
//! `CapA` | `StillCaptureAvailability` | 0xc
//! `CMPA` | `ClearMediaPlayerAudio` | 0xc
//! `CMPC` | `ClearMediaPlayerClip` | 0xc
//! `CMPS` | `ChangeMediaPlayerSetup` | 0x10
//! `CSTL` | `ClearMediaPlayerStill` | 0xc
//! `MPAS` | `MediaPlayerAudioEntry` | 0x5c
//! `MPfM` | `MediaPlayerImageFrameMultiEntry` | 0x12 + (0x1 * entries)
//! `MPSp` | `MediaPlayerSetup` | 0x14
//! `RCPS` | `MediaPlayerClipPlayStatus` | 0x10
//! `SCPS` | `SetMediaPlayerClipPlayStatus` | 0x10
//! `SMPA` | `SetMediaPlayerAudio` | 0x4c
//! `SMPC` | `SetMediaPlayerClip` | 0x4c
//! `SMPS` | `SetMediaPlayerStill` | 0x4c

use super::str_from_utf8_null;
use binrw::binrw;
use std::fmt::Debug;

pub const CAPTURE_STILL: CaptureStill = CaptureStill {};

/// `Capt`: capture still image from ME (`CaptureStill`)
///
/// ## Packet format
///
/// No payload.
#[binrw]
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub struct CaptureStill {}

#[binrw]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MediaPlayerSourceID {
    #[brw(magic = 1u8)]
    Still(#[brw(pad_after = 1)] u8),
    #[brw(magic = 2u8)]
    VideoClip(#[brw(pad_before = 1)] u8),
}

/// `MPCE`: Media player current source event (`MediaPlayerCurrentSource`)
///
/// ## Packet format
///
/// * `u8`: media player ID
/// * `u8`: type (0x01 = still, 0x02 = clip)
/// * `u8`: still image index
/// * `u8`: video clip index
#[binrw]
#[brw(big)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct MediaPlayerSource {
    /// Media player ID
    pub id: u8,
    pub source: MediaPlayerSourceID,
}

/// `MPSS`: Set media player source (`MediaPlayerSetSource`)
///
/// ## Packet format
///
/// * `u8`: setting mask:
///   * 0x01: enable
///   * 0x02: still image
///   * 0x04: video clip
/// * `u8`: media player ID
/// * `u8`: type (0x01 = still, 0x02 = clip)
/// * `u8`: still image index
/// * `u8`: video clip index
/// * 3 bytes padding
#[binrw]
#[brw(big)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct SetMediaPlayerSource {
    // We only read the "enable" bit from this, but write all bits.
    #[br(temp)]
    #[bw(calc(self.mask()))]
    setting_mask: u8,

    #[br(calc(setting_mask & 0x01 != 0))]
    #[bw(ignore)]
    pub enable: bool,

    /// Media player ID to control
    pub id: u8,
    // pub enable: bool,
    #[brw(pad_after = 3)]
    pub source: MediaPlayerSourceID,
}

impl SetMediaPlayerSource {
    /// Calculates the `setting_mask` for this [SetMediaPlayerSource] command.
    fn mask(&self) -> u8 {
        let mut o = 0;
        if self.enable {
            o |= 0x01;
        }
        match self.source {
            MediaPlayerSourceID::Still(_) => {
                o |= 0x02;
            }
            MediaPlayerSourceID::VideoClip(_) => {
                o |= 0x04;
            }
        }
        o
    }
}

/// `_mpl`: Media player capabilities (`CapabilitiesMediaPlayer`)
///
/// ## Packet format
///
/// * `u8`: still frame count
/// * `u8`: clip count
/// * `bool`: supports still image capture
/// * `u8`: ?
#[binrw]
#[brw(big)]
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub struct MediaPlayerCapabilities {
    pub still_count: u8,
    pub clip_count: u8,
    #[brw(pad_after = 1)]
    #[br(map = |v: u8| v != 0)]
    #[bw(map = |v: &bool| Into::<u8>::into(*v))]
    pub supports_still_capture: bool,
}

/// `MPfe`: media player frame description (`MediaPlayerImageFrameEntry`)
///
/// ## Packet format
///
/// Packets are padded to 4 byte boundaries.
///
/// * `u8`: store ID
/// * 1 byte padding
/// * `u16`: storage index / slot
/// * `u8`: is valid
/// * `char[16]`: MD5 hash of image frame
/// * 1 byte padding
/// * `u16`: image name length
/// * image name
/// * 0 - 3 bytes of padding
#[binrw]
#[brw(big)]
#[derive(Default, PartialEq, Eq, Clone)]
pub struct MediaPlayerFrameDescription {
    #[brw(pad_after = 1)]
    pub store_id: u8,
    pub index: u16,
    #[br(map = |v: u8| v != 0)]
    #[bw(map = |v: &bool| Into::<u8>::into(*v))]
    pub is_valid: bool,
    #[brw(pad_after = 1)]
    pub md5: [u8; 16],

    #[br(temp)]
    #[bw(try_calc(u16::try_from(name.len())))]
    name_length: u16,

    #[br(count = name_length, try_map = |v: Vec<u8>| str_from_utf8_null(&v).map(str::to_string))]
    #[bw(map = |v: &String| { v.as_bytes().to_vec() })]
    #[brw(align_after = 4)]
    pub name: String,
}

impl Debug for MediaPlayerFrameDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut b = f.debug_struct("MediaPlayerFrameDescription");

        b.field("store_id", &self.store_id)
            .field("index", &self.index)
            .field("is_valid", &self.is_valid);
        if self.is_valid {
            b.field("md5", &hex::encode(self.md5))
                .field("name", &self.name);
        }
        b.finish()
    }
}

/// `MPCS`: media pool clip description (`MediaPoolClipDescription`)
///
/// Sent once per clip slot after connecting, and again when a clip is set or
/// cleared. This is the only place the wire protocol carries a clip name:
/// `MPfe` describes stills, and `RCPS` only reports playback state.
///
/// ## Packet format
///
/// Fixed length, 0x4c bytes.
///
/// * `u8`: clip index
/// * `u8`: is used (`1` when the slot holds a clip)
/// * `char[64]`: clip name, null-terminated, padded with nulls
/// * `u16`: frame count
#[binrw]
#[brw(big)]
#[derive(Default, PartialEq, Eq, Clone)]
pub struct MediaPoolClipDescription {
    /// Clip slot this describes.
    pub index: u8,

    #[br(map = |v: u8| v != 0)]
    #[bw(map = |v: &bool| Into::<u8>::into(*v))]
    pub is_used: bool,

    #[br(try_map = |v: [u8; 64]| str_from_utf8_null(&v).map(str::to_string))]
    #[bw(map = |v: &String| name_bytes(v))]
    pub name: String,

    pub frames: u16,
}

/// Pad `name` into the fixed 64 byte field.
fn name_bytes(name: &str) -> [u8; 64] {
    let mut out = [0u8; 64];
    let bytes = name.as_bytes();
    let len = bytes.len().min(64);
    out[..len].copy_from_slice(&bytes[..len]);
    out
}

impl Debug for MediaPoolClipDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut b = f.debug_struct("MediaPoolClipDescription");

        b.field("index", &self.index)
            .field("is_used", &self.is_used)
            .field("frames", &self.frames);
        if self.is_used {
            b.field("name", &self.name);
        }
        b.finish()
    }
}

#[cfg(test)]
mod test {
    use binrw::{BinRead, BinWrite};
    use std::io::Cursor;

    use super::*;
    use crate::{
        atom::{Atom, Payload},
        Result,
    };

    #[test]
    fn capture() -> Result<()> {
        let cmd: Vec<u8> = hex::decode("0008000043617074")?;
        let capture = Atom::read(&mut Cursor::new(&cmd))?;
        let Payload::CaptureStill(capture) = capture.payload else {
            panic!("wrong command type");
        };
        assert_eq!(CAPTURE_STILL, capture);

        let o = Atom::new(CAPTURE_STILL);
        let mut out = Cursor::new(Vec::with_capacity(cmd.len()));
        o.write(&mut out)?;
        assert_eq!(cmd, out.into_inner());
        Ok(())
    }

    #[test]
    fn media_player_source_changed() -> Result<()> {
        let expected = MediaPlayerSource {
            id: 0,
            source: MediaPlayerSourceID::Still(5),
        };
        let cmd: Vec<u8> = hex::decode("000c00004d50434500010500")?;
        let mpce = Atom::read(&mut Cursor::new(&cmd))?;
        let Payload::MediaPlayerSource(mpce) = mpce.payload else {
            panic!("wrong command type");
        };
        assert_eq!(expected, mpce);

        let o = Atom::new(expected);
        let mut out = Cursor::new(Vec::with_capacity(cmd.len()));
        o.write(&mut out)?;
        assert_eq!(cmd, out.into_inner());
        Ok(())
    }

    #[test]
    fn set_media_player_source() -> Result<()> {
        let expected = SetMediaPlayerSource {
            id: 0,
            enable: true,
            source: MediaPlayerSourceID::Still(5),
        };
        let cmd: Vec<u8> = hex::decode("001000004d5053530300010500000000")?;
        let mpss = Atom::read(&mut Cursor::new(&cmd))?;
        let Payload::SetMediaPlayerSource(mpss) = mpss.payload else {
            panic!("wrong command type");
        };
        assert_eq!(expected, mpss);

        let o = Atom::new(expected);
        let mut out = Cursor::new(Vec::with_capacity(cmd.len()));
        o.write(&mut out)?;
        assert_eq!(cmd, out.into_inner());
        Ok(())
    }

    #[test]
    fn media_player_capabilites() -> Result<()> {
        let expected = MediaPlayerCapabilities {
            still_count: 20,
            clip_count: 0,
            supports_still_capture: true,
        };
        let cmd: Vec<u8> = hex::decode("000c00005f6d706c14000100")?;
        let mpl = Atom::read(&mut Cursor::new(&cmd))?;
        let Payload::MediaPlayerCapabilities(mpl) = mpl.payload else {
            panic!("wrong command type");
        };
        assert_eq!(expected, mpl);

        let o = Atom::new(expected);
        let mut out = Cursor::new(Vec::with_capacity(cmd.len()));
        o.write(&mut out)?;
        assert_eq!(cmd, out.into_inner());
        Ok(())
    }

    #[test]
    fn media_pool_clip_description() -> Result<()> {
        let _ = tracing_subscriber::fmt().try_init();

        // Occupied slot. Layout after the 8 byte header:
        // index (1), is_used (1), name (64, null padded), frames (2) = 0x48 payload.
        let mut payload = Vec::new();
        payload.push(1); // index
        payload.push(1); // is_used
        let mut name = [0u8; 64];
        name[..16].copy_from_slice(b"opening-loop.mov");
        payload.extend_from_slice(&name);
        payload.extend_from_slice(&1500u16.to_be_bytes());
        assert_eq!(payload.len(), 68);

        let mut cmd = Vec::new();
        cmd.extend_from_slice(&76u16.to_be_bytes()); // 0x4c, including the header
        cmd.extend_from_slice(&[0, 0]);
        cmd.extend_from_slice(b"MPCS");
        cmd.extend_from_slice(&payload);

        let expected = MediaPoolClipDescription {
            index: 1,
            is_used: true,
            name: "opening-loop.mov".to_string(),
            frames: 1500,
        };
        let mpcs = Atom::read(&mut Cursor::new(&cmd))?;
        let Payload::MediaPoolClipDescription(mpcs) = mpcs.payload else {
            panic!("wrong command type");
        };
        assert_eq!(expected, mpcs);

        let o = Atom::new(expected);
        let mut out = Cursor::new(Vec::with_capacity(cmd.len()));
        o.write(&mut out)?;
        assert_eq!(cmd, out.into_inner());

        Ok(())
    }

    #[test]
    fn media_pool_clip_description_empty_slot() -> Result<()> {
        let _ = tracing_subscriber::fmt().try_init();

        let mut cmd: Vec<u8> = hex::decode("004c00004d504353030000")?;
        cmd.resize(0x4c, 0);
        let mpcs = Atom::read(&mut Cursor::new(&cmd))?;
        let Payload::MediaPoolClipDescription(mpcs) = mpcs.payload else {
            panic!("wrong command type");
        };
        assert_eq!(
            mpcs,
            MediaPoolClipDescription {
                index: 3,
                is_used: false,
                name: "".to_string(),
                frames: 0,
            }
        );

        Ok(())
    }

    #[test]
    fn media_player_frame_description() -> Result<()> {
        let _ = tracing_subscriber::fmt().try_init();
        // All of these samples had uninitialised memory :(
        // Occupied slot
        let expected = MediaPlayerFrameDescription {
            store_id: 0,
            index: 5,
            is_valid: true,
            md5: [
                0xb1, 0xa6, 0x19, 0x4d, 0x4f, 0x52, 0xb4, 0x49, 0xfd, 0x51, 0x98, 0x70, 0xa6, 0x3c,
                0xb3, 0xc2,
            ],
            name: "tram-1080p.rle".to_string(),
        };
        let cmd: Vec<u8> = hex::decode("003000004d5066650000000501b1a6194d4f52b449fd519870a63cb3c200000e7472616d2d31303830702e726c650000")?;
        let mpfe = Atom::read(&mut Cursor::new(&cmd))?;
        let Payload::MediaPlayerFrameDescription(mpfe) = mpfe.payload else {
            panic!("wrong command type");
        };
        assert_eq!(expected, mpfe);

        let o = Atom::new(expected);
        let mut out = Cursor::new(Vec::with_capacity(cmd.len()));
        o.write(&mut out)?;
        assert_eq!(cmd, out.into_inner());

        // Empty slot
        let expected = MediaPlayerFrameDescription {
            store_id: 0,
            index: 2,
            is_valid: false,
            md5: [0; 16],
            name: "".to_string(),
        };
        let cmd: Vec<u8> =
            hex::decode("002000004d506665000000020000000000000000000000000000000000000000")?;
        let mpfe = Atom::read(&mut Cursor::new(&cmd))?;
        let Payload::MediaPlayerFrameDescription(mpfe) = mpfe.payload else {
            panic!("wrong command type");
        };
        assert_eq!(expected, mpfe);

        let o = Atom::new(expected);
        let mut out = Cursor::new(Vec::with_capacity(cmd.len()));
        o.write(&mut out)?;
        assert_eq!(cmd, out.into_inner());

        // Checking word alignment, 1 byte name
        let expected = MediaPlayerFrameDescription {
            store_id: 0,
            index: 0,
            is_valid: true,
            md5: [
                0xb1, 0xa6, 0x19, 0x4d, 0x4f, 0x52, 0xb4, 0x49, 0xfd, 0x51, 0x98, 0x70, 0xa6, 0x3c,
                0xb3, 0xc2,
            ],
            name: "A".to_string(),
        };
        let cmd: Vec<u8> = hex::decode(
            "002400004d5066650000000001b1a6194d4f52b449fd519870a63cb3c200000141000000",
        )?;
        let mpfe = Atom::read(&mut Cursor::new(&cmd))?;
        let Payload::MediaPlayerFrameDescription(mpfe) = mpfe.payload else {
            panic!("wrong command type");
        };
        assert_eq!(expected, mpfe);

        let o = Atom::new(expected);
        let mut out = Cursor::new(Vec::with_capacity(cmd.len()));
        o.write(&mut out)?;
        assert_eq!(cmd, out.into_inner());

        // Checking word alignment, 1 byte name
        let expected = MediaPlayerFrameDescription {
            store_id: 0,
            index: 1,
            is_valid: true,
            md5: [
                0xb1, 0xa6, 0x19, 0x4d, 0x4f, 0x52, 0xb4, 0x49, 0xfd, 0x51, 0x98, 0x70, 0xa6, 0x3c,
                0xb3, 0xc2,
            ],
            name: "AB".to_string(),
        };
        let cmd: Vec<u8> = hex::decode(
            "002400004d5066650000000101b1a6194d4f52b449fd519870a63cb3c200000241420000",
        )?;
        let mpfe = Atom::read(&mut Cursor::new(&cmd))?;
        let Payload::MediaPlayerFrameDescription(mpfe) = mpfe.payload else {
            panic!("wrong command type");
        };
        assert_eq!(expected, mpfe);

        let o = Atom::new(expected);
        let mut out = Cursor::new(Vec::with_capacity(cmd.len()));
        o.write(&mut out)?;
        assert_eq!(cmd, out.into_inner());

        // Checking word alignment, 3 byte name
        let expected = MediaPlayerFrameDescription {
            store_id: 0,
            index: 2,
            is_valid: true,
            md5: [
                0xb1, 0xa6, 0x19, 0x4d, 0x4f, 0x52, 0xb4, 0x49, 0xfd, 0x51, 0x98, 0x70, 0xa6, 0x3c,
                0xb3, 0xc2,
            ],
            name: "ABC".to_string(),
        };
        let cmd: Vec<u8> = hex::decode(
            "002400004d5066650000000201b1a6194d4f52b449fd519870a63cb3c200000341424300",
        )?;
        let mpfe = Atom::read(&mut Cursor::new(&cmd))?;
        let Payload::MediaPlayerFrameDescription(mpfe) = mpfe.payload else {
            panic!("wrong command type");
        };
        assert_eq!(expected, mpfe);

        let o = Atom::new(expected);
        let mut out = Cursor::new(Vec::with_capacity(cmd.len()));
        o.write(&mut out)?;
        assert_eq!(cmd, out.into_inner());

        // Checking word alignment, 4 byte name
        let expected = MediaPlayerFrameDescription {
            store_id: 0,
            index: 3,
            is_valid: true,
            md5: [
                0xb1, 0xa6, 0x19, 0x4d, 0x4f, 0x52, 0xb4, 0x49, 0xfd, 0x51, 0x98, 0x70, 0xa6, 0x3c,
                0xb3, 0xc2,
            ],
            name: "ABCD".to_string(),
        };
        let cmd: Vec<u8> = hex::decode(
            "002400004d5066650000000301b1a6194d4f52b449fd519870a63cb3c200000441424344",
        )?;
        let mpfe = Atom::read(&mut Cursor::new(&cmd))?;
        let Payload::MediaPlayerFrameDescription(mpfe) = mpfe.payload else {
            panic!("wrong command type");
        };
        assert_eq!(expected, mpfe);

        let o = Atom::new(expected);
        let mut out = Cursor::new(Vec::with_capacity(cmd.len()));
        o.write(&mut out)?;
        assert_eq!(cmd, out.into_inner());

        // Checking word alignment, 5 byte name
        let expected = MediaPlayerFrameDescription {
            store_id: 0,
            index: 4,
            is_valid: true,
            md5: [
                0xb1, 0xa6, 0x19, 0x4d, 0x4f, 0x52, 0xb4, 0x49, 0xfd, 0x51, 0x98, 0x70, 0xa6, 0x3c,
                0xb3, 0xc2,
            ],
            name: "ABCDE".to_string(),
        };
        let cmd: Vec<u8> = hex::decode(
            "002800004d5066650000000401b1a6194d4f52b449fd519870a63cb3c20000054142434445000000",
        )?;
        let mpfe = Atom::read(&mut Cursor::new(&cmd))?;
        let Payload::MediaPlayerFrameDescription(mpfe) = mpfe.payload else {
            panic!("wrong command type");
        };
        assert_eq!(expected, mpfe);

        let o = Atom::new(expected);
        let mut out = Cursor::new(Vec::with_capacity(cmd.len()));
        o.write(&mut out)?;
        assert_eq!(cmd, out.into_inner());

        Ok(())
    }
}
