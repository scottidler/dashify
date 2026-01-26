use regex::Regex;

/// Options for controlling dashify behavior
#[derive(Debug, Clone, Default)]
pub struct DashifyOptions {
    /// When true, convert underscores to dashes
    pub force_dash: bool,
}

/// Dashify a filename according to the specification.
/// This only transforms the filename, not the path.
pub fn dashify(filename: &str, options: &DashifyOptions) -> String {
    // Check for early exit conditions
    if should_leave_alone(filename, options) {
        return filename.to_string();
    }

    // Split into name and extension(s)
    let (name, ext) = split_name_and_extension(filename);

    // Process the name part
    let processed = process_name(&name, options);

    // Rejoin with extension (lowercased)
    if ext.is_empty() {
        processed
    } else {
        format!("{}.{}", processed, ext.to_lowercase())
    }
}

/// Determine if a filename should be left completely alone
fn should_leave_alone(filename: &str, options: &DashifyOptions) -> bool {
    // Leading or trailing spaces - leave alone
    if filename.starts_with(' ') || filename.ends_with(' ') {
        return true;
    }

    // Dunder pattern: __word__.ext
    if is_dunder_pattern(filename) {
        return true;
    }

    // Contains non-ASCII (unicode) - leave alone
    if !filename.is_ascii() {
        return true;
    }

    // Hidden file without other issues (starts with . and rest is clean)
    if let Some(rest) = filename.strip_prefix('.') {
        if !filename.contains(' ') {
            // If the rest is clean (no spaces, no uppercase that needs handling), leave alone
            if is_clean_name(rest) && !has_camel_case(rest.split('.').next().unwrap_or("")) {
                return true;
            }
        }
    }

    // Extension-only files like ".txt"
    if filename.starts_with('.') && !filename[1..].contains('.') && is_clean_name(&filename[1..]) {
        return true;
    }

    // Multiple dots only like "...txt"
    if filename.chars().filter(|&c| c != '.').count() == 0
        || (filename.starts_with('.') && filename[1..].chars().all(|c| c == '.' || c.is_ascii_lowercase()))
    {
        let non_dot_part: String = filename.chars().filter(|&c| c != '.').collect();
        if non_dot_part.chars().all(|c| c.is_ascii_lowercase()) && !filename.contains(' ') {
            // Check if it's something like "...txt" - leave alone
            if filename.starts_with("...") {
                return true;
            }
        }
    }

    // Files that are already clean (lowercase, proper separators, no issues)
    if is_already_clean(filename, options) {
        return true;
    }

    // ALL CAPS filenames without spaces (like README.md, CHANGELOG.md)
    if is_all_caps_filename(filename) {
        return true;
    }

    // Semver-style files like v2.0.1-release.txt
    if is_semver_style(filename) {
        return true;
    }

    // tar.gz style files
    if filename.ends_with(".tar.gz") || filename.ends_with(".tar.bz2") || filename.ends_with(".tar.xz") {
        let name_part = filename.rsplit_once('.').map(|(n, _)| n).unwrap_or(filename);
        let name_part = name_part.rsplit_once('.').map(|(n, _)| n).unwrap_or(name_part);
        if is_clean_name(name_part) {
            return true;
        }
    }

    false
}

/// Check if filename matches dunder pattern: __word__.ext or just __word__
fn is_dunder_pattern(filename: &str) -> bool {
    let name = filename.split('.').next().unwrap_or(filename);
    if name.starts_with("__") && name.ends_with("__") && name.len() > 4 {
        let inner = &name[2..name.len() - 2];
        // Inner part should be a simple word (letters/numbers/underscores but not starting/ending with _)
        if !inner.is_empty()
            && !inner.starts_with('_')
            && !inner.ends_with('_')
            && inner.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return true;
        }
    }
    false
}

/// Check if a name is "clean" (lowercase alphanumeric with dashes/underscores/dots)
fn is_clean_name(name: &str) -> bool {
    if name.is_empty() {
        return true;
    }
    name.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_' || c == '.')
}

/// Check if filename is already properly dashified
fn is_already_clean(filename: &str, options: &DashifyOptions) -> bool {
    // Must not have any uppercase, spaces, or special chars that need conversion
    if filename.chars().any(|c| c.is_ascii_uppercase()) {
        return false;
    }

    // If force_dash is enabled, any underscore means it needs processing
    if options.force_dash && filename.contains('_') {
        return false;
    }

    // Must not have consecutive dashes or underscores
    if filename.contains("--") || filename.contains("___") {
        return false;
    }

    // Must not have consecutive dots (except at start like "...txt")
    if filename.contains("..") && !filename.starts_with("..") {
        return false;
    }

    // Must not have spaces or special chars
    let bad_chars = [
        ' ', '+', ',', '(', ')', '[', ']', '{', '}', '\'', '"', '@', '#', '$', '%', '&', '!',
    ];
    if filename.chars().any(|c| bad_chars.contains(&c)) {
        return false;
    }

    // Must not have mixed separator sequences like -_-
    if filename.contains("-_") || filename.contains("_-") {
        return false;
    }

    // Must not have camel case or numbers needing separation
    let name_part = filename.split('.').next().unwrap_or(filename);
    if has_letter_number_transition(name_part) {
        return false;
    }

    true
}

/// Check if the filename part (before extension) is ALL CAPS
fn is_all_caps_filename(filename: &str) -> bool {
    let name = filename.split('.').next().unwrap_or(filename);
    !name.is_empty()
        && name.chars().all(|c| c.is_ascii_uppercase() || c == '_' || c == '-')
        && name.chars().any(|c| c.is_ascii_uppercase())
        && !filename.contains(' ')
}

/// Check if file looks like semver style (v1.2.3-something.txt)
fn is_semver_style(filename: &str) -> bool {
    let re = Regex::new(r"^v?\d+\.\d+(\.\d+)?(-[a-z0-9-]+)?\.[a-z]+$").unwrap();
    re.is_match(filename)
}

/// Check if name contains camelCase pattern
fn has_camel_case(name: &str) -> bool {
    let chars: Vec<char> = name.chars().collect();
    for i in 1..chars.len() {
        if chars[i].is_ascii_uppercase() && chars[i - 1].is_ascii_lowercase() {
            return true;
        }
    }
    false
}

/// Check if there's a letter-number or number-letter transition that needs dashing
fn has_letter_number_transition(name: &str) -> bool {
    let chars: Vec<char> = name.chars().collect();
    for i in 1..chars.len() {
        let prev = chars[i - 1];
        let curr = chars[i];
        if (prev.is_ascii_alphabetic() && curr.is_ascii_digit())
            || (prev.is_ascii_digit() && curr.is_ascii_alphabetic())
        {
            // Check if there's already a separator
            return true;
        }
    }
    false
}

/// Split filename into name and extension
/// Handles cases like .hidden_file, file.tar.gz, etc.
fn split_name_and_extension(filename: &str) -> (String, String) {
    // Handle hidden files
    if let Some(rest) = filename.strip_prefix('.') {
        if let Some(dot_pos) = rest.rfind('.') {
            let name = format!(".{}", &rest[..dot_pos]);
            let ext = &rest[dot_pos + 1..];
            return (name, ext.to_string());
        } else {
            return (filename.to_string(), String::new());
        }
    }

    // Handle regular files
    if let Some(dot_pos) = filename.rfind('.') {
        if dot_pos > 0 {
            let name = &filename[..dot_pos];
            let ext = &filename[dot_pos + 1..];
            return (name.to_string(), ext.to_string());
        }
    }

    (filename.to_string(), String::new())
}

/// Process the name part of a filename
fn process_name(name: &str, options: &DashifyOptions) -> String {
    // Handle hidden file prefix
    let (prefix, working_name) = if let Some(rest) = name.strip_prefix('.') { (".", rest) } else { ("", name) };

    // Check if original ends with a separator (to preserve trailing separators)
    let original_ends_with_separator = working_name.ends_with('-') || working_name.ends_with('_');

    let mut processed = working_name.to_string();

    // Step 1: Split CamelCase/PascalCase (before other transformations)
    processed = split_camel_case(&processed);

    // Step 2: Split number transitions
    processed = split_numbers(&processed);

    // Step 3: Remove brackets and braces
    processed = processed.replace(['[', ']', '{', '}'], "");

    // Step 4: Replace special chars with dashes
    let dash_chars = [' ', '+', ',', '(', ')', '\'', '"', '@', '#', '$', '%', '&', '!'];
    for c in dash_chars {
        processed = processed.replace(c, "-");
    }

    // Step 4.5: If force_dash, convert underscores to dashes
    if options.force_dash {
        processed = processed.replace('_', "-");
    }

    // Step 5: Handle underscore collapsing (but preserve single underscores)
    // First collapse mixed separator sequences
    processed = collapse_mixed_separators(&processed);

    // Step 6: Collapse multiple dashes
    let re_dashes = Regex::new(r"-+").unwrap();
    processed = re_dashes.replace_all(&processed, "-").to_string();

    // Step 7: Collapse multiple underscores (only if not force_dash)
    if !options.force_dash {
        let re_underscores = Regex::new(r"_+").unwrap();
        processed = re_underscores.replace_all(&processed, "_").to_string();
    }

    // Step 8: Collapse double dots
    while processed.contains("..") {
        processed = processed.replace("..", ".");
    }

    // Step 9: Trim trailing dashes/underscores ONLY if original didn't end with one
    if !original_ends_with_separator {
        processed = processed.trim_end_matches('-').to_string();
        processed = processed.trim_end_matches('_').to_string();
    }

    // Step 10: Lowercase everything
    processed = processed.to_lowercase();

    format!("{}{}", prefix, processed)
}

/// Split CamelCase and PascalCase into dash-separated words
fn split_camel_case(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.is_empty() {
        return String::new();
    }

    let mut result = String::new();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if i == 0 {
            result.push(c);
            i += 1;
            continue;
        }

        let prev = chars[i - 1];

        // Transition from lowercase to uppercase
        if prev.is_ascii_lowercase() && c.is_ascii_uppercase() {
            // Count how many lowercase chars came before
            let mut lowercase_count = 0;
            for j in (0..i).rev() {
                if chars[j].is_ascii_lowercase() {
                    lowercase_count += 1;
                } else {
                    break;
                }
            }

            // Only split if there were 2+ lowercase chars before (handles iPhone case)
            if lowercase_count >= 2 {
                result.push('-');
            }
            result.push(c);
        }
        // Transition from uppercase to uppercase+lowercase (like XMLParser -> XML-Parser)
        else if prev.is_ascii_uppercase()
            && c.is_ascii_uppercase()
            && i + 1 < chars.len()
            && chars[i + 1].is_ascii_lowercase()
        {
            result.push('-');
            result.push(c);
        }
        // Transition from uppercase sequence to lowercase (end of acronym)
        else if prev.is_ascii_uppercase() && c.is_ascii_lowercase() {
            // Check if we need to insert dash before previous char
            // This is handled by the case above
            result.push(c);
        } else {
            result.push(c);
        }

        i += 1;
    }

    result
}

/// Split numbers from letters with dashes
fn split_numbers(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.is_empty() {
        return String::new();
    }

    let mut result = String::new();

    for i in 0..chars.len() {
        let c = chars[i];

        if i > 0 {
            let prev = chars[i - 1];

            // Letter to digit transition
            if prev.is_ascii_alphabetic() && c.is_ascii_digit() {
                // Don't add dash if previous char is already a separator
                if prev != '-' && prev != '_' {
                    result.push('-');
                }
            }
            // Digit to letter transition
            else if prev.is_ascii_digit() && c.is_ascii_alphabetic() {
                // Don't add dash if previous char is already a separator
                result.push('-');
            }
        }

        result.push(c);
    }

    result
}

/// Collapse mixed sequences of separators (dash, underscore, space combinations)
fn collapse_mixed_separators(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '-' || c == '_' {
            // Look ahead to see if we have a mixed sequence
            let mut has_dash = c == '-';

            while let Some(&next) = chars.peek() {
                if next == '-' || next == '_' {
                    if next == '-' {
                        has_dash = true;
                    }
                    chars.next();
                } else {
                    break;
                }
            }

            // If we have any dashes in the sequence, output dash
            // If we have only underscores, output underscore
            if has_dash {
                result.push('-');
            } else {
                result.push('_');
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_opts() -> DashifyOptions {
        DashifyOptions::default()
    }

    fn force_dash_opts() -> DashifyOptions {
        DashifyOptions { force_dash: true }
    }

    // ============================================================
    // 1. Basic Separators
    // ============================================================

    #[test]
    fn test_1_1_space_becomes_dash() {
        assert_eq!(dashify("file name.txt", &default_opts()), "file-name.txt");
    }

    #[test]
    fn test_1_2_underscore_left_alone() {
        assert_eq!(dashify("file_name.txt", &default_opts()), "file_name.txt");
    }

    #[test]
    fn test_1_3_plus_becomes_dash() {
        assert_eq!(dashify("file+name.txt", &default_opts()), "file-name.txt");
    }

    #[test]
    fn test_1_4_comma_becomes_dash() {
        assert_eq!(dashify("file,name.txt", &default_opts()), "file-name.txt");
    }

    #[test]
    fn test_1_5_dot_in_middle_left_alone() {
        assert_eq!(dashify("file.name.txt", &default_opts()), "file.name.txt");
    }

    // ============================================================
    // 2. CamelCase / PascalCase
    // ============================================================

    #[test]
    fn test_2_1_pascal_case_split() {
        assert_eq!(
            dashify("ConsiderationsWhileProjectPlanning.doc", &default_opts()),
            "considerations-while-project-planning.doc"
        );
    }

    #[test]
    fn test_2_2_camel_case_split() {
        assert_eq!(
            dashify("camelCaseFileName.txt", &default_opts()),
            "camel-case-file-name.txt"
        );
    }

    #[test]
    fn test_2_3_acronym_at_start() {
        assert_eq!(dashify("XMLParser.java", &default_opts()), "xml-parser.java");
    }

    #[test]
    fn test_2_4_acronym_in_middle() {
        assert_eq!(dashify("getHTTPResponse.js", &default_opts()), "get-http-response.js");
    }

    #[test]
    fn test_2_5_two_letter_acronym() {
        assert_eq!(dashify("IOStream.py", &default_opts()), "io-stream.py");
    }

    #[test]
    fn test_2_6_iphone_style() {
        assert_eq!(dashify("iPhone.txt", &default_opts()), "iphone.txt");
    }

    #[test]
    fn test_2_7_acronym_then_word() {
        assert_eq!(dashify("PDFReader.pdf", &default_opts()), "pdf-reader.pdf");
    }

    // ============================================================
    // 3. Mixed/Redundant Separators
    // ============================================================

    #[test]
    fn test_3_1_mixed_separators_collapse() {
        assert_eq!(dashify("-_-.txt", &default_opts()), "-.txt");
    }

    #[test]
    fn test_3_2_multiple_dashes_collapse() {
        assert_eq!(dashify("file--name.txt", &default_opts()), "file-name.txt");
    }

    #[test]
    fn test_3_3_multiple_underscores_collapse() {
        assert_eq!(dashify("file___name.txt", &default_opts()), "file_name.txt");
    }

    #[test]
    fn test_3_4_space_dash_space() {
        assert_eq!(dashify("file - name.txt", &default_opts()), "file-name.txt");
    }

    #[test]
    fn test_3_5_space_underscore_space() {
        assert_eq!(dashify("file _ name.txt", &default_opts()), "file-name.txt");
    }

    #[test]
    fn test_3_6_space_plus_space() {
        assert_eq!(dashify("file + name.txt", &default_opts()), "file-name.txt");
    }

    #[test]
    fn test_3_7_leading_trailing_dashes_collapse() {
        assert_eq!(dashify("--file--.txt", &default_opts()), "-file-.txt");
    }

    #[test]
    fn test_3_8_leading_trailing_underscores_collapse() {
        assert_eq!(dashify("___file___.txt", &default_opts()), "_file_.txt");
    }

    #[test]
    fn test_3_8_dunder_pattern_preserved() {
        assert_eq!(dashify("__anything__.py", &default_opts()), "__anything__.py");
    }

    #[test]
    fn test_3_8_dunder_pattern_preserved_complex() {
        assert_eq!(dashify("__init__.py", &default_opts()), "__init__.py");
    }

    // ============================================================
    // 4. Special Characters
    // ============================================================

    #[test]
    fn test_4_1_parentheses_become_dash() {
        assert_eq!(dashify("file(name).txt", &default_opts()), "file-name.txt");
    }

    #[test]
    fn test_4_2_brackets_removed() {
        assert_eq!(dashify("file[name].txt", &default_opts()), "filename.txt");
    }

    #[test]
    fn test_4_3_braces_removed() {
        assert_eq!(dashify("file{name}.txt", &default_opts()), "filename.txt");
    }

    #[test]
    fn test_4_4_single_quote_becomes_dash() {
        assert_eq!(dashify("file'name.txt", &default_opts()), "file-name.txt");
    }

    #[test]
    fn test_4_5_double_quote_becomes_dash() {
        assert_eq!(dashify("file\"name.txt", &default_opts()), "file-name.txt");
    }

    #[test]
    fn test_4_6_at_becomes_dash() {
        assert_eq!(dashify("file@name.txt", &default_opts()), "file-name.txt");
    }

    #[test]
    fn test_4_7_hash_becomes_dash() {
        assert_eq!(dashify("file#name.txt", &default_opts()), "file-name.txt");
    }

    #[test]
    fn test_4_8_dollar_becomes_dash() {
        assert_eq!(dashify("file$name.txt", &default_opts()), "file-name.txt");
    }

    #[test]
    fn test_4_9_percent_becomes_dash() {
        assert_eq!(dashify("file%name.txt", &default_opts()), "file-name.txt");
    }

    #[test]
    fn test_4_10_ampersand_becomes_dash() {
        assert_eq!(dashify("file&name.txt", &default_opts()), "file-name.txt");
    }

    #[test]
    fn test_4_11_exclamation_becomes_dash() {
        assert_eq!(dashify("file!name.txt", &default_opts()), "file-name.txt");
    }

    // ============================================================
    // 5. Case Handling
    // ============================================================

    #[test]
    fn test_5_1_all_caps_with_space_lowercased() {
        assert_eq!(dashify("FILE NAME.TXT", &default_opts()), "file-name.txt");
    }

    #[test]
    fn test_5_2_title_case_lowercased() {
        assert_eq!(dashify("File Name.txt", &default_opts()), "file-name.txt");
    }

    #[test]
    fn test_5_3_mixed_case_extension_lowercased() {
        assert_eq!(dashify("file name.TXT", &default_opts()), "file-name.txt");
    }

    #[test]
    fn test_5_4_readme_preserved() {
        assert_eq!(dashify("README.md", &default_opts()), "README.md");
    }

    #[test]
    fn test_5_5_changelog_preserved() {
        assert_eq!(dashify("CHANGELOG.md", &default_opts()), "CHANGELOG.md");
    }

    // ============================================================
    // 6. Numbers
    // ============================================================

    #[test]
    fn test_6_1_numbers_in_middle_dash_separated() {
        assert_eq!(dashify("file123name.txt", &default_opts()), "file-123-name.txt");
    }

    #[test]
    fn test_6_2_numbers_at_start() {
        assert_eq!(dashify("123file.txt", &default_opts()), "123-file.txt");
    }

    #[test]
    fn test_6_3_numbers_at_end() {
        assert_eq!(dashify("file123.txt", &default_opts()), "file-123.txt");
    }

    #[test]
    fn test_6_4_number_between_pascal_words() {
        assert_eq!(dashify("File2Name.txt", &default_opts()), "file-2-name.txt");
    }

    #[test]
    fn test_6_5_version_number() {
        assert_eq!(dashify("version2.0.txt", &default_opts()), "version-2.0.txt");
    }

    #[test]
    fn test_6_6_semver_style_left_alone() {
        assert_eq!(dashify("v2.0.1-release.txt", &default_opts()), "v2.0.1-release.txt");
    }

    // ============================================================
    // 7. Extensions
    // ============================================================

    #[test]
    fn test_7_1_uppercase_extension_lowercased() {
        assert_eq!(dashify("My Document.PDF", &default_opts()), "my-document.pdf");
    }

    #[test]
    fn test_7_2_multiple_dots_in_name() {
        assert_eq!(dashify("My.Document.Name.txt", &default_opts()), "my.document.name.txt");
    }

    #[test]
    fn test_7_3_no_extension_with_underscore_left_alone() {
        assert_eq!(dashify("no_extension", &default_opts()), "no_extension");
    }

    #[test]
    fn test_7_4_tar_gz_left_alone() {
        assert_eq!(dashify("file.tar.gz", &default_opts()), "file.tar.gz");
    }

    #[test]
    fn test_7_5_hidden_file_left_alone() {
        assert_eq!(dashify(".hidden_file", &default_opts()), ".hidden_file");
    }

    #[test]
    fn test_7_6_hidden_file_with_space() {
        assert_eq!(dashify(".Hidden File.txt", &default_opts()), ".hidden-file.txt");
    }

    // ============================================================
    // 8. Edge Cases
    // ============================================================

    #[test]
    fn test_8_1_single_char_left_alone() {
        assert_eq!(dashify("a.txt", &default_opts()), "a.txt");
    }

    #[test]
    fn test_8_2_dash_only_left_alone() {
        assert_eq!(dashify("-.txt", &default_opts()), "-.txt");
    }

    #[test]
    fn test_8_3_extension_only_left_alone() {
        assert_eq!(dashify(".txt", &default_opts()), ".txt");
    }

    #[test]
    fn test_8_4_multiple_dots_only_left_alone() {
        assert_eq!(dashify("...txt", &default_opts()), "...txt");
    }

    #[test]
    fn test_8_5_double_dots_collapsed() {
        assert_eq!(dashify("file..name.txt", &default_opts()), "file.name.txt");
    }

    #[test]
    fn test_8_6_leading_space_left_alone() {
        assert_eq!(dashify(" file.txt", &default_opts()), " file.txt");
    }

    #[test]
    fn test_8_7_trailing_space_left_alone() {
        assert_eq!(dashify("file.txt ", &default_opts()), "file.txt ");
    }

    #[test]
    fn test_8_8_space_before_extension_left_alone() {
        assert_eq!(dashify(" .txt", &default_opts()), " .txt");
    }

    // ============================================================
    // 9. Unicode
    // ============================================================

    #[test]
    fn test_9_1_unicode_cafe_left_alone() {
        assert_eq!(dashify("café.txt", &default_opts()), "café.txt");
    }

    #[test]
    fn test_9_2_unicode_naive_left_alone() {
        assert_eq!(dashify("naïve.txt", &default_opts()), "naïve.txt");
    }

    #[test]
    fn test_9_3_unicode_japanese_left_alone() {
        assert_eq!(dashify("日本語.txt", &default_opts()), "日本語.txt");
    }

    #[test]
    fn test_9_4_unicode_german_left_alone() {
        assert_eq!(dashify("Über-Datei.txt", &default_opts()), "Über-Datei.txt");
    }

    // ============================================================
    // 10. Already Dashified
    // ============================================================

    #[test]
    fn test_10_1_already_dashified_left_alone() {
        assert_eq!(
            dashify("already-dashified.txt", &default_opts()),
            "already-dashified.txt"
        );
    }

    #[test]
    fn test_10_2_dashified_but_caps_lowercased() {
        assert_eq!(
            dashify("Already-Dashified.txt", &default_opts()),
            "already-dashified.txt"
        );
    }

    // ============================================================
    // Additional edge cases from original examples
    // ============================================================

    #[test]
    fn test_original_example_plus_signs() {
        assert_eq!(
            dashify("considerations+while+project+planning.doc", &default_opts()),
            "considerations-while-project-planning.doc"
        );
    }

    #[test]
    fn test_original_example_plus_signs_title_case() {
        assert_eq!(
            dashify("Consideration+While+Project+Planning.doc", &default_opts()),
            "consideration-while-project-planning.doc"
        );
    }

    // ============================================================
    // 11. Force Dash Tests
    // ============================================================

    #[test]
    fn test_11_1_force_dash_converts_underscores() {
        assert_eq!(dashify("file_name.txt", &force_dash_opts()), "file-name.txt");
    }

    #[test]
    fn test_11_2_force_dash_multiple_underscores() {
        assert_eq!(dashify("file__name.txt", &force_dash_opts()), "file-name.txt");
    }

    #[test]
    fn test_11_3_force_dash_mixed_separators() {
        assert_eq!(dashify("file_name-here.txt", &force_dash_opts()), "file-name-here.txt");
    }

    #[test]
    fn test_11_4_force_dash_snake_case() {
        assert_eq!(
            dashify("some_snake_case.yaml", &force_dash_opts()),
            "some-snake-case.yaml"
        );
    }

    #[test]
    fn test_11_5_force_dash_with_numbers() {
        assert_eq!(dashify("my_project_v2.md", &force_dash_opts()), "my-project-v-2.md");
    }
}
