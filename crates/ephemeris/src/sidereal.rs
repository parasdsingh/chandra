use serde::{Deserialize, Serialize};

/// Ayanamsa options offered in settings.
///
/// Swiss Ephemeris implements 47 of these. Only the ones with a real user base
/// are exposed: an ayanamsa nobody uses is a way to get wrong answers, not a
/// feature. Lahiri is the default (`docs/DECISIONS.md` D-003).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ayanamsa {
    #[default]
    Lahiri,
    LahiriIcrc,
    Raman,
    Krishnamurti,
    TrueChitra,
    TrueRevati,
    TruePushya,
    Yukteshwar,
    JnBhasin,
    FaganBradley,
    Suryasiddhanta,
    Aryabhata,
}

impl Ayanamsa {
    pub const ALL: [Ayanamsa; 12] = [
        Ayanamsa::Lahiri,
        Ayanamsa::LahiriIcrc,
        Ayanamsa::Raman,
        Ayanamsa::Krishnamurti,
        Ayanamsa::TrueChitra,
        Ayanamsa::TrueRevati,
        Ayanamsa::TruePushya,
        Ayanamsa::Yukteshwar,
        Ayanamsa::JnBhasin,
        Ayanamsa::FaganBradley,
        Ayanamsa::Suryasiddhanta,
        Ayanamsa::Aryabhata,
    ];

    /// `SE_SIDM_*` identifier.
    pub(crate) const fn se_mode(self) -> i32 {
        match self {
            Ayanamsa::FaganBradley => 0,
            Ayanamsa::Lahiri => 1,
            Ayanamsa::Raman => 3,
            Ayanamsa::Krishnamurti => 5,
            Ayanamsa::Yukteshwar => 7,
            Ayanamsa::JnBhasin => 8,
            Ayanamsa::Suryasiddhanta => 21,
            Ayanamsa::Aryabhata => 23,
            Ayanamsa::TrueChitra => 27,
            Ayanamsa::TrueRevati => 28,
            Ayanamsa::TruePushya => 29,
            Ayanamsa::LahiriIcrc => 46,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Ayanamsa::Lahiri => "Lahiri (Chitrapaksha)",
            Ayanamsa::LahiriIcrc => "Lahiri (ICRC)",
            Ayanamsa::Raman => "Raman",
            Ayanamsa::Krishnamurti => "Krishnamurti (KP)",
            Ayanamsa::TrueChitra => "True Chitra",
            Ayanamsa::TrueRevati => "True Revati",
            Ayanamsa::TruePushya => "True Pushya",
            Ayanamsa::Yukteshwar => "Yukteshwar",
            Ayanamsa::JnBhasin => "J. N. Bhasin",
            Ayanamsa::FaganBradley => "Fagan-Bradley",
            Ayanamsa::Suryasiddhanta => "Surya Siddhanta",
            Ayanamsa::Aryabhata => "Aryabhata",
        }
    }

    pub const fn key(self) -> &'static str {
        match self {
            Ayanamsa::Lahiri => "lahiri",
            Ayanamsa::LahiriIcrc => "lahiri_icrc",
            Ayanamsa::Raman => "raman",
            Ayanamsa::Krishnamurti => "krishnamurti",
            Ayanamsa::TrueChitra => "true_chitra",
            Ayanamsa::TrueRevati => "true_revati",
            Ayanamsa::TruePushya => "true_pushya",
            Ayanamsa::Yukteshwar => "yukteshwar",
            Ayanamsa::JnBhasin => "jn_bhasin",
            Ayanamsa::FaganBradley => "fagan_bradley",
            Ayanamsa::Suryasiddhanta => "suryasiddhanta",
            Ayanamsa::Aryabhata => "aryabhata",
        }
    }
}

/// Which lunar node to use for Rahu and Ketu.
///
/// The mean node moves uniformly retrograde at -3'11"/day. The true node
/// oscillates around it and periodically turns direct for a few days, which is
/// visible in the transit calendar and is the reason both are offered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    #[default]
    True,
    Mean,
}

impl NodeType {
    pub const fn label(self) -> &'static str {
        match self {
            NodeType::True => "True node",
            NodeType::Mean => "Mean node",
        }
    }

    pub const fn key(self) -> &'static str {
        match self {
            NodeType::True => "true",
            NodeType::Mean => "mean",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SiderealConfig {
    pub ayanamsa: Ayanamsa,
    pub node_type: NodeType,
}
