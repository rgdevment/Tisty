use tisty_markdown::to_tisty;

#[test]
fn a_pen_the_old_tag_wrote_becomes_tisty_words() {
    assert_eq!(to_tisty("a <mark>b</mark> c"), "a ==b== c");
    assert_eq!(
        to_tisty("a <mark data-pen=\"green\">b</mark> c"),
        "a ==b=={green} c"
    );
    assert_eq!(to_tisty("<mark data-pen=\"yellow\">b</mark>"), "==b==");
}

#[test]
fn what_a_pen_held_keeps_its_own_marks() {
    assert_eq!(
        to_tisty("<mark data-pen=\"blue\">**bold** and [a](https://a.b)</mark>"),
        "==**bold** and [a](https://a.b)=={blue}"
    );
}

#[test]
fn a_space_inside_the_old_tag_moves_out_of_the_walls() {
    assert_eq!(to_tisty("a<mark> b </mark>c"), "a ==b== c");
    assert_eq!(to_tisty("a <mark></mark>c"), "a c");
}

#[test]
fn an_underline_and_an_icon_say_it_the_short_way() {
    assert_eq!(to_tisty("<u>sí</u> y <U>no</U>"), "++sí++ y ++no++");
    assert_eq!(
        to_tisty("<span data-ico=\"dog\" data-hue=\"orange\">:dog:</span> perro"),
        "::dog orange:: perro"
    );
    assert_eq!(
        to_tisty("<span data-ico=\"star\">:star:</span>"),
        "::star::"
    );
}

#[test]
fn a_hue_the_palette_lacks_is_dropped_and_an_unknown_icon_is_left_alone() {
    assert_eq!(
        to_tisty("<span data-ico=\"star\" data-hue=\"chartreuse\">:star:</span>"),
        "::star::"
    );
    let stranger = "<span data-ico=\"nadaquever\">:x:</span>";
    assert_eq!(to_tisty(stranger), stranger);
}

#[test]
fn code_keeps_the_tags_it_shows_as_written() {
    let body = "Write `<mark>x</mark>` like this:\n\n```html\n<u>y</u>\n```\n";
    assert_eq!(to_tisty(body), body);
}

#[test]
fn a_tag_left_open_is_not_guessed_at() {
    let body = "a <mark>b and on\n\nnext";
    assert_eq!(to_tisty(body), body);
}

#[test]
fn tables_lists_and_quotes_turn_too_and_nothing_else_moves() {
    let body = "| a | b |\n| - | - |\n| <mark data-pen=\"pink\">x</mark> | y |\n\n- <u>z</u>\n\n> [!TIP]\n> <span data-ico=\"leaf\" data-hue=\"green\">:leaf:</span> ok\n";
    assert_eq!(
        to_tisty(body),
        "| a | b |\n| - | - |\n| ==x=={pink} | y |\n\n- ++z++\n\n> [!TIP]\n> ::leaf green:: ok\n"
    );
}

#[test]
fn turning_twice_is_turning_once() {
    let body = "<mark data-pen=\"green\">a</mark> <u>b</u> <span data-ico=\"star\">:star:</span>";
    let once = to_tisty(body);
    assert_eq!(to_tisty(&once), once);
}
