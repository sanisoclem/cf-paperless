use mail_parser::{MessageParser, MessagePart, MimeHeaders};

pub struct Extensions(Vec<String>);

impl Extensions {
    pub fn parse(list: &str) -> Extensions {
        Extensions(
            list.split(',')
                .map(|extension| extension.trim().trim_start_matches('.'))
                .filter(|extension| !extension.is_empty())
                .map(str::to_owned)
                .collect(),
        )
    }

    fn contains(&self, extension: &str) -> bool {
        self.0
            .iter()
            .any(|allowed| allowed.eq_ignore_ascii_case(extension))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Document {
    pub name: String,
    pub content_type: Option<String>,
    pub content: Vec<u8>,
}

pub fn documents(raw_email: &[u8], allowed: &Extensions) -> Vec<Document> {
    MessageParser::default()
        .parse(raw_email)
        .map(|email| {
            email
                .attachments()
                .filter_map(|part| document(part, allowed))
                .collect()
        })
        .unwrap_or_default()
}

fn document(part: &MessagePart, allowed: &Extensions) -> Option<Document> {
    if is_inline(part) {
        return None;
    }
    let name = name(part)?;
    if !allowed.contains(extension(&name)?) {
        return None;
    }
    Some(Document {
        name,
        content_type: content_type(part),
        content: part.contents().to_vec(),
    })
}

fn is_inline(part: &MessagePart) -> bool {
    part.content_disposition()
        .is_some_and(|disposition| disposition.is_inline())
}

fn name(part: &MessagePart) -> Option<String> {
    match part.attachment_name() {
        Some(name) => Some(name.replace('/', "_")),
        None => Some(format!("attachment.{}", part.content_type()?.subtype()?)),
    }
}

fn extension(name: &str) -> Option<&str> {
    name.rsplit_once('.').map(|(_, extension)| extension)
}

fn content_type(part: &MessagePart) -> Option<String> {
    let content_type = part.content_type()?;
    let subtype = content_type.subtype()?;
    Some(format!("{}/{subtype}", content_type.ctype()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const EMAIL: &str = "From: me@proton.me\r
To: docs@shokugami.place\r
Subject: Bills\r
MIME-Version: 1.0\r
Content-Type: multipart/mixed; boundary=\"b\"\r
\r
--b\r
Content-Type: text/plain\r
\r
See attached.\r
--b\r
Content-Type: application/pdf\r
Content-Disposition: attachment; filename=\"bill.pdf\"\r
\r
bill\r
--b\r
Content-Type: image/png\r
Content-Disposition: inline; filename=\"logo.png\"\r
\r
logo\r
--b\r
Content-Type: application/octet-stream\r
Content-Disposition: attachment; filename=\"scan.PDF\"\r
\r
scan\r
--b\r
Content-Type: image/jpeg\r
Content-Disposition: attachment\r
\r
photo\r
--b\r
Content-Type: text/csv\r
Content-Disposition: attachment; filename=\"2026/usage.csv\"\r
\r
csv\r
--b--\r
";

    fn names(allowed: &str) -> Vec<String> {
        documents(EMAIL.as_bytes(), &Extensions::parse(allowed))
            .into_iter()
            .map(|document| document.name)
            .collect()
    }

    #[test]
    fn keeps_attachments_with_allowed_extensions_and_skips_inline_images() {
        assert_eq!(
            names(" .PDF, jpeg, png ,"),
            vec!["bill.pdf", "scan.PDF", "attachment.jpeg"]
        );
    }

    #[test]
    fn replaces_slashes_so_each_document_is_one_file() {
        assert_eq!(names("csv"), vec!["2026_usage.csv"]);
    }

    #[test]
    fn keeps_nothing_when_no_extensions_are_allowed() {
        assert_eq!(names(""), Vec::<String>::new());
    }

    #[test]
    fn returns_nothing_for_unparseable_input() {
        assert_eq!(documents(b"", &Extensions::parse("pdf")), vec![]);
    }
}
