// This contract is used as the reference via The Wizard on Stylus. It
// should be changed to compete in the hackathon!

extern crate alloc;

use stylus_sdk::{alloy_primitives::*, prelude::*};

use alloc::{collections::BTreeMap, vec, vec::Vec};

use siphasher::sip::SipHasher13;

/* ~~~~~~~~~~~~~~ BOARD IMPLEMENTATION ~~~~~~~~~~~~~~ */

const BOARD_SIZE: u32 = 0x1FFFF;

const MAX_TRIES: u32 = 10_000;

const CHECKS_NEEDED: u32 = 2;

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum Piece {
    PAWN,
    KNIGHT,
    BISHOP,
    CASTLE,
    QUEEN,
    KING,
}

impl From<Piece> for u8 {
    fn from(v: Piece) -> Self {
        v as u8
    }
}

impl TryFrom<u8> for Piece {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Piece::PAWN),
            1 => Ok(Piece::KNIGHT),
            2 => Ok(Piece::BISHOP),
            3 => Ok(Piece::CASTLE),
            4 => Ok(Piece::QUEEN),
            5 => Ok(Piece::KING),
            _ => Err(()),
        }
    }
}

/// Board that this game is played on. Could be of any size. This could
/// be optimised for gas golfing.
pub type Board = BTreeMap<u32, (Piece, u32)>;

fn pos_to_xy(row_size: u32, p: u32) -> (u32, u32) {
    (p % row_size, p / row_size)
}

fn xy_to_pos(row_size: u32, x: u32, y: u32) -> u32 {
    y.wrapping_mul(row_size).wrapping_add(x)
}

fn in_bounds(row_size: u32, x: u32, y: u32) -> bool {
    x < row_size && y < row_size
}

// Find the in check threats for the king given, returning the nonces of
// the threats.
fn in_check_threats(board: &Board, row_size: u32, king_pos: u32) -> Vec<u32> {
    let (king_x, king_y) = pos_to_xy(row_size, king_pos);
    let mut threats = vec![];
    // The following code takes the position of the king, then searches for pieces in
    // positions that might threaten the king, then adding them as threats if they're
    // the kind of piece to be a threat.
    macro_rules! piece_add_threat_if_valid {
        ($piece:ident, $x:expr, $y:expr) => {
            if in_bounds(row_size, $x, $y) {
                if let Some((Piece::$piece, n)) = board.get(&xy_to_pos(row_size, $x, $y)) {
                    threats.push(*n);
                }
            }
        };
    }
    // Pawn
    for dx in [-1, 1] {
        let x = king_x.wrapping_add_signed(dx);
        let y = king_y.wrapping_sub(1);
        piece_add_threat_if_valid!(PAWN, x, y);
    }
    // Knight
    for (dx, dy) in [
        (-2, -1),
        (-2, 1),
        (-1, -2),
        (-1, 2),
        (1, -2),
        (1, 2),
        (2, -1),
        (2, 1),
    ] {
        piece_add_threat_if_valid!(
            KNIGHT,
            king_x.wrapping_add_signed(dx),
            king_y.wrapping_add_signed(dy)
        );
    }
    // Rook/Queen
    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
        let mut x = king_x;
        let mut y = king_y;
        loop {
            x = x.wrapping_add_signed(dx);
            y = y.wrapping_add_signed(dy);
            // It's true that the macro does this check as well, but any compiler
            // would optimise this out, so we leave it for brevity reasons.
            if !in_bounds(row_size, x, y) {
                break;
            }
            piece_add_threat_if_valid!(CASTLE, x, y);
            piece_add_threat_if_valid!(QUEEN, x, y);
        }
    }
    // Bishop/Queen
    for (dx, dy) in [(-1, -1), (-1, 1), (1, -1), (1, 1)] {
        let mut x = king_x;
        let mut y = king_y;
        loop {
            x = x.wrapping_add_signed(dx);
            y = y.wrapping_add_signed(dy);
            if !in_bounds(row_size, x, y) {
                break;
            }
            piece_add_threat_if_valid!(BISHOP, x, y);
            piece_add_threat_if_valid!(QUEEN, x, y);
        }
    }
    // King
    for dx in [-1, 0, 1] {
        for dy in [-1, 0, 1] {
            // Make sure we're not hcecking the king against itself, and that we're
            // not in the corner.
            let x = king_x.wrapping_add_signed(dx);
            let y = king_y.wrapping_add_signed(dy);
            if dx == 0 && dy == 0 || xy_to_pos(row_size, x, y) == king_pos {
                continue;
            }
            piece_add_threat_if_valid!(KING, x, y);
        }
    }
    threats
}

pub fn solve(starting_hash: &[u8], start: u32) -> Option<(u32, u32)> {
    let row_size = BOARD_SIZE.isqrt();
    let mut board = BTreeMap::new();
    let mut last_king = None;
    for i in start..MAX_TRIES {
        let e = hash(starting_hash, i);
        let king_id: u8 = Piece::KING.into();
        let p_id: u8 = (e % (king_id as u64 + 1)).try_into().unwrap();
        let p = Piece::try_from(p_id).unwrap();
        let offset: u32 = (e >> 32).try_into().unwrap();
        let pos: u32 = offset % BOARD_SIZE;
        board.insert(pos, (p, i));
        if p == Piece::KING {
            last_king = Some((pos, i));
        }
        if let Some((last_king_pos, last_king_nonce)) = last_king {
            let mut threats = in_check_threats(&board, row_size, last_king_pos);
            if threats.len() >= CHECKS_NEEDED as usize {
                threats.push(last_king_nonce);
                return Some((*threats.iter().min().unwrap(), i));
            }
        }
    }
    None
}

fn hash(x: &[u8], i: u32) -> u64 {
    // The siphasher uses a u32 for the key, but we instead lift the count,
    // which is also a 32 bit number to keep things simple and small.
    let mut w = [0u8; 16];
    w[12..].copy_from_slice(&i.to_be_bytes());
    SipHasher13::new_with_key(&w).hash(x)
}

/* ~~~~~~~~~~~~~~ CONTRACT ENTRYPOINT ~~~~~~~~~~~~~~ */

#[storage]
#[entrypoint]
pub struct Storage {}

#[public]
impl Storage {
    pub fn prove(&self, hash: FixedBytes<32>, from: u32) -> Result<(u32, u32), Vec<u8>> {
        Ok(solve(hash.as_slice(), from).unwrap())
    }
}

/* ~~~~~~~~~~~~~~ ALGORITHM TESTING ~~~~~~~~~~~~~~ */

#[cfg(all(test, not(target_arch = "wasm32")))]
mod test {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_solve(
            checks_needed in 1u32..=4,
            board_size in 100u32..0x1FFFF,
            starting_hash in any::<[u8; 64]>()
        ) {
            let (l, h) = solve(board_size, checks_needed, 1000, &starting_hash, 0).unwrap();
            assert_eq!(
                (l, h),
                solve(board_size, checks_needed, 1000, &starting_hash, l).unwrap()
            );
        }
    }
}
