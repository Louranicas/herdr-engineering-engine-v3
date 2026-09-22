use super::{Class, Error, Profile, profile};
use crate::contracts::UuidV4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Health {
    Useful,
    Unavailable,
    Unknown,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UsefulResult {
    Passed,
    Failed,
    Unknown,
    Cancelled,
    TimedOut,
}
/// Type-specific receiver facts; a listener, loaded image or zero exit alone is
/// never a useful request result. Libraries intentionally have no daemon state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Facts {
    Daemon {
        alive: Option<bool>,
        request: UsefulResult,
    },
    OneShot {
        exit_code: Option<i32>,
        output: UsefulResult,
    },
    Library {
        loaded: Option<bool>,
        call: UsefulResult,
    },
    RemoteEndpoint {
        reachable: Option<bool>,
        request: UsefulResult,
    },
    NeuralOperator {
        model_matches: Option<bool>,
        result: UsefulResult,
    },
}
impl Facts {
    #[must_use]
    pub const fn class(&self) -> Class {
        match self {
            Self::Daemon { .. } => Class::Daemon,
            Self::OneShot { .. } => Class::OneShot,
            Self::Library { .. } => Class::Library,
            Self::RemoteEndpoint { .. } => Class::RemoteEndpoint,
            Self::NeuralOperator { .. } => Class::NeuralOperator,
        }
    }
    fn useful(&self) -> Health {
        let (prerequisite, result) = match self {
            Self::Daemon { alive, request } => (*alive, *request),
            Self::OneShot { exit_code, output } => (exit_code.map(|code| code == 0), *output),
            Self::Library { loaded, call } => (*loaded, *call),
            Self::RemoteEndpoint { reachable, request } => (*reachable, *request),
            Self::NeuralOperator {
                model_matches,
                result,
            } => (*model_matches, *result),
        };
        if matches!(
            result,
            UsefulResult::Unknown | UsefulResult::Cancelled | UsefulResult::TimedOut
        ) {
            return Health::Unknown;
        }
        match (prerequisite, result) {
            (Some(true), UsefulResult::Passed) => Health::Useful,
            (Some(false), _) | (_, UsefulResult::Failed) => Health::Unavailable,
            _ => Health::Unknown,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProbeObservation {
    pub facts: Facts,
    pub probe_id: String,
    pub probe_version: u16,
    pub actual_identity: Option<String>,
    pub immutable_revision: Option<String>,
    pub observed_unix_ms: Option<u64>,
    pub latency_ms: Option<u64>,
    pub usage_ms: Option<u64>,
    pub cost_microunits: Option<u64>,
    pub cleanup_settled: bool,
    pub evidence_ref: String,
}
impl ProbeObservation {
    pub(super) fn validate(&self) -> Result<(), Error> {
        UuidV4::parse(&self.evidence_ref).map_err(|_| Error::Binding)?;
        profile::text(&self.probe_id, 128)?;
        if self.probe_version != 1
            || [self.latency_ms, self.usage_ms]
                .into_iter()
                .flatten()
                .any(|n| n > 60_000)
            || self.cost_microunits.is_some_and(|n| n != 0)
        {
            return Err(Error::Bounds);
        }
        for value in [&self.actual_identity, &self.immutable_revision]
            .into_iter()
            .flatten()
        {
            profile::text(value, 256)?;
        }
        if self.immutable_revision.is_some() && self.actual_identity.is_none() {
            return Err(Error::Binding);
        }
        if let Facts::OneShot {
            exit_code: Some(code),
            ..
        } = self.facts
            && !(0..=255).contains(&code)
        {
            return Err(Error::Bounds);
        }
        Ok(())
    }
    pub(super) fn health(&self, profile: &Profile) -> Health {
        if !self.cleanup_settled
            || self.cost_microunits != Some(0)
            || self.usage_ms.is_none()
            || self.actual_identity.as_deref() != Some(profile.actual_identity.as_str())
            || self.immutable_revision.as_deref() != Some(profile.immutable_revision.as_str())
            || self.probe_id != profile.probe_id
            || self.probe_version != profile.probe_version
        {
            return Health::Unknown;
        }
        self.facts.useful()
    }
}
