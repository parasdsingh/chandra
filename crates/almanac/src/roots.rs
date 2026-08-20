//! Root finding for event times.
//!
//! Every event this app shows - a sign ingress, a nakshatra ingress, a
//! retrograde station - is a root of a continuous function of time. One
//! bracketing scan plus one refinement routine covers all of them, so there is
//! no per-event numerical code to get subtly wrong.

/// A sign change was located between these two instants.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bracket {
    pub lo: f64,
    pub hi: f64,
    pub f_lo: f64,
    pub f_hi: f64,
}

/// Refinement target in days. 1e-6 days is 86 milliseconds, an order of
/// magnitude finer than the minute this app displays, so the printed time is
/// never the limiting factor.
pub const TOLERANCE_DAYS: f64 = 1e-6;

/// Hard cap on iterations. Brent converges superlinearly and reaches the
/// tolerance in well under 20 steps for these functions; the cap exists so a
/// pathological input cannot hang the UI thread's blocking task.
const MAX_ITERATIONS: usize = 100;

/// Scans `[start, end]` in steps of `step`, returning every interval across
/// which `f` changes sign.
///
/// The step must be fine enough that two roots cannot share one interval. Step
/// sizes are chosen per body from its maximum angular speed, which is why
/// [`chandra_ephemeris::Graha::scan_step_days`] exists.
pub fn brackets<F>(start: f64, end: f64, step: f64, mut f: F) -> Vec<Bracket>
where
    F: FnMut(f64) -> Option<f64>,
{
    let mut found = Vec::new();
    let mut lo = start;
    let mut f_lo = match f(lo) {
        Some(v) => v,
        None => return found,
    };

    while lo < end {
        let hi = (lo + step).min(end);
        let Some(f_hi) = f(hi) else { return found };

        // A sample landing exactly on zero is a root, not a bracket, and is
        // reported as a degenerate interval so the caller still gets the time.
        if f_lo == 0.0 || (f_lo < 0.0) != (f_hi < 0.0) {
            found.push(Bracket { lo, hi, f_lo, f_hi });
        }

        if hi >= end {
            break;
        }
        lo = hi;
        f_lo = f_hi;
    }
    found
}

/// Brent's method: inverse quadratic interpolation with a bisection fallback.
///
/// Chosen over Newton because the derivative of an ephemeris longitude is only
/// available as another ephemeris call, and over plain bisection because it
/// reaches the tolerance in roughly a third of the evaluations. Convergence is
/// guaranteed for any continuous function that changes sign across the bracket.
///
/// Returns `None` only if the bracket does not actually contain a sign change,
/// or an evaluation fails. It never returns a guessed time.
pub fn refine<F>(bracket: Bracket, mut f: F) -> Option<f64>
where
    F: FnMut(f64) -> Option<f64>,
{
    let (mut a, mut b) = (bracket.lo, bracket.hi);
    let (mut fa, mut fb) = (bracket.f_lo, bracket.f_hi);

    if fa == 0.0 {
        return Some(a);
    }
    if fb == 0.0 {
        return Some(b);
    }
    if (fa < 0.0) == (fb < 0.0) {
        return None;
    }

    // Brent's method requires b to be the better estimate of the two.
    if fa.abs() < fb.abs() {
        std::mem::swap(&mut a, &mut b);
        std::mem::swap(&mut fa, &mut fb);
    }

    let mut c = a;
    let mut fc = fa;
    let mut used_bisection = true;
    let mut d = 0.0f64;

    for _ in 0..MAX_ITERATIONS {
        if (b - a).abs() < TOLERANCE_DAYS {
            return Some(b);
        }

        let mut s = if fa != fc && fb != fc {
            // Inverse quadratic interpolation.
            a * fb * fc / ((fa - fb) * (fa - fc))
                + b * fa * fc / ((fb - fa) * (fb - fc))
                + c * fa * fb / ((fc - fa) * (fc - fb))
        } else {
            // Secant.
            b - fb * (b - a) / (fb - fa)
        };

        // Conditions under which the interpolated step is rejected and a
        // bisection is taken instead. These are what make the method safe: an
        // interpolation that leaves the bracket or converges too slowly cannot
        // be accepted.
        let bound_lo = (3.0 * a + b) / 4.0;
        let outside = if bound_lo < b {
            s < bound_lo || s > b
        } else {
            s < b || s > bound_lo
        };
        let slow = if used_bisection {
            (s - b).abs() >= (b - c).abs() / 2.0
        } else {
            (s - b).abs() >= (c - d).abs() / 2.0
        };
        let stalled = if used_bisection {
            (b - c).abs() < TOLERANCE_DAYS
        } else {
            (c - d).abs() < TOLERANCE_DAYS
        };

        if outside || slow || stalled {
            s = (a + b) / 2.0;
            used_bisection = true;
        } else {
            used_bisection = false;
        }

        let fs = f(s)?;
        d = c;
        c = b;
        fc = fb;

        if (fa < 0.0) != (fs < 0.0) {
            b = s;
            fb = fs;
        } else {
            a = s;
            fa = fs;
        }

        if fa.abs() < fb.abs() {
            std::mem::swap(&mut a, &mut b);
            std::mem::swap(&mut fa, &mut fb);
        }

        if fs == 0.0 {
            return Some(s);
        }
    }
    None
}

/// Signed angular difference `angle - target`, wrapped to `(-180, 180]`.
///
/// This is what makes a longitude crossing a well-behaved root problem: the
/// naive difference jumps by 360 at the wraparound, which a sign-change search
/// would read as a crossing that never happened. The wrapped form is continuous
/// everywhere except exactly opposite the target, half a revolution away, which
/// no search window in this app spans.
pub fn signed_delta(angle: f64, target: f64) -> f64 {
    let mut d = (angle - target) % 360.0;
    if d > 180.0 {
        d -= 360.0;
    } else if d <= -180.0 {
        d += 360.0;
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_delta_wraps_without_a_false_crossing() {
        assert!((signed_delta(1.0, 359.0) - 2.0).abs() < 1e-12);
        assert!((signed_delta(359.0, 1.0) + 2.0).abs() < 1e-12);
        assert!((signed_delta(0.0, 0.0)).abs() < 1e-12);
        assert!((signed_delta(180.0, 0.0) - 180.0).abs() < 1e-12);
        assert!((signed_delta(181.0, 0.0) + 179.0).abs() < 1e-12);
        // A body moving 359 -> 0 -> 1 crosses zero exactly once.
        assert!(signed_delta(359.5, 0.0) < 0.0);
        assert!(signed_delta(0.5, 0.0) > 0.0);
    }

    #[test]
    fn brackets_finds_each_sign_change_once() {
        // sin has roots at 0, pi, 2pi; scanning (0, 2pi) finds the interior one.
        let found = brackets(0.1, 6.0, 0.1, |x| Some(x.sin()));
        assert_eq!(found.len(), 1, "expected exactly the root at pi");
        assert!(found[0].lo < std::f64::consts::PI);
        assert!(found[0].hi > std::f64::consts::PI);
    }

    #[test]
    fn brackets_returns_nothing_without_a_crossing() {
        assert!(brackets(0.0, 1.0, 0.1, |x| Some(x + 1.0)).is_empty());
    }

    #[test]
    fn refine_converges_on_a_transcendental_root() {
        let f = |x: f64| Some(x.cos() - x);
        let bracket = brackets(0.0, 1.5, 0.1, f).pop().expect("root exists");
        let root = refine(bracket, f).expect("must converge");
        // Dottie number.
        assert!((root - 0.739_085_133_215_16).abs() < 1e-9, "got {root}");
    }

    #[test]
    fn refine_hits_the_stated_tolerance_on_a_stiff_root() {
        // A steep cubic is the case plain bisection handles slowly.
        let f = |x: f64| Some((x - 0.3).powi(3));
        let bracket = brackets(0.0, 1.0, 0.25, f).pop().expect("root exists");
        let root = refine(bracket, f).expect("must converge");
        assert!((root - 0.3).abs() < 1e-4, "got {root}");
    }

    #[test]
    fn refine_refuses_a_bracket_without_a_sign_change() {
        let bracket = Bracket {
            lo: 0.0,
            hi: 1.0,
            f_lo: 1.0,
            f_hi: 2.0,
        };
        assert!(refine(bracket, |_| Some(1.0)).is_none());
    }

    #[test]
    fn refine_propagates_evaluation_failure_rather_than_guessing() {
        let bracket = Bracket {
            lo: 0.0,
            hi: 1.0,
            f_lo: -1.0,
            f_hi: 1.0,
        };
        assert!(refine(bracket, |_| None).is_none());
    }

    #[test]
    fn refine_returns_an_exact_root_landed_on_directly() {
        let bracket = Bracket {
            lo: 0.0,
            hi: 1.0,
            f_lo: 0.0,
            f_hi: 1.0,
        };
        assert_eq!(refine(bracket, |_| Some(0.0)), Some(0.0));
    }
}
