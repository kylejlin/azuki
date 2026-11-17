use super::layout::*;

struct DecoratedLine<'a> {
    before: Vec<PlacedComment<'a>>,
    trailing: Vec<PlacedComment<'a>>,
    blank_before: bool,
    blank_after_comments: bool,
}

struct PlacedComment<'a> {
    raw: &'a str,
    blank_before: bool,
}

pub(super) fn render(layout: Layout<'_>) -> String {
    let mut lines: Vec<_> = layout
        .lines
        .iter()
        .map(|_| DecoratedLine {
            before: Vec::new(),
            trailing: Vec::new(),
            blank_before: false,
            blank_after_comments: false,
        })
        .collect();
    let mut token_lines = vec![0; layout.tokens.len()];
    for (index, line) in layout.lines.iter().enumerate() {
        token_lines[line.start..line.end].fill(index);
    }

    for (index, token) in layout.tokens.iter().enumerate() {
        let right = token_lines[index];
        let line = &layout.lines[right];
        let left = index.checked_sub(1).map(|left| token_lines[left]);
        let mut has_leading = false;
        for comment in &token.leading.comments {
            let placed = PlacedComment {
                raw: comment.raw,
                blank_before: comment.blank_before && !line.follows_prefix,
            };
            if let Some(left) =
                left.filter(|&left| comment.trailing && layout.lines[left].end == index)
            {
                lines[left].trailing.push(placed);
            } else {
                // Keep comments above the header itself, after any Rust
                // prefix. This preserves both comment order and trailing
                // comments on the prefix without adding an empty line.
                lines[right].before.push(placed);
                has_leading = true;
            }
        }
        if index == line.start && !line.follows_prefix {
            if has_leading {
                lines[right].blank_after_comments = token.leading.blank_before;
            } else {
                lines[right].blank_before = token.leading.blank_before;
            }
        }
    }

    let mut end_comments = Vec::new();
    for comment in &layout.end.comments {
        let placed = PlacedComment {
            raw: comment.raw,
            blank_before: comment.blank_before,
        };
        if comment.trailing
            && let Some(last) = lines.last_mut()
        {
            last.trailing.push(placed);
        } else {
            end_comments.push(placed);
        }
    }

    let mut out = String::new();
    for (line, decorated) in layout.lines.iter().zip(lines) {
        for (index, comment) in decorated.before.iter().enumerate() {
            push_line(
                &mut out,
                comment.raw,
                line.comment_indent,
                comment.blank_before || (index == 0 && decorated.blank_before),
            );
        }
        let mut text = line_text(&layout.tokens[line.start..line.end]);
        for comment in decorated.trailing {
            text.push(' ');
            text.push_str(comment.raw);
        }
        let blank_before = if decorated.before.is_empty() {
            decorated.blank_before
        } else {
            decorated.blank_after_comments
        };
        push_line(&mut out, &text, line.indent, blank_before);
    }
    for comment in end_comments {
        push_line(&mut out, comment.raw, 0, comment.blank_before);
    }
    out
}

fn push_line(out: &mut String, text: &str, indent: usize, blank_before: bool) {
    if blank_before && !out.is_empty() && !out.ends_with("\n\n") {
        out.push('\n');
    }
    out.extend(std::iter::repeat_n(' ', indent * 4));
    out.push_str(text);
    out.push('\n');
}
