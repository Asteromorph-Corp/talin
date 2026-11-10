//! [Hashline](https://github.com/kebbbnnn/hashline) rendering and line-addressed partial updates.

/// Reserved hash for the virtual end-of-file anchor that terminates every [`hashline_view`].
/// Real anchors are 4 hex chars, so this never collides with a content line.
const EOF_HASH: &str = "eof";

/// A short content hash used as a hashline anchor: 4 hex chars over the trimmed line.
/// Empty lines are folded to a reserved token.
fn line_hash(line: &str) -> String {
    let trimmed = line.trim();
    let basis = if trimmed.is_empty() {
        "<empty>"
    } else {
        trimmed
    };
    let mut hash: u32 = 0x811c_9dc5;
    for b in basis.bytes() {
        hash ^= b as u32;
        hash = hash.wrapping_mul(0x0100_0193);
    }
    format!("{:04x}", hash & 0xffff)
}

/// Renders `content` in [hashline](https://github.com/kebbbnnn/hashline) form: each line as
/// `linenum:hash| text`, with 1-based line numbers, followed by a trailing end-of-file line.
pub(super) fn hashline_view(content: &str) -> String {
    let mut lines: Vec<String> = if content.is_empty() {
        Vec::new()
    } else {
        content
            .split('\n')
            .enumerate()
            .map(|(i, line)| format!("{}:{}| {line}", i + 1, line_hash(line)))
            .collect()
    };
    lines.push(format!("{}:{EOF_HASH}| <eof>", lines.len() + 1));
    lines.join("\n")
}

/// Applies a hashline edit: replaces the inclusive line range `[start, end]` with `content`.
pub(super) fn apply_hashline(
    buffer: &str,
    start: &str,
    end: &str,
    content: &str,
) -> Result<String, String> {
    let lines: Vec<&str> = if buffer.is_empty() {
        Vec::new()
    } else {
        buffer.split('\n').collect()
    };
    // The view appends a virtual end-of-file line at this index; it is addressable but never part
    // of the output, so a range ending at it simply drops the tail.
    let eof_idx = lines.len();
    let start_idx = resolve_anchor(&lines, start, eof_idx)?;
    let end_idx = resolve_anchor(&lines, end, eof_idx)?;
    if end_idx < start_idx {
        return Err(format!(
            "`end` anchor `{end}` precedes `start` anchor `{start}`"
        ));
    }

    let mut out: Vec<&str> = Vec::with_capacity(lines.len());
    out.extend_from_slice(&lines[..start_idx]);
    out.extend(content.split('\n'));
    out.extend_from_slice(&lines[(end_idx + 1).min(lines.len())..]);
    Ok(out.join("\n"))
}

/// Resolves a `linenum:hash` anchor to a 0-based line index, picking the matching line nearest to
/// the hinted line number (proximity search). The reserved [`EOF_HASH`] resolves to `eof_idx`, the
/// virtual line one past the end of `lines`.
fn resolve_anchor(lines: &[&str], anchor: &str, eof_idx: usize) -> Result<usize, String> {
    let (num, hash) = anchor
        .split_once(':')
        .ok_or_else(|| format!("invalid anchor `{anchor}` (expected `linenum:hash`)"))?;
    let hint = num
        .trim()
        .parse::<usize>()
        .map_err(|_| format!("invalid line number in anchor `{anchor}`"))?
        .saturating_sub(1);
    let hash = hash.trim();

    if hash == EOF_HASH {
        return Ok(eof_idx);
    }

    lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line_hash(line) == hash)
        .min_by_key(|(i, _)| i.abs_diff(hint))
        .map(|(i, _)| i)
        .ok_or_else(|| format!("no line matches anchor `{anchor}`"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_range() {
        let buffer = "alpha\nbeta\ngamma";
        let start = format!("2:{}", line_hash("beta"));

        assert_eq!(
            apply_hashline(buffer, &start, &start, "BETA").unwrap(),
            "alpha\nBETA\ngamma"
        );
    }

    #[test]
    fn replaces_multiline_range() {
        let buffer = "alpha\nbeta\ngamma\ndelta";
        let start = format!("2:{}", line_hash("beta"));
        let end = format!("3:{}", line_hash("gamma"));

        assert_eq!(
            apply_hashline(buffer, &start, &end, "one\ntwo").unwrap(),
            "alpha\none\ntwo\ndelta"
        );
    }

    #[test]
    fn errors_on_unknown_anchor() {
        let buffer = "alpha\nbeta";

        assert!(apply_hashline(buffer, "1:zzzz", "1:zzzz", "x").is_err());
    }

    #[test]
    fn view_formats_anchors() {
        let view = hashline_view("alpha\nbeta");
        let lines: Vec<&str> = view.lines().collect();

        assert!(lines[0].starts_with("1:") && lines[0].ends_with("| alpha"));
        assert!(lines[1].starts_with("2:") && lines[1].ends_with("| beta"));
        assert_eq!(lines[2], "3:eof| <eof>");
    }

    #[test]
    fn empty_view_is_eof_only() {
        assert_eq!(hashline_view(""), "1:eof| <eof>");
    }

    #[test]
    fn appends_into_empty_buffer_at_eof() {
        assert_eq!(
            apply_hashline("", "1:eof", "1:eof", "hello\nworld").unwrap(),
            "hello\nworld"
        );
    }

    #[test]
    fn appends_at_eof() {
        assert_eq!(
            apply_hashline("alpha\nbeta", "3:eof", "3:eof", "gamma").unwrap(),
            "alpha\nbeta\ngamma"
        );
    }

    #[test]
    fn replaces_range_through_eof() {
        let start = format!("2:{}", line_hash("beta"));

        assert_eq!(
            apply_hashline("alpha\nbeta", &start, "3:eof", "BETA").unwrap(),
            "alpha\nBETA"
        );
    }
}
