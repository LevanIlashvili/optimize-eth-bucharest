extern crate alloc;

use stylus_sdk::{alloy_primitives::*, prelude::*};

use alloc::vec;
use alloc::vec::Vec;

use libbucharesthashing::{immutables::*, prover, prover::Piece};

use std::collections::HashSet;

pub type Board = Vec<(u32, Piece, u32)>;

fn pos_to_xy(row_size: u32, p: u32) -> (u32, u32) {
    (p % row_size, p / row_size)
}

fn is_solved(row_size: u32, king_pos: u32, piece_pos: u32, piece: Piece) -> bool {
    let (king_x, king_y) = pos_to_xy(row_size, king_pos);
    let (piece_x, piece_y) = pos_to_xy(row_size, piece_pos);

    let dx = if king_x > piece_x {
        king_x - piece_x
    } else {
        piece_x - king_x
    };
    let dy = if king_y > piece_y {
        king_y - piece_y
    } else {
        piece_y - king_y
    };

    if dx == 0 && dy == 0 {
        return false;
    }

    match piece {
        Piece::PAWN => piece_y + 1 == king_y && dx == 1,
        Piece::CASTLE => dx == 0 || dy == 0,
        Piece::QUEEN => dx == 0 || dy == 0 || dx == dy,
        Piece::BISHOP => dx == dy,
        Piece::KNIGHT => dx * dy == 2,
        Piece::KING => dx <= 1 && dy <= 1,
    }
}

pub fn solve(starting_hash: &[u8], start: u32) -> Option<(u32, u32)> {
    let row_size = BOARD_SIZE.isqrt();
    let mut board: Board = Vec::new();
    let mut last_king: Option<(u32, u32)> = None; 

    for i in start..MAX_TRIES {
        let e = prover::hash(starting_hash, i);
        let king_id: u8 = Piece::KING.into();
        let p_id: u8 = (e % (king_id as u64 + 1)).try_into().unwrap();
        let p = Piece::try_from(p_id).unwrap();
        let offset: u32 = (e >> 32).try_into().unwrap();
        let pos: u32 = offset % BOARD_SIZE;

        board.push((i, p, pos));

        if p == Piece::KING {
            last_king = Some((pos, i));
        }

        if let Some((current_king_pos, current_king_nonce)) = last_king {
            // Simulate BTreeMap overwrite by only considering the latest piece at each position
            let mut threats: Vec<u32> = Vec::new();
            let mut visited_pos: HashSet<u32> = HashSet::new();

            for &(nonce, piece, piece_pos) in board.iter().rev() {
                // If we've already processed a newer piece at this position, skip
                if visited_pos.contains(&piece_pos) {
                    continue;
                }
                visited_pos.insert(piece_pos);

                // Check if this (latest) piece at piece_pos threatens the king
                if nonce != current_king_nonce && is_solved(row_size, current_king_pos, piece_pos, piece) {
                    threats.push(nonce);
                }
            }

            if threats.len() >= CHECKS_NEEDED as usize {
                threats.push(current_king_nonce);

                if let Some(first_threat) = threats.iter().min() {
                    return Some((*first_threat, i));
                }
            }
        }
    }
    None
}

#[storage]
#[entrypoint]
pub struct Storage {}

#[public]
impl Storage {
    pub fn prove(&self, hash: FixedBytes<32>, from: u32) -> Result<(u32, u32), Vec<u8>> {
        match solve(hash.as_slice(), from) {
             Some(result) => Ok(result),
             None => Err(vec![])
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod test {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig { cases: 5000, ..Default::default() })]
        #[test]
        fn test_solve(starting_hash in prop::array::uniform32(any::<u8>())) {
            if let Some((e_l, e_h)) = solve(&starting_hash, 0) {
                
                if let Some((t_l, t_h)) = solve(&starting_hash, e_l) {
                   assert_eq!((e_l, e_h), (t_l, t_h), "user contract not consistent. {e_l} != {t_l} or {e_h} != {t_h}");
                } else {
                   panic!("Second solve failed where first succeeded");
                }

                if let Some((p_l, p_h)) = prover::default_solve(&starting_hash, e_l) {
                    assert_eq!(
                        (e_l, e_h), (p_l, p_h),
                        "user contract inconsistent with reference. {e_l} != {p_l} or {e_h} != {p_h}"
                    );
                } else {
                    panic!("Reference solve failed where user solve succeeded");
                }
            }
        }
    }
}