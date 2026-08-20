//! Transit events within a month: ingress, station, combustion.
//!
//! All three are sign changes of some continuous function of time, so all three
//! use the same scan-then-refine machinery in [`crate::roots`].

use chandra_ephemeris::{Engine, Graha, Source};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::roots::{self, Bracket};
use crate::spans::Division;
use crate::time::{CivilDay, DateKey, Moment};
use crate::zodiac::{Nakshatra, Rashi};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    RashiIngress,
    NakshatraIngress,
    /// Direct motion turns retrograde.
    RetrogradeStation,
    /// Retrograde motion turns direct.
    DirectStation,
    /// Enters the Sun's rays and is lost to visibility.
    CombustionStart,
    CombustionEnd,
}

impl EventKind {
    /// Instants are a single moment; spans continue until their partner event.
    /// The two are drawn differently, so the distinction belongs in the model.
    pub const fn is_span_start(self) -> bool {
        matches!(
            self,
            EventKind::RetrogradeStation | EventKind::CombustionStart
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub kind: EventKind,
    pub graha: Graha,
    pub date: DateKey,
    pub at: Moment,
    /// What the event moves into, where that makes sense: the rashi or nakshatra
    /// entered. Empty for stations and combustion.
    pub target: Option<String>,
    pub source: Source,
}

/// Combustion orbs in degrees of elongation from the Sun.
///
/// Values follow Brihat Parashara Hora Shastra. Some traditions narrow the orb
/// for a retrograde Budha or Shukra; that variant is applied by splitting the
/// scan at stations so the orb is constant across each interval and no spurious
/// crossing is manufactured at the discontinuity.
fn combustion_orb(graha: Graha, retrograde: bool) -> Option<f64> {
    Some(match graha {
        Graha::Chandra => 12.0,
        Graha::Mangala => 17.0,
        Graha::Budha => {
            if retrograde {
                12.0
            } else {
                14.0
            }
        }
        Graha::Guru => 11.0,
        Graha::Shukra => {
            if retrograde {
                8.0
            } else {
                10.0
            }
        }
        Graha::Shani => 15.0,
        // The Sun cannot be combust by itself, and the nodes are geometric
        // points with no visibility to lose.
        Graha::Surya | Graha::Rahu | Graha::Ketu => return None,
    })
}

/// Every event for one graha between two instants.
pub fn events_in_range(
    engine: &Engine,
    graha: Graha,
    start_jd: f64,
    end_jd: f64,
    day_for: &dyn Fn(f64) -> Result<(DateKey, Moment)>,
) -> Result<Vec<Event>> {
    let step = graha.scan_step_days();
    let mut events = Vec::new();

    events.extend(ingress_events(
        engine,
        graha,
        Division::Rashi,
        start_jd,
        end_jd,
        step,
        day_for,
    )?);
    events.extend(ingress_events(
        engine,
        graha,
        Division::Nakshatra,
        start_jd,
        end_jd,
        step,
        day_for,
    )?);

    let stations = station_events(engine, graha, start_jd, end_jd, step, day_for)?;
    let station_times: Vec<f64> = stations.iter().map(|(jd, _)| *jd).collect();
    events.extend(stations.into_iter().map(|(_, event)| event));

    events.extend(combustion_events(
        engine,
        graha,
        start_jd,
        end_jd,
        step,
        &station_times,
        day_for,
    )?);

    events.sort_by(|a, b| a.at.unix_ms.cmp(&b.at.unix_ms));
    Ok(events)
}

fn ingress_events(
    engine: &Engine,
    graha: Graha,
    division: Division,
    start_jd: f64,
    end_jd: f64,
    step: f64,
    day_for: &dyn Fn(f64) -> Result<(DateKey, Moment)>,
) -> Result<Vec<Event>> {
    let arc = division.arc();
    let count = division.count();
    let index_at = |jd: f64| -> Option<(usize, f64, Source)> {
        engine.position(jd, graha).ok().map(|p| {
            (
                ((p.longitude.rem_euclid(360.0) / arc) as usize).min(count - 1),
                p.longitude,
                p.source,
            )
        })
    };

    let mut events = Vec::new();
    let mut jd = start_jd;
    let mut previous = index_at(jd).ok_or(Error::NoCrossing {
        what: "position",
        graha: graha.name(),
        near: jd,
        window_days: 0.0,
    })?;

    while jd < end_jd {
        let next_jd = (jd + step).min(end_jd);
        let Some(current) = index_at(next_jd) else {
            break;
        };

        if current.0 != previous.0 {
            let ascending = (current.0 + count - previous.0) % count == 1;
            let boundary = if ascending { current.0 } else { previous.0 } as f64 * arc;

            let bracket = Bracket {
                lo: jd,
                hi: next_jd,
                f_lo: roots::signed_delta(previous.1, boundary),
                f_hi: roots::signed_delta(current.1, boundary),
            };

            if let Some(exact) = roots::refine(bracket, |t| {
                engine
                    .position(t, graha)
                    .ok()
                    .map(|p| roots::signed_delta(p.longitude, boundary))
            }) {
                let (date, moment) = day_for(exact)?;
                let entered = ((boundary / arc).round() as usize
                    + if ascending { 0 } else { count - 1 })
                    % count;
                events.push(Event {
                    kind: match division {
                        Division::Rashi => EventKind::RashiIngress,
                        Division::Nakshatra => EventKind::NakshatraIngress,
                    },
                    graha,
                    date,
                    at: moment,
                    target: Some(match division {
                        Division::Rashi => Rashi::ALL[entered].name().to_string(),
                        Division::Nakshatra => Nakshatra::ALL[entered].name().to_string(),
                    }),
                    source: current.2,
                });
            }
        }

        previous = current;
        jd = next_jd;
    }
    Ok(events)
}

/// Returns each station with its exact instant, so combustion scanning can split
/// on the same times and keep its orb constant across every interval.
fn station_events(
    engine: &Engine,
    graha: Graha,
    start_jd: f64,
    end_jd: f64,
    step: f64,
    day_for: &dyn Fn(f64) -> Result<(DateKey, Moment)>,
) -> Result<Vec<(f64, Event)>> {
    if !graha.can_station() {
        return Ok(Vec::new());
    }

    let speed = |jd: f64| engine.position(jd, graha).ok().map(|p| p.speed);
    let brackets = roots::brackets(start_jd, end_jd, step, speed);

    let mut events = Vec::new();
    for bracket in brackets {
        let Some(exact) = roots::refine(bracket, speed) else {
            continue;
        };
        let (date, moment) = day_for(exact)?;
        events.push((
            exact,
            Event {
                // Speed falling through zero is the turn to retrograde.
                kind: if bracket.f_lo > 0.0 {
                    EventKind::RetrogradeStation
                } else {
                    EventKind::DirectStation
                },
                graha,
                date,
                at: moment,
                target: None,
                source: engine.position(exact, graha)?.source,
            },
        ));
    }
    Ok(events)
}

fn combustion_events(
    engine: &Engine,
    graha: Graha,
    start_jd: f64,
    end_jd: f64,
    step: f64,
    station_times: &[f64],
    day_for: &dyn Fn(f64) -> Result<(DateKey, Moment)>,
) -> Result<Vec<Event>> {
    if combustion_orb(graha, false).is_none() {
        return Ok(Vec::new());
    }

    // Split at stations so the orb is constant within each interval. Without
    // this, the orb stepping between its direct and retrograde values at a
    // station would look like a crossing and invent an event.
    let mut boundaries = vec![start_jd];
    boundaries.extend(
        station_times
            .iter()
            .copied()
            .filter(|&t| t > start_jd && t < end_jd),
    );
    boundaries.push(end_jd);

    let mut events = Vec::new();
    for window in boundaries.windows(2) {
        let (from, to) = (window[0], window[1]);
        let midpoint = (from + to) / 2.0;
        let retrograde = engine.position(midpoint, graha)?.is_retrograde();
        let Some(orb) = combustion_orb(graha, retrograde) else {
            continue;
        };

        // Positive outside the Sun's rays, negative within them.
        let separation = |jd: f64| -> Option<f64> {
            let bodies = engine.positions(jd, &[graha, Graha::Surya]).ok()?;
            let elongation = roots::signed_delta(bodies[0].longitude, bodies[1].longitude).abs();
            Some(elongation - orb)
        };

        for bracket in roots::brackets(from, to, step, separation) {
            let Some(exact) = roots::refine(bracket, separation) else {
                continue;
            };
            let (date, moment) = day_for(exact)?;
            events.push(Event {
                // Separation decreasing through the orb is the start of
                // combustion.
                kind: if bracket.f_lo > 0.0 {
                    EventKind::CombustionStart
                } else {
                    EventKind::CombustionEnd
                },
                graha,
                date,
                at: moment,
                target: None,
                source: engine.position(exact, graha)?.source,
            });
        }
    }
    Ok(events)
}

/// Convenience wrapper binding [`events_in_range`] to a month's civil days.
pub fn events_in_month(engine: &Engine, graha: Graha, days: &[CivilDay]) -> Result<Vec<Event>> {
    let Some(first) = days.first() else {
        return Ok(Vec::new());
    };
    let last = days.last().expect("non-empty");

    let day_for = |jd: f64| -> Result<(DateKey, Moment)> {
        // Attribute the event to the civil day containing it, and express the
        // moment relative to that same day so `day_offset` is zero.
        let date = first.date_of(jd)?;
        let containing = days
            .iter()
            .find(|d| d.date == date)
            .unwrap_or(if jd < first.start_jd { first } else { last });
        Ok((date, containing.moment(jd)?))
    };

    events_in_range(engine, graha, first.start_jd, last.end_jd, &day_for)
}
