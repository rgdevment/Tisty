use super::{DOC, paper_lines, papers};

#[test]
fn the_line_a_page_is_named_on_is_the_line_the_order_is_read_from() {
    let body = format!(
        "# Libro

intro

![Uno]({DOC}a-0001)

medio

![Dos]({DOC}a-0002)
"
    );
    assert_eq!(
        paper_lines(&body),
        vec![("a-0001".to_string(), 4), ("a-0002".to_string(), 8)]
    );
}

#[test]
fn a_page_named_only_inside_a_fence_is_named_on_no_line_at_all() {
    for fence in ["```", "~~~"] {
        let body = format!(
            "# Libro

{fence}md
![Uno]({DOC}a-0001)
{fence}

![Dos]({DOC}a-0002)
"
        );
        assert_eq!(
            papers(&body),
            vec!["a-0002".to_string()],
            "{fence}: a line in code is not a way in"
        );
        assert_eq!(
            paper_lines(&body),
            vec![("a-0002".to_string(), 6)],
            "{fence}: and it is named on no line at all"
        );
    }
}

#[test]
fn what_keeps_a_file_alive_is_read_more_widely_than_what_decides_the_order() {
    let body = "# Libro

~~~md
![Plano](attachments/plano.png)
~~~
";
    assert_eq!(
        super::extract(body)
            .into_iter()
            .map(|one| one.target)
            .collect::<Vec<_>>(),
        vec!["attachments/plano.png".to_string()],
        "a file named anywhere is a file somebody still means to keep"
    );
}

#[test]
fn a_fence_written_inside_a_quote_is_still_code() {
    let body = format!(
        "# Libro

> ```
> ![Uno]({DOC}a-0001)
> ```

![Dos]({DOC}a-0002)
"
    );
    assert_eq!(papers(&body), vec!["a-0002".to_string()]);
}

#[test]
fn a_plain_link_to_a_page_is_a_mention_of_it_not_a_place_for_it() {
    let body = format!(
        "# Libro

Como conte en [Uno]({DOC}a-0001), ya esta.

![Dos]({DOC}a-0002)
"
    );
    assert_eq!(paper_lines(&body), vec![("a-0002".to_string(), 4)]);
    assert_eq!(papers(&body), vec!["a-0002".to_string()]);
    assert_eq!(
        super::extract(&body).len(),
        2,
        "both are still references, which is what keeps what they point at alive"
    );
}

#[test]
fn a_fence_that_closes_wider_than_it_opened_holds_until_it_does() {
    let body = format!(
        "# Libro

````md
```
![Uno]({DOC}a-0001)
```
````

![Dos]({DOC}a-0002)
"
    );
    assert_eq!(papers(&body), vec!["a-0002".to_string()]);
}

#[test]
fn a_page_named_twice_is_named_on_both_lines_and_read_from_the_first() {
    let body = format!(
        "![Uno]({DOC}a-0001)

otra

![Uno]({DOC}a-0001)
"
    );
    assert_eq!(
        paper_lines(&body),
        vec![("a-0001".to_string(), 0), ("a-0001".to_string(), 4)]
    );
    assert_eq!(papers(&body), vec!["a-0001".to_string()]);
}
