use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use comma_connect_api::RouteFiles;

/// Upload status for one file type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusSymbol {
    pub letter: char,
    pub color: Color,
    pub dim: bool,
}

/// Compute the upload status symbol group for a route.
///
/// `segment_count` is maxqlog + 1 (segments are 0-indexed).
/// Order: l c L F W D
/// Normal color = all segments uploaded, dim = partial, space = none.
pub fn upload_status_symbols(files: &RouteFiles, segment_count: i32) -> Vec<StatusSymbol> {
    let kinds = [
        ('l', Color::DarkGray, files.qlogs.len()),
        ('c', Color::Yellow, files.qcameras.len()),
        ('L', Color::Green, files.logs.len()),
        ('F', Color::Cyan, files.cameras.len()),
        ('W', Color::Magenta, files.ecameras.len()),
        ('D', Color::Red, files.dcameras.len()),
    ];
    kinds
        .into_iter()
        .map(|(letter, color, uploaded)| {
            let dim = if segment_count <= 0 {
                false
            } else if (uploaded as i32) >= segment_count {
                false // full
            } else if uploaded > 0 {
                true // partial
            } else {
                false // none, will be space
            };
            let ch = if uploaded == 0 { ' ' } else { letter };
            StatusSymbol {
                letter: ch,
                color,
                dim,
            }
        })
        .collect()
}

/// Convert status symbols to ratatui Spans for display.
pub fn status_spans(symbols: &[StatusSymbol]) -> Vec<Span<'static>> {
    symbols
        .iter()
        .map(|s| {
            let mut style = Style::default().fg(s.color);
            if s.dim {
                style = style.add_modifier(Modifier::DIM);
            }
            Span::styled(s.letter.to_string(), style)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use comma_connect_api::RouteFiles;
    use ratatui::style::Color;

    fn files_with(qlogs: usize, qcams: usize, logs: usize, cams: usize, ecams: usize, dcams: usize) -> RouteFiles {
        RouteFiles {
            qlogs: vec!["x".into(); qlogs],
            qcameras: vec!["x".into(); qcams],
            logs: vec!["x".into(); logs],
            cameras: vec!["x".into(); cams],
            ecameras: vec!["x".into(); ecams],
            dcameras: vec!["x".into(); dcams],
            ..Default::default()
        }
    }

    #[test]
    fn full_and_partial_and_none() {
        // maxqlog=2 => 3 segments
        let files = files_with(3, 3, 1, 0, 2, 0);
        let symbols = upload_status_symbols(&files, 3);
        assert_eq!(symbols.len(), 6);
        // l full normal grey
        assert_eq!(symbols[0].letter, 'l');
        assert_eq!(symbols[0].color, Color::DarkGray);
        assert!(!symbols[0].dim);
        // c full normal yellow
        assert_eq!(symbols[1].letter, 'c');
        assert_eq!(symbols[1].color, Color::Yellow);
        assert!(!symbols[1].dim);
        // L partial dim green
        assert_eq!(symbols[2].letter, 'L');
        assert_eq!(symbols[2].color, Color::Green);
        assert!(symbols[2].dim);
        // F none space
        assert_eq!(symbols[3].letter, ' ');
        // W partial dim magenta
        assert_eq!(symbols[4].letter, 'W');
        assert_eq!(symbols[4].color, Color::Magenta);
        assert!(symbols[4].dim);
        // D none space
        assert_eq!(symbols[5].letter, ' ');
    }

    #[test]
    fn all_full() {
        let files = files_with(2, 2, 2, 2, 2, 2);
        let symbols = upload_status_symbols(&files, 2);
        assert!(symbols.iter().all(|s| s.letter != ' ' && !s.dim));
        assert_eq!(symbols[0].letter, 'l');
        assert_eq!(symbols[3].letter, 'F');
    }

    #[test]
    fn all_none() {
        let files = files_with(0, 0, 0, 0, 0, 0);
        let symbols = upload_status_symbols(&files, 5);
        assert!(symbols.iter().all(|s| s.letter == ' '));
    }
}
