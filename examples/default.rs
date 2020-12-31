extern crate cursive;
extern crate cursive_hexview;

use cursive::views::{Dialog, DummyView, LinearLayout, TextView};
use cursive_hexview::{DisplayState, HexView};

fn main() {
    let mut cur = cursive::default();
    let explanation = TextView::new("Use the keys + - ↑ ↓ ← → 0-9 a-f for the HexView.\nUse q to exit.");
    let view = HexView::new().display_state(DisplayState::Editable);

    cur.add_layer(
        Dialog::around(LinearLayout::vertical().child(explanation).child(DummyView).child(view)).title("HexView"),
    );
    cur.add_global_callback('q', |cur| cur.quit());
    cur.run();
}
