use shogi_core::{Move, PartialPosition, Square};
use shogi_usi_parser::FromUsi;

use shogi_official_kifu::*;

fn main() {
    // An example found in https://www.shogi.or.jp/faq/kihuhyouki.html.
    // A
    let pos = PartialPosition::from_usi("+R8/9/9/1+R7/9/9/9/9/4K1k2 b - 1").unwrap();
    let mv = Move::Normal {
        from: Square::new(9, 1).unwrap(),
        to: Square::new(8, 2).unwrap(),
        promote: false,
    };
    let result = display_single_move(&pos, mv);
    println!("{}", result.unwrap());
    let mv = Move::Normal {
        from: Square::new(8, 4).unwrap(),
        to: Square::new(8, 2).unwrap(),
        promote: false,
    };
    let result = display_single_move(&pos, mv);
    println!("{}", result.unwrap());
}
