use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;
use std::{fmt, path::Path};

use console::{style, Style};
use pdf::{content::Op, file::FileOptions};
use similar::{ChangeTag, TextDiff};

struct Line(Option<usize>);

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            None => write!(f, "    "),
            Some(idx) => write!(f, "{:<4}", idx + 1),
        }
    }
}

fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 3 {
        eprintln!("usage: pdf-text-diff [old.pdf] [new.pdf]");
        exit(1);
    }

    let old = pdf_to_string(
        PathBuf::from_str(args[1].to_str().unwrap())
            .unwrap()
            .as_path(),
    );
    let new = pdf_to_string(
        PathBuf::from_str(args[2].to_str().unwrap())
            .unwrap()
            .as_path(),
    );
    let diff = TextDiff::from_lines(&old, &new);

    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            println!("{:-^1$}", "-", 80);
        }
        for op in group {
            for change in diff.iter_inline_changes(op) {
                let (sign, s) = match change.tag() {
                    ChangeTag::Delete => ("-", Style::new().red()),
                    ChangeTag::Insert => ("+", Style::new().green()),
                    ChangeTag::Equal => (" ", Style::new().dim()),
                };
                print!(
                    "{}{} |{}",
                    style(Line(change.old_index())).dim(),
                    style(Line(change.new_index())).dim(),
                    s.apply_to(sign).bold(),
                );
                for (emphasized, value) in change.iter_strings_lossy() {
                    if emphasized {
                        print!("{}", s.apply_to(value).underlined());
                    } else {
                        print!("{}", s.apply_to(value));
                    }
                }
                if change.missing_newline() {
                    println!();
                }
            }
        }
    }
}

fn pdf_to_string(path: &Path) -> String {
    let file = FileOptions::cached().open(path).unwrap();

    let resolver = file.resolver();

    let mut out = String::new();

    let mut last_y: Option<f32> = None;

    for page in file.pages() {
        let page = page.unwrap();

        for op in page
            .contents
            .as_ref()
            .unwrap()
            .operations(&resolver)
            .unwrap()
        {
            // println!("o: {:?}", op);

            match op {
                // Op::EndMarkedContent => {
                //     println!("EndMarkedContent");
                //     out.push_str("\n")
                // }
                Op::TextDraw { text } => {
                    // println!("TextDraw: {:?}", text);

                    out.push_str(&String::from_utf8_lossy(text.as_bytes()));
                }
                Op::TextNewline => out.push_str("\n"),
                Op::TextDrawAdjusted { array } => {
                    let mut text = String::new();

                    for e in array {
                        match e {
                            pdf::content::TextDrawAdjusted::Text(t) => {
                                text.push_str(&String::from_utf8_lossy(t.as_bytes()));
                            }
                            pdf::content::TextDrawAdjusted::Spacing(_) => (),
                        }
                    }

                    // println!("TextDrawAdjusted: {}", text);

                    out.push_str(&text);
                }
                // Op::TextScaling { horiz_scale } => todo!(),
                Op::SetTextMatrix { matrix } => {
                    // println!("SetTextMatrix: {:?}", matrix);

                    let y = matrix.d + matrix.f;

                    if last_y.map(|last_y| y != last_y).unwrap_or_else(|| true) {
                        if last_y.is_some() {
                            out.push_str("\n");
                        }

                        last_y = Some(y);
                    }
                }
                // Op::BeginMarkedContent { tag, properties } => todo!(),
                // Op::MarkedContentPoint { tag, properties } => todo!(),
                // Op::Close => todo!(),
                // Op::MoveTo { p } => todo!(),
                // Op::LineTo { p } => todo!(),
                // Op::CurveTo { c1, c2, p } => todo!(),
                // Op::Rect { rect } => todo!(),
                // Op::EndPath => todo!(),
                // Op::Stroke => todo!(),
                // Op::FillAndStroke { winding } => todo!(),
                // Op::Fill { winding } => todo!(),
                // Op::Shade { name } => todo!(),
                // Op::Clip { winding } => todo!(),
                // Op::Save => todo!(),
                // Op::Restore => todo!(),
                // Op::Transform { matrix } => todo!(),
                // Op::LineWidth { width } => todo!(),
                // Op::Dash { pattern, phase } => todo!(),
                // Op::LineJoin { join } => todo!(),
                // Op::LineCap { cap } => todo!(),
                // Op::MiterLimit { limit } => todo!(),
                // Op::Flatness { tolerance } => todo!(),
                // Op::GraphicsState { name } => todo!(),
                // Op::StrokeColor { color } => todo!(),
                // Op::FillColor { color } => todo!(),
                // Op::FillColorSpace { name } => todo!(),
                // Op::StrokeColorSpace { name } => todo!(),
                // Op::RenderingIntent { intent } => todo!(),
                // Op::BeginText => todo!(),
                // Op::EndText => ,
                // Op::CharSpacing { char_space } => todo!(),
                // Op::WordSpacing { word_space } => todo!(),
                // Op::Leading { leading } => todo!(),
                // Op::TextFont { name, size } => todo!(),
                // Op::TextRenderMode { mode } => todo!(),
                // Op::TextRise { rise } => todo!(),
                // Op::MoveTextPosition { translation } => todo!(),
                // Op::XObject { name } => todo!(),
                // Op::InlineImage { image } => todo!(),
                _ => (),
            }
        }

        // out.push_str("\n[page break]\n");
    }

    out
}
