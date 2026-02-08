use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

/// A fixed-size bitboard supporting up to 32×32 = 1024 positions.
/// Stored as 16 × u64 words, entirely on the stack.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Bitboard {
    words: [u64; 16],
}

impl Bitboard {
    /// All bits zero.
    #[inline]
    pub const fn empty() -> Self {
        Bitboard { words: [0; 16] }
    }

    /// Single bit set at `index`.
    #[inline]
    pub fn single(index: usize) -> Self {
        debug_assert!(index < 1024);
        let mut bb = Self::empty();
        bb.words[index / 64] = 1u64 << (index % 64);
        bb
    }

    /// Construct from raw words.
    #[inline]
    pub const fn from_words(words: [u64; 16]) -> Self {
        Bitboard { words }
    }

    /// Test whether bit `index` is set.
    #[inline]
    pub fn get(&self, index: usize) -> bool {
        debug_assert!(index < 1024);
        (self.words[index / 64] >> (index % 64)) & 1 != 0
    }

    /// Set bit `index` to 1.
    #[inline]
    pub fn set(&mut self, index: usize) {
        debug_assert!(index < 1024);
        self.words[index / 64] |= 1u64 << (index % 64);
    }

    /// Clear bit `index` to 0.
    #[inline]
    pub fn clear(&mut self, index: usize) {
        debug_assert!(index < 1024);
        self.words[index / 64] &= !(1u64 << (index % 64));
    }

    /// True if no bits are set.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.words.iter().all(|&w| w == 0)
    }

    /// True if any bit is set.
    #[inline]
    pub fn is_nonzero(&self) -> bool {
        self.words.iter().any(|&w| w != 0)
    }

    /// Population count — number of set bits.
    #[inline]
    pub fn count(&self) -> u32 {
        self.words.iter().map(|w| w.count_ones()).sum()
    }

    /// Index of the lowest set bit, or `None` if empty.
    #[inline]
    pub fn lowest_bit_index(&self) -> Option<usize> {
        for (i, &w) in self.words.iter().enumerate() {
            if w != 0 {
                return Some(i * 64 + w.trailing_zeros() as usize);
            }
        }
        None
    }

    /// Shift all bits left (toward higher indices) by `n` positions.
    /// Bits shifted beyond 1023 are lost.
    #[inline]
    pub fn shift_left(&self, n: usize) -> Self {
        if n == 0 {
            return *self;
        }
        if n >= 1024 {
            return Self::empty();
        }
        let word_shift = n / 64;
        let bit_shift = n % 64;
        let mut out = [0u64; 16];

        if bit_shift == 0 {
            for i in word_shift..16 {
                out[i] = self.words[i - word_shift];
            }
        } else {
            for i in word_shift..16 {
                out[i] = self.words[i - word_shift] << bit_shift;
                if i > word_shift {
                    out[i] |= self.words[i - word_shift - 1] >> (64 - bit_shift);
                }
            }
        }
        Bitboard { words: out }
    }

    /// Shift all bits right (toward lower indices) by `n` positions.
    /// Bits shifted below 0 are lost.
    #[inline]
    pub fn shift_right(&self, n: usize) -> Self {
        if n == 0 {
            return *self;
        }
        if n >= 1024 {
            return Self::empty();
        }
        let word_shift = n / 64;
        let bit_shift = n % 64;
        let mut out = [0u64; 16];

        if bit_shift == 0 {
            for i in 0..16 - word_shift {
                out[i] = self.words[i + word_shift];
            }
        } else {
            for i in 0..16 - word_shift {
                out[i] = self.words[i + word_shift] >> bit_shift;
                if i + word_shift + 1 < 16 {
                    out[i] |= self.words[i + word_shift + 1] << (64 - bit_shift);
                }
            }
        }
        Bitboard { words: out }
    }

    /// Iterate over indices of set bits.
    #[inline]
    pub fn iter_ones(&self) -> BitIterator {
        BitIterator {
            words: self.words,
            word_index: 0,
        }
    }
}

impl BitAnd for Bitboard {
    type Output = Bitboard;
    #[inline]
    fn bitand(self, rhs: Bitboard) -> Bitboard {
        let mut out = [0u64; 16];
        for i in 0..16 {
            out[i] = self.words[i] & rhs.words[i];
        }
        Bitboard { words: out }
    }
}

impl BitAndAssign for Bitboard {
    #[inline]
    fn bitand_assign(&mut self, rhs: Bitboard) {
        for i in 0..16 {
            self.words[i] &= rhs.words[i];
        }
    }
}

impl BitOr for Bitboard {
    type Output = Bitboard;
    #[inline]
    fn bitor(self, rhs: Bitboard) -> Bitboard {
        let mut out = [0u64; 16];
        for i in 0..16 {
            out[i] = self.words[i] | rhs.words[i];
        }
        Bitboard { words: out }
    }
}

impl BitOrAssign for Bitboard {
    #[inline]
    fn bitor_assign(&mut self, rhs: Bitboard) {
        for i in 0..16 {
            self.words[i] |= rhs.words[i];
        }
    }
}

impl Not for Bitboard {
    type Output = Bitboard;
    #[inline]
    fn not(self) -> Bitboard {
        let mut out = [0u64; 16];
        for i in 0..16 {
            out[i] = !self.words[i];
        }
        Bitboard { words: out }
    }
}

/// Iterator over set-bit indices in a `Bitboard`.
pub struct BitIterator {
    words: [u64; 16],
    word_index: usize,
}

impl Iterator for BitIterator {
    type Item = usize;
    #[inline]
    fn next(&mut self) -> Option<usize> {
        while self.word_index < 16 {
            let w = self.words[self.word_index];
            if w != 0 {
                let bit = w.trailing_zeros() as usize;
                // Clear lowest set bit
                self.words[self.word_index] = w & (w - 1);
                return Some(self.word_index * 64 + bit);
            }
            self.word_index += 1;
        }
        None
    }
}

/// Precomputed masks for a given board geometry. Created once per Game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoardGeometry {
    pub width: usize,
    pub height: usize,
    pub area: usize,
    /// Mask with 1s at all valid board positions (indices 0..area).
    pub board_mask: Bitboard,
    /// board_mask minus column 0 (used to prevent left-wrap in right-shift neighbor).
    pub not_col0: Bitboard,
    /// board_mask minus last column (used to prevent right-wrap in left-shift neighbor).
    pub not_col_last: Bitboard,
}

impl BoardGeometry {
    /// Build geometry for a `width × height` board.
    pub fn new(width: usize, height: usize) -> Self {
        debug_assert!(width >= 2 && width <= 32);
        debug_assert!(height >= 2 && height <= 32);
        let area = width * height;

        let mut board_mask = Bitboard::empty();
        for i in 0..area {
            board_mask.set(i);
        }

        let mut not_col0 = board_mask;
        for row in 0..height {
            not_col0.clear(row * width); // column 0
        }

        let mut not_col_last = board_mask;
        for row in 0..height {
            not_col_last.clear(row * width + width - 1); // last column
        }

        BoardGeometry {
            width,
            height,
            area,
            board_mask,
            not_col0,
            not_col_last,
        }
    }

    /// Compute the set of all orthogonal neighbors of every bit in `bb`.
    #[inline]
    pub fn neighbors(&self, bb: &Bitboard) -> Bitboard {
        // right: col+1 = shift left by 1, mask off column 0 wraps
        let right = bb.shift_left(1) & self.not_col0;
        // left: col-1 = shift right by 1, mask off last-column wraps
        let left = bb.shift_right(1) & self.not_col_last;
        // down: row+1 = shift left by width
        let down = bb.shift_left(self.width);
        // up: row-1 = shift right by width
        let up = bb.shift_right(self.width);
        (right | left | down | up) & self.board_mask
    }

    /// Flood-fill from `seed` through `mask`. Returns the connected component
    /// of `seed` within `mask`.
    #[inline]
    pub fn flood_fill(&self, seed: Bitboard, mask: Bitboard) -> Bitboard {
        let mut filled = seed & mask;
        loop {
            let expanded = (filled | self.neighbors(&filled)) & mask;
            if expanded == filled {
                return filled;
            }
            filled = expanded;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let bb = Bitboard::empty();
        assert!(bb.is_empty());
        assert_eq!(bb.count(), 0);
        assert!(bb.lowest_bit_index().is_none());
    }

    #[test]
    fn test_single() {
        let bb = Bitboard::single(0);
        assert!(bb.get(0));
        assert!(!bb.get(1));
        assert_eq!(bb.count(), 1);
        assert_eq!(bb.lowest_bit_index(), Some(0));

        let bb2 = Bitboard::single(63);
        assert!(bb2.get(63));
        assert!(!bb2.get(62));
        assert!(!bb2.get(64));

        let bb3 = Bitboard::single(64);
        assert!(bb3.get(64));
        assert!(!bb3.get(63));

        let bb4 = Bitboard::single(1023);
        assert!(bb4.get(1023));
        assert_eq!(bb4.count(), 1);
    }

    #[test]
    fn test_set_clear() {
        let mut bb = Bitboard::empty();
        bb.set(100);
        assert!(bb.get(100));
        assert_eq!(bb.count(), 1);
        bb.clear(100);
        assert!(!bb.get(100));
        assert!(bb.is_empty());
    }

    #[test]
    fn test_bitwise_ops() {
        let a = Bitboard::single(5) | Bitboard::single(10);
        let b = Bitboard::single(10) | Bitboard::single(20);

        let and = a & b;
        assert!(and.get(10));
        assert!(!and.get(5));
        assert!(!and.get(20));

        let or = a | b;
        assert!(or.get(5));
        assert!(or.get(10));
        assert!(or.get(20));
    }

    #[test]
    fn test_shift_left() {
        let bb = Bitboard::single(0);
        let shifted = bb.shift_left(1);
        assert!(shifted.get(1));
        assert!(!shifted.get(0));

        // Cross word boundary: 63 -> 64
        let bb2 = Bitboard::single(63);
        let shifted2 = bb2.shift_left(1);
        assert!(shifted2.get(64));
        assert!(!shifted2.get(63));

        // Cross word boundary: 127 -> 128
        let bb3 = Bitboard::single(127);
        let shifted3 = bb3.shift_left(1);
        assert!(shifted3.get(128));
        assert!(!shifted3.get(127));
    }

    #[test]
    fn test_shift_right() {
        let bb = Bitboard::single(1);
        let shifted = bb.shift_right(1);
        assert!(shifted.get(0));
        assert!(!shifted.get(1));

        // Cross word boundary: 64 -> 63
        let bb2 = Bitboard::single(64);
        let shifted2 = bb2.shift_right(1);
        assert!(shifted2.get(63));
        assert!(!shifted2.get(64));

        // Shift from 0 -> lost
        let bb3 = Bitboard::single(0);
        let shifted3 = bb3.shift_right(1);
        assert!(shifted3.is_empty());
    }

    #[test]
    fn test_shift_by_width() {
        // Simulate shift by width=9 (row shift on 9x9 board)
        let bb = Bitboard::single(4); // col=4, row=0
        let shifted = bb.shift_left(9);
        assert!(shifted.get(13)); // col=4, row=1
        assert!(!shifted.get(4));
    }

    #[test]
    fn test_iter_ones() {
        let bb = Bitboard::single(3) | Bitboard::single(64) | Bitboard::single(200);
        let indices: Vec<usize> = bb.iter_ones().collect();
        assert_eq!(indices, vec![3, 64, 200]);
    }

    #[test]
    fn test_iter_ones_empty() {
        let bb = Bitboard::empty();
        let indices: Vec<usize> = bb.iter_ones().collect();
        assert!(indices.is_empty());
    }

    #[test]
    fn test_geometry_9x9() {
        let geo = BoardGeometry::new(9, 9);
        assert_eq!(geo.area, 81);
        assert_eq!(geo.board_mask.count(), 81);

        // Column 0 positions: 0, 9, 18, 27, ...
        for row in 0..9 {
            assert!(!geo.not_col0.get(row * 9));
            assert!(geo.not_col0.get(row * 9 + 1));
        }

        // Last column positions: 8, 17, 26, ...
        for row in 0..9 {
            assert!(!geo.not_col_last.get(row * 9 + 8));
            assert!(geo.not_col_last.get(row * 9 + 7));
        }
    }

    #[test]
    fn test_neighbors_center() {
        let geo = BoardGeometry::new(9, 9);
        // Center of 9x9: col=4, row=4 -> index = 4*9+4 = 40
        let center = Bitboard::single(40);
        let nbrs = geo.neighbors(&center);

        // Expected: right=41, left=39, up=31, down=49
        assert!(nbrs.get(41));
        assert!(nbrs.get(39));
        assert!(nbrs.get(31));
        assert!(nbrs.get(49));
        assert_eq!(nbrs.count(), 4);
    }

    #[test]
    fn test_neighbors_corner() {
        let geo = BoardGeometry::new(9, 9);
        // Top-left corner: col=0, row=0 -> index = 0
        let corner = Bitboard::single(0);
        let nbrs = geo.neighbors(&corner);

        // Expected: right=1, down=9 (no left, no up)
        assert!(nbrs.get(1));
        assert!(nbrs.get(9));
        assert_eq!(nbrs.count(), 2);
    }

    #[test]
    fn test_neighbors_no_wrap() {
        let geo = BoardGeometry::new(9, 9);
        // Right edge: col=8, row=1 -> index = 1*9+8 = 17
        let edge = Bitboard::single(17);
        let nbrs = geo.neighbors(&edge);

        // Expected: left=16, up=8, down=26 (no right — must not wrap to col=0 of next row)
        assert!(nbrs.get(16)); // left
        assert!(nbrs.get(8));  // up
        assert!(nbrs.get(26)); // down
        assert!(!nbrs.get(18)); // must NOT wrap
        assert_eq!(nbrs.count(), 3);
    }

    #[test]
    fn test_neighbors_left_edge() {
        let geo = BoardGeometry::new(9, 9);
        // Left edge: col=0, row=2 -> index = 2*9+0 = 18
        let edge = Bitboard::single(18);
        let nbrs = geo.neighbors(&edge);

        // Expected: right=19, up=9, down=27 (no left — must not wrap to col=8 of previous row)
        assert!(nbrs.get(19)); // right
        assert!(nbrs.get(9));  // up
        assert!(nbrs.get(27)); // down
        assert!(!nbrs.get(17)); // must NOT wrap
        assert_eq!(nbrs.count(), 3);
    }

    #[test]
    fn test_flood_fill_single() {
        let geo = BoardGeometry::new(5, 5);
        let seed = Bitboard::single(0);
        let mask = seed;
        let result = geo.flood_fill(seed, mask);
        assert_eq!(result, seed);
    }

    #[test]
    fn test_flood_fill_group() {
        let geo = BoardGeometry::new(5, 5);
        // Create a group: (0,0), (1,0), (2,0) -> indices 0, 1, 2
        let mask = Bitboard::single(0) | Bitboard::single(1) | Bitboard::single(2);
        let seed = Bitboard::single(0);
        let result = geo.flood_fill(seed, mask);
        assert_eq!(result, mask);
    }

    #[test]
    fn test_flood_fill_disconnected() {
        let geo = BoardGeometry::new(5, 5);
        // Two disconnected stones: (0,0) and (3,3) -> indices 0 and 18
        let mask = Bitboard::single(0) | Bitboard::single(18);
        let seed = Bitboard::single(0);
        let result = geo.flood_fill(seed, mask);
        // Should only reach the seed's connected component
        assert!(result.get(0));
        assert!(!result.get(18));
        assert_eq!(result.count(), 1);
    }

    #[test]
    fn test_not() {
        let bb = Bitboard::single(5);
        let notbb = !bb;
        assert!(!notbb.get(5));
        assert!(notbb.get(0));
        assert!(notbb.get(6));
    }

    #[test]
    fn test_non_square_board() {
        let geo = BoardGeometry::new(5, 3);
        assert_eq!(geo.area, 15);
        assert_eq!(geo.board_mask.count(), 15);

        // Corner (4, 2) -> index = 2*5+4 = 14
        let corner = Bitboard::single(14);
        let nbrs = geo.neighbors(&corner);
        // Expected: left=13, up=9
        assert!(nbrs.get(13));
        assert!(nbrs.get(9));
        assert_eq!(nbrs.count(), 2);
    }

    #[test]
    fn test_assign_ops() {
        let mut bb = Bitboard::single(1);
        bb |= Bitboard::single(2);
        assert!(bb.get(1));
        assert!(bb.get(2));

        bb &= Bitboard::single(2);
        assert!(!bb.get(1));
        assert!(bb.get(2));
    }
}
