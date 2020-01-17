extern crate cursive;
extern crate cursive_hexview;

use cursive::views::{Dialog, DummyView, LinearLayout, TextView};
use cursive::Cursive;
use cursive_hexview::{DisplayState, HexView};
use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

fn read_file(path: &Path) -> Result<Vec<u8>, io::Error> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

fn main() {
    let arg = env::args()
        .nth(1)
        .expect("Please provide the file to read from as first argument");
    let path = Path::new(&arg);

    let mut cur = Cursive::default();
    let explanation = TextView::new("Use the keys ↑ ↓ ← → to navigate around.\nUse q to exit.");
    let view = HexView::new_from_iter(read_file(path).expect("Cannot read file")).display_state(DisplayState::Enabled);

    cur.add_layer(
        Dialog::around(LinearLayout::vertical().child(explanation).child(DummyView).child(view)).title("HexView"),
    );
    cur.add_global_callback('q', |cur| cur.quit());
    cur.run();
}
