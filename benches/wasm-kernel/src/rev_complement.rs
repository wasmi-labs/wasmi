// Adapted version from benchmarks game. In particular
// rayon is removed.
//
// https://benchmarksgame-team.pages.debian.net/benchmarksgame/program/revcomp-rust-3.html

// The Computer Language Benchmarks Game
// https://salsa.debian.org/benchmarksgame-team/benchmarksgame/
//
// contributed by the Rust Project Developers
// contributed by Cristi Cobzarenco
// contributed by TeXitoi
// contributed by Matt Brubeck

use std::io::BufRead;
use std::mem::{replace, take};
use std::{cmp, io};

/// Lookup table to find the complement of a single FASTA code.
fn build_table() -> [u8; 256] {
	let mut table = [0; 256];
	for (i, x) in table.iter_mut().enumerate() {
		*x = match i as u8 as char {
			'A' | 'a' => 'T',
			'C' | 'c' => 'G',
			'G' | 'g' => 'C',
			'T' | 't' => 'A',
			'U' | 'u' => 'A',
			'M' | 'm' => 'K',
			'R' | 'r' => 'Y',
			'W' | 'w' => 'W',
			'S' | 's' => 'S',
			'Y' | 'y' => 'R',
			'K' | 'k' => 'M',
			'V' | 'v' => 'B',
			'H' | 'h' => 'D',
			'D' | 'd' => 'H',
			'B' | 'b' => 'V',
			'N' | 'n' => 'N',
			i => i,
		} as u8;
	}
	table
}

/// Utilities for splitting chunks off of slices.
trait SplitOff {
	fn split_off_left(&mut self, n: usize) -> Self;
	fn split_off_right(&mut self, n: usize) -> Self;
}
impl<'a, T> SplitOff for &'a mut [T] {
	/// Split the left `n` items from self and return them as a separate slice.
	fn split_off_left(&mut self, n: usize) -> Self {
		let n = cmp::min(self.len(), n);
		let data = take(self);
		let (left, data) = data.split_at_mut(n);
		*self = data;
		left
	}
	/// Split the right `n` items from self and return them as a separate slice.
	fn split_off_right(&mut self, n: usize) -> Self {
		let len = self.len();
		let n = cmp::min(len, n);
		let data = take(self);
		let (data, right) = data.split_at_mut(len - n);
		*self = data;
		right
	}
}

/// Length of a normal line including the terminating \n.
const LINE_LEN: usize = 61;

/// Compute the reverse complement for two contiguous chunks without line breaks.
fn reverse_chunks(left: &mut [u8], right: &mut [u8], table: &[u8; 256]) {
	for (x, y) in left.iter_mut().zip(right.iter_mut().rev()) {
		*y = table[replace(x, table[*y as usize]) as usize];
	}
}

/// Compute the reverse complement on chunks from opposite ends of a sequence.
///
/// `left` must start at the beginning of a line. If there are an odd number of
/// bytes, `right` will initially be 1 byte longer than `left`; otherwise they
/// will have equal lengths.
fn reverse_complement_left_right(
	mut left: &mut [u8],
	mut right: &mut [u8],
	trailing_len: usize,
	table: &[u8; 256],
) {
	// Each iteration swaps one line from the start of the sequence with one
	// from the end.
	while !left.is_empty() || !right.is_empty() {
		// Get the chunk up to the newline in `right`.
		let mut a = left.split_off_left(trailing_len);
		let mut b = right.split_off_right(trailing_len);
		right.split_off_right(1); // Skip the newline in `right`.

		// If we've reached the middle of the sequence here and there is an
		// odd number of bytes remaining, the odd one will be on the right.
		if b.len() > a.len() {
			let mid = b.split_off_left(1);
			mid[0] = table[mid[0] as usize];
		}

		reverse_chunks(a, b, table);

		// Get the chunk up to the newline in `left`.
		let n = LINE_LEN - 1 - trailing_len;
		a = left.split_off_left(n);
		b = right.split_off_right(n);
		left.split_off_left(1); // Skip the newline in `left`.

		// If we've reached the middle of the sequence and there is an odd
		// number of bytes remaining, the odd one will now be on the left.
		if a.len() > b.len() {
			let mid = a.split_off_right(1);
			mid[0] = table[mid[0] as usize]
		}

		reverse_chunks(a, b, table);
	}
}

/// Compute the reverse complement of one sequence.
fn reverse_complement(seq: &mut [u8], table: &[u8; 256]) {
	let len = seq.len() - 1;
	let seq = &mut seq[..len]; // Drop the last newline
	let trailing_len = len % LINE_LEN;
	let (left, right) = seq.split_at_mut(len / 2);
	reverse_complement_left_right(left, right, trailing_len, table);
}

/// Read sequences from stdin and print the reverse complement to stdout.
pub fn run(input: &[u8]) -> Vec<u8> {
	let mut buf = Vec::with_capacity(input.len());

	let mut input = io::Cursor::new(input);

	// Read the first header line.
	input.read_until(b'\n', &mut buf).unwrap();

	// Read sequence data line-by-line, splitting on headers.
	let mut line_start = buf.len();
	let mut seq_start = line_start;
	let mut seqs = vec![];
	while input.read_until(b'\n', &mut buf).unwrap() > 0 {
		if buf[line_start] == b'>' {
			// Found the start of a new sequence.
			seqs.push(seq_start..line_start);
			seq_start = buf.len();
		}
		line_start = buf.len();
	}
	seqs.push(seq_start..buf.len());

	// Compute the reverse complements of each sequence.
	let table = build_table();
	for seq in seqs {
		reverse_complement(&mut buf[seq], &table);
	}

	buf
}
