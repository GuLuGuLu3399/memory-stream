mod markdown;
mod zip_reader;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportCard {
    pub title: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub body_md: String,
    pub source_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportImage {
    pub filename: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImportError {
    IoError(String),
    ParseError(String),
    InvalidFormat(String),
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportError::IoError(msg) => write!(f, "IO Error: {}", msg),
            ImportError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
            ImportError::InvalidFormat(msg) => write!(f, "Invalid Format: {}", msg),
        }
    }
}

impl std::error::Error for ImportError {}

pub use markdown::parse_markdown_file;
pub use zip_reader::extract_zip_archive;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    fn create_test_zip(files: Vec<(&str, &str)>) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempdir().unwrap();
        let zip_path = dir.path().join("test.zip");
        let zip_file = File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(zip_file);
        let options = SimpleFileOptions::default();

        for (name, content) in files {
            zip.start_file(name, options).unwrap();
            write!(zip, "{}", content).unwrap();
        }
        zip.finish().unwrap();
        (dir, zip_path)
    }

    #[test]
    fn test_extract_multiple_md_files() {
        let (_dir, zip_path) = create_test_zip(vec![
            ("card1.md", "# Card 1\n\nContent 1"),
            ("card2.md", "# Card 2\n\nContent 2"),
        ]);

        let (cards, images) = extract_zip_archive(&zip_path).unwrap();
        assert_eq!(cards.len(), 2);
        assert_eq!(images.len(), 0);

        let titles: Vec<&str> = cards.iter().map(|c| c.title.as_str()).collect();
        assert!(titles.contains(&"Card 1"));
        assert!(titles.contains(&"Card 2"));
    }

    #[test]
    fn test_extract_md_with_frontmatter() {
        let (_dir, zip_path) = create_test_zip(vec![(
            "meta.md",
            "---\ntitle: Meta Card\ncategory: Test\ntags:\n  - tag1\n---\n\n# Content",
        )]);

        let (cards, _) = extract_zip_archive(&zip_path).unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].title, "Meta Card");
        assert_eq!(cards[0].category, Some("Test".to_string()));
        assert_eq!(cards[0].tags, vec!["tag1"]);
    }

    #[test]
    fn test_extract_images() {
        let dir = tempdir().unwrap();
        let zip_path = dir.path().join("images.zip");
        let zip_file = File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(zip_file);
        let options = SimpleFileOptions::default();

        zip.start_file("image1.png", options).unwrap();
        writeln!(zip, "PNG DATA").unwrap();
        zip.start_file("image2.jpg", options).unwrap();
        writeln!(zip, "JPG DATA").unwrap();
        zip.start_file("doc.md", options).unwrap();
        writeln!(zip, "# Doc").unwrap();
        zip.finish().unwrap();

        let (cards, images) = extract_zip_archive(&zip_path).unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(images.len(), 2);
        assert!(images.iter().any(|i| i.filename == "image1.png"));
        assert!(images.iter().any(|i| i.filename == "image2.jpg"));
    }

    #[test]
    fn test_skip_binary_files() {
        let dir = tempdir().unwrap();
        let zip_path = dir.path().join("test.zip");
        let zip_file = File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(zip_file);
        let options = SimpleFileOptions::default();

        zip.start_file("binary.exe", options).unwrap();
        writeln!(zip, "EXE DATA").unwrap();
        zip.finish().unwrap();

        let (cards, images) = extract_zip_archive(&zip_path).unwrap();
        assert_eq!(cards.len(), 0);
        assert_eq!(images.len(), 0);
    }

    // =========================================================================
    // END-TO-END INTEGRATION TESTS
    // =========================================================================

    /// Test corrupted ZIP file returns graceful error without panic
    #[test]
    fn test_corrupted_zip_graceful_error() {
        let dir = tempdir().unwrap();
        let zip_path = dir.path().join("corrupted.zip");
        let mut file = File::create(&zip_path).unwrap();
        // Write invalid ZIP data
        file.write_all(b"This is not a valid ZIP file content")
            .unwrap();

        let result = extract_zip_archive(&zip_path);
        assert!(result.is_err());
        match result {
            Err(ImportError::ParseError(msg)) => {
                assert!(msg.contains("ZIP") || msg.contains("archive") || msg.contains("index"));
            }
            Err(ImportError::IoError(_)) => {
                // Also acceptable - IO error during read
            }
            _ => panic!("Expected ParseError or IoError for corrupted ZIP"),
        }
    }

    /// Test batch parsing of 100+ MD files
    #[test]
    fn test_batch_100_plus_md_files() {
        let dir = tempdir().unwrap();
        let zip_path = dir.path().join("batch.zip");
        let zip_file = File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(zip_file);
        let options = SimpleFileOptions::default();

        let num_files = 150;
        for i in 0..num_files {
            let filename = format!("card_{:03}.md", i);
            zip.start_file(&filename, options).unwrap();
            let content = format!(
                "---\ntitle: Card {}\ncategory: Batch\ntags:\n  - batch\n  - test\n---\n\n# Card {}\n\nContent for card {}.",
                i, i, i
            );
            write!(zip, "{}", content).unwrap();
        }
        zip.finish().unwrap();

        let (cards, images) = extract_zip_archive(&zip_path).unwrap();
        assert_eq!(cards.len(), num_files);
        assert_eq!(images.len(), 0);

        // Verify all cards have correct metadata
        for card in &cards {
            assert!(card.title.starts_with("Card "));
            assert_eq!(card.category, Some("Batch".to_string()));
            assert_eq!(card.tags, vec!["batch", "test"]);
            assert!(!card.body_md.is_empty());
        }
    }

    /// Test ZIP with nested directory structure
    #[test]
    fn test_nested_directories_in_zip() {
        let (_dir, zip_path) = create_test_zip(vec![
            ("docs/readme.md", "# README\n\nThis is the readme."),
            ("docs/guide.md", "# Guide\n\nThis is the guide."),
            (
                "docs/advanced/tutorial.md",
                "# Tutorial\n\nAdvanced tutorial.",
            ),
        ]);

        let (cards, images) = extract_zip_archive(&zip_path).unwrap();
        assert_eq!(cards.len(), 3);
        assert_eq!(images.len(), 0);

        let titles: Vec<&str> = cards.iter().map(|c| c.title.as_str()).collect();
        assert!(titles.contains(&"README"));
        assert!(titles.contains(&"Guide"));
        assert!(titles.contains(&"Tutorial"));

        // Verify source_path includes directory structure
        assert!(cards.iter().any(|c| c.source_path == "docs/readme.md"));
        assert!(cards
            .iter()
            .any(|c| c.source_path == "docs/advanced/tutorial.md"));
    }

    /// Test ZIP with mixed content: MD, images, and binary files
    #[test]
    fn test_mixed_content_zip() {
        let dir = tempdir().unwrap();
        let zip_path = dir.path().join("mixed.zip");
        let zip_file = File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(zip_file);
        let options = SimpleFileOptions::default();

        // Add markdown files
        zip.start_file("note1.md", options).unwrap();
        write!(zip, "# Note 1\n\nFirst note.").unwrap();
        zip.start_file("note2.md", options).unwrap();
        write!(
            zip,
            "---\ntitle: Custom Title\n---\n\n# Heading\n\nSecond note."
        )
        .unwrap();

        // Add images
        zip.start_file("photo.png", options).unwrap();
        write!(zip, "PNG_BINARY_DATA_HERE").unwrap();
        zip.start_file("diagram.jpg", options).unwrap();
        write!(zip, "JPG_BINARY_DATA_HERE").unwrap();
        zip.start_file("animation.gif", options).unwrap();
        write!(zip, "GIF_BINARY_DATA_HERE").unwrap();
        zip.start_file("modern.webp", options).unwrap();
        write!(zip, "WEBP_BINARY_DATA_HERE").unwrap();

        // Add binary files (should be skipped)
        zip.start_file("document.pdf", options).unwrap();
        write!(zip, "PDF_DATA").unwrap();
        zip.start_file("archive.zip", options).unwrap();
        write!(zip, "NESTED_ZIP").unwrap();

        zip.finish().unwrap();

        let (cards, images) = extract_zip_archive(&zip_path).unwrap();
        assert_eq!(cards.len(), 2);
        assert_eq!(images.len(), 4); // png, jpg, gif, webp

        // Verify image formats
        let image_names: Vec<&str> = images.iter().map(|i| i.filename.as_str()).collect();
        assert!(image_names.contains(&"photo.png"));
        assert!(image_names.contains(&"diagram.jpg"));
        assert!(image_names.contains(&"animation.gif"));
        assert!(image_names.contains(&"modern.webp"));
    }

    /// Test MD file with only frontmatter (no body content)
    #[test]
    fn test_md_with_only_frontmatter() {
        let (_dir, zip_path) = create_test_zip(vec![(
            "meta_only.md",
            "---\ntitle: Metadata Only\ncategory: Meta\ntags:\n  - empty\n---\n",
        )]);

        let (cards, _) = extract_zip_archive(&zip_path).unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].title, "Metadata Only");
        assert_eq!(cards[0].category, Some("Meta".to_string()));
        assert_eq!(cards[0].tags, vec!["empty"]);
        assert!(cards[0].body_md.is_empty());
    }

    /// Test MD file with empty tags array
    #[test]
    fn test_md_with_empty_tags() {
        let (_dir, zip_path) = create_test_zip(vec![(
            "no_tags.md",
            "---\ntitle: No Tags\ncategory: Test\ntags: []\n---\n\n# Content",
        )]);

        let (cards, _) = extract_zip_archive(&zip_path).unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].title, "No Tags");
        assert_eq!(cards[0].tags.len(), 0);
    }

    /// Test MD file with multiline body content
    #[test]
    fn test_md_with_multiline_body() {
        let body = r#"## Introduction

This is a **multi-paragraph** document.

### Features

- Feature 1
- Feature 2
- Feature 3

```rust
fn main() {
    println!("Hello, world!");
}
```

> A blockquote for emphasis.

[Link to example](https://example.com)"#;

        let (_dir, zip_path) = create_test_zip(vec![(
            "full.md",
            &format!(
                "---\ntitle: Full Document\ncategory: Docs\ntags:\n  - markdown\n  - full\n---\n\n{}",
                body
            ),
        )]);

        let (cards, _) = extract_zip_archive(&zip_path).unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].title, "Full Document");
        assert!(cards[0].body_md.contains("## Introduction"));
        assert!(cards[0].body_md.contains("```rust"));
        assert!(cards[0].body_md.contains("println!"));
    }

    /// Test MD file fallback to filename when no title and no heading
    #[test]
    fn test_md_fallback_to_filename() {
        let (_dir, zip_path) = create_test_zip(vec![
            ("NoTitle.md", "Just some content without heading."),
            ("another_one.md", "More content here."),
        ]);

        let (cards, _) = extract_zip_archive(&zip_path).unwrap();
        assert_eq!(cards.len(), 2);

        let titles: Vec<&str> = cards.iter().map(|c| c.title.as_str()).collect();
        assert!(titles.contains(&"NoTitle"));
        assert!(titles.contains(&"another_one"));
    }

    /// Test MD file with special characters in frontmatter
    #[test]
    fn test_md_with_special_characters() {
        let (_dir, zip_path) = create_test_zip(vec![(
            "special.md",
            "---\ntitle: 'Title with \"quotes\" and symbols: @#$%'\ncategory: Special-Chars_123\ntags:\n  - 'tag with spaces'\n  - unicode-中文\n---\n\n# Special Content",
        )]);

        let (cards, _) = extract_zip_archive(&zip_path).unwrap();
        assert_eq!(cards.len(), 1);
        assert!(cards[0].title.contains("quotes"));
        assert!(cards[0].title.contains("@"));
        assert_eq!(cards[0].category, Some("Special-Chars_123".to_string()));
        assert!(cards[0].tags.contains(&"tag with spaces".to_string()));
        assert!(cards[0].tags.contains(&"unicode-中文".to_string()));
    }

    /// Test parsing individual markdown file directly
    #[test]
    fn test_parse_markdown_file_direct() {
        use super::parse_markdown_file;
        use std::io::Write;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("direct_test.md");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "---").unwrap();
        writeln!(file, "title: Direct Parse").unwrap();
        writeln!(file, "category: Unit").unwrap();
        writeln!(file, "tags:").unwrap();
        writeln!(file, "  - direct").unwrap();
        writeln!(file, "---").unwrap();
        writeln!(file, "# Heading").unwrap();
        writeln!(file, "Direct content.").unwrap();

        let card = parse_markdown_file(&file_path).unwrap();
        assert_eq!(card.title, "Direct Parse");
        assert_eq!(card.category, Some("Unit".to_string()));
        assert_eq!(card.tags, vec!["direct"]);
        assert!(card.body_md.contains("Direct content."));
        assert!(card.source_path.contains("direct_test.md"));
    }

    /// Test ImportCard serialization/deserialization
    #[test]
    fn test_import_card_serialization() {
        let card = ImportCard {
            title: "Test Card".to_string(),
            category: Some("Testing".to_string()),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            body_md: "# Content\n\nBody text.".to_string(),
            source_path: "/path/to/file.md".to_string(),
        };

        let json = serde_json::to_string(&card).unwrap();
        assert!(json.contains("Test Card"));
        assert!(json.contains("Testing"));
        assert!(json.contains("tag1"));

        let deserialized: ImportCard = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.title, card.title);
        assert_eq!(deserialized.category, card.category);
        assert_eq!(deserialized.tags, card.tags);
        assert_eq!(deserialized.body_md, card.body_md);
        assert_eq!(deserialized.source_path, card.source_path);
    }

    /// Test ImportImage serialization/deserialization
    #[test]
    fn test_import_image_serialization() {
        let image = ImportImage {
            filename: "test.png".to_string(),
            data: vec![0x89, 0x50, 0x4E, 0x47], // PNG magic bytes
        };

        let json = serde_json::to_string(&image).unwrap();
        assert!(json.contains("test.png"));
        assert!(json.contains("data"));

        let deserialized: ImportImage = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.filename, image.filename);
        assert_eq!(deserialized.data, image.data);
    }

    /// Test empty ZIP returns empty results
    #[test]
    fn test_empty_zip() {
        let dir = tempdir().unwrap();
        let zip_path = dir.path().join("empty.zip");
        let zip_file = File::create(&zip_path).unwrap();
        let _zip = ZipWriter::new(zip_file).finish().unwrap();

        let (cards, images) = extract_zip_archive(&zip_path).unwrap();
        assert_eq!(cards.len(), 0);
        assert_eq!(images.len(), 0);
    }

    /// Test ZIP with many images and few MD files
    #[test]
    fn test_many_images_few_md() {
        let dir = tempdir().unwrap();
        let zip_path = dir.path().join("images_heavy.zip");
        let zip_file = File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(zip_file);
        let options = SimpleFileOptions::default();

        // Add single MD
        zip.start_file("index.md", options).unwrap();
        write!(zip, "# Image Gallery\n\nGallery with many images.").unwrap();

        // Add 50 images
        for i in 0..50 {
            let ext = if i % 3 == 0 {
                "png"
            } else if i % 3 == 1 {
                "jpg"
            } else {
                "webp"
            };
            let filename = format!("img_{:03}.{}", i, ext);
            zip.start_file(&filename, options).unwrap();
            write!(zip, "IMAGE_DATA_{}", i).unwrap();
        }

        zip.finish().unwrap();

        let (cards, images) = extract_zip_archive(&zip_path).unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(images.len(), 50);
        assert_eq!(cards[0].title, "Image Gallery");
    }

    /// Test large file handling within ZIP
    #[test]
    fn test_large_content_in_zip() {
        let large_body = "x".repeat(100_000); // 100KB of content
        let content = format!(
            "---\ntitle: Large File\ncategory: Big\ntags:\n  - large\n---\n\n# Large Content\n\n{}",
            large_body
        );

        let (_dir, zip_path) = create_test_zip(vec![("large.md", &content)]);

        let (cards, _) = extract_zip_archive(&zip_path).unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].title, "Large File");
        assert_eq!(
            cards[0].body_md.len(),
            large_body.len() + "# Large Content\n\n".len()
        );
    }

    /// Test MD with YAML-style tags (flow style)
    #[test]
    fn test_md_with_flow_style_tags() {
        let (_dir, zip_path) = create_test_zip(vec![(
            "flow.md",
            "---\ntitle: Flow Style\ntags: [tag1, tag2, tag3]\n---\n\n# Flow",
        )]);

        let (cards, _) = extract_zip_archive(&zip_path).unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].title, "Flow Style");
        assert_eq!(cards[0].tags, vec!["tag1", "tag2", "tag3"]);
    }

    /// Test MD with multiple h1 headings (uses first one)
    #[test]
    fn test_md_multiple_h1_headings() {
        let (_dir, zip_path) = create_test_zip(vec![(
            "multi_h1.md",
            "# First Heading\n\nContent one.\n\n# Second Heading\n\nContent two.",
        )]);

        let (cards, _) = extract_zip_archive(&zip_path).unwrap();
        assert_eq!(cards.len(), 1);
        // Should use first heading as title
        assert_eq!(cards[0].title, "First Heading");
    }

    /// Test MD with h2/h3 headings but no h1 (uses filename)
    #[test]
    fn test_md_no_h1_uses_filename() {
        let (_dir, zip_path) = create_test_zip(vec![(
            "NoH1File.md",
            "## Introduction\n\nSome intro.\n\n### Details\n\nMore details.",
        )]);

        let (cards, _) = extract_zip_archive(&zip_path).unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].title, "NoH1File");
    }
}
