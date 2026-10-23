use binrw::binrw;

/// Switcher port types.
#[binrw]
#[brw(big, repr = u8)]
#[derive(Debug, FromPrimitive, ToPrimitive, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum PortType {
    /// External port (see [ExternalPortType][super::ExternalPortType] for specifics).
    External = 0x00,
    /// Black video generator port.
    Black = 0x01,
    /// Colour bars generator port.
    ColourBars = 0x02,
    /// Colour generator port.
    ColourGenerator = 0x03,
    /// Media player fill port.
    MediaPlayerFill = 0x04,
    /// Media player cut port.
    MediaPlayerKey = 0x05,
    /// SuperSource port.
    SuperSource = 0x06,
    /// External direct-mode port, which bypasses all switching.
    ExternalDirect = 0x07,
    /// Mix effect block output port.
    MEOutput = 0x80,
    /// Auxiliary output port.
    Auxiliary = 0x81,
    // TODO: may be "key cut output"?
    Mask = 0x82,
    /// MultiView output port.
    Multiview = 0x83,
    /// Audio monitor output. Present from protocol 2.31 (ATEM software 9.4).
    AudioMonitor = 0x84,
}
