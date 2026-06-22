//! Resources — загрузка word packs, quotes.
//! Sprint 6: WordPackLoader (txt), QuoteLoader (toml).

pub mod quotes;
pub mod words;

pub use quotes::{quote_loader, Quote, QuoteLoader, QuotePack};
pub use words::{word_pack_loader, WordPack, WordPackLoader};
