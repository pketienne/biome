/// Generate a GitHub-style slug from heading text.
///
/// Rules:
/// - Convert to lowercase
/// - Remove anything that is not alphanumeric, space, or hyphen
/// - Replace spaces with hyphens
/// - Collapse consecutive hyphens
pub fn heading_slug(text: &str) -> String {
    let lower = text.to_lowercase();
    let mut slug = String::with_capacity(lower.len());

    for ch in lower.chars() {
        if ch.is_alphanumeric() {
            slug.push(ch);
        } else if ch == ' ' || ch == '-' || ch == '\t' {
            slug.push('-');
        }
        // other characters are simply dropped
    }

    // Collapse consecutive hyphens
    let mut result = String::with_capacity(slug.len());
    let mut prev_hyphen = false;
    for ch in slug.chars() {
        if ch == '-' {
            if !prev_hyphen {
                result.push('-');
            }
            prev_hyphen = true;
        } else {
            result.push(ch);
            prev_hyphen = false;
        }
    }

    // Trim leading/trailing hyphens
    result.trim_matches('-').to_string()
}

/// Extract heading text from an ATX heading line (strip # prefix and optional closing #s).
pub fn extract_atx_heading_text(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('#') {
        return None;
    }

    let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
    if hash_count > 6 {
        return None;
    }

    let after_hashes = &trimmed[hash_count..];
    // Must be followed by space or end of line
    if !after_hashes.is_empty() && !after_hashes.starts_with(' ') && !after_hashes.starts_with('\t')
    {
        return None;
    }

    let content = after_hashes.trim();
    // Remove optional closing #s
    let content = content.trim_end_matches('#').trim_end();
    Some(content.to_string())
}

/// Collect all heading slugs from a document's text (skipping fenced code blocks).
pub fn collect_heading_slugs(text: &str) -> Vec<String> {
    use crate::utils::fence_utils::FenceTracker;

    let mut slugs = Vec::new();
    let mut tracker = FenceTracker::new();

    for (line_idx, line) in text.lines().enumerate() {
        tracker.process_line(line_idx, line);

        if !tracker.is_inside_fence() {
            if let Some(heading_text) = extract_atx_heading_text(line) {
                slugs.push(heading_slug(&heading_text));
            }
        }
    }

    slugs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_slug() {
        assert_eq!(heading_slug("Hello World"), "hello-world");
    }

    #[test]
    fn slug_special_chars() {
        assert_eq!(heading_slug("What's New?"), "whats-new");
        assert_eq!(heading_slug("API v2.0"), "api-v20");
    }

    #[test]
    fn slug_hyphens() {
        assert_eq!(heading_slug("kebab-case-title"), "kebab-case-title");
        assert_eq!(heading_slug("multiple   spaces"), "multiple-spaces");
    }

    #[test]
    fn extract_heading() {
        assert_eq!(
            extract_atx_heading_text("# Hello"),
            Some("Hello".to_string())
        );
        assert_eq!(
            extract_atx_heading_text("## World ##"),
            Some("World".to_string())
        );
        assert_eq!(
            extract_atx_heading_text("### Test"),
            Some("Test".to_string())
        );
    }

    #[test]
    fn extract_heading_no_space() {
        assert_eq!(extract_atx_heading_text("#NoSpace"), None);
    }

    #[test]
    fn extract_not_a_heading() {
        assert_eq!(extract_atx_heading_text("Not a heading"), None);
    }

    #[test]
    fn collect_slugs() {
        let text = "# Hello\n\nSome text\n\n## World\n\n```\n# Not a heading\n```\n";
        let slugs = collect_heading_slugs(text);
        assert_eq!(slugs, vec!["hello", "world"]);
    }
}
