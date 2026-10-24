//! # Input properties; 1/2 atoms
//!
//! ## Unimplemented atoms (1)
//!
//! FourCC | Atom name | Length
//! ------ | --------- | ------
//! `CInL` | `ChangeInputProperties` | 0x28
use crate::{
    atom::{colour::video_source_to_generator_id, str_from_utf8_null},
    structs::{ExternalPortType, PortType, VideoSource},
    Result,
};
use binrw::{binrw, BinRead, BinWrite};
use modular_bitfield::{bitfield, specifiers::B4};
use std::fmt::Debug;

/// Where an input may be routed, other than a mix effect.
///
/// This is the high byte of [`InputProperties::availability3`] (`InPr` payload
/// offset 34). Bit positions match LibAtem `SourceAvailability` and
/// sofie-atem-connection `SourceAvailability`. The official SDK's
/// `BMDSwitcherInputAvailability` is a separate combined mask, with its own
/// numbering.
///
/// Bits, least significant first:
///
/// | Bit | Flag | Notes |
/// | --- | ---- | ----- |
/// | 0 | auxiliary | Any auxiliary output |
/// | 1 | multiviewer | Multiviewer window |
/// | 2 | super_source_art | SuperSource foreground / background |
/// | 3 | super_source_box | SuperSource box |
/// | 4 | key_source | Key source |
/// | 5 | auxiliary1 | Aux 1 output mask on some models (LibAtem `Aux1Output`) |
/// | 6 | auxiliary2 | Aux 2 output mask on some models (LibAtem `Aux2Output`) |
/// | 7 | webcam_out | Webcam output (LibAtem `WebcamOutput`) |
#[bitfield(bits = 8)]
#[repr(u8)]
#[derive(Specifier, BinRead, BinWrite, Debug, Default, PartialEq, Eq, Clone, Copy)]
#[br(map = From::<u8>::from)]
#[bw(map = |&x| Into::<u8>::into(x))]
pub struct SourceAvailability {
    /// Source can be routed to an auxiliary output.
    pub auxiliary: bool,
    /// Source can be shown in a multiviewer window.
    pub multiviewer: bool,
    /// Source can be used as SuperSource art (foreground or background).
    pub super_source_art: bool,
    /// Source can be placed in a SuperSource box.
    pub super_source_box: bool,
    /// Source can be used as a key source.
    pub key_source: bool,
    /// Source can be routed through the Aux 1 output mask.
    ///
    /// Observed on some newer models (for example the ATEM Mini Extreme).
    /// LibAtem calls this `Aux1Output`; sofie-atem-connection calls it
    /// `Auxiliary1`.
    pub auxiliary1: bool,
    /// Source can be routed through the Aux 2 output mask.
    ///
    /// Observed on some newer models (for example the ATEM Mini Extreme).
    /// LibAtem calls this `Aux2Output`; sofie-atem-connection calls it
    /// `Auxiliary2`.
    pub auxiliary2: bool,
    /// Source can be routed to the webcam output.
    ///
    /// LibAtem calls this `WebcamOutput`; sofie-atem-connection calls it
    /// `WebcamOut`.
    pub webcam_out: bool,
}

/// Which mix effects an input may be used on.
///
/// This is the low byte of [`InputProperties::availability3`] (`InPr` payload
/// offset 35). Bit positions match LibAtem `MeAvailability` and
/// sofie-atem-connection `MeAvailability` (M/E 1 through M/E 4). The upper
/// four bits are reserved and are preserved so the raw byte round-trips.
#[bitfield(bits = 8)]
#[repr(u8)]
#[derive(Specifier, BinRead, BinWrite, Debug, Default, PartialEq, Eq, Clone, Copy)]
#[br(map = From::<u8>::from)]
#[bw(map = |&x| Into::<u8>::into(x))]
pub struct MeAvailability {
    /// Available on mix effect 1.
    pub me1: bool,
    /// Available on mix effect 2.
    pub me2: bool,
    /// Available on mix effect 3.
    pub me3: bool,
    /// Available on mix effect 4.
    pub me4: bool,
    #[skip]
    __: B4,
}

/// `InPr`: Input properties (`InputProperties`)
///
/// [`Self::availability3`] is the big-endian pair of availability bytes.
/// The high byte is [`SourceAvailability`] and the low byte is
/// [`MeAvailability`].
#[binrw]
#[brw(big)]
#[derive(PartialEq, Eq, Clone)]
pub struct InputProperties {
    pub video_source: VideoSource,
    pub long_name: [u8; 20],
    pub short_name: [u8; 4],
    #[br(map = |v: u8| v != 0)]
    #[bw(map = |v: &bool| Into::<u8>::into(*v))]
    pub input_names_are_default: bool,
    // pub available_external_port_types: u8, ?? may be wrong, always 0, never used
    // 0x24; ATEM API treats this as a mix of u8 and u16
    #[brw(align_before = 2)]
    pub available_external_port_types: ExternalPortType,
    // 0x26
    pub external_port_type: ExternalPortType,
    // 0x28 (this is 2 bytes, but we only retain the high byte)
    #[brw(pad_after = 1)]
    pub port_type: PortType,
    // 0x2a. Big-endian: high byte is SourceAvailability (payload offset 34),
    // low byte is MeAvailability (payload offset 35).
    pub availability3: u16,
}

impl Debug for InputProperties {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputProperties")
            .field("video_source", &self.video_source)
            .field(
                "long_name",
                &self
                    .get_long_name()
                    .map(str::to_string)
                    .unwrap_or_else(|_| self.long_name.escape_ascii().to_string()),
            )
            .field(
                "short_name",
                &self
                    .get_short_name()
                    .map(str::to_string)
                    .unwrap_or_else(|_| self.short_name.escape_ascii().to_string()),
            )
            .field("input_names_are_default", &self.input_names_are_default)
            .field(
                "available_external_port_types",
                &self.available_external_port_types,
            )
            .field("external_port_type", &self.external_port_type)
            .field("port_type", &self.port_type)
            .field("availability3", &self.availability3)
            .field("source_availability", &self.source_availability())
            .field("me_availability", &self.me_availability())
            .finish()
    }
}

impl InputProperties {
    #[inline]
    pub fn get_long_name(&self) -> Result<&str> {
        str_from_utf8_null(&self.long_name)
    }

    #[inline]
    pub fn get_short_name(&self) -> Result<&str> {
        str_from_utf8_null(&self.short_name)
    }

    /// Source-routing availability (auxiliary, multiviewer, SuperSource, and key).
    ///
    /// This is the high byte of [`Self::availability3`].
    #[inline]
    pub fn source_availability(&self) -> SourceAvailability {
        SourceAvailability::from((self.availability3 >> 8) as u8)
    }

    /// Mix-effect availability (M/E 1 through M/E 4).
    ///
    /// This is the low byte of [`Self::availability3`].
    #[inline]
    pub fn me_availability(&self) -> MeAvailability {
        MeAvailability::from(self.availability3 as u8)
    }

    /// Gets the colour generator ID for this video source.
    ///
    /// Returns [`None`] if this is not a colour generator.
    pub fn colour_generator_id(&self) -> Option<u8> {
        if self.port_type != PortType::ColourGenerator {
            return None;
        }
        video_source_to_generator_id(self.video_source).ok()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::atom::{Atom, Payload};
    use binrw::{BinRead, BinWrite};
    use std::io::Cursor;

    fn round_trip_atom(cmd: &[u8]) -> Result<InputProperties> {
        let atom = Atom::read(&mut Cursor::new(cmd))?;
        let Payload::InputProperties(input_properties) = atom.payload else {
            panic!("wrong command type");
        };

        let mut out = Cursor::new(Vec::with_capacity(cmd.len()));
        Atom::new(input_properties.clone()).write(&mut out)?;
        assert_eq!(cmd, out.into_inner());
        Ok(input_properties)
    }

    fn round_trip_payload(payload: &[u8]) -> Result<InputProperties> {
        let input_properties = InputProperties::read(&mut Cursor::new(payload))?;
        let mut out = Cursor::new(Vec::with_capacity(payload.len()));
        input_properties.write(&mut out)?;
        assert_eq!(payload, out.into_inner());
        Ok(input_properties)
    }

    fn assert_availability(
        input_properties: &InputProperties,
        raw: u16,
        source: SourceAvailability,
        me: MeAvailability,
    ) {
        assert_eq!(raw, input_properties.availability3);
        assert_eq!(source, input_properties.source_availability());
        assert_eq!(me, input_properties.me_availability());
        assert_eq!(
            raw,
            (u16::from(u8::from(input_properties.source_availability())) << 8)
                | u16::from(u8::from(input_properties.me_availability()))
        );
    }

    #[test]
    fn input_properties() -> Result<()> {
        let cmd = hex::decode("002c0000496e5072000143616d657261203100000000000000000000000043414d3101000002000200001101")?;
        let input_properties = round_trip_atom(&cmd)?;

        assert_eq!(VideoSource::Input1, input_properties.video_source);
        assert_eq!("Camera 1", input_properties.get_long_name()?);
        assert_eq!("CAM1", input_properties.get_short_name()?);
        assert!(input_properties.input_names_are_default);
        assert_eq!(
            ExternalPortType::new().with_hdmi(true),
            input_properties.available_external_port_types
        );
        assert_eq!(
            ExternalPortType::new().with_hdmi(true),
            input_properties.external_port_type
        );
        assert_eq!(PortType::External, input_properties.port_type);
        assert_eq!(None, input_properties.colour_generator_id());
        // 0x1101: auxiliary | key source, M/E 1.
        assert_availability(
            &input_properties,
            0x1101,
            SourceAvailability::new()
                .with_auxiliary(true)
                .with_key_source(true),
            MeAvailability::new().with_me1(true),
        );

        Ok(())
    }

    #[test]
    fn colour_bars() -> Result<()> {
        let pl = hex::decode(
            "03e8436f6c6f722042617273000000000000000000004241525301000100010002001001",
        )?;
        let input_properties = round_trip_payload(&pl)?;
        assert_eq!(VideoSource::ColourBars, input_properties.video_source);
        assert_eq!("Color Bars", input_properties.get_long_name()?);
        assert_eq!("BARS", input_properties.get_short_name()?);
        assert!(input_properties.input_names_are_default);
        assert_eq!(
            ExternalPortType::new().with_internal(true),
            input_properties.available_external_port_types
        );
        assert_eq!(
            ExternalPortType::new().with_internal(true),
            input_properties.external_port_type
        );
        assert_eq!(PortType::ColourBars, input_properties.port_type);
        assert_eq!(None, input_properties.colour_generator_id());
        // 0x1001: key source, M/E 1.
        assert_availability(
            &input_properties,
            0x1001,
            SourceAvailability::new().with_key_source(true),
            MeAvailability::new().with_me1(true),
        );

        Ok(())
    }

    #[test]
    fn edited_input() -> Result<()> {
        let pl = hex::decode(
            "00014c6170746f702048444d490000000000000000004c41500000000002000200001101",
        )?;
        let input_properties = round_trip_payload(&pl)?;
        assert_eq!(VideoSource::Input1, input_properties.video_source);
        assert_eq!("Laptop HDMI", input_properties.get_long_name()?);
        assert_eq!("LAP", input_properties.get_short_name()?);
        assert!(!input_properties.input_names_are_default);
        assert_eq!(
            ExternalPortType::new().with_hdmi(true),
            input_properties.available_external_port_types
        );
        assert_eq!(
            ExternalPortType::new().with_hdmi(true),
            input_properties.external_port_type
        );
        assert_eq!(PortType::External, input_properties.port_type);
        assert_eq!(None, input_properties.colour_generator_id());
        assert_availability(
            &input_properties,
            0x1101,
            SourceAvailability::new()
                .with_auxiliary(true)
                .with_key_source(true),
            MeAvailability::new().with_me1(true),
        );
        Ok(())
    }

    #[test]
    fn direct() -> Result<()> {
        let pl = hex::decode(
            "2af94c6170746f702048444d492044697265637400004449520000000002000207000100",
        )?;
        let input_properties = round_trip_payload(&pl)?;
        assert_eq!(VideoSource::Input1Direct, input_properties.video_source);
        assert_eq!("Laptop HDMI Direct", input_properties.get_long_name()?);
        assert_eq!("DIR", input_properties.get_short_name()?);
        assert!(!input_properties.input_names_are_default);
        assert_eq!(
            ExternalPortType::new().with_hdmi(true),
            input_properties.available_external_port_types
        );
        assert_eq!(
            ExternalPortType::new().with_hdmi(true),
            input_properties.external_port_type
        );
        assert_eq!(PortType::ExternalDirect, input_properties.port_type);
        assert_eq!(None, input_properties.colour_generator_id());
        // 0x0100: auxiliary only. Direct inputs are not placed on a mix effect.
        assert_availability(
            &input_properties,
            0x0100,
            SourceAvailability::new().with_auxiliary(true),
            MeAvailability::new(),
        );
        Ok(())
    }

    #[test]
    fn colour_generator() -> Result<()> {
        let expected = InputProperties {
            video_source: VideoSource::Colour1,
            long_name: *b"Color 1\0\0\0\0\0\0\0\0\0\0\0\0\0",
            short_name: *b"COL1",
            input_names_are_default: true,
            available_external_port_types: ExternalPortType::new().with_internal(true),
            external_port_type: ExternalPortType::new().with_internal(true),
            port_type: PortType::ColourGenerator,
            availability3: 1,
        };
        let cmd = hex::decode("002c0000496e507207d1436f6c6f72203100000000000000000000000000434f4c3101000100010003000001")?;
        let inpr = round_trip_atom(&cmd)?;

        assert_eq!(expected, inpr);
        assert_eq!(Some(0), inpr.colour_generator_id());
        // 0x0001: no source-routing flags, M/E 1.
        assert_availability(
            &inpr,
            0x0001,
            SourceAvailability::new(),
            MeAvailability::new().with_me1(true),
        );
        Ok(())
    }

    #[test]
    fn availability_bit_positions() {
        assert_eq!(
            SourceAvailability::from(1 << 0),
            SourceAvailability::new().with_auxiliary(true)
        );
        assert_eq!(
            SourceAvailability::from(1 << 1),
            SourceAvailability::new().with_multiviewer(true)
        );
        assert_eq!(
            SourceAvailability::from(1 << 2),
            SourceAvailability::new().with_super_source_art(true)
        );
        assert_eq!(
            SourceAvailability::from(1 << 3),
            SourceAvailability::new().with_super_source_box(true)
        );
        assert_eq!(
            SourceAvailability::from(1 << 4),
            SourceAvailability::new().with_key_source(true)
        );
        assert_eq!(
            SourceAvailability::from(1 << 5),
            SourceAvailability::new().with_auxiliary1(true)
        );
        assert_eq!(
            SourceAvailability::from(1 << 6),
            SourceAvailability::new().with_auxiliary2(true)
        );
        assert_eq!(
            SourceAvailability::from(1 << 7),
            SourceAvailability::new().with_webcam_out(true)
        );
        assert_eq!(0xff, u8::from(SourceAvailability::from(0xff)));

        assert_eq!(
            MeAvailability::from(1 << 0),
            MeAvailability::new().with_me1(true)
        );
        assert_eq!(
            MeAvailability::from(1 << 1),
            MeAvailability::new().with_me2(true)
        );
        assert_eq!(
            MeAvailability::from(1 << 2),
            MeAvailability::new().with_me3(true)
        );
        assert_eq!(
            MeAvailability::from(1 << 3),
            MeAvailability::new().with_me4(true)
        );
        // Undefined high nibble stays intact.
        let reserved = MeAvailability::from(0xf0);
        assert!(!reserved.me1());
        assert!(!reserved.me2());
        assert!(!reserved.me3());
        assert!(!reserved.me4());
        assert_eq!(0xf0, u8::from(reserved));
    }
}
