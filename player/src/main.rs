mod parser;

use resvg::{tiny_skia, usvg};
use crate::parser::parse;

fn main() {
    let d = parse("<meta-ente><fps>24</fps><ente>true</ente></meta-ente>\
    <frames>\
        <frame>\
            <svg version='1.1' xmlns='http://www.w3.org/2000/svg' width='720' height='720'>\
                <rect x='0' y='0' width='720' height='720' fill='red' />
            </svg>\
        </frame>\
    </frames>");

    let tree = usvg::Tree::from_data(
        d.frames[0].as_bytes(),
        &usvg::Options::default()
    ).unwrap();

    let size = tree.size();
    let mut pixmap = tiny_skia::Pixmap::new(
        size.width() as u32,
        size.height() as u32
    ).unwrap();

    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
    tiny_skia::Pixmap::save_png(&pixmap, "test.png").unwrap();
}

