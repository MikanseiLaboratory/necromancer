# necromancer_protocol &emsp; [![Latest Version]][crates.io] [![Docs version]][docs.rs]

`necromancer_protocol` implements the low-level data structures for Blackmagic Design ATEM video
switchers' control protocol (BEP/BURP), targeting protocol versions 2.30 through 2.32
(ATEM software 8.1.1 through 9.6).

<div class="warning">

**WARNING:** This is still a work in progress.

As this library is based on reverse engineered data structures, the API is subject to change at any
time.

[The `necromancer` crate][necromancer] provides a higher-level API which can actually make a
connection to an ATEM switcher, and manage state; which is probably what you want to use.

</div>

This crate contains protocol documentation as `rustdoc` comments, which you can build with:

```sh
cargo doc --document-private-items --no-deps --open
```

I've also written some [decompiling notes][decompiling] for `BMDSwitcherAPI`.

## Cargo features

- `clap`: Adds clap `ValueEnum` derive macros to some enums.
- `palette`: Adds helpers for using `palette`, enabled by default.
- `serde`: Adds Serde serialisation and deserialisation derive macros to some types.

## References and related work

While this still required a large amount of my own reverse engineering (as most public resources
are out of date), these resources were helpful to get started:

- [skaarhoj][] ([archive link][0]) partially documents an old version of the ATEM protocol (v2.15?),
  and has an Arduino implementation of the old protocol.
  
  They don't publicly publish documentation or code anymore for the current version of the protocol.

- [libqatemcontrol][] implements an old version of the ATEM protocol in C++.

- [blackmagic-camera-control][] describes some similar control protocol to `CCmd` commands for
  camera control over Bluetooth Low Energy.

- The [ATEM Switchers SDK][sdk] (`BMDSwitcherAPI`) has a proprietary C++ COM library which
  implements the protocols. All of ATEM's software _also_ uses this library.

- [Blackmagic SDI Camera Control Protocol][sdi] describes some of the camera control protocol an SDI
  and Bluetooth Low Energy layer.

[0]: https://web.archive.org/web/20180629094026/http://skaarhoj.com/fileadmin/BMDPROTOCOL.html
[blackmagic-camera-control]: https://github.com/coral/blackmagic-camera-control
[crates.io]: https://crates.io/crates/necromancer_protocol
[decompiling]: https://github.com/micolous/necromancer/blob/main/doc/decompiling.md
[Docs version]: https://img.shields.io/docsrs/necromancer_protocol.svg
[docs.rs]: https://docs.rs/necromancer_protocol/
[Latest Version]: https://img.shields.io/crates/v/necromancer_protocol.svg
[libqatemcontrol]: https://github.com/petersimonsson/libqatemcontrol
[necromancer]: https://github.com/micolous/necromancer/tree/main/necromancer
[sdi]: https://documents.blackmagicdesign.com/DeveloperManuals/BlackmagicCameraControl.pdf
[sdk]: https://www.blackmagicdesign.com/au/developer/product/atem
[skaarhoj]: https://www.skaarhoj.com/discover/blackmagic-atem-switcher-protocol
