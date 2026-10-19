pub fn detect_mime_type(filename: &str, bytes: &[u8]) -> String {
    if let Some(mime) = detect_from_magic_bytes(bytes) {
        return mime;
    }

    detect_from_extension(filename)
}

fn detect_from_magic_bytes(bytes: &[u8]) -> Option<String> {
    match bytes {
        [0x25, 0x50, 0x44, 0x46, ..] => Some("application/pdf".to_string()),
        [0xFF, 0xD8, 0xFF, ..] => Some("image/jpeg".to_string()),
        [0x89, 0x50, 0x4E, 0x47, ..] => Some("image/png".to_string()),
        [0x47, 0x49, 0x46, 0x38, ..] => Some("image/gif".to_string()),
        [0x50, 0x4B, 0x03, 0x04, ..] => Some("application/zip".to_string()),
        [0x52, 0x61, 0x72, 0x21, ..] => Some("application/x-rar-compressed".to_string()),
        [0x1F, 0x8B, ..] => Some("application/gzip".to_string()),
        _ if bytes.starts_with(b"<?xml") => Some("application/xml".to_string()),
        _ if bytes.starts_with(b"{") || bytes.starts_with(b"[") => {
            Some("application/json".to_string())
        }
        _ => None,
    }
}

fn detect_from_extension(filename: &str) -> String {
    let extension = filename
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "pdf" => "application/pdf",
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "zip" => "application/zip",
        "tar" => "application/x-tar",
        "gz" => "application/gzip",
        "7z" => "application/x-7z-compressed",
        "rar" => "application/x-rar-compressed",
        "txt" => "text/plain",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "xml" => "application/xml",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        _ => "application/octet-stream",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_pdf() {
        let bytes = vec![0x25, 0x50, 0x44, 0x46];
        assert_eq!(
            detect_mime_type("file.pdf", &bytes),
            "application/pdf"
        );
    }

    #[test]
    fn test_detect_jpeg() {
        let bytes = vec![0xFF, 0xD8, 0xFF, 0xE0];
        assert_eq!(detect_mime_type("photo.jpg", &bytes), "image/jpeg");
    }

    #[test]
    fn test_detect_png() {
        let bytes = vec![0x89, 0x50, 0x4E, 0x47];
        assert_eq!(detect_mime_type("image.png", &bytes), "image/png");
    }

    #[test]
    fn test_detect_from_extension() {
        let bytes = vec![0x00, 0x00, 0x00, 0x00];
        assert_eq!(detect_mime_type("doc.txt", &bytes), "text/plain");
        assert_eq!(detect_mime_type("data.json", &bytes), "application/json");
    }

    #[test]
    fn test_fallback_to_octet_stream() {
        let bytes = vec![0x00, 0x00, 0x00, 0x00];
        assert_eq!(
            detect_mime_type("unknown.xyz", &bytes),
            "application/octet-stream"
        );
    }

    #[test]
    fn short_json_is_detected_by_content() {
        assert_eq!(detect_mime_type("data", b"{}"), "application/json");
        assert_eq!(detect_mime_type("data", b"["), "application/json");
    }

    #[test]
    fn two_byte_gzip_header_is_detected() {
        assert_eq!(detect_mime_type("data", &[0x1F, 0x8B]), "application/gzip");
    }

    #[test]
    fn empty_body_falls_back_to_the_extension() {
        assert_eq!(detect_mime_type("notes.txt", &[]), "text/plain");
    }
}
