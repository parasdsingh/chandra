//! A minimal SVG path-data parser.
//!
//! Glyphs are authored as path strings in `docs/DESIGN.md` because that is how
//! they can be read, reviewed and adjusted as drawings rather than as arithmetic.
//! Only the commands that data uses are supported - deliberately not arcs, since
//! the design spec approximates every arc with cubics so curvature is identical
//! in the source and on screen.

use tiny_skia::{Path, PathBuilder};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum PathError {
    #[error("unsupported path command {0:?}; only M, L, H, V, C, Q and Z are handled")]
    UnsupportedCommand(char),

    #[error("command {command:?} needs {expected} numbers, found {found}")]
    WrongArity {
        command: char,
        expected: usize,
        found: usize,
    },

    #[error("{0:?} is not a number")]
    NotANumber(String),

    #[error("path data does not begin with a move command")]
    NoInitialMove,

    #[error("path data is empty or produced no geometry")]
    Empty,
}

/// Parses SVG path data into a `tiny-skia` path.
pub fn parse(data: &str) -> Result<Path, PathError> {
    let mut builder = PathBuilder::new();
    let mut cursor = (0.0f32, 0.0f32);
    // Where the current subpath began, so `Z` returns to the right point when a
    // path has several subpaths - Surya's ring and centre dot, for instance.
    let mut subpath_start = (0.0f32, 0.0f32);
    let mut started = false;

    for (command, numbers) in tokenize(data)? {
        let relative = command.is_ascii_lowercase();
        let base = if relative { cursor } else { (0.0, 0.0) };

        match command.to_ascii_uppercase() {
            'M' => {
                expect(command, &numbers, 2)?;
                cursor = (base.0 + numbers[0], base.1 + numbers[1]);
                subpath_start = cursor;
                builder.move_to(cursor.0, cursor.1);
                started = true;
            }
            'L' => {
                require_started(started)?;
                expect(command, &numbers, 2)?;
                cursor = (base.0 + numbers[0], base.1 + numbers[1]);
                builder.line_to(cursor.0, cursor.1);
            }
            'H' => {
                require_started(started)?;
                expect(command, &numbers, 1)?;
                cursor = (base.0 + numbers[0], cursor.1);
                builder.line_to(cursor.0, cursor.1);
            }
            'V' => {
                require_started(started)?;
                expect(command, &numbers, 1)?;
                cursor = (cursor.0, base.1 + numbers[0]);
                builder.line_to(cursor.0, cursor.1);
            }
            'C' => {
                require_started(started)?;
                expect(command, &numbers, 6)?;
                let control_a = (base.0 + numbers[0], base.1 + numbers[1]);
                let control_b = (base.0 + numbers[2], base.1 + numbers[3]);
                cursor = (base.0 + numbers[4], base.1 + numbers[5]);
                builder.cubic_to(
                    control_a.0,
                    control_a.1,
                    control_b.0,
                    control_b.1,
                    cursor.0,
                    cursor.1,
                );
            }
            'Q' => {
                require_started(started)?;
                expect(command, &numbers, 4)?;
                let control = (base.0 + numbers[0], base.1 + numbers[1]);
                cursor = (base.0 + numbers[2], base.1 + numbers[3]);
                builder.quad_to(control.0, control.1, cursor.0, cursor.1);
            }
            'Z' => {
                require_started(started)?;
                expect(command, &numbers, 0)?;
                builder.close();
                cursor = subpath_start;
            }
            other => return Err(PathError::UnsupportedCommand(other)),
        }
    }

    builder.finish().ok_or(PathError::Empty)
}

fn require_started(started: bool) -> Result<(), PathError> {
    started.then_some(()).ok_or(PathError::NoInitialMove)
}

fn expect(command: char, numbers: &[f32], arity: usize) -> Result<(), PathError> {
    if numbers.len() == arity {
        Ok(())
    } else {
        Err(PathError::WrongArity {
            command,
            expected: arity,
            found: numbers.len(),
        })
    }
}

/// Splits path data into `(command, numbers)` pairs.
///
/// Repeated coordinate sets after one command letter, which SVG allows, are
/// expanded into separate pairs so the caller sees one command per set.
fn tokenize(data: &str) -> Result<Vec<(char, Vec<f32>)>, PathError> {
    let mut pairs: Vec<(char, Vec<f32>)> = Vec::new();
    let mut current: Option<(char, Vec<f32>)> = None;
    let mut number = String::new();

    let flush_number =
        |number: &mut String, current: &mut Option<(char, Vec<f32>)>| -> Result<(), PathError> {
            if number.is_empty() {
                return Ok(());
            }
            let value: f32 = number
                .parse()
                .map_err(|_| PathError::NotANumber(number.clone()))?;
            if let Some((_, numbers)) = current.as_mut() {
                numbers.push(value);
            }
            number.clear();
            Ok(())
        };

    for character in data.chars() {
        match character {
            'A' | 'a' | 'S' | 's' | 'T' | 't' => {
                return Err(PathError::UnsupportedCommand(character))
            }
            c if c.is_ascii_alphabetic() => {
                flush_number(&mut number, &mut current)?;
                if let Some(pair) = current.take() {
                    pairs.push(pair);
                }
                current = Some((c, Vec::new()));
            }
            // A minus sign starts a new number unless it follows an exponent.
            '-' if !number.is_empty() && !number.ends_with(['e', 'E']) => {
                flush_number(&mut number, &mut current)?;
                number.push('-');
            }
            c if c.is_ascii_digit() || c == '.' || c == '-' || c == 'e' || c == 'E' => {
                number.push(c);
            }
            ',' | ' ' | '\t' | '\n' | '\r' => flush_number(&mut number, &mut current)?,
            other => return Err(PathError::UnsupportedCommand(other)),
        }
    }
    flush_number(&mut number, &mut current)?;
    if let Some(pair) = current.take() {
        pairs.push(pair);
    }

    // Expand repeated coordinate sets: "L 1,2 3,4" is two line commands.
    let mut expanded = Vec::with_capacity(pairs.len());
    for (command, numbers) in pairs {
        let arity = match command.to_ascii_uppercase() {
            'M' | 'L' => 2,
            'H' | 'V' => 1,
            'C' => 6,
            'Q' => 4,
            'Z' => 0,
            other => return Err(PathError::UnsupportedCommand(other)),
        };

        if arity == 0 || numbers.len() <= arity {
            expanded.push((command, numbers));
            continue;
        }
        if numbers.len() % arity != 0 {
            return Err(PathError::WrongArity {
                command,
                expected: arity,
                found: numbers.len(),
            });
        }
        for (index, chunk) in numbers.chunks(arity).enumerate() {
            // A repeated `M` continues as `L`, per the SVG specification.
            let effective = match (command, index) {
                ('M', n) if n > 0 => 'L',
                ('m', n) if n > 0 => 'l',
                (c, _) => c,
            };
            expanded.push((effective, chunk.to_vec()));
        }
    }

    if expanded.is_empty() {
        return Err(PathError::Empty);
    }
    Ok(expanded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_simple_closed_triangle() {
        let path = parse("M 0,0 L 10,0 L 10,10 Z").expect("valid");
        let bounds = path.bounds();
        assert_eq!((bounds.left(), bounds.top()), (0.0, 0.0));
        assert_eq!((bounds.right(), bounds.bottom()), (10.0, 10.0));
    }

    #[test]
    fn handles_relative_commands() {
        let absolute = parse("M 5,5 L 15,5").expect("valid");
        let relative = parse("M 5,5 l 10,0").expect("valid");
        assert_eq!(absolute.bounds(), relative.bounds());
    }

    #[test]
    fn handles_horizontal_and_vertical_shorthands() {
        let path = parse("M 0,0 H 10 V 10").expect("valid");
        let bounds = path.bounds();
        assert_eq!((bounds.right(), bounds.bottom()), (10.0, 10.0));
    }

    #[test]
    fn expands_repeated_coordinate_sets() {
        let repeated = parse("M 0,0 L 10,0 10,10").expect("valid");
        let explicit = parse("M 0,0 L 10,0 L 10,10").expect("valid");
        assert_eq!(repeated.bounds(), explicit.bounds());
    }

    #[test]
    fn a_repeated_move_continues_as_a_line() {
        // Per the SVG specification. Treating it as another move would break the
        // subpath into disconnected points.
        let path = parse("M 0,0 5,5 10,10").expect("valid");
        assert_eq!(path.bounds().right(), 10.0);
    }

    #[test]
    fn close_returns_to_the_start_of_the_current_subpath() {
        // Two subpaths; the second Z must return to (20,20), not to (0,0).
        let path = parse("M 0,0 L 5,0 Z M 20,20 L 25,20 Z").expect("valid");
        let bounds = path.bounds();
        assert_eq!((bounds.left(), bounds.top()), (0.0, 0.0));
        assert_eq!((bounds.right(), bounds.bottom()), (25.0, 20.0));
    }

    #[test]
    fn negative_numbers_split_without_a_separator() {
        let path = parse("M 0,0 L-10,-10").expect("valid");
        let bounds = path.bounds();
        assert_eq!((bounds.left(), bounds.top()), (-10.0, -10.0));
    }

    #[test]
    fn arcs_are_refused_rather_than_silently_dropped() {
        // The design spec forbids arcs so that curvature matches between the
        // source drawing and the render. Accepting and ignoring one would give a
        // subtly wrong glyph with no error.
        assert_eq!(
            parse("M 0,0 A 5,5 0 0 1 10,10"),
            Err(PathError::UnsupportedCommand('A'))
        );
        assert_eq!(
            parse("M 0,0 S 1,1 2,2"),
            Err(PathError::UnsupportedCommand('S'))
        );
    }

    #[test]
    fn malformed_input_is_rejected() {
        assert_eq!(parse(""), Err(PathError::Empty));
        assert_eq!(parse("L 1,2"), Err(PathError::NoInitialMove));
        assert!(matches!(
            parse("M 1"),
            Err(PathError::WrongArity { command: 'M', .. })
        ));
        assert!(matches!(
            parse("M 0,0 C 1,2,3"),
            Err(PathError::WrongArity { command: 'C', .. })
        ));
    }
}
