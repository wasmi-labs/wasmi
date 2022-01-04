//! Initially it supposed to be like [1]. However it turned out
//! that executing this code in wasmi way too slow.
//!
//! [1]: https://benchmarksgame-team.pages.debian.net/benchmarksgame/program/regexredux-rust-2.html

lazy_static! {
	static ref REGEX: ::regex::bytes::Regex =
		::regex::bytes::Regex::new("agggtaa[cgt]|[acg]ttaccct").unwrap();
}

pub fn prepare() {
	::lazy_static::initialize(&REGEX);
}

pub fn run(seq: &[u8]) -> usize {
	REGEX.find_iter(seq).count()
}
