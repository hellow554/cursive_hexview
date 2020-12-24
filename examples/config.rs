extern crate cursive;
extern crate cursive_hexview;

use cursive::views::{Dialog, DummyView, LinearLayout, TextView};
use cursive_hexview::{DisplayState, HexView, HexViewConfig};

fn main() {
    let mut cur = cursive::default();
    let explanation = TextView::new("Use the keys + - ↑ ↓ ← → 0-9 a-f for the HexView.\nUse q to exit.");
    let data = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".as_bytes();

    //A view with implicit default config
    let view1 = HexView::new_from_iter(data).display_state(DisplayState::Editable);

    //A view with a single config change, everything else default, chainable version
    let config = HexViewConfig {
        hex_ascii_separator: " - ",
        ..Default::default()
    };
    let view2 = HexView::new_from_iter(data)
        .config(config)
        .display_state(DisplayState::Editable);

    //A view with several config changes, non chainable version
    let mut view3 = HexView::new_from_iter(data).display_state(DisplayState::Editable);
    let config2 = HexViewConfig {
        bytes_per_line: 8,
        bytes_per_group: 4,
        byte_group_separator: "     ",
        ..Default::default()
    };
    view3.set_config(config2);

    cur.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(explanation)
                .child(DummyView)
                .child(view1)
                .child(DummyView)
                .child(view2)
                .child(DummyView)
                .child(view3),
        )
        .title("HexView"),
    );
    cur.add_global_callback('q', |cur| cur.quit());
    cur.run();
}
