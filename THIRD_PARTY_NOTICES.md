# Third-Party Notices

This file inventories third-party packages distributed or embedded by Racoon Typper. Racoon Typper project-owned code, resources, and metadata are Apache-2.0; these dependencies retain their original licenses and are not relicensed.

The authoritative machine-readable report is `licenses/dependencies.json`. License expressions and source URLs are captured from the locked Rust metadata and npm installation. Each package's original notice/license remains authoritative at its source distribution.

## Policy

- Unknown licenses fail the policy check.
- GPL/LGPL/AGPL expressions fail unless an explicit, reviewed permissive-choice exception is recorded.
- `r-efi` is the only current exception: its expression offers MIT and Apache-2.0 alternatives; the LGPL alternative is not selected.

## Rust dependency license summary

| License expression | Packages |
|---|---:|
| (MIT OR Apache-2.0) AND Unicode-3.0 | 1 |
| 0BSD OR MIT OR Apache-2.0 | 1 |
| Apache-2.0 | 3 |
| Apache-2.0 / MIT | 1 |
| Apache-2.0 AND ISC | 1 |
| Apache-2.0 AND MIT | 1 |
| Apache-2.0 OR BSL-1.0 | 1 |
| Apache-2.0 OR ISC OR MIT | 3 |
| Apache-2.0 OR MIT | 37 |
| Apache-2.0 WITH LLVM-exception | 1 |
| Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | 5 |
| Apache-2.0/MIT | 3 |
| BSD-2-Clause OR Apache-2.0 OR MIT | 2 |
| BSD-3-Clause | 3 |
| BSD-3-Clause AND MIT | 1 |
| BSD-3-Clause OR Apache-2.0 | 2 |
| BSD-3-Clause OR MIT OR Apache-2.0 | 2 |
| BSD-3-Clause/MIT | 1 |
| BSL-1.0 | 2 |
| CC0-1.0 OR MIT-0 OR Apache-2.0 | 1 |
| CDLA-Permissive-2.0 | 2 |
| ISC | 3 |
| MIT | 120 |
| MIT OR Apache-2.0 | 252 |
| MIT OR Apache-2.0 OR LGPL-2.1-or-later | 2 |
| MIT OR Apache-2.0 OR Zlib | 5 |
| MIT OR Zlib OR Apache-2.0 | 1 |
| MIT/Apache-2.0 | 27 |
| MPL-2.0 | 5 |
| Unicode-3.0 | 18 |
| Unlicense OR MIT | 5 |
| Unlicense/MIT | 2 |
| Zlib | 2 |
| Zlib OR Apache-2.0 OR MIT | 18 |

Total Rust packages: 534

## npm dependency license summary

| License expression | Packages |
|---|---:|
| 0BSD | 1 |
| Apache-2.0 | 3 |
| Apache-2.0 OR MIT | 13 |
| BSD-3-Clause | 1 |
| ISC | 1 |
| MIT | 87 |
| MIT OR Apache-2.0 | 4 |

Total npm packages: 110

## Human-readable package inventory

The following tables identify every locked package, its original license expression, and its source reference. Rust package authors are included when declared by Cargo metadata. The machine-readable inventory additionally records npm integrity hashes and all available metadata.

### Rust packages

| Package | License expression | Source | Declared authors |
|---|---|---|---|
| adler2@2.0.1 | 0BSD OR MIT OR Apache-2.0 | https://github.com/oyvindln/adler2 | Jonas Schievink <jonasschievink@gmail.com>, oyvindln <oyvindln@users.noreply.github.com> |
| ahash@0.8.12 | MIT OR Apache-2.0 | https://github.com/tkaitchuck/ahash | Tom Kaitchuck <Tom.Kaitchuck@gmail.com> |
| aho-corasick@1.1.4 | Unlicense OR MIT | https://github.com/BurntSushi/aho-corasick | Andrew Gallant <jamslam@gmail.com> |
| alloc-no-stdlib@2.0.4 | BSD-3-Clause | https://github.com/dropbox/rust-alloc-no-stdlib | Daniel Reiter Horn <danielrh@dropbox.com> |
| alloc-stdlib@0.2.4 | BSD-3-Clause | https://github.com/dropbox/rust-alloc-no-stdlib | Daniel Reiter Horn <danielrh@dropbox.com> |
| android_system_properties@0.1.5 | MIT/Apache-2.0 | https://github.com/nical/android_system_properties | Nicolas Silva <nical@fastmail.com> |
| anyhow@1.0.102 | MIT OR Apache-2.0 | https://github.com/dtolnay/anyhow | David Tolnay <dtolnay@gmail.com> |
| arbitrary@1.4.2 | MIT OR Apache-2.0 | https://github.com/rust-fuzz/arbitrary/ | The Rust-Fuzz Project Developers, Nick Fitzgerald <fitzgen@gmail.com>, Manish Goregaokar <manishsmail@gmail.com>, Simonas Kazlauskas <arbitrary@kazlauskas.me>, Brian L. Troutwine <brian@troutwine.us>, Corey Farwell <coreyf@rwell.org> |
| arboard@3.6.1 | MIT OR Apache-2.0 | https://github.com/1Password/arboard |  |
| async-trait@0.1.89 | MIT OR Apache-2.0 | https://github.com/dtolnay/async-trait | David Tolnay <dtolnay@gmail.com> |
| atk-sys@0.18.2 | MIT | https://github.com/gtk-rs/gtk3-rs | The gtk-rs Project Developers |
| atk@0.18.2 | MIT | https://github.com/gtk-rs/gtk3-rs | The gtk-rs Project Developers |
| atomic-waker@1.1.2 | Apache-2.0 OR MIT | https://github.com/smol-rs/atomic-waker | Stjepan Glavina <stjepang@gmail.com>, Contributors to futures-rs |
| autocfg@1.5.1 | Apache-2.0 OR MIT | https://github.com/cuviper/autocfg | Josh Stone <cuviper@gmail.com> |
| base64@0.21.7 | MIT OR Apache-2.0 | https://github.com/marshallpierce/rust-base64 | Alice Maz <alice@alicemaz.com>, Marshall Pierce <marshall@mpierce.org> |
| base64@0.22.1 | MIT OR Apache-2.0 | https://github.com/marshallpierce/rust-base64 | Marshall Pierce <marshall@mpierce.org> |
| bit-set@0.8.0 | Apache-2.0 OR MIT | https://github.com/contain-rs/bit-set | Alexis Beingessner <a.beingessner@gmail.com> |
| bit-vec@0.8.0 | Apache-2.0 OR MIT | https://github.com/contain-rs/bit-vec | Alexis Beingessner <a.beingessner@gmail.com> |
| bitflags@1.3.2 | MIT/Apache-2.0 | https://github.com/bitflags/bitflags | The Rust Project Developers |
| bitflags@2.13.0 | MIT OR Apache-2.0 | https://github.com/bitflags/bitflags | The Rust Project Developers |
| block-buffer@0.10.4 | MIT OR Apache-2.0 | https://github.com/RustCrypto/utils | RustCrypto Developers |
| block2@0.6.2 | MIT | https://github.com/madsmtm/objc2 | Mads Marquart <mads@marquart.dk> |
| brotli-decompressor@5.0.3 | BSD-3-Clause/MIT | https://github.com/dropbox/rust-brotli-decompressor | Daniel Reiter Horn <danielrh@dropbox.com>, The Brotli Authors |
| brotli@8.0.4 | BSD-3-Clause AND MIT | https://github.com/dropbox/rust-brotli | Daniel Reiter Horn <danielrh@dropbox.com>, The Brotli Authors |
| bs58@0.5.1 | MIT/Apache-2.0 | https://github.com/Nullus157/bs58-rs |  |
| bumpalo@3.20.3 | MIT OR Apache-2.0 | https://github.com/fitzgen/bumpalo | Nick Fitzgerald <fitzgen@gmail.com> |
| bytemuck@1.25.0 | Zlib OR Apache-2.0 OR MIT | https://github.com/Lokathor/bytemuck | Lokathor <zefria@gmail.com> |
| byteorder-lite@0.1.0 | Unlicense OR MIT | https://github.com/image-rs/byteorder-lite |  |
| byteorder@1.5.0 | Unlicense OR MIT | https://github.com/BurntSushi/byteorder | Andrew Gallant <jamslam@gmail.com> |
| bytes@1.12.0 | MIT | https://github.com/tokio-rs/bytes | Carl Lerche <me@carllerche.com>, Sean McArthur <sean@seanmonstar.com> |
| cairo-rs@0.18.5 | MIT | https://github.com/gtk-rs/gtk-rs-core | The gtk-rs Project Developers |
| cairo-sys-rs@0.18.2 | MIT | https://github.com/gtk-rs/gtk-rs-core | The gtk-rs Project Developers |
| camino@1.2.3 | MIT OR Apache-2.0 | https://github.com/camino-rs/camino | Without Boats <saoirse@without.boats>, Ashley Williams <ashley666ashley@gmail.com>, Steve Klabnik <steve@steveklabnik.com>, Rain <rain@sunshowers.io> |
| cargo_metadata@0.19.2 | MIT | https://github.com/oli-obk/cargo_metadata | Oliver Schneider <git-spam-no-reply9815368754983@oli-obk.de> |
| cargo_toml@0.22.3 | Apache-2.0 OR MIT | https://gitlab.com/lib.rs/cargo_toml | Kornel <kornel@geekhood.net> |
| cargo-platform@0.1.9 | MIT OR Apache-2.0 | https://github.com/rust-lang/cargo |  |
| cc@1.2.65 | MIT OR Apache-2.0 | https://github.com/rust-lang/cc-rs | Alex Crichton <alex@alexcrichton.com> |
| cesu8@1.1.0 | Apache-2.0/MIT | https://github.com/emk/cesu8-rs | Eric Kidd <git@randomhacks.net> |
| cfb@0.7.3 | MIT | https://github.com/mdsteele/rust-cfb | Matthew D. Steele <mdsteele@alum.mit.edu> |
| cfg_aliases@0.2.1 | MIT | https://github.com/katharostech/cfg_aliases | Zicklag <zicklag@katharostech.com> |
| cfg-expr@0.15.8 | MIT OR Apache-2.0 | https://github.com/EmbarkStudios/cfg-expr | Embark <opensource@embark-studios.com>, Jake Shadle <jake.shadle@embark-studios.com> |
| cfg-if@1.0.4 | MIT OR Apache-2.0 | https://github.com/rust-lang/cfg-if | Alex Crichton <alex@alexcrichton.com> |
| chacha20@0.10.1 | MIT OR Apache-2.0 | https://github.com/RustCrypto/stream-ciphers | RustCrypto Developers |
| chrono@0.4.45 | MIT OR Apache-2.0 | https://github.com/chronotope/chrono |  |
| clipboard-win@5.4.1 | BSL-1.0 | https://github.com/DoumanAsh/clipboard-win | Douman <douman@gmx.se> |
| combine@4.6.7 | MIT | https://github.com/Marwes/combine | Markus Westerlind <marwes91@gmail.com> |
| cookie@0.18.1 | MIT OR Apache-2.0 | https://github.com/SergioBenitez/cookie-rs | Sergio Benitez <sb@sergio.bz>, Alex Crichton <alex@alexcrichton.com> |
| core-foundation-sys@0.8.7 | MIT OR Apache-2.0 | https://github.com/servo/core-foundation-rs | The Servo Project Developers |
| core-foundation@0.10.1 | MIT OR Apache-2.0 | https://github.com/servo/core-foundation-rs | The Servo Project Developers |
| core-graphics-types@0.2.0 | MIT OR Apache-2.0 | https://github.com/servo/core-foundation-rs | The Servo Project Developers |
| core-graphics@0.25.0 | MIT OR Apache-2.0 | https://github.com/servo/core-foundation-rs | The Servo Project Developers |
| cpufeatures@0.2.17 | MIT OR Apache-2.0 | https://github.com/RustCrypto/utils | RustCrypto Developers |
| cpufeatures@0.3.0 | MIT OR Apache-2.0 | https://github.com/RustCrypto/utils | RustCrypto Developers |
| crc32fast@1.5.0 | MIT OR Apache-2.0 | https://github.com/srijs/rust-crc32fast | Sam Rijs <srijs@airpost.net>, Alex Crichton <alex@alexcrichton.com> |
| crossbeam-channel@0.5.15 | MIT OR Apache-2.0 | https://github.com/crossbeam-rs/crossbeam |  |
| crossbeam-utils@0.8.21 | MIT OR Apache-2.0 | https://github.com/crossbeam-rs/crossbeam |  |
| crunchy@0.2.4 | MIT | https://github.com/eira-fransham/crunchy | Eira Fransham <jackefransham@gmail.com> |
| crypto-common@0.1.7 | MIT OR Apache-2.0 | https://github.com/RustCrypto/traits | RustCrypto Developers |
| cssparser-macros@0.6.1 | MPL-2.0 | https://github.com/servo/rust-cssparser | Simon Sapin <simon.sapin@exyr.org> |
| cssparser@0.36.0 | MPL-2.0 | https://github.com/servo/rust-cssparser | Simon Sapin <simon.sapin@exyr.org> |
| ctor-proc-macro@0.0.7 | Apache-2.0 OR MIT | https://github.com/mmastrac/rust-ctor | Matt Mastracci <matthew@mastracci.com> |
| ctor@0.8.0 | Apache-2.0 OR MIT | https://github.com/mmastrac/rust-ctor | Matt Mastracci <matthew@mastracci.com> |
| darling_core@0.23.0 | MIT | https://github.com/TedDriggs/darling | Ted Driggs <ted.driggs@outlook.com> |
| darling_macro@0.23.0 | MIT | https://github.com/TedDriggs/darling | Ted Driggs <ted.driggs@outlook.com> |
| darling@0.23.0 | MIT | https://github.com/TedDriggs/darling | Ted Driggs <ted.driggs@outlook.com> |
| dbus@0.9.11 | Apache-2.0/MIT | https://github.com/diwic/dbus-rs | David Henningsson <diwic@ubuntu.com> |
| deranged@0.5.8 | MIT OR Apache-2.0 | https://github.com/jhpratt/deranged | Jacob Pratt <jacob@jhpratt.dev> |
| derive_arbitrary@1.4.2 | MIT OR Apache-2.0 | https://github.com/rust-fuzz/arbitrary | The Rust-Fuzz Project Developers, Nick Fitzgerald <fitzgen@gmail.com>, Manish Goregaokar <manishsmail@gmail.com>, Andre Bogus <bogusandre@gmail.com>, Corey Farwell <coreyf@rwell.org> |
| derive_more-impl@2.1.1 | MIT | https://github.com/JelteF/derive_more | Jelte Fennema <github-tech@jeltef.nl> |
| derive_more@2.1.1 | MIT | https://github.com/JelteF/derive_more | Jelte Fennema <github-tech@jeltef.nl> |
| digest@0.10.7 | MIT OR Apache-2.0 | https://github.com/RustCrypto/traits | RustCrypto Developers |
| dirs-sys@0.5.0 | MIT OR Apache-2.0 | https://github.com/dirs-dev/dirs-sys-rs | Simon Ochsenreither <simon@ochsenreither.de> |
| dirs@6.0.0 | MIT OR Apache-2.0 | https://github.com/soc/dirs-rs | Simon Ochsenreither <simon@ochsenreither.de> |
| dispatch2@0.3.1 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 | Mads Marquart <mads@marquart.dk>, Mary <mary@mary.zone> |
| displaydoc@0.2.6 | MIT OR Apache-2.0 | https://github.com/yaahc/displaydoc | Jane Lusby <jlusby@yaah.dev> |
| dlopen2_derive@0.4.3 | MIT | https://github.com/OpenByteDev/dlopen2 | Szymon Wieloch <szymon.wieloch@gmail.com>, OpenByte <development.openbyte@gmail.com> |
| dlopen2@0.8.2 | MIT | https://github.com/OpenByteDev/dlopen2 | Szymon Wieloch <szymon.wieloch@gmail.com>, Ahmed Masud <ahmed.masud@saf.ai>, OpenByte <development.openbyte@gmail.com> |
| dom_query@0.27.0 | MIT | https://github.com/niklak/dom_query | niklak <morgenpurple@gmail.com>, importcjj <importcjj@gmail.com> |
| downcast-rs@1.2.1 | MIT/Apache-2.0 | https://github.com/marcianx/downcast-rs | Ashish Myles <marcianx@gmail.com>, Runji Wang <wangrunji0408@163.com> |
| dpi@0.1.2 | Apache-2.0 AND MIT | https://github.com/rust-windowing/winit |  |
| dtoa-short@0.3.5 | MPL-2.0 | https://github.com/upsuper/dtoa-short | Xidorn Quan <me@upsuper.org> |
| dtoa@1.0.11 | MIT OR Apache-2.0 | https://github.com/dtolnay/dtoa | David Tolnay <dtolnay@gmail.com> |
| dtor-proc-macro@0.0.6 | Apache-2.0 OR MIT | https://github.com/mmastrac/rust-ctor | Matt Mastracci <matthew@mastracci.com> |
| dtor@0.3.0 | Apache-2.0 OR MIT | https://github.com/mmastrac/rust-ctor | Matt Mastracci <matthew@mastracci.com> |
| dunce@1.0.5 | CC0-1.0 OR MIT-0 OR Apache-2.0 | https://gitlab.com/kornelski/dunce | Kornel <kornel@geekhood.net> |
| dyn-clone@1.0.20 | MIT OR Apache-2.0 | https://github.com/dtolnay/dyn-clone | David Tolnay <dtolnay@gmail.com> |
| embed_plist@1.2.2 | MIT OR Apache-2.0 | https://github.com/nvzqz/embed-plist-rs | Nikolai Vazquez <hello@nikolaivazquez.com> |
| embed-resource@3.0.9 | MIT | https://github.com/nabijaczleweli/rust-embed-resource | наб <nabijaczleweli@nabijaczleweli.xyz>, Cat Plus Plus <piotrlegnica@piotrl.pl>, Liigo <liigo@qq.com>, azyobuzin <azyobuzin@users.sourceforge.jp>, Peter Atashian <retep998@gmail.com>, pravic <ehysta@gmail.com>, Gabriel Majeri <gabriel.majeri6@gmail.com>, SonnyX, Johan Andersson <repi@repi.se>, Jordan Poles <jpdev.noreply@gmail.com>, MSxDOS <melcodos@gmail.com>, Jim McGrath <jimmc2@gmail.com>, roblabla <unfiltered@roblab.la>, Jasper Bekkers <jasper@traverseresearch.nl>, Richard Markiewicz <rmarkiewicz@devolutions.net>, Emerson de Freitas Barcelos <emersonfxbx@gmail.com>, Li Keqing <me@kaze.ai>, Alexis Bourget <alexis.bourget@gmail.com>, Michael Farrell <micolous+git@gmail.com>, Jacob Okamoto <oko@oko.io>, Marijn Suijten <marijn@traverseresearch.nl>, Lucas Nogueira <lucas@tauri.app>, CharlesChen0823 <yongchen0823@gmail.com>, Daniel Schaefer <dhs@frame.work>, Rene Leonhardt, ssrlive, Kan-Ru Chen <kanru@kanru.info>, Tony <legendmastertony@gmail.com> |
| equivalent@1.0.2 | Apache-2.0 OR MIT | https://github.com/indexmap-rs/equivalent |  |
| erased-serde@0.4.10 | MIT OR Apache-2.0 | https://github.com/dtolnay/erased-serde | David Tolnay <dtolnay@gmail.com> |
| errno@0.3.14 | MIT OR Apache-2.0 | https://github.com/lambda-fairy/rust-errno | Chris Wong <lambda.fairy@gmail.com>, Dan Gohman <dev@sunfishcode.online> |
| error-code@3.3.2 | BSL-1.0 | https://github.com/DoumanAsh/error-code | Douman <douman@gmx.se> |
| fallible-iterator@0.3.0 | MIT/Apache-2.0 | https://github.com/sfackler/rust-fallible-iterator | Steven Fackler <sfackler@gmail.com> |
| fallible-streaming-iterator@0.1.9 | MIT/Apache-2.0 | https://github.com/sfackler/fallible-streaming-iterator | Steven Fackler <sfackler@gmail.com> |
| fastrand@2.4.1 | Apache-2.0 OR MIT | https://github.com/smol-rs/fastrand | Stjepan Glavina <stjepang@gmail.com> |
| fax@0.2.7 | MIT | https://github.com/pdf-rs/fax | Sebastian K <s3bk@protonmail.com> |
| fdeflate@0.3.7 | MIT OR Apache-2.0 | https://github.com/image-rs/fdeflate | The image-rs Developers |
| field-offset@0.3.6 | MIT OR Apache-2.0 | https://github.com/Diggsey/rust-field-offset | Diggory Blake <diggsey@googlemail.com> |
| filetime@0.2.29 | MIT/Apache-2.0 | https://github.com/alexcrichton/filetime | Alex Crichton <alex@alexcrichton.com> |
| find-msvc-tools@0.1.9 | MIT OR Apache-2.0 | https://github.com/rust-lang/cc-rs |  |
| fixedbitset@0.5.7 | MIT OR Apache-2.0 | https://github.com/petgraph/fixedbitset | bluss |
| flate2@1.1.9 | MIT OR Apache-2.0 | https://github.com/rust-lang/flate2-rs | Alex Crichton <alex@alexcrichton.com>, Josh Triplett <josh@joshtriplett.org> |
| fnv@1.0.7 | Apache-2.0 / MIT | https://github.com/servo/rust-fnv | Alex Crichton <alex@alexcrichton.com> |
| foldhash@0.1.5 | Zlib | https://github.com/orlp/foldhash | Orson Peters <orsonpeters@gmail.com> |
| foldhash@0.2.0 | Zlib | https://github.com/orlp/foldhash | Orson Peters <orsonpeters@gmail.com> |
| foreign-types-macros@0.2.3 | MIT/Apache-2.0 | https://github.com/sfackler/foreign-types | Steven Fackler <sfackler@gmail.com> |
| foreign-types-shared@0.3.1 | MIT/Apache-2.0 | https://github.com/sfackler/foreign-types | Steven Fackler <sfackler@gmail.com> |
| foreign-types@0.5.0 | MIT/Apache-2.0 | https://github.com/sfackler/foreign-types | Steven Fackler <sfackler@gmail.com> |
| form_urlencoded@1.2.2 | MIT OR Apache-2.0 | https://github.com/servo/rust-url | The rust-url developers |
| futures-channel@0.3.32 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |  |
| futures-core@0.3.32 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |  |
| futures-executor@0.3.32 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |  |
| futures-io@0.3.32 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |  |
| futures-macro@0.3.32 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |  |
| futures-sink@0.3.32 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |  |
| futures-task@0.3.32 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |  |
| futures-util@0.3.32 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |  |
| gdk-pixbuf-sys@0.18.0 | MIT | https://github.com/gtk-rs/gtk-rs-core | The gtk-rs Project Developers |
| gdk-pixbuf@0.18.5 | MIT | https://github.com/gtk-rs/gtk-rs-core | The gtk-rs Project Developers |
| gdk-sys@0.18.2 | MIT | https://github.com/gtk-rs/gtk3-rs | The gtk-rs Project Developers |
| gdk@0.18.2 | MIT | https://github.com/gtk-rs/gtk3-rs | The gtk-rs Project Developers |
| gdkwayland-sys@0.18.2 | MIT | https://github.com/gtk-rs/gtk3-rs | The gtk-rs Project Developers |
| gdkx11-sys@0.18.2 | MIT | https://github.com/gtk-rs/gtk3-rs | The gtk-rs Project Developers |
| gdkx11@0.18.2 | MIT | https://github.com/gtk-rs/gtk3-rs | The gtk-rs Project Developers |
| generic-array@0.14.7 | MIT | https://github.com/fizyk20/generic-array.git | Bartłomiej Kamiński <fizyk20@gmail.com>, Aaron Trent <novacrazy@gmail.com> |
| gethostname@1.1.0 | Apache-2.0 | https://codeberg.org/swsnr/gethostname.rs.git | Sebastian Wiesner <sebastian@swsnr.de> |
| getrandom@0.2.17 | MIT OR Apache-2.0 | https://github.com/rust-random/getrandom | The Rand Project Developers |
| getrandom@0.3.4 | MIT OR Apache-2.0 | https://github.com/rust-random/getrandom | The Rand Project Developers |
| getrandom@0.4.3 | MIT OR Apache-2.0 | https://github.com/rust-random/getrandom | The Rand Project Developers |
| gio-sys@0.18.1 | MIT | https://github.com/gtk-rs/gtk-rs-core | The gtk-rs Project Developers |
| gio@0.18.4 | MIT | https://github.com/gtk-rs/gtk-rs-core | The gtk-rs Project Developers |
| glib-macros@0.18.5 | MIT | https://github.com/gtk-rs/gtk-rs-core | The gtk-rs Project Developers |
| glib-sys@0.18.1 | MIT | https://github.com/gtk-rs/gtk-rs-core | The gtk-rs Project Developers |
| glib@0.18.5 | MIT | https://github.com/gtk-rs/gtk-rs-core | The gtk-rs Project Developers |
| glob@0.3.3 | MIT OR Apache-2.0 | https://github.com/rust-lang/glob | The Rust Project Developers |
| gobject-sys@0.18.0 | MIT | https://github.com/gtk-rs/gtk-rs-core | The gtk-rs Project Developers |
| gtk-sys@0.18.2 | MIT | https://github.com/gtk-rs/gtk3-rs | The gtk-rs Project Developers |
| gtk@0.18.2 | MIT | https://github.com/gtk-rs/gtk3-rs | The gtk-rs Project Developers |
| gtk3-macros@0.18.2 | MIT | https://github.com/gtk-rs/gtk3-rs | The gtk-rs Project Developers |
| half@2.7.1 | MIT OR Apache-2.0 | https://github.com/VoidStarKat/half-rs | Kathryn Long <squeeself@gmail.com> |
| hashbrown@0.12.3 | MIT OR Apache-2.0 | https://github.com/rust-lang/hashbrown | Amanieu d'Antras <amanieu@gmail.com> |
| hashbrown@0.14.5 | MIT OR Apache-2.0 | https://github.com/rust-lang/hashbrown | Amanieu d'Antras <amanieu@gmail.com> |
| hashbrown@0.15.5 | MIT OR Apache-2.0 | https://github.com/rust-lang/hashbrown | Amanieu d'Antras <amanieu@gmail.com> |
| hashbrown@0.17.1 | MIT OR Apache-2.0 | https://github.com/rust-lang/hashbrown |  |
| hashlink@0.9.1 | MIT OR Apache-2.0 | https://github.com/kyren/hashlink | kyren <kerriganw@gmail.com> |
| heck@0.4.1 | MIT OR Apache-2.0 | https://github.com/withoutboats/heck | Without Boats <woboats@gmail.com> |
| heck@0.5.0 | MIT OR Apache-2.0 | https://github.com/withoutboats/heck |  |
| hex@0.4.3 | MIT OR Apache-2.0 | https://github.com/KokaKiwi/rust-hex | KokaKiwi <kokakiwi@kokakiwi.net> |
| html5ever@0.38.0 | MIT OR Apache-2.0 | https://github.com/servo/html5ever | The html5ever Project Developers |
| http-body-util@0.1.3 | MIT | https://github.com/hyperium/http-body | Carl Lerche <me@carllerche.com>, Lucio Franco <luciofranco14@gmail.com>, Sean McArthur <sean@seanmonstar.com> |
| http-body@1.0.1 | MIT | https://github.com/hyperium/http-body | Carl Lerche <me@carllerche.com>, Lucio Franco <luciofranco14@gmail.com>, Sean McArthur <sean@seanmonstar.com> |
| http@1.4.2 | MIT OR Apache-2.0 | https://github.com/hyperium/http | Alex Crichton <alex@alexcrichton.com>, Carl Lerche <me@carllerche.com>, Sean McArthur <sean@seanmonstar.com> |
| httparse@1.10.1 | MIT OR Apache-2.0 | https://github.com/seanmonstar/httparse | Sean McArthur <sean@seanmonstar.com> |
| hyper-rustls@0.27.9 | Apache-2.0 OR ISC OR MIT | https://github.com/rustls/hyper-rustls |  |
| hyper-util@0.1.20 | MIT | https://github.com/hyperium/hyper-util | Sean McArthur <sean@seanmonstar.com> |
| hyper@1.10.1 | MIT | https://github.com/hyperium/hyper | Sean McArthur <sean@seanmonstar.com> |
| iana-time-zone-haiku@0.1.2 | MIT OR Apache-2.0 | https://github.com/strawlab/iana-time-zone | René Kijewski <crates.io@k6i.de> |
| iana-time-zone@0.1.65 | MIT OR Apache-2.0 | https://github.com/strawlab/iana-time-zone | Andrew Straw <strawman@astraw.com>, René Kijewski <rene.kijewski@fu-berlin.de>, Ryan Lopopolo <rjl@hyperbo.la> |
| ico@0.5.0 | MIT | https://github.com/mdsteele/rust-ico | Matthew D. Steele <mdsteele@alum.mit.edu> |
| icu_collections@2.2.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x | The ICU4X Project Developers |
| icu_locale_core@2.2.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x | The ICU4X Project Developers |
| icu_normalizer_data@2.2.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x | The ICU4X Project Developers |
| icu_normalizer@2.2.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x | The ICU4X Project Developers |
| icu_properties_data@2.2.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x | The ICU4X Project Developers |
| icu_properties@2.2.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x | The ICU4X Project Developers |
| icu_provider@2.2.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x | The ICU4X Project Developers |
| ident_case@1.0.1 | MIT/Apache-2.0 | https://github.com/TedDriggs/ident_case | Ted Driggs <ted.driggs@outlook.com> |
| idna_adapter@1.2.2 | Apache-2.0 OR MIT | https://github.com/hsivonen/idna_adapter | The rust-url developers |
| idna@1.1.0 | MIT OR Apache-2.0 | https://github.com/servo/rust-url/ | The rust-url developers |
| image@0.25.10 | MIT OR Apache-2.0 | https://github.com/image-rs/image | The image-rs Developers |
| indexmap@1.9.3 | Apache-2.0 OR MIT | https://github.com/bluss/indexmap |  |
| indexmap@2.14.0 | Apache-2.0 OR MIT | https://github.com/indexmap-rs/indexmap |  |
| infer@0.19.0 | MIT | https://github.com/bojand/infer | Bojan <dbojan@gmail.com> |
| ipnet@2.12.0 | MIT OR Apache-2.0 | https://github.com/krisprice/ipnet | Kris Price <kris@krisprice.nz> |
| itoa@1.0.18 | MIT OR Apache-2.0 | https://github.com/dtolnay/itoa | David Tolnay <dtolnay@gmail.com> |
| javascriptcore-rs-sys@1.1.1 | MIT | https://github.com/tauri-apps/javascriptcore-rs | The Gtk-rs Project Developers |
| javascriptcore-rs@1.1.2 | MIT | https://github.com/tauri-apps/javascriptcore-rs |  |
| jni-macros@0.22.4 | MIT OR Apache-2.0 | https://github.com/jni-rs/jni-rs |  |
| jni-sys-macros@0.4.1 | MIT OR Apache-2.0 | https://github.com/jni-rs/jni-sys | Robert Bragg <robert@sixbynine.org> |
| jni-sys@0.3.1 | MIT OR Apache-2.0 | https://github.com/jni-rs/jni-sys | Steven Fackler <sfackler@gmail.com> |
| jni-sys@0.4.1 | MIT OR Apache-2.0 | https://github.com/jni-rs/jni-sys | Steven Fackler <sfackler@gmail.com>, Robert Bragg <robert@sixbynine.org> |
| jni@0.21.1 | MIT/Apache-2.0 | https://github.com/jni-rs/jni-rs | Josh Chase <josh@prevoty.com> |
| jni@0.22.4 | MIT OR Apache-2.0 | https://github.com/jni-rs/jni-rs | jni team |
| js-sys@0.3.102 | MIT OR Apache-2.0 | https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/js-sys | The wasm-bindgen Developers |
| json-patch@3.0.1 | MIT/Apache-2.0 | https://github.com/idubrov/json-patch | Ivan Dubrov <dubrov.ivan@gmail.com> |
| jsonptr@0.6.3 | MIT OR Apache-2.0 | https://github.com/chanced/jsonptr | chance dinkins, André Sá de Mello <codasm@pm.me> |
| keyboard-types@0.7.0 | MIT OR Apache-2.0 | https://github.com/pyfisch/keyboard-types | Pyfisch <pyfisch@posteo.org> |
| libappindicator-sys@0.9.0 | Apache-2.0 OR MIT | registry+https://github.com/rust-lang/crates.io-index |  |
| libappindicator@0.9.0 | Apache-2.0 OR MIT | registry+https://github.com/rust-lang/crates.io-index |  |
| libc@0.2.186 | MIT OR Apache-2.0 | https://github.com/rust-lang/libc | The Rust Project Developers |
| libdbus-sys@0.2.7 | Apache-2.0/MIT | https://github.com/diwic/dbus-rs | David Henningsson <diwic@ubuntu.com> |
| libloading@0.7.4 | ISC | https://github.com/nagisa/rust_libloading/ | Simonas Kazlauskas <libloading@kazlauskas.me> |
| libredox@0.1.17 | MIT | https://gitlab.redox-os.org/redox-os/libredox.git | 4lDO2 <4lDO2@protonmail.com> |
| libsqlite3-sys@0.28.0 | MIT | https://github.com/rusqlite/rusqlite | The rusqlite developers |
| linux-raw-sys@0.12.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | https://github.com/sunfishcode/linux-raw-sys | Dan Gohman <dev@sunfishcode.online> |
| litemap@0.8.2 | Unicode-3.0 | https://github.com/unicode-org/icu4x | The ICU4X Project Developers |
| lock_api@0.4.14 | MIT OR Apache-2.0 | https://github.com/Amanieu/parking_lot | Amanieu d'Antras <amanieu@gmail.com> |
| log@0.4.33 | MIT OR Apache-2.0 | https://github.com/rust-lang/log | The Rust Project Developers |
| lru-slab@0.1.2 | MIT OR Apache-2.0 OR Zlib | https://github.com/Ralith/lru-slab | Benjamin Saunders <ben.e.saunders@gmail.com> |
| markup5ever@0.38.0 | MIT OR Apache-2.0 | https://github.com/servo/html5ever | The html5ever Project Developers |
| memchr@2.8.2 | Unlicense OR MIT | https://github.com/BurntSushi/memchr | Andrew Gallant <jamslam@gmail.com>, bluss |
| memoffset@0.9.1 | MIT | https://github.com/Gilnaa/memoffset | Gilad Naaman <gilad.naaman@gmail.com> |
| mime@0.3.17 | MIT OR Apache-2.0 | https://github.com/hyperium/mime | Sean McArthur <sean@seanmonstar.com> |
| minisign-verify@0.2.5 | MIT | https://github.com/jedisct1/rust-minisign-verify | Frank Denis <github@pureftpd.org> |
| miniz_oxide@0.8.9 | MIT OR Zlib OR Apache-2.0 | https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide | Frommi <daniil.liferenko@gmail.com>, oyvindln <oyvindln@users.noreply.github.com>, Rich Geldreich richgel99@gmail.com |
| mio@1.2.1 | MIT | https://github.com/tokio-rs/mio | Carl Lerche <me@carllerche.com>, Thomas de Zeeuw <thomasdezeeuw@gmail.com>, Tokio Contributors <team@tokio.rs> |
| moxcms@0.8.1 | BSD-3-Clause OR Apache-2.0 | https://github.com/awxkee/moxcms.git | Radzivon Bartoshyk |
| muda@0.19.3 | Apache-2.0 OR MIT | https://github.com/tauri-apps/muda |  |
| ndk-sys@0.6.0+11769913 | MIT OR Apache-2.0 | https://github.com/rust-mobile/ndk | The Rust Windowing contributors |
| ndk@0.9.0 | MIT OR Apache-2.0 | https://github.com/rust-mobile/ndk | The Rust Mobile contributors |
| new_debug_unreachable@1.0.6 | MIT | https://github.com/mbrubeck/rust-debug-unreachable | Matt Brubeck <mbrubeck@limpet.net>, Jonathan Reem <jonathan.reem@gmail.com> |
| nom@8.0.0 | MIT | https://github.com/rust-bakery/nom | contact@geoffroycouprie.com |
| num_enum_derive@0.7.6 | BSD-3-Clause OR MIT OR Apache-2.0 | https://github.com/illicitonion/num_enum | Daniel Wagner-Hall <dawagner@gmail.com>, Daniel Henry-Mantilla <daniel.henry.mantilla@gmail.com>, Vincent Esche <regexident@gmail.com> |
| num_enum@0.7.6 | BSD-3-Clause OR MIT OR Apache-2.0 | https://github.com/illicitonion/num_enum | Daniel Wagner-Hall <dawagner@gmail.com>, Daniel Henry-Mantilla <daniel.henry.mantilla@gmail.com>, Vincent Esche <regexident@gmail.com> |
| num-conv@0.2.2 | MIT OR Apache-2.0 | https://github.com/jhpratt/num-conv | Jacob Pratt <jacob@jhpratt.dev> |
| num-traits@0.2.19 | MIT OR Apache-2.0 | https://github.com/rust-num/num-traits | The Rust Project Developers |
| objc2-app-kit@0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |  |
| objc2-cloud-kit@0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |  |
| objc2-core-data@0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |  |
| objc2-core-foundation@0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |  |
| objc2-core-graphics@0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |  |
| objc2-core-image@0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |  |
| objc2-core-location@0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |  |
| objc2-core-text@0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |  |
| objc2-encode@4.1.0 | MIT | https://github.com/madsmtm/objc2 | Mads Marquart <mads@marquart.dk> |
| objc2-exception-helper@0.1.1 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 | Mads Marquart <mads@marquart.dk> |
| objc2-foundation@0.3.2 | MIT | https://github.com/madsmtm/objc2 |  |
| objc2-io-surface@0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |  |
| objc2-osa-kit@0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |  |
| objc2-quartz-core@0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |  |
| objc2-ui-kit@0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |  |
| objc2-user-notifications@0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |  |
| objc2-web-kit@0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |  |
| objc2@0.6.4 | MIT | https://github.com/madsmtm/objc2 | Mads Marquart <mads@marquart.dk> |
| once_cell@1.21.4 | MIT OR Apache-2.0 | https://github.com/matklad/once_cell | Aleksey Kladov <aleksey.kladov@gmail.com> |
| openssl-probe@0.2.1 | MIT OR Apache-2.0 | https://github.com/rustls/openssl-probe | Alex Crichton <alex@alexcrichton.com> |
| option-ext@0.2.0 | MPL-2.0 | https://github.com/soc/option-ext.git | Simon Ochsenreither <simon@ochsenreither.de> |
| os_pipe@1.2.3 | MIT | https://github.com/oconnor663/os_pipe.rs | Jack O'Connor |
| osakit@0.3.1 | MIT OR Apache-2.0 | https://github.com/mdevils/rust-osakit | Marat Dulin <mdevils@gmail.com> |
| pango-sys@0.18.0 | MIT | https://github.com/gtk-rs/gtk-rs-core | The gtk-rs Project Developers |
| pango@0.18.3 | MIT | https://github.com/gtk-rs/gtk-rs-core | The gtk-rs Project Developers |
| parking_lot_core@0.9.12 | MIT OR Apache-2.0 | https://github.com/Amanieu/parking_lot | Amanieu d'Antras <amanieu@gmail.com> |
| parking_lot@0.12.5 | MIT OR Apache-2.0 | https://github.com/Amanieu/parking_lot | Amanieu d'Antras <amanieu@gmail.com> |
| percent-encoding@2.3.2 | MIT OR Apache-2.0 | https://github.com/servo/rust-url/ | The rust-url developers |
| petgraph@0.8.3 | MIT OR Apache-2.0 | https://github.com/petgraph/petgraph | bluss, mitchmindtree |
| phf_codegen@0.13.1 | MIT | https://github.com/rust-phf/rust-phf | Steven Fackler <sfackler@gmail.com> |
| phf_generator@0.13.1 | MIT | https://github.com/rust-phf/rust-phf | Steven Fackler <sfackler@gmail.com> |
| phf_macros@0.13.1 | MIT | https://github.com/rust-phf/rust-phf | Steven Fackler <sfackler@gmail.com> |
| phf_shared@0.13.1 | MIT | https://github.com/rust-phf/rust-phf | Steven Fackler <sfackler@gmail.com> |
| phf@0.13.1 | MIT | https://github.com/rust-phf/rust-phf | Steven Fackler <sfackler@gmail.com> |
| pin-project-lite@0.2.17 | Apache-2.0 OR MIT | https://github.com/taiki-e/pin-project-lite |  |
| pkg-config@0.3.33 | MIT OR Apache-2.0 | https://github.com/rust-lang/pkg-config-rs | Alex Crichton <alex@alexcrichton.com> |
| plist@1.9.0 | MIT | https://github.com/ebarnard/rust-plist/ | Ed Barnard <eabarnard@gmail.com> |
| png@0.17.16 | MIT OR Apache-2.0 | https://github.com/image-rs/image-png | The image-rs Developers |
| png@0.18.1 | MIT OR Apache-2.0 | https://github.com/image-rs/image-png | The image-rs Developers |
| potential_utf@0.1.5 | Unicode-3.0 | https://github.com/unicode-org/icu4x | The ICU4X Project Developers |
| powerfmt@0.2.0 | MIT OR Apache-2.0 | https://github.com/jhpratt/powerfmt | Jacob Pratt <jacob@jhpratt.dev> |
| precomputed-hash@0.1.1 | MIT | https://github.com/emilio/precomputed-hash | Emilio Cobos Álvarez <emilio@crisal.io> |
| proc-macro-crate@1.3.1 | MIT OR Apache-2.0 | https://github.com/bkchr/proc-macro-crate | Bastian Köcher <git@kchr.de> |
| proc-macro-crate@2.0.0 | MIT OR Apache-2.0 | https://github.com/bkchr/proc-macro-crate | Bastian Köcher <git@kchr.de> |
| proc-macro-crate@3.5.0 | MIT OR Apache-2.0 | https://github.com/bkchr/proc-macro-crate | Bastian Köcher <git@kchr.de> |
| proc-macro-error-attr@1.0.4 | MIT OR Apache-2.0 | https://gitlab.com/CreepySkeleton/proc-macro-error | CreepySkeleton <creepy-skeleton@yandex.ru> |
| proc-macro-error@1.0.4 | MIT OR Apache-2.0 | https://gitlab.com/CreepySkeleton/proc-macro-error | CreepySkeleton <creepy-skeleton@yandex.ru> |
| proc-macro2@1.0.106 | MIT OR Apache-2.0 | https://github.com/dtolnay/proc-macro2 | David Tolnay <dtolnay@gmail.com>, Alex Crichton <alex@alexcrichton.com> |
| pxfm@0.1.30 | BSD-3-Clause OR Apache-2.0 | https://github.com/awxkee/pxfm | Radzivon Bartoshyk |
| quick-error@2.0.1 | MIT/Apache-2.0 | http://github.com/tailhook/quick-error | Paul Colomiets <paul@colomiets.name>, Colin Kiegel <kiegel@gmx.de> |
| quick-xml@0.39.4 | MIT | https://github.com/tafia/quick-xml |  |
| quick-xml@0.41.0 | MIT | https://github.com/tafia/quick-xml |  |
| quinn-proto@0.11.16 | MIT OR Apache-2.0 | https://github.com/quinn-rs/quinn |  |
| quinn-udp@0.5.15 | MIT OR Apache-2.0 | https://github.com/quinn-rs/quinn |  |
| quinn@0.11.11 | MIT OR Apache-2.0 | https://github.com/quinn-rs/quinn |  |
| quote@1.0.45 | MIT OR Apache-2.0 | https://github.com/dtolnay/quote | David Tolnay <dtolnay@gmail.com> |
| r-efi@5.3.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later | https://github.com/r-efi/r-efi |  |
| r-efi@6.0.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later | https://github.com/r-efi/r-efi |  |
| rand_core@0.10.1 | MIT OR Apache-2.0 | https://github.com/rust-random/rand_core | The Rand Project Developers |
| rand_pcg@0.10.2 | MIT OR Apache-2.0 | https://github.com/rust-random/rngs | The Rand Project Developers |
| rand@0.10.2 | MIT OR Apache-2.0 | https://github.com/rust-random/rand | The Rand Project Developers, The Rust Project Developers |
| raw-window-handle@0.6.2 | MIT OR Apache-2.0 OR Zlib | https://github.com/rust-windowing/raw-window-handle | Osspial <osspial@gmail.com> |
| redox_syscall@0.5.18 | MIT | https://gitlab.redox-os.org/redox-os/syscall | Jeremy Soller <jackpot51@gmail.com> |
| redox_users@0.5.2 | MIT | https://gitlab.redox-os.org/redox-os/users | Jose Narvaez <goyox86@gmail.com>, Wesley Hershberger <mggmugginsmc@gmail.com> |
| ref-cast-impl@1.0.25 | MIT OR Apache-2.0 | https://github.com/dtolnay/ref-cast | David Tolnay <dtolnay@gmail.com> |
| ref-cast@1.0.25 | MIT OR Apache-2.0 | https://github.com/dtolnay/ref-cast | David Tolnay <dtolnay@gmail.com> |
| refinery-core@0.8.16 | MIT OR Apache-2.0 | https://github.com/rust-db/refinery | Katharina Fey <kookie@spacekookie.de>, João Oliveira <hello@jxs.pt> |
| refinery-macros@0.8.16 | MIT OR Apache-2.0 | https://github.com/rust-db/refinery | Katharina Fey <kookie@spacekookie.de>, João Oliveira <hello@jxs.pt> |
| refinery@0.8.16 | MIT | https://github.com/rust-db/refinery | Katharina Fey <kookie@spacekookie.de>, João Oliveira <hello@jxs.pt> |
| regex-automata@0.4.14 | MIT OR Apache-2.0 | https://github.com/rust-lang/regex | The Rust Project Developers, Andrew Gallant <jamslam@gmail.com> |
| regex-syntax@0.8.11 | MIT OR Apache-2.0 | https://github.com/rust-lang/regex | The Rust Project Developers, Andrew Gallant <jamslam@gmail.com> |
| regex@1.12.4 | MIT OR Apache-2.0 | https://github.com/rust-lang/regex | The Rust Project Developers, Andrew Gallant <jamslam@gmail.com> |
| reqwest@0.12.28 | MIT OR Apache-2.0 | https://github.com/seanmonstar/reqwest | Sean McArthur <sean@seanmonstar.com> |
| reqwest@0.13.4 | MIT OR Apache-2.0 | https://github.com/seanmonstar/reqwest | Sean McArthur <sean@seanmonstar.com> |
| rfd@0.16.0 | MIT | https://github.com/PolyMeilex/rfd | Poly <marynczak.bartlomiej@gmail.com> |
| ring@0.17.14 | Apache-2.0 AND ISC | https://github.com/briansmith/ring |  |
| rusqlite@0.31.0 | MIT | https://github.com/rusqlite/rusqlite | The rusqlite developers |
| rustc_version@0.4.1 | MIT OR Apache-2.0 | https://github.com/djc/rustc-version-rs |  |
| rustc-hash@2.1.2 | Apache-2.0 OR MIT | https://github.com/rust-lang/rustc-hash | The Rust Project Developers |
| rustix@1.1.4 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | https://github.com/bytecodealliance/rustix | Dan Gohman <dev@sunfishcode.online>, Jakub Konka <kubkon@jakubkonka.com> |
| rustls-native-certs@0.8.4 | Apache-2.0 OR ISC OR MIT | https://github.com/rustls/rustls-native-certs |  |
| rustls-pki-types@1.15.0 | MIT OR Apache-2.0 | https://github.com/rustls/pki-types |  |
| rustls-platform-verifier-android@0.1.1 | MIT OR Apache-2.0 | https://github.com/rustls/rustls-platform-verifier |  |
| rustls-platform-verifier@0.7.0 | MIT OR Apache-2.0 | https://github.com/rustls/rustls-platform-verifier |  |
| rustls-webpki@0.103.13 | ISC | https://github.com/rustls/webpki |  |
| rustls@0.23.42 | Apache-2.0 OR ISC OR MIT | https://github.com/rustls/rustls |  |
| rustversion@1.0.22 | MIT OR Apache-2.0 | https://github.com/dtolnay/rustversion | David Tolnay <dtolnay@gmail.com> |
| ryu@1.0.23 | Apache-2.0 OR BSL-1.0 | https://github.com/dtolnay/ryu | David Tolnay <dtolnay@gmail.com> |
| same-file@1.0.6 | Unlicense/MIT | https://github.com/BurntSushi/same-file | Andrew Gallant <jamslam@gmail.com> |
| schannel@0.1.29 | MIT | https://github.com/steffengy/schannel-rs | Steven Fackler <sfackler@gmail.com>, Steffen Butzer <steffen.butzer@outlook.com> |
| schemars_derive@0.8.22 | MIT | https://github.com/GREsau/schemars | Graham Esau <gesau@hotmail.co.uk> |
| schemars@0.8.22 | MIT | https://github.com/GREsau/schemars | Graham Esau <gesau@hotmail.co.uk> |
| schemars@0.9.0 | MIT | https://github.com/GREsau/schemars | Graham Esau <gesau@hotmail.co.uk> |
| schemars@1.2.1 | MIT | https://github.com/GREsau/schemars | Graham Esau <gesau@hotmail.co.uk> |
| scopeguard@1.2.0 | MIT OR Apache-2.0 | https://github.com/bluss/scopeguard | bluss |
| security-framework-sys@2.17.0 | MIT OR Apache-2.0 | https://github.com/kornelski/rust-security-framework | Steven Fackler <sfackler@gmail.com>, Kornel <kornel@geekhood.net> |
| security-framework@3.7.0 | MIT OR Apache-2.0 | https://github.com/kornelski/rust-security-framework | Steven Fackler <sfackler@gmail.com>, Kornel <kornel@geekhood.net> |
| selectors@0.36.1 | MPL-2.0 | https://github.com/servo/stylo | The Servo Project Developers |
| semver@1.0.28 | MIT OR Apache-2.0 | https://github.com/dtolnay/semver | David Tolnay <dtolnay@gmail.com> |
| serde_core@1.0.228 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde | Erick Tryzelaar <erick.tryzelaar@gmail.com>, David Tolnay <dtolnay@gmail.com> |
| serde_derive_internals@0.29.1 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde | Erick Tryzelaar <erick.tryzelaar@gmail.com>, David Tolnay <dtolnay@gmail.com> |
| serde_derive@1.0.228 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde | Erick Tryzelaar <erick.tryzelaar@gmail.com>, David Tolnay <dtolnay@gmail.com> |
| serde_json@1.0.150 | MIT OR Apache-2.0 | https://github.com/serde-rs/json | Erick Tryzelaar <erick.tryzelaar@gmail.com>, David Tolnay <dtolnay@gmail.com> |
| serde_repr@0.1.20 | MIT OR Apache-2.0 | https://github.com/dtolnay/serde-repr | David Tolnay <dtolnay@gmail.com> |
| serde_spanned@0.6.9 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |  |
| serde_spanned@1.1.1 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |  |
| serde_urlencoded@0.7.1 | MIT/Apache-2.0 | https://github.com/nox/serde_urlencoded | Anthony Ramine <n.oxyde@gmail.com> |
| serde_with_macros@3.21.0 | MIT OR Apache-2.0 | https://github.com/jonasbb/serde_with/ | Jonas Bushart |
| serde_with@3.21.0 | MIT OR Apache-2.0 | https://github.com/jonasbb/serde_with/ | Jonas Bushart, Marcin Kaźmierczak |
| serde-untagged@0.1.9 | MIT OR Apache-2.0 | https://github.com/dtolnay/serde-untagged | David Tolnay <dtolnay@gmail.com> |
| serde@1.0.228 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde | Erick Tryzelaar <erick.tryzelaar@gmail.com>, David Tolnay <dtolnay@gmail.com> |
| serialize-to-javascript-impl@0.1.2 | MIT OR Apache-2.0 | https://github.com/chippers/serialize-to-javascript | Chip Reed <chip@chip.sh> |
| serialize-to-javascript@0.1.2 | MIT OR Apache-2.0 | https://github.com/chippers/serialize-to-javascript | Chip Reed <chip@chip.sh> |
| servo_arc@0.4.3 | MIT OR Apache-2.0 | https://github.com/servo/stylo | The Servo Project Developers |
| sha2@0.10.9 | MIT OR Apache-2.0 | https://github.com/RustCrypto/hashes | RustCrypto Developers |
| shlex@2.0.1 | MIT OR Apache-2.0 | https://github.com/comex/rust-shlex | comex <comexk@gmail.com>, Fenhl <fenhl@fenhl.net>, Adrian Taylor <adetaylor@chromium.org>, Alex Touchet <alextouchet@outlook.com>, Daniel Parks <dp+git@oxidized.org>, Garrett Berg <googberg@gmail.com> |
| simd_cesu8@1.2.0 | Apache-2.0 OR MIT | https://github.com/seancroach/simd_cesu8 | Sean C. Roach <me@seancroach.dev> |
| simd-adler32@0.3.9 | MIT | https://github.com/mcountryman/simd-adler32 | Marvin Countryman <me@maar.vin> |
| simdutf8@0.1.5 | MIT OR Apache-2.0 | https://github.com/rusticstuff/simdutf8 | Hans Kratz <hans@appfour.com> |
| siphasher@1.0.3 | MIT/Apache-2.0 | https://github.com/jedisct1/rust-siphash | Frank Denis <github@pureftpd.org> |
| slab@0.4.12 | MIT | https://github.com/tokio-rs/slab | Carl Lerche <me@carllerche.com> |
| smallvec@1.15.2 | MIT OR Apache-2.0 | https://github.com/servo/rust-smallvec | The Servo Project Developers |
| socket2@0.6.4 | MIT OR Apache-2.0 | https://github.com/rust-lang/socket2 | Alex Crichton <alex@alexcrichton.com>, Thomas de Zeeuw <thomasdezeeuw@gmail.com> |
| softbuffer@0.4.8 | MIT OR Apache-2.0 | https://github.com/rust-windowing/softbuffer |  |
| soup3-sys@0.5.0 | MIT | https://gitlab.gnome.org/World/Rust/soup3-rs | The Gtk-rs Project Developers |
| soup3@0.5.0 | MIT | https://gitlab.gnome.org/World/Rust/soup3-rs |  |
| stable_deref_trait@1.2.1 | MIT OR Apache-2.0 | https://github.com/storyyeller/stable_deref_trait | Robert Grosse <n210241048576@gmail.com> |
| string_cache_codegen@0.6.1 | MIT OR Apache-2.0 | https://github.com/servo/string-cache | The Servo Project Developers |
| string_cache@0.9.0 | MIT OR Apache-2.0 | https://github.com/servo/string-cache | The Servo Project Developers |
| strsim@0.11.1 | MIT | https://github.com/rapidfuzz/strsim-rs | Danny Guo <danny@dannyguo.com>, maxbachmann <oss@maxbachmann.de> |
| subtle@2.6.1 | BSD-3-Clause | https://github.com/dalek-cryptography/subtle | Isis Lovecruft <isis@patternsinthevoid.net>, Henry de Valence <hdevalence@hdevalence.ca> |
| swift-rs@1.0.7 | MIT OR Apache-2.0 | https://github.com/Brendonovich/swift-rs | The swift-rs contributors |
| syn@1.0.109 | MIT OR Apache-2.0 | https://github.com/dtolnay/syn | David Tolnay <dtolnay@gmail.com> |
| syn@2.0.118 | MIT OR Apache-2.0 | https://github.com/dtolnay/syn | David Tolnay <dtolnay@gmail.com> |
| sync_wrapper@1.0.2 | Apache-2.0 | https://github.com/Actyx/sync_wrapper | Actyx AG <developer@actyx.io> |
| synstructure@0.13.2 | MIT | https://github.com/mystor/synstructure | Nika Layzell <nika@thelayzells.com> |
| system-deps@6.2.2 | MIT OR Apache-2.0 | https://github.com/gdesmott/system-deps | Guillaume Desmottes <guillaume.desmottes@collabora.com>, Josh Triplett <josh@joshtriplett.org> |
| tao-macros@0.1.3 | MIT OR Apache-2.0 | https://github.com/tauri-apps/tao | Tauri Programme within The Commons Conservancy |
| tao@0.35.3 | Apache-2.0 | https://github.com/tauri-apps/tao | Tauri Programme within The Commons Conservancy, The winit contributors |
| tar@0.4.46 | MIT OR Apache-2.0 | https://github.com/composefs/tar-rs | Alex Crichton <alex@alexcrichton.com> |
| target-lexicon@0.12.16 | Apache-2.0 WITH LLVM-exception | https://github.com/bytecodealliance/target-lexicon | Dan Gohman <sunfish@mozilla.com> |
| tauri-build@2.6.3 | Apache-2.0 OR MIT | https://github.com/tauri-apps/tauri | Tauri Programme within The Commons Conservancy |
| tauri-codegen@2.6.3 | Apache-2.0 OR MIT | https://github.com/tauri-apps/tauri | Tauri Programme within The Commons Conservancy |
| tauri-macros@2.6.3 | Apache-2.0 OR MIT | https://github.com/tauri-apps/tauri | Tauri Programme within The Commons Conservancy |
| tauri-plugin-clipboard-manager@2.3.2 | Apache-2.0 OR MIT | https://github.com/tauri-apps/plugins-workspace | Tauri Programme within The Commons Conservancy |
| tauri-plugin-dialog@2.7.2 | Apache-2.0 OR MIT | https://github.com/tauri-apps/plugins-workspace | Tauri Programme within The Commons Conservancy |
| tauri-plugin-fs@2.5.1 | Apache-2.0 OR MIT | https://github.com/tauri-apps/plugins-workspace | Tauri Programme within The Commons Conservancy |
| tauri-plugin-updater@2.10.1 | Apache-2.0 OR MIT | https://github.com/tauri-apps/plugins-workspace | Tauri Programme within The Commons Conservancy |
| tauri-plugin@2.6.3 | Apache-2.0 OR MIT | https://github.com/tauri-apps/tauri | Tauri Programme within The Commons Conservancy |
| tauri-runtime-wry@2.11.3 | Apache-2.0 OR MIT | https://github.com/tauri-apps/tauri | Tauri Programme within The Commons Conservancy |
| tauri-runtime@2.11.3 | Apache-2.0 OR MIT | https://github.com/tauri-apps/tauri | Tauri Programme within The Commons Conservancy |
| tauri-utils@2.9.3 | Apache-2.0 OR MIT | https://github.com/tauri-apps/tauri | Tauri Programme within The Commons Conservancy |
| tauri-winres@0.3.6 | MIT | https://github.com/tauri-apps/winres | Tauri Programme within The Commons Conservancy, Max Resch <resch.max@gmail.com> |
| tauri@2.11.3 | Apache-2.0 OR MIT | https://github.com/tauri-apps/tauri | Tauri Programme within The Commons Conservancy |
| tempfile@3.27.0 | MIT OR Apache-2.0 | https://github.com/Stebalien/tempfile | Steven Allen <steven@stebalien.com>, The Rust Project Developers, Ashley Mannix <ashleymannix@live.com.au>, Jason White <me@jasonwhite.io> |
| tendril@0.5.0 | MIT OR Apache-2.0 | https://github.com/servo/html5ever | Keegan McAllister <mcallister.keegan@gmail.com>, Simon Sapin <simon.sapin@exyr.org>, Chris Morgan <me@chrismorgan.info> |
| thiserror-impl@1.0.69 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror | David Tolnay <dtolnay@gmail.com> |
| thiserror-impl@2.0.18 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror | David Tolnay <dtolnay@gmail.com> |
| thiserror@1.0.69 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror | David Tolnay <dtolnay@gmail.com> |
| thiserror@2.0.18 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror | David Tolnay <dtolnay@gmail.com> |
| tiff@0.11.3 | MIT | https://github.com/image-rs/image-tiff | The image-rs Developers |
| time-core@0.1.9 | MIT OR Apache-2.0 | https://github.com/time-rs/time | Jacob Pratt <open-source@jhpratt.dev>, Time contributors |
| time-macros@0.2.29 | MIT OR Apache-2.0 | https://github.com/time-rs/time | Jacob Pratt <open-source@jhpratt.dev>, Time contributors |
| time@0.3.49 | MIT OR Apache-2.0 | https://github.com/time-rs/time | Jacob Pratt <open-source@jhpratt.dev>, Time contributors |
| tinystr@0.8.3 | Unicode-3.0 | https://github.com/unicode-org/icu4x | The ICU4X Project Developers |
| tinyvec_macros@0.1.1 | MIT OR Apache-2.0 OR Zlib | https://github.com/Soveu/tinyvec_macros | Soveu <marx.tomasz@gmail.com> |
| tinyvec@1.11.0 | Zlib OR Apache-2.0 OR MIT | https://github.com/Lokathor/tinyvec | Lokathor <zefria@gmail.com> |
| tokio-rustls@0.26.4 | MIT OR Apache-2.0 | https://github.com/rustls/tokio-rustls |  |
| tokio-util@0.7.18 | MIT | https://github.com/tokio-rs/tokio | Tokio Contributors <team@tokio.rs> |
| tokio@1.52.3 | MIT | https://github.com/tokio-rs/tokio | Tokio Contributors <team@tokio.rs> |
| toml_datetime@0.6.11 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |  |
| toml_datetime@0.7.5+spec-1.1.0 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |  |
| toml_datetime@1.1.1+spec-1.1.0 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |  |
| toml_edit@0.19.15 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml | Andronik Ordian <write@reusable.software>, Ed Page <eopage@gmail.com> |
| toml_edit@0.20.7 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml | Andronik Ordian <write@reusable.software>, Ed Page <eopage@gmail.com> |
| toml_edit@0.22.27 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |  |
| toml_edit@0.25.12+spec-1.1.0 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |  |
| toml_parser@1.1.2+spec-1.1.0 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |  |
| toml_write@0.1.2 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |  |
| toml_writer@1.1.1+spec-1.1.0 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |  |
| toml@0.8.23 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |  |
| toml@0.9.12+spec-1.1.0 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |  |
| toml@1.1.2+spec-1.1.0 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |  |
| tower-http@0.6.11 | MIT | https://github.com/tower-rs/tower-http | Tower Maintainers <team@tower-rs.com> |
| tower-layer@0.3.3 | MIT | https://github.com/tower-rs/tower | Tower Maintainers <team@tower-rs.com> |
| tower-service@0.3.3 | MIT | https://github.com/tower-rs/tower | Tower Maintainers <team@tower-rs.com> |
| tower@0.5.3 | MIT | https://github.com/tower-rs/tower | Tower Maintainers <team@tower-rs.com> |
| tracing-core@0.1.36 | MIT | https://github.com/tokio-rs/tracing | Tokio Contributors <team@tokio.rs> |
| tracing@0.1.44 | MIT | https://github.com/tokio-rs/tracing | Eliza Weisman <eliza@buoyant.io>, Tokio Contributors <team@tokio.rs> |
| tray-icon@0.24.1 | MIT OR Apache-2.0 | https://github.com/tauri-apps/tray-icon |  |
| tree_magic_mini@3.2.2 | MIT | https://github.com/mbrubeck/tree_magic/ | Matt Brubeck <mbrubeck@limpet.net>, Allison Hancock <aahancoc@umich.edu> |
| try-lock@0.2.5 | MIT | https://github.com/seanmonstar/try-lock | Sean McArthur <sean@seanmonstar.com> |
| typeid@1.0.3 | MIT OR Apache-2.0 | https://github.com/dtolnay/typeid | David Tolnay <dtolnay@gmail.com> |
| typenum@1.20.1 | MIT OR Apache-2.0 | https://github.com/paholg/typenum |  |
| unic-char-property@0.9.0 | MIT/Apache-2.0 | https://github.com/open-i18n/rust-unic/ | The UNIC Project Developers |
| unic-char-range@0.9.0 | MIT/Apache-2.0 | https://github.com/open-i18n/rust-unic/ | The UNIC Project Developers |
| unic-common@0.9.0 | MIT/Apache-2.0 | https://github.com/open-i18n/rust-unic/ | The UNIC Project Developers |
| unic-ucd-ident@0.9.0 | MIT/Apache-2.0 | https://github.com/open-i18n/rust-unic/ | The UNIC Project Developers |
| unic-ucd-version@0.9.0 | MIT/Apache-2.0 | https://github.com/open-i18n/rust-unic/ | The UNIC Project Developers |
| unicode-ident@1.0.24 | (MIT OR Apache-2.0) AND Unicode-3.0 | https://github.com/dtolnay/unicode-ident | David Tolnay <dtolnay@gmail.com> |
| unicode-segmentation@1.13.3 | MIT OR Apache-2.0 | https://github.com/unicode-rs/unicode-segmentation | kwantam <kwantam@gmail.com>, Manish Goregaokar <manishsmail@gmail.com> |
| untrusted@0.9.0 | ISC | https://github.com/briansmith/untrusted | Brian Smith <brian@briansmith.org> |
| url@2.5.8 | MIT OR Apache-2.0 | https://github.com/servo/rust-url | The rust-url developers |
| urlpattern@0.3.0 | MIT | https://github.com/denoland/rust-urlpattern | the Deno authors, crowlKats <crowlkats@toaxl.com> |
| utf-8@0.7.6 | MIT OR Apache-2.0 | https://github.com/SimonSapin/rust-utf8 | Simon Sapin <simon.sapin@exyr.org> |
| utf8_iter@1.0.4 | Apache-2.0 OR MIT | https://github.com/hsivonen/utf8_iter | Henri Sivonen <hsivonen@hsivonen.fi> |
| uuid@1.23.3 | Apache-2.0 OR MIT | https://github.com/uuid-rs/uuid | Ashley Mannix<ashleymannix@live.com.au>, Dylan DPC<dylan.dpc@gmail.com>, Hunar Roop Kahlon<hunar.roop@gmail.com> |
| vcpkg@0.2.15 | MIT/Apache-2.0 | https://github.com/mcgoo/vcpkg-rs | Jim McGrath <jimmc2@gmail.com> |
| version_check@0.9.5 | MIT/Apache-2.0 | https://github.com/SergioBenitez/version_check | Sergio Benitez <sb@sergio.bz> |
| version-compare@0.2.1 | MIT | https://gitlab.com/timvisee/version-compare | Tim Visee <3a4fb3964f@sinenomine.email> |
| vswhom-sys@0.1.3 | MIT | https://github.com/nabijaczleweli/vswhom-sys.rs | наб <nabijaczleweli@nabijaczleweli.xyz>, forrestsmithfb <forrest.smith@fb.com> |
| vswhom@0.1.0 | MIT | https://github.com/nabijaczleweli/vswhom.rs | nabijaczleweli <nabijaczleweli@gmail.com> |
| walkdir@2.5.0 | Unlicense/MIT | https://github.com/BurntSushi/walkdir | Andrew Gallant <jamslam@gmail.com> |
| want@0.3.1 | MIT | https://github.com/seanmonstar/want | Sean McArthur <sean@seanmonstar.com> |
| wasi@0.11.1+wasi-snapshot-preview1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | https://github.com/bytecodealliance/wasi | The Cranelift Project Developers |
| wasip2@1.0.4+wasi-0.2.12 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | https://github.com/bytecodealliance/wasi-rs |  |
| wasm-bindgen-futures@0.4.75 | MIT OR Apache-2.0 | https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/futures | The wasm-bindgen Developers |
| wasm-bindgen-macro-support@0.2.125 | MIT OR Apache-2.0 | https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro-support | The wasm-bindgen Developers |
| wasm-bindgen-macro@0.2.125 | MIT OR Apache-2.0 | https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro | The wasm-bindgen Developers |
| wasm-bindgen-shared@0.2.125 | MIT OR Apache-2.0 | https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/shared | The wasm-bindgen Developers |
| wasm-bindgen@0.2.125 | MIT OR Apache-2.0 | https://github.com/wasm-bindgen/wasm-bindgen | The wasm-bindgen Developers |
| wasm-streams@0.5.0 | MIT OR Apache-2.0 | https://github.com/MattiasBuelens/wasm-streams/ | Mattias Buelens <mattias@buelens.com> |
| wayland-backend@0.3.16 | MIT | https://github.com/smithay/wayland-rs | Elinor Berger <elinor@safaradeg.net> |
| wayland-client@0.31.15 | MIT | https://github.com/smithay/wayland-rs | Elinor Berger <elinor@safaradeg.net> |
| wayland-protocols-wlr@0.3.12 | MIT | https://github.com/smithay/wayland-rs | Elinor Berger <elinor@safaradeg.net> |
| wayland-protocols@0.32.13 | MIT | https://github.com/smithay/wayland-rs | Elinor Berger <elinor@safaradeg.net> |
| wayland-scanner@0.31.11 | MIT | https://github.com/smithay/wayland-rs | Elinor Berger <elinor@safaradeg.net> |
| wayland-sys@0.31.11 | MIT | https://github.com/smithay/wayland-rs | Elinor Berger <elinor@safaradeg.net> |
| web_atoms@0.2.5 | MIT OR Apache-2.0 | https://github.com/servo/html5ever | The html5ever Project Developers |
| web-sys@0.3.102 | MIT OR Apache-2.0 | https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/web-sys | The wasm-bindgen Developers |
| web-time@1.1.0 | MIT OR Apache-2.0 | https://github.com/daxpedda/web-time |  |
| webkit2gtk-sys@2.0.2 | MIT | https://github.com/tauri-apps/webkit2gtk-rs |  |
| webkit2gtk@2.0.2 | MIT | https://github.com/tauri-apps/webkit2gtk-rs |  |
| webpki-root-certs@1.0.9 | CDLA-Permissive-2.0 | https://github.com/rustls/webpki-roots |  |
| webpki-roots@1.0.8 | CDLA-Permissive-2.0 | https://github.com/rustls/webpki-roots |  |
| webview2-com-macros@0.8.1 | MIT | https://github.com/wravery/webview2-rs |  |
| webview2-com-sys@0.38.2 | MIT | https://github.com/wravery/webview2-rs |  |
| webview2-com@0.38.2 | MIT | https://github.com/wravery/webview2-rs |  |
| weezl@0.1.12 | MIT OR Apache-2.0 | https://github.com/image-rs/weezl | The image-rs Developers |
| winapi-i686-pc-windows-gnu@0.4.0 | MIT/Apache-2.0 | https://github.com/retep998/winapi-rs | Peter Atashian <retep998@gmail.com> |
| winapi-util@0.1.11 | Unlicense OR MIT | https://github.com/BurntSushi/winapi-util | Andrew Gallant <jamslam@gmail.com> |
| winapi-x86_64-pc-windows-gnu@0.4.0 | MIT/Apache-2.0 | https://github.com/retep998/winapi-rs | Peter Atashian <retep998@gmail.com> |
| winapi@0.3.9 | MIT/Apache-2.0 | https://github.com/retep998/winapi-rs | Peter Atashian <retep998@gmail.com> |
| window-vibrancy@0.6.0 | Apache-2.0 OR MIT | https://github.com/tauri-apps/tauri-plugin-vibrancy | Tauri Programme within The Commons Conservancy |
| windows_aarch64_gnullvm@0.42.2 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs | Microsoft |
| windows_aarch64_gnullvm@0.52.6 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs | Microsoft |
| windows_aarch64_gnullvm@0.53.1 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |  |
| windows_aarch64_msvc@0.42.2 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs | Microsoft |
| windows_aarch64_msvc@0.52.6 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs | Microsoft |
| windows_aarch64_msvc@0.53.1 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |  |
| windows_i686_gnu@0.42.2 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs | Microsoft |
| windows_i686_gnu@0.52.6 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs | Microsoft |
| windows_i686_gnu@0.53.1 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |  |
| windows_i686_gnullvm@0.52.6 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs | Microsoft |
| windows_i686_gnullvm@0.53.1 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |  |
| windows_i686_msvc@0.42.2 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs | Microsoft |
| windows_i686_msvc@0.52.6 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs | Microsoft |
| windows_i686_msvc@0.53.1 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |  |
| windows_x86_64_gnu@0.42.2 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs | Microsoft |
| windows_x86_64_gnu@0.52.6 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs | Microsoft |
| windows_x86_64_gnu@0.53.1 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |  |
| windows_x86_64_gnullvm@0.42.2 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs | Microsoft |
| windows_x86_64_gnullvm@0.52.6 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs | Microsoft |
| windows_x86_64_gnullvm@0.53.1 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |  |
| windows_x86_64_msvc@0.42.2 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs | Microsoft |
| windows_x86_64_msvc@0.52.6 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs | Microsoft |
| windows_x86_64_msvc@0.53.1 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |  |
| windows-collections@0.2.0 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |  |
| windows-core@0.61.2 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs | Microsoft |
| windows-core@0.62.2 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |  |
| windows-future@0.2.1 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |  |
| windows-implement@0.60.2 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |  |
| windows-interface@0.59.3 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |  |
| windows-link@0.1.3 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs | Microsoft |
| windows-link@0.2.1 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |  |
| windows-numerics@0.2.0 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |  |
| windows-result@0.3.4 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs | Microsoft |
| windows-result@0.4.1 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |  |
| windows-strings@0.4.2 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs | Microsoft |
| windows-strings@0.5.1 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |  |
| windows-sys@0.45.0 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs | Microsoft |
| windows-sys@0.52.0 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs | Microsoft |
| windows-sys@0.59.0 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs | Microsoft |
| windows-sys@0.60.2 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs | Microsoft |
| windows-sys@0.61.2 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |  |
| windows-targets@0.42.2 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs | Microsoft |
| windows-targets@0.52.6 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs | Microsoft |
| windows-targets@0.53.5 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |  |
| windows-threading@0.1.0 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs | Microsoft |
| windows-version@0.1.7 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |  |
| windows@0.61.3 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs | Microsoft |
| winnow@0.5.40 | MIT | https://github.com/winnow-rs/winnow |  |
| winnow@0.7.15 | MIT | https://github.com/winnow-rs/winnow |  |
| winnow@1.0.3 | MIT | https://github.com/winnow-rs/winnow |  |
| winreg@0.55.0 | MIT | https://github.com/gentoo90/winreg-rs | Igor Shaula <gentoo90@gmail.com> |
| wit-bindgen@0.57.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | https://github.com/bytecodealliance/wit-bindgen | Alex Crichton <alex@alexcrichton.com> |
| wl-clipboard-rs@0.9.3 | MIT/Apache-2.0 | https://github.com/YaLTeR/wl-clipboard-rs | Ivan Molodetskikh <yalterz@gmail.com> |
| writeable@0.6.3 | Unicode-3.0 | https://github.com/unicode-org/icu4x | The ICU4X Project Developers |
| wry@0.55.1 | Apache-2.0 OR MIT | https://github.com/tauri-apps/wry | Tauri Programme within The Commons Conservancy |
| x11-dl@2.21.0 | MIT | https://github.com/AltF02/x11-rs.git | daggerbot <daggerbot@gmail.com>, Erle Pereira <erle@erlepereira.com>, AltF02 <contact@altf2.dev> |
| x11@2.21.0 | MIT | https://github.com/AltF02/x11-rs.git | daggerbot <daggerbot@gmail.com>, Erle Pereira <erle@erlepereira.com>, AltF02 <contact@altf2.dev> |
| x11rb-protocol@0.13.2 | MIT OR Apache-2.0 | https://github.com/psychon/x11rb | Uli Schlachter <psychon@znc.in>, Eduardo Sánchez Muñoz <eduardosm-dev@e64.io>, notgull <jtnunley01@gmail.com> |
| x11rb@0.13.2 | MIT OR Apache-2.0 | https://github.com/psychon/x11rb | Uli Schlachter <psychon@znc.in>, Eduardo Sánchez Muñoz <eduardosm-dev@e64.io>, notgull <jtnunley01@gmail.com> |
| xattr@1.6.1 | MIT OR Apache-2.0 | https://github.com/Stebalien/xattr | Steven Allen <steven@stebalien.com> |
| yoke-derive@0.8.2 | Unicode-3.0 | https://github.com/unicode-org/icu4x | Manish Goregaokar <manishsmail@gmail.com> |
| yoke@0.8.3 | Unicode-3.0 | https://github.com/unicode-org/icu4x | Manish Goregaokar <manishsmail@gmail.com> |
| zerocopy-derive@0.8.52 | BSD-2-Clause OR Apache-2.0 OR MIT | https://github.com/google/zerocopy | Joshua Liebow-Feeser <joshlf@google.com>, Jack Wrenn <jswrenn@amazon.com> |
| zerocopy@0.8.52 | BSD-2-Clause OR Apache-2.0 OR MIT | https://github.com/google/zerocopy | Joshua Liebow-Feeser <joshlf@google.com>, Jack Wrenn <jswrenn@amazon.com> |
| zerofrom-derive@0.1.7 | Unicode-3.0 | https://github.com/unicode-org/icu4x | Manish Goregaokar <manishsmail@gmail.com> |
| zerofrom@0.1.8 | Unicode-3.0 | https://github.com/unicode-org/icu4x | The ICU4X Project Developers |
| zeroize@1.9.0 | Apache-2.0 OR MIT | https://github.com/RustCrypto/utils | The RustCrypto Project Developers |
| zerotrie@0.2.4 | Unicode-3.0 | https://github.com/unicode-org/icu4x | The ICU4X Project Developers |
| zerovec-derive@0.11.3 | Unicode-3.0 | https://github.com/unicode-org/icu4x | Manish Goregaokar <manishsmail@gmail.com> |
| zerovec@0.11.6 | Unicode-3.0 | https://github.com/unicode-org/icu4x | The ICU4X Project Developers |
| zip@4.6.1 | MIT | https://github.com/zip-rs/zip2.git | Mathijs van de Nes <git@mathijs.vd-nes.nl>, Marli Frost <marli@frost.red>, Ryan Levick <ryan.levick@gmail.com>, Chris Hennick <hennickc@amazon.com> |
| zmij@1.0.21 | MIT | https://github.com/dtolnay/zmij | David Tolnay <dtolnay@gmail.com> |
| zune-core@0.5.3 | MIT OR Apache-2.0 OR Zlib | https://github.com/etemesi254/zune-image |  |
| zune-jpeg@0.5.15 | MIT OR Apache-2.0 OR Zlib | https://github.com/etemesi254/zune-image/tree/dev/crates/zune-jpeg | caleb <etemesicaleb@gmail.com> |

### npm packages

| Package | License expression | Source | Declared authors |
|---|---|---|---|
| @esbuild/aix-ppc64@0.21.5 | MIT | https://registry.npmjs.org/@esbuild/aix-ppc64/-/aix-ppc64-0.21.5.tgz |  |
| @esbuild/android-arm@0.21.5 | MIT | https://registry.npmjs.org/@esbuild/android-arm/-/android-arm-0.21.5.tgz |  |
| @esbuild/android-arm64@0.21.5 | MIT | https://registry.npmjs.org/@esbuild/android-arm64/-/android-arm64-0.21.5.tgz |  |
| @esbuild/android-x64@0.21.5 | MIT | https://registry.npmjs.org/@esbuild/android-x64/-/android-x64-0.21.5.tgz |  |
| @esbuild/darwin-arm64@0.21.5 | MIT | https://registry.npmjs.org/@esbuild/darwin-arm64/-/darwin-arm64-0.21.5.tgz |  |
| @esbuild/darwin-x64@0.21.5 | MIT | https://registry.npmjs.org/@esbuild/darwin-x64/-/darwin-x64-0.21.5.tgz |  |
| @esbuild/freebsd-arm64@0.21.5 | MIT | https://registry.npmjs.org/@esbuild/freebsd-arm64/-/freebsd-arm64-0.21.5.tgz |  |
| @esbuild/freebsd-x64@0.21.5 | MIT | https://registry.npmjs.org/@esbuild/freebsd-x64/-/freebsd-x64-0.21.5.tgz |  |
| @esbuild/linux-arm@0.21.5 | MIT | https://registry.npmjs.org/@esbuild/linux-arm/-/linux-arm-0.21.5.tgz |  |
| @esbuild/linux-arm64@0.21.5 | MIT | https://registry.npmjs.org/@esbuild/linux-arm64/-/linux-arm64-0.21.5.tgz |  |
| @esbuild/linux-ia32@0.21.5 | MIT | https://registry.npmjs.org/@esbuild/linux-ia32/-/linux-ia32-0.21.5.tgz |  |
| @esbuild/linux-loong64@0.21.5 | MIT | https://registry.npmjs.org/@esbuild/linux-loong64/-/linux-loong64-0.21.5.tgz |  |
| @esbuild/linux-mips64el@0.21.5 | MIT | https://registry.npmjs.org/@esbuild/linux-mips64el/-/linux-mips64el-0.21.5.tgz |  |
| @esbuild/linux-ppc64@0.21.5 | MIT | https://registry.npmjs.org/@esbuild/linux-ppc64/-/linux-ppc64-0.21.5.tgz |  |
| @esbuild/linux-riscv64@0.21.5 | MIT | https://registry.npmjs.org/@esbuild/linux-riscv64/-/linux-riscv64-0.21.5.tgz |  |
| @esbuild/linux-s390x@0.21.5 | MIT | https://registry.npmjs.org/@esbuild/linux-s390x/-/linux-s390x-0.21.5.tgz |  |
| @esbuild/linux-x64@0.21.5 | MIT | https://registry.npmjs.org/@esbuild/linux-x64/-/linux-x64-0.21.5.tgz |  |
| @esbuild/netbsd-x64@0.21.5 | MIT | https://registry.npmjs.org/@esbuild/netbsd-x64/-/netbsd-x64-0.21.5.tgz |  |
| @esbuild/openbsd-x64@0.21.5 | MIT | https://registry.npmjs.org/@esbuild/openbsd-x64/-/openbsd-x64-0.21.5.tgz |  |
| @esbuild/sunos-x64@0.21.5 | MIT | https://registry.npmjs.org/@esbuild/sunos-x64/-/sunos-x64-0.21.5.tgz |  |
| @esbuild/win32-arm64@0.21.5 | MIT | https://registry.npmjs.org/@esbuild/win32-arm64/-/win32-arm64-0.21.5.tgz |  |
| @esbuild/win32-ia32@0.21.5 | MIT | https://registry.npmjs.org/@esbuild/win32-ia32/-/win32-ia32-0.21.5.tgz |  |
| @esbuild/win32-x64@0.21.5 | MIT | https://registry.npmjs.org/@esbuild/win32-x64/-/win32-x64-0.21.5.tgz |  |
| @jridgewell/gen-mapping@0.3.13 | MIT | https://registry.npmjs.org/@jridgewell/gen-mapping/-/gen-mapping-0.3.13.tgz |  |
| @jridgewell/remapping@2.3.5 | MIT | https://registry.npmjs.org/@jridgewell/remapping/-/remapping-2.3.5.tgz |  |
| @jridgewell/resolve-uri@3.1.2 | MIT | https://registry.npmjs.org/@jridgewell/resolve-uri/-/resolve-uri-3.1.2.tgz |  |
| @jridgewell/sourcemap-codec@1.5.5 | MIT | https://registry.npmjs.org/@jridgewell/sourcemap-codec/-/sourcemap-codec-1.5.5.tgz |  |
| @jridgewell/trace-mapping@0.3.31 | MIT | https://registry.npmjs.org/@jridgewell/trace-mapping/-/trace-mapping-0.3.31.tgz |  |
| @rollup/rollup-android-arm-eabi@4.62.2 | MIT | https://registry.npmjs.org/@rollup/rollup-android-arm-eabi/-/rollup-android-arm-eabi-4.62.2.tgz |  |
| @rollup/rollup-android-arm64@4.62.2 | MIT | https://registry.npmjs.org/@rollup/rollup-android-arm64/-/rollup-android-arm64-4.62.2.tgz |  |
| @rollup/rollup-darwin-arm64@4.62.2 | MIT | https://registry.npmjs.org/@rollup/rollup-darwin-arm64/-/rollup-darwin-arm64-4.62.2.tgz |  |
| @rollup/rollup-darwin-x64@4.62.2 | MIT | https://registry.npmjs.org/@rollup/rollup-darwin-x64/-/rollup-darwin-x64-4.62.2.tgz |  |
| @rollup/rollup-freebsd-arm64@4.62.2 | MIT | https://registry.npmjs.org/@rollup/rollup-freebsd-arm64/-/rollup-freebsd-arm64-4.62.2.tgz |  |
| @rollup/rollup-freebsd-x64@4.62.2 | MIT | https://registry.npmjs.org/@rollup/rollup-freebsd-x64/-/rollup-freebsd-x64-4.62.2.tgz |  |
| @rollup/rollup-linux-arm-gnueabihf@4.62.2 | MIT | https://registry.npmjs.org/@rollup/rollup-linux-arm-gnueabihf/-/rollup-linux-arm-gnueabihf-4.62.2.tgz |  |
| @rollup/rollup-linux-arm-musleabihf@4.62.2 | MIT | https://registry.npmjs.org/@rollup/rollup-linux-arm-musleabihf/-/rollup-linux-arm-musleabihf-4.62.2.tgz |  |
| @rollup/rollup-linux-arm64-gnu@4.62.2 | MIT | https://registry.npmjs.org/@rollup/rollup-linux-arm64-gnu/-/rollup-linux-arm64-gnu-4.62.2.tgz |  |
| @rollup/rollup-linux-arm64-musl@4.62.2 | MIT | https://registry.npmjs.org/@rollup/rollup-linux-arm64-musl/-/rollup-linux-arm64-musl-4.62.2.tgz |  |
| @rollup/rollup-linux-loong64-gnu@4.62.2 | MIT | https://registry.npmjs.org/@rollup/rollup-linux-loong64-gnu/-/rollup-linux-loong64-gnu-4.62.2.tgz |  |
| @rollup/rollup-linux-loong64-musl@4.62.2 | MIT | https://registry.npmjs.org/@rollup/rollup-linux-loong64-musl/-/rollup-linux-loong64-musl-4.62.2.tgz |  |
| @rollup/rollup-linux-ppc64-gnu@4.62.2 | MIT | https://registry.npmjs.org/@rollup/rollup-linux-ppc64-gnu/-/rollup-linux-ppc64-gnu-4.62.2.tgz |  |
| @rollup/rollup-linux-ppc64-musl@4.62.2 | MIT | https://registry.npmjs.org/@rollup/rollup-linux-ppc64-musl/-/rollup-linux-ppc64-musl-4.62.2.tgz |  |
| @rollup/rollup-linux-riscv64-gnu@4.62.2 | MIT | https://registry.npmjs.org/@rollup/rollup-linux-riscv64-gnu/-/rollup-linux-riscv64-gnu-4.62.2.tgz |  |
| @rollup/rollup-linux-riscv64-musl@4.62.2 | MIT | https://registry.npmjs.org/@rollup/rollup-linux-riscv64-musl/-/rollup-linux-riscv64-musl-4.62.2.tgz |  |
| @rollup/rollup-linux-s390x-gnu@4.62.2 | MIT | https://registry.npmjs.org/@rollup/rollup-linux-s390x-gnu/-/rollup-linux-s390x-gnu-4.62.2.tgz |  |
| @rollup/rollup-linux-x64-gnu@4.62.2 | MIT | https://registry.npmjs.org/@rollup/rollup-linux-x64-gnu/-/rollup-linux-x64-gnu-4.62.2.tgz |  |
| @rollup/rollup-linux-x64-musl@4.62.2 | MIT | https://registry.npmjs.org/@rollup/rollup-linux-x64-musl/-/rollup-linux-x64-musl-4.62.2.tgz |  |
| @rollup/rollup-openbsd-x64@4.62.2 | MIT | https://registry.npmjs.org/@rollup/rollup-openbsd-x64/-/rollup-openbsd-x64-4.62.2.tgz |  |
| @rollup/rollup-openharmony-arm64@4.62.2 | MIT | https://registry.npmjs.org/@rollup/rollup-openharmony-arm64/-/rollup-openharmony-arm64-4.62.2.tgz |  |
| @rollup/rollup-win32-arm64-msvc@4.62.2 | MIT | https://registry.npmjs.org/@rollup/rollup-win32-arm64-msvc/-/rollup-win32-arm64-msvc-4.62.2.tgz |  |
| @rollup/rollup-win32-ia32-msvc@4.62.2 | MIT | https://registry.npmjs.org/@rollup/rollup-win32-ia32-msvc/-/rollup-win32-ia32-msvc-4.62.2.tgz |  |
| @rollup/rollup-win32-x64-gnu@4.62.2 | MIT | https://registry.npmjs.org/@rollup/rollup-win32-x64-gnu/-/rollup-win32-x64-gnu-4.62.2.tgz |  |
| @rollup/rollup-win32-x64-msvc@4.62.2 | MIT | https://registry.npmjs.org/@rollup/rollup-win32-x64-msvc/-/rollup-win32-x64-msvc-4.62.2.tgz |  |
| @sveltejs/acorn-typescript@1.0.10 | MIT | https://registry.npmjs.org/@sveltejs/acorn-typescript/-/acorn-typescript-1.0.10.tgz |  |
| @sveltejs/load-config@0.1.1 | MIT | https://registry.npmjs.org/@sveltejs/load-config/-/load-config-0.1.1.tgz |  |
| @sveltejs/vite-plugin-svelte-inspector@3.0.1 | MIT | https://registry.npmjs.org/@sveltejs/vite-plugin-svelte-inspector/-/vite-plugin-svelte-inspector-3.0.1.tgz |  |
| @sveltejs/vite-plugin-svelte@4.0.4 | MIT | https://registry.npmjs.org/@sveltejs/vite-plugin-svelte/-/vite-plugin-svelte-4.0.4.tgz |  |
| @tauri-apps/api@2.11.1 | Apache-2.0 OR MIT | https://registry.npmjs.org/@tauri-apps/api/-/api-2.11.1.tgz |  |
| @tauri-apps/cli-darwin-arm64@2.11.3 | Apache-2.0 OR MIT | https://registry.npmjs.org/@tauri-apps/cli-darwin-arm64/-/cli-darwin-arm64-2.11.3.tgz |  |
| @tauri-apps/cli-darwin-x64@2.11.3 | Apache-2.0 OR MIT | https://registry.npmjs.org/@tauri-apps/cli-darwin-x64/-/cli-darwin-x64-2.11.3.tgz |  |
| @tauri-apps/cli-linux-arm-gnueabihf@2.11.3 | Apache-2.0 OR MIT | https://registry.npmjs.org/@tauri-apps/cli-linux-arm-gnueabihf/-/cli-linux-arm-gnueabihf-2.11.3.tgz |  |
| @tauri-apps/cli-linux-arm64-gnu@2.11.3 | Apache-2.0 OR MIT | https://registry.npmjs.org/@tauri-apps/cli-linux-arm64-gnu/-/cli-linux-arm64-gnu-2.11.3.tgz |  |
| @tauri-apps/cli-linux-arm64-musl@2.11.3 | Apache-2.0 OR MIT | https://registry.npmjs.org/@tauri-apps/cli-linux-arm64-musl/-/cli-linux-arm64-musl-2.11.3.tgz |  |
| @tauri-apps/cli-linux-riscv64-gnu@2.11.3 | Apache-2.0 OR MIT | https://registry.npmjs.org/@tauri-apps/cli-linux-riscv64-gnu/-/cli-linux-riscv64-gnu-2.11.3.tgz |  |
| @tauri-apps/cli-linux-x64-gnu@2.11.3 | Apache-2.0 OR MIT | https://registry.npmjs.org/@tauri-apps/cli-linux-x64-gnu/-/cli-linux-x64-gnu-2.11.3.tgz |  |
| @tauri-apps/cli-linux-x64-musl@2.11.3 | Apache-2.0 OR MIT | https://registry.npmjs.org/@tauri-apps/cli-linux-x64-musl/-/cli-linux-x64-musl-2.11.3.tgz |  |
| @tauri-apps/cli-win32-arm64-msvc@2.11.3 | Apache-2.0 OR MIT | https://registry.npmjs.org/@tauri-apps/cli-win32-arm64-msvc/-/cli-win32-arm64-msvc-2.11.3.tgz |  |
| @tauri-apps/cli-win32-ia32-msvc@2.11.3 | Apache-2.0 OR MIT | https://registry.npmjs.org/@tauri-apps/cli-win32-ia32-msvc/-/cli-win32-ia32-msvc-2.11.3.tgz |  |
| @tauri-apps/cli-win32-x64-msvc@2.11.3 | Apache-2.0 OR MIT | https://registry.npmjs.org/@tauri-apps/cli-win32-x64-msvc/-/cli-win32-x64-msvc-2.11.3.tgz |  |
| @tauri-apps/cli@2.11.3 | Apache-2.0 OR MIT | https://registry.npmjs.org/@tauri-apps/cli/-/cli-2.11.3.tgz |  |
| @tauri-apps/plugin-clipboard-manager@2.3.2 | MIT OR Apache-2.0 | https://registry.npmjs.org/@tauri-apps/plugin-clipboard-manager/-/plugin-clipboard-manager-2.3.2.tgz |  |
| @tauri-apps/plugin-dialog@2.7.2 | MIT OR Apache-2.0 | https://registry.npmjs.org/@tauri-apps/plugin-dialog/-/plugin-dialog-2.7.2.tgz |  |
| @tauri-apps/plugin-fs@2.5.1 | MIT OR Apache-2.0 | https://registry.npmjs.org/@tauri-apps/plugin-fs/-/plugin-fs-2.5.1.tgz |  |
| @tauri-apps/plugin-updater@2.10.1 | MIT OR Apache-2.0 | https://registry.npmjs.org/@tauri-apps/plugin-updater/-/plugin-updater-2.10.1.tgz |  |
| @tsconfig/svelte@5.0.8 | MIT | https://registry.npmjs.org/@tsconfig/svelte/-/svelte-5.0.8.tgz |  |
| @types/estree@1.0.9 | MIT | https://registry.npmjs.org/@types/estree/-/estree-1.0.9.tgz |  |
| @types/trusted-types@2.0.7 | MIT | https://registry.npmjs.org/@types/trusted-types/-/trusted-types-2.0.7.tgz |  |
| acorn@8.17.0 | MIT | https://registry.npmjs.org/acorn/-/acorn-8.17.0.tgz |  |
| aria-query@5.3.1 | Apache-2.0 | https://registry.npmjs.org/aria-query/-/aria-query-5.3.1.tgz |  |
| axobject-query@4.1.0 | Apache-2.0 | https://registry.npmjs.org/axobject-query/-/axobject-query-4.1.0.tgz |  |
| chokidar@4.0.3 | MIT | https://registry.npmjs.org/chokidar/-/chokidar-4.0.3.tgz |  |
| clsx@2.1.1 | MIT | https://registry.npmjs.org/clsx/-/clsx-2.1.1.tgz |  |
| debug@4.4.3 | MIT | https://registry.npmjs.org/debug/-/debug-4.4.3.tgz |  |
| deepmerge@4.3.1 | MIT | https://registry.npmjs.org/deepmerge/-/deepmerge-4.3.1.tgz |  |
| devalue@5.8.1 | MIT | https://registry.npmjs.org/devalue/-/devalue-5.8.1.tgz |  |
| esbuild@0.21.5 | MIT | https://registry.npmjs.org/esbuild/-/esbuild-0.21.5.tgz |  |
| esm-env@1.2.2 | MIT | https://registry.npmjs.org/esm-env/-/esm-env-1.2.2.tgz |  |
| esrap@2.2.12 | MIT | https://registry.npmjs.org/esrap/-/esrap-2.2.12.tgz |  |
| fdir@6.5.0 | MIT | https://registry.npmjs.org/fdir/-/fdir-6.5.0.tgz |  |
| fsevents@2.3.3 | MIT | https://registry.npmjs.org/fsevents/-/fsevents-2.3.3.tgz |  |
| is-reference@3.0.3 | MIT | https://registry.npmjs.org/is-reference/-/is-reference-3.0.3.tgz |  |
| kleur@4.1.5 | MIT | https://registry.npmjs.org/kleur/-/kleur-4.1.5.tgz |  |
| locate-character@3.0.0 | MIT | https://registry.npmjs.org/locate-character/-/locate-character-3.0.0.tgz |  |
| magic-string@0.30.21 | MIT | https://registry.npmjs.org/magic-string/-/magic-string-0.30.21.tgz |  |
| mri@1.2.0 | MIT | https://registry.npmjs.org/mri/-/mri-1.2.0.tgz |  |
| ms@2.1.3 | MIT | https://registry.npmjs.org/ms/-/ms-2.1.3.tgz |  |
| nanoid@3.3.14 | MIT | https://registry.npmjs.org/nanoid/-/nanoid-3.3.14.tgz |  |
| picocolors@1.1.1 | ISC | https://registry.npmjs.org/picocolors/-/picocolors-1.1.1.tgz |  |
| postcss@8.5.15 | MIT | https://registry.npmjs.org/postcss/-/postcss-8.5.15.tgz |  |
| readdirp@4.1.2 | MIT | https://registry.npmjs.org/readdirp/-/readdirp-4.1.2.tgz |  |
| rollup@4.62.2 | MIT | https://registry.npmjs.org/rollup/-/rollup-4.62.2.tgz |  |
| sade@1.8.1 | MIT | https://registry.npmjs.org/sade/-/sade-1.8.1.tgz |  |
| source-map-js@1.2.1 | BSD-3-Clause | https://registry.npmjs.org/source-map-js/-/source-map-js-1.2.1.tgz |  |
| svelte-check@4.6.0 | MIT | https://registry.npmjs.org/svelte-check/-/svelte-check-4.6.0.tgz |  |
| svelte@5.56.3 | MIT | https://registry.npmjs.org/svelte/-/svelte-5.56.3.tgz |  |
| tslib@2.8.1 | 0BSD | https://registry.npmjs.org/tslib/-/tslib-2.8.1.tgz |  |
| typescript@5.9.3 | Apache-2.0 | https://registry.npmjs.org/typescript/-/typescript-5.9.3.tgz |  |
| vite@5.4.21 | MIT | https://registry.npmjs.org/vite/-/vite-5.4.21.tgz |  |
| vitefu@1.1.3 | MIT | https://registry.npmjs.org/vitefu/-/vitefu-1.1.3.tgz |  |
| zimmerframe@1.1.4 | MIT | https://registry.npmjs.org/zimmerframe/-/zimmerframe-1.1.4.tgz |  |

The complete machine-readable inventory is stored in `licenses/dependencies.json`.

### Reviewed exception

- `r-efi` — `MIT OR Apache-2.0 OR LGPL-2.1-or-later`; the project policy selects the permissive MIT/Apache-2.0 alternatives and does not distribute the LGPL alternative as project content.

