pub enum DispositionType {
    Attachment,
    Inline,
    Auto,
}

pub fn get_content_disposition(
    filename: &str,
    disposition_type: DispositionType,
    mime_type: &str,
) -> String {
    match disposition_type {
        DispositionType::Attachment => format!("attachment; filename=\"{}\"", filename),
        DispositionType::Inline => format!("inline; filename=\"{}\"", filename),
        DispositionType::Auto => {
            if is_displayable_in_browser(mime_type) {
                format!("inline; filename=\"{}\"", filename)
            } else {
                format!("attachment; filename=\"{}\"", filename)
            }
        }
    }
}

fn is_displayable_in_browser(mime_type: &str) -> bool {
    matches!(
        mime_type,
        "image/jpeg"
            | "image/png"
            | "image/gif"
            | "image/webp"
            | "image/svg+xml"
            | "application/pdf"
            | "text/plain"
            | "text/html"
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

    #[test]
    fn test_attachment_disposition() {
        let result = get_content_disposition("file.txt", DispositionType::Attachment, "text/plain");
        assert_eq!(result, "attachment; filename=\"file.txt\"");
    }

    #[test]
    fn test_inline_disposition() {
        let result = get_content_disposition("image.png", DispositionType::Inline, "image/png");
        assert_eq!(result, "inline; filename=\"image.png\"");
    }

    #[test]
    fn test_auto_disposition_displayable() {
        let result = get_content_disposition("photo.jpg", DispositionType::Auto, "image/jpeg");
        assert_eq!(result, "inline; filename=\"photo.jpg\"");
    }

    #[test]
    fn test_auto_disposition_non_displayable() {
        let result = get_content_disposition("archive.zip", DispositionType::Auto, "application/zip");
        assert_eq!(result, "attachment; filename=\"archive.zip\"");
    }

    #[test]
    fn test_displayable_mime_types() {
        assert!(is_displayable_in_browser("image/jpeg"));
        assert!(is_displayable_in_browser("image/png"));
        assert!(is_displayable_in_browser("image/gif"));
        assert!(is_displayable_in_browser("image/webp"));
        assert!(is_displayable_in_browser("image/svg+xml"));
        assert!(is_displayable_in_browser("application/pdf"));
        assert!(is_displayable_in_browser("text/plain"));
        assert!(is_displayable_in_browser("video/mp4"));
        assert!(is_displayable_in_browser("audio/mpeg"));
    }

    #[test]
    fn test_non_displayable_mime_types() {
        assert!(!is_displayable_in_browser("application/zip"));
        assert!(!is_displayable_in_browser("application/octet-stream"));
        assert!(!is_displayable_in_browser("application/x-rar-compressed"));
        assert!(!is_displayable_in_browser("application/vnd.ms-excel"));
    }

    #[test]
    fn test_pdf_is_displayable() {
        let result = get_content_disposition("document.pdf", DispositionType::Auto, "application/pdf");
        assert_eq!(result, "inline; filename=\"document.pdf\"");
    }

    #[test]
    fn test_filename_with_spaces() {
        let result = get_content_disposition("my file.txt", DispositionType::Attachment, "text/plain");
        assert_eq!(result, "attachment; filename=\"my file.txt\"");
    }

    #[test]
    fn test_filename_with_special_chars() {
        let result = get_content_disposition("report-2024.pdf", DispositionType::Inline, "application/pdf");
        assert_eq!(result, "inline; filename=\"report-2024.pdf\"");
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
