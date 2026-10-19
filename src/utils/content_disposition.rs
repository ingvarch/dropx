use crate::utils::constants::INLINE_ACTIVE_CONTENT_TYPE;

pub enum DispositionType {
    Attachment,
    Inline,
    Auto,
}

pub struct ContentHeaders {
    pub content_type: String,
    pub content_disposition: String,
}

pub fn content_headers(
    filename: &str,
    disposition_type: DispositionType,
    mime_type: &str,
) -> ContentHeaders {
    let (kind, content_type) = match disposition_type {
        DispositionType::Attachment => ("attachment", mime_type),
        DispositionType::Inline if is_active_content(mime_type) => ("inline", INLINE_ACTIVE_CONTENT_TYPE),
        DispositionType::Inline => ("inline", mime_type),
        DispositionType::Auto if is_displayable_in_browser(mime_type) => ("inline", mime_type),
        DispositionType::Auto => ("attachment", mime_type),
    };
    ContentHeaders {
        content_type: content_type.to_string(),
        content_disposition: format!("{}; filename=\"{}\"", kind, filename),
    }
}

// Types a browser runs scripts in when it opens them as a page.
fn is_active_content(mime_type: &str) -> bool {
    matches!(mime_type, "text/html" | "image/svg+xml" | "application/xml")
}

fn is_displayable_in_browser(mime_type: &str) -> bool {
    matches!(
        mime_type,
        "image/jpeg"
            | "image/png"
            | "image/gif"
            | "image/webp"
            | "application/pdf"
            | "text/plain"
            | "text/css"
            | "text/javascript"
            | "application/javascript"
            | "video/mp4"
            | "video/webm"
            | "audio/mpeg"
            | "audio/ogg"
            | "audio/wav"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACTIVE_TYPES: [&str; 3] = ["text/html", "image/svg+xml", "application/xml"];

    #[test]
    fn test_active_types_are_attachments_on_auto() {
        for mime in ACTIVE_TYPES {
            let result = content_headers("page", DispositionType::Auto, mime);
            assert_eq!(result.content_type, mime);
            assert_eq!(result.content_disposition, "attachment; filename=\"page\"");
        }
    }

    #[test]
    fn test_active_types_are_plain_text_on_inline() {
        for mime in ACTIVE_TYPES {
            let result = content_headers("page", DispositionType::Inline, mime);
            assert_eq!(result.content_type, "text/plain; charset=utf-8");
            assert_eq!(result.content_disposition, "inline; filename=\"page\"");
        }
    }

    #[test]
    fn test_active_types_keep_their_type_on_attachment() {
        for mime in ACTIVE_TYPES {
            let result = content_headers("page", DispositionType::Attachment, mime);
            assert_eq!(result.content_type, mime);
            assert_eq!(result.content_disposition, "attachment; filename=\"page\"");
        }
    }

    #[test]
    fn test_passive_types_are_unchanged_on_every_route() {
        for (mime, auto) in [("image/png", "inline"), ("text/plain", "inline"), ("application/zip", "attachment")] {
            let expected = [
                (DispositionType::Auto, auto),
                (DispositionType::Inline, "inline"),
                (DispositionType::Attachment, "attachment"),
            ];
            for (route, disposition) in expected {
                let result = content_headers("f", route, mime);
                assert_eq!(result.content_type, mime);
                assert_eq!(result.content_disposition, format!("{}; filename=\"f\"", disposition));
            }
        }
    }

    #[test]
    fn test_attachment_disposition() {
        let result = content_headers("file.txt", DispositionType::Attachment, "text/plain");
        assert_eq!(result.content_disposition, "attachment; filename=\"file.txt\"");
    }

    #[test]
    fn test_inline_disposition() {
        let result = content_headers("image.png", DispositionType::Inline, "image/png");
        assert_eq!(result.content_disposition, "inline; filename=\"image.png\"");
    }

    #[test]
    fn test_auto_disposition_displayable() {
        let result = content_headers("photo.jpg", DispositionType::Auto, "image/jpeg");
        assert_eq!(result.content_disposition, "inline; filename=\"photo.jpg\"");
    }

    #[test]
    fn test_auto_disposition_non_displayable() {
        let result = content_headers("archive.zip", DispositionType::Auto, "application/zip");
        assert_eq!(result.content_disposition, "attachment; filename=\"archive.zip\"");
    }

    #[test]
    fn test_displayable_mime_types() {
        assert!(is_displayable_in_browser("image/jpeg"));
        assert!(is_displayable_in_browser("image/png"));
        assert!(is_displayable_in_browser("image/gif"));
        assert!(is_displayable_in_browser("image/webp"));
        assert!(is_displayable_in_browser("application/pdf"));
        assert!(is_displayable_in_browser("text/plain"));
        assert!(is_displayable_in_browser("video/mp4"));
        assert!(is_displayable_in_browser("audio/mpeg"));
    }

    #[test]
    fn test_non_displayable_mime_types() {
        assert!(!is_displayable_in_browser("text/html"));
        assert!(!is_displayable_in_browser("image/svg+xml"));
        assert!(!is_displayable_in_browser("application/zip"));
        assert!(!is_displayable_in_browser("application/octet-stream"));
        assert!(!is_displayable_in_browser("application/x-rar-compressed"));
        assert!(!is_displayable_in_browser("application/vnd.ms-excel"));
    }

    #[test]
    fn test_pdf_is_displayable() {
        let result = content_headers("document.pdf", DispositionType::Auto, "application/pdf");
        assert_eq!(result.content_disposition, "inline; filename=\"document.pdf\"");
    }

    #[test]
    fn test_filename_with_spaces() {
        let result = content_headers("my file.txt", DispositionType::Attachment, "text/plain");
        assert_eq!(result.content_disposition, "attachment; filename=\"my file.txt\"");
    }

    #[test]
    fn test_filename_with_special_chars() {
        let result = content_headers("report-2024.pdf", DispositionType::Inline, "application/pdf");
        assert_eq!(result.content_disposition, "inline; filename=\"report-2024.pdf\"");
    }

    #[test]
    fn test_video_is_displayable() {
        assert!(is_displayable_in_browser("video/mp4"));
        assert!(is_displayable_in_browser("video/webm"));
    }

    #[test]
    fn test_audio_is_displayable() {
        assert!(is_displayable_in_browser("audio/mpeg"));
        assert!(is_displayable_in_browser("audio/ogg"));
        assert!(is_displayable_in_browser("audio/wav"));
    }
}
