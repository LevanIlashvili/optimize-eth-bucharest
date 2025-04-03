extern crate alloc;

use stylus_sdk::{alloy_primitives::*, prelude::*};

use alloc::{vec, vec::Vec};

use libbucharesthashing::{immutables::*, prover, prover::Piece};

pub struct BoardEntry {
    piece: Option<(Piece, u32)>,
}

pub struct Board {
    entries: Vec<BoardEntry>,
    size: u32,
    kings: Vec<(u32, u32)>,
    potential_threats: Vec<(u32, Piece)>,
}

impl Board {
    fn new(size: u32) -> Self {
        let mut entries = Vec::with_capacity(size as usize);
        for _ in 0..size {
            entries.push(BoardEntry { piece: None });
        }
        let kings_capacity = 4;         
        let threats_capacity = (size / 2) as usize;

        Board {
            entries,
            size,
            kings: Vec::with_capacity(kings_capacity),
            potential_threats: Vec::with_capacity(threats_capacity),
        }
    }

    fn insert(&mut self, pos: u32, piece: Piece, nonce: u32) {
        if pos < self.size {
            self.entries[pos as usize].piece = Some((piece, nonce));
            
            if piece == Piece::KING {
                self.kings.push((pos, nonce));
            } else {
                self.potential_threats.push((pos, piece));
            }
        }
    }

    fn get(&self, pos: u32) -> Option<&(Piece, u32)> {
        if pos < self.size {
            self.entries[pos as usize].piece.as_ref()
        } else {
            None
        }
    }
    
    fn latest_king(&self) -> Option<(u32, u32)> {
        self.kings.last().copied()
    }
}

fn pos_to_xy(row_size: u32, p: u32) -> (u32, u32) {
    (p % row_size, p / row_size)
}

fn xy_to_pos(row_size: u32, x: u32, y: u32) -> u32 {
    y.wrapping_mul(row_size).wrapping_add(x)
}

fn in_bounds(row_size: u32, x: u32, y: u32) -> bool {
    x < row_size && y < row_size
}

fn is_path_clear_straight(board: &Board, row_size: u32, x1: u32, y1: u32, x2: u32, y2: u32) -> bool {
    if x1 == x2 { // Vertical
        let start = if y1 < y2 { y1 + 1 } else { y2 + 1 };
        let end = if y1 < y2 { y2 } else { y1 };
        for y in start..end {
            if board.get(xy_to_pos(row_size, x1, y)).is_some() {
                return false;
            }
        }
    } else { // Horizontal (y1 == y2 assumed)
        let start = if x1 < x2 { x1 + 1 } else { x2 + 1 };
        let end = if x1 < x2 { x2 } else { x1 };
        for x in start..end {
            if board.get(xy_to_pos(row_size, x, y1)).is_some() {
                return false;
            }
        }
    }
    true
}

fn is_path_clear_diagonal(board: &Board, row_size: u32, x1: u32, y1: u32, x2: u32, y2: u32) -> bool {
    let step_x = if x1 < x2 { 1 } else { -1 };
    let step_y = if y1 < y2 { 1 } else { -1 };

    let mut x = x1 as i32 + step_x;
    let mut y = y1 as i32 + step_y;

    while (x as u32) != x2 || (y as u32) != y2 {
        if !in_bounds(row_size, x as u32, y as u32) { // Should not happen if dx==dy, but safe check
             break;
        }
        let check_pos = xy_to_pos(row_size, x as u32, y as u32);
        if board.get(check_pos).is_some() {
            return false;
        }
        x += step_x;
        y += step_y;
    }
    true
}

fn in_check_threats(board: &Board, row_size: u32, king_pos: u32) -> Vec<u32> {
    let (king_x, king_y) = pos_to_xy(row_size, king_pos);
    let needed_threats = CHECKS_NEEDED as usize;
    let mut threats = Vec::with_capacity(needed_threats);
    
    macro_rules! add_threat_and_check {
        ($nonce:expr) => {
            threats.push($nonce);
            if threats.len() >= needed_threats {
                return threats;
            }
        };
    }
    
    for &(pos, piece) in &board.potential_threats {
        match piece {
            Piece::PAWN => {
                let (piece_x, piece_y) = pos_to_xy(row_size, pos);
                if (king_y == piece_y + 1) && 
                   ((king_x == piece_x + 1) || (king_x == piece_x - 1)) {
                    if let Some((_, n)) = board.get(pos) {
                        add_threat_and_check!(*n);
                    }
                }
            },
            Piece::KNIGHT => {
                let (piece_x, piece_y) = pos_to_xy(row_size, pos);
                let dx = if piece_x > king_x { piece_x - king_x } else { king_x - piece_x };
                let dy = if piece_y > king_y { piece_y - king_y } else { king_y - piece_y };
                
                if (dx == 1 && dy == 2) || (dx == 2 && dy == 1) {
                    if let Some((_, n)) = board.get(pos) {
                        add_threat_and_check!(*n);
                    }
                }
            },
            Piece::BISHOP => {
                let (piece_x, piece_y) = pos_to_xy(row_size, pos);
                let dx = if piece_x > king_x { piece_x - king_x } else { king_x - piece_x };
                let dy = if piece_y > king_y { piece_y - king_y } else { king_y - piece_y };
                
                if dx == dy {
                    if is_path_clear_diagonal(board, row_size, king_x, king_y, piece_x, piece_y) {
                        if let Some((_, n)) = board.get(pos) {
                            add_threat_and_check!(*n);
                        }
                    }
                }
            },
            Piece::CASTLE => {
                let (piece_x, piece_y) = pos_to_xy(row_size, pos);
                
                if piece_x == king_x || piece_y == king_y {
                     if is_path_clear_straight(board, row_size, king_x, king_y, piece_x, piece_y) {
                        if let Some((_, n)) = board.get(pos) {
                            add_threat_and_check!(*n);
                        }
                    }
                }
            },
            Piece::QUEEN => {
                let (piece_x, piece_y) = pos_to_xy(row_size, pos);
                let dx = if piece_x > king_x { piece_x - king_x } else { king_x - piece_x };
                let dy = if piece_y > king_y { piece_y - king_y } else { king_y - piece_y };
                
                let mut clear_path = false;
                if piece_x == king_x || piece_y == king_y {
                    clear_path = is_path_clear_straight(board, row_size, king_x, king_y, piece_x, piece_y);
                } else if dx == dy {
                    clear_path = is_path_clear_diagonal(board, row_size, king_x, king_y, piece_x, piece_y);
                }
                
                if clear_path {
                    if let Some((_, n)) = board.get(pos) {
                        add_threat_and_check!(*n);
                    }
                }
            },
            Piece::KING => {
                let (piece_x, piece_y) = pos_to_xy(row_size, pos);
                let dx = if piece_x > king_x { piece_x - king_x } else { king_x - piece_x };
                let dy = if piece_y > king_y { piece_y - king_y } else { king_y - piece_y };
                
                if dx <= 1 && dy <= 1 && (dx > 0 || dy > 0) {
                    if let Some((_, n)) = board.get(pos) {
                        add_threat_and_check!(*n);
                    }
                }
            },
        }
    }
    
    threats
}

pub fn solve(starting_hash: &[u8], start: u32) -> Option<(u32, u32)> {
    let row_size = BOARD_SIZE.isqrt();
    let mut board = Board::new(BOARD_SIZE);
    
    for i in start..MAX_TRIES {
        let e = prover::hash(starting_hash, i);
        let king_id: u8 = Piece::KING.into();
        let p_id: u8 = (e % (king_id as u64 + 1)).try_into().unwrap();
        let p = Piece::try_from(p_id).unwrap();
        let offset: u32 = (e >> 32).try_into().unwrap();
        let pos: u32 = offset % BOARD_SIZE;
        
        board.insert(pos, p, i);
        
        if p == Piece::KING {
            if let Some((king_pos, king_nonce)) = board.latest_king() {
                let mut threats = in_check_threats(&board, row_size, king_pos);
                if threats.len() >= CHECKS_NEEDED as usize {
                    threats.push(king_nonce);
                    return Some((*threats.iter().min().unwrap(), i));
                }
            }
        } 
        else if let Some((king_pos, king_nonce)) = board.latest_king() {
            let (king_x, king_y) = pos_to_xy(row_size, king_pos);
            let (piece_x, piece_y) = pos_to_xy(row_size, pos);
            
            let dx = if piece_x > king_x { piece_x - king_x } else { king_x - piece_x };
            let dy = if piece_y > king_y { piece_y - king_y } else { king_y - piece_y };
            
            let is_potential_threat = match p {
                Piece::PAWN => king_y == piece_y + 1 && (king_x == piece_x + 1 || king_x == piece_x - 1),
                Piece::KNIGHT => (dx == 1 && dy == 2) || (dx == 2 && dy == 1),
                Piece::BISHOP => dx == dy,
                Piece::CASTLE => piece_x == king_x || piece_y == king_y,
                Piece::QUEEN => piece_x == king_x || piece_y == king_y || dx == dy,
                Piece::KING => dx <= 1 && dy <= 1,
            };
            
            if is_potential_threat {
                let mut threats = in_check_threats(&board, row_size, king_pos);
                if threats.len() >= CHECKS_NEEDED as usize {
                    threats.push(king_nonce);
                    return Some((*threats.iter().min().unwrap(), i));
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
        Ok(solve(hash.as_slice(), from).unwrap())
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod test {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_solve(starting_hash in any::<[u8; 64]>()) {
            let (e_l, e_h) = solve(&starting_hash, 0).unwrap();
            let (t_l, t_h) = solve(&starting_hash, e_l).unwrap();
            assert_eq!((e_l, e_h), (t_l, t_h), "user contract not consistent. {e_l} != {t_l} or {e_h} != {t_h}");
            let (p_l, p_h) = prover::default_solve(&starting_hash, e_l).unwrap();
            assert_eq!(
                (e_l, e_h), (p_l, p_h),
                "user contract inconsistent with reference. {e_l} != {p_l} or {e_h} != {p_h}"
            );
        }
    }
}