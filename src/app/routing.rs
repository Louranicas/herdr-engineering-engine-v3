//! `task.preview`'s composition (B07): route's declared policy and recipes (`routes.toml`, one
//! parse) joined with the caller's roster snapshot, screened by route's own filters, reported as
//! eligible recipes and coded exclusions. Nothing here writes, reserves or starts anything.
//!
//! Design and its three independent reviews: `~/hee3-evidence/T28/B07-task-preview-20260925/`.
//!
//! * **One parse, composed once.** [`compose`] is pure over the configuration's bytes; [`read`]
//!   is the only I/O, through [`crate::app::custody`]. A [`Policy`] is derived per preview,
//!   because the baseline's locality is a roster fact that can change while the process runs.
//! * **A recipe restates no roster fact.** Its record's locality and current observation come
//!   from the snapshot; its figures and the classes it serves from the declaration.
//! * **Capabilities: the task's requirement that the recipe serves.** A task requires exactly its
//!   class, and suitability for a class is a recipe fact (`serves`; no roster observer can observe
//!   it), so route is handed `serves ∩ requirement`: at most one label, valid by construction, so no
//!   admitted roster record can make route refuse the whole preview. Roster labels do not enter:
//!   under RC01 nothing a task requires is a roster capability, and a path they could take would
//!   decide nothing (review G1). **Deferred, named:** when a class first requires a non-class
//!   capability, the roster's evidenced set ([`roster::evidenced_capabilities`], the one
//!   declared-and-observed rule) joins here, with the declared set when no observation is
//!   current, so availability (R05), not capability (R02), reports the missing fact.
//! * **Eligibility is route's screening, read from its explanation.** No second filter: the steps
//!   route records are mapped to the wire's codes (design P7). A recipe that passed every filter
//!   but lacks a ranking figure is eligible (eligibility is the filters; ranking is the choice).

use crate::app::custody::{DirectoryError, FileError, PrivateDirectory};
use crate::contracts::control::{ErrorCode, Fault, Retry};
use crate::contracts::rc01::MAX_INPUT_TOKENS;
use crate::contracts::roster::{self, Freshness, Snapshot};
use crate::route::{
    self, ConfigError, DeclaredRecipe, Observation, PrivacyClass, Recipe, Rule, Step, Task,
};
use crate::task::control::{ADMITTED_CLASSES, Spec};
use serde_json::{Value, json};
use std::path::Path;

/// Where the operator installs the route configuration, under the home directory: its own 0700
/// directory, mirroring the grants' (design R3.3).
pub const ROUTING_DIRECTORY: &str = ".config/herdr-engineering-engine-v3/routing";
/// The route configuration's file name in [`ROUTING_DIRECTORY`].
pub const ROUTES_FILE: &str = "routes.toml";
/// The largest route configuration read, and the acquisition bound for everything parsed from it
/// (`toml` parses the whole file before route counts its rows). The shipped file with its anchor
/// block plus 128 recipe rows, every value at its widest, fits with room (B07-P8). It is not a
/// bound on formatting: a file padded with whitespace or comments past it is refused as too large.
pub const MAX_ROUTE_CONFIG_BYTES: u64 = 262_144;

// Availability maps `Expired` to an observation and lets route's R05 bound decide staleness; that
// is sound only while route's bound cannot exceed the roster's TTL ceiling.
const _: () = assert!(route::MAX_STALENESS_MS <= roster::MAX_TTL_MS);

/// Why preview cannot be composed. Each reads as its own static constraint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Unready {
    /// No routing directory, or no `routes.toml` in it.
    NotInstalled,
    /// Something is installed and is not trusted or not servable: custody, size, encoding, a
    /// refused declaration or recipe, a class the catalogue does not admit, or a baseline that
    /// names no declared recipe.
    Refused,
}

impl Unready {
    /// The static constraint a preview refusal names.
    #[must_use]
    pub const fn constraint(self) -> &'static str {
        match self {
            Self::NotInstalled => "route configuration not installed",
            Self::Refused => "route configuration refused",
        }
    }

    /// The `unavailable` refusal a preview answers with while routing is not composed.
    #[must_use]
    pub fn fault(self) -> Fault {
        unavailable(self.constraint())
    }
}

/// A composed route configuration: parsed once, every declared class admitted, the baseline among
/// the declared recipes.
#[derive(Clone, Debug)]
pub struct Routing {
    parsed: route::Routing,
}

/// Compose the configuration `source` (pure).
///
/// # Errors
///
/// [`Unready::Refused`] for bytes that are not UTF-8, a declaration or recipe `route` refuses, a
/// recipe serving a class outside [`ADMITTED_CLASSES`], a baseline that names no declared recipe,
/// and a baseline recipe lacking a figure the ranking reads (a configuration fact no roster change
/// can mend, so it is refused here and at start, never per preview — review D1).
pub fn compose(source: &[u8]) -> Result<Routing, Unready> {
    let text = std::str::from_utf8(source).map_err(|_| Unready::Refused)?;
    let parsed = route::Routing::parse(text).map_err(|_| Unready::Refused)?;
    let admitted = parsed.recipes().iter().all(|recipe| {
        recipe
            .serves
            .iter()
            .all(|class| ADMITTED_CLASSES.contains(&class.as_str()))
    });
    let baseline_usable = parsed
        .declared_baseline()
        .is_some_and(|baseline| parsed.carries_ranking(baseline));
    if !admitted || !baseline_usable {
        return Err(Unready::Refused);
    }
    Ok(Routing { parsed })
}

/// Read and compose `directory`/[`ROUTES_FILE`] under custody (the only I/O here).
///
/// # Errors
///
/// [`Unready::NotInstalled`] when the directory or the file is absent; [`Unready::Refused`] for a
/// custody failure (not the operator's 0700 directory or 0600 file, a link), an oversize file, an
/// unreadable one, and every refusal of [`compose`].
pub fn read(directory: &Path) -> Result<Routing, Unready> {
    let held = match PrivateDirectory::open(directory) {
        Ok(held) => held,
        Err(DirectoryError::NotFound) => return Err(Unready::NotInstalled),
        Err(DirectoryError::Custody | DirectoryError::Io(_)) => return Err(Unready::Refused),
    };
    match held.read(ROUTES_FILE, MAX_ROUTE_CONFIG_BYTES) {
        Ok(bytes) => compose(&bytes),
        Err(FileError::NotFound) => Err(Unready::NotInstalled),
        Err(FileError::Custody | FileError::TooLarge | FileError::Io(_)) => Err(Unready::Refused),
    }
}

/// Every preview `unavailable`: one message, the constraint naming which reason.
fn unavailable(constraint: &'static str) -> Fault {
    Fault::of(
        ErrorCode::Unavailable,
        Retry::AfterCondition,
        "task.preview cannot answer: its route configuration or baseline is not available",
    )
    .because(constraint)
}

fn internal() -> Fault {
    Fault::of(
        ErrorCode::Internal,
        Retry::Never,
        "route composition produced a value outside its own contract",
    )
}

/// A declared recipe joined with the roster facts route needs, owned so `Recipe` can borrow it.
struct Joined<'r> {
    declared: &'r DeclaredRecipe,
    revision: String,
    capabilities: Vec<&'r str>,
    locality: roster::Locality,
    availability: Observation,
}

impl Joined<'_> {
    fn recipe(&self) -> Recipe<'_> {
        Recipe {
            id: &self.declared.id,
            revision: &self.revision,
            capabilities: &self.capabilities,
            context_limit_tokens: self.declared.context_limit_tokens,
            locality: self.locality,
            availability: self.availability,
            cost_microunits: self.declared.cost_microunits,
            quality_basis_points: self.declared.quality_basis_points,
            latency_ms: self.declared.latency_ms,
        }
    }
}

/// Join `declared` with its record in `snapshot`, or `None` when that record is absent from the
/// caller's snapshot or disabled (the recipe is then `unavailable`, and never handed to route).
fn join<'r>(
    declared: &'r DeclaredRecipe,
    snapshot: &'r Snapshot,
    required: &[&'r str],
) -> Option<Joined<'r>> {
    let record = snapshot
        .records
        .iter()
        .find(|record| record.head.record_id == declared.roster_record)?;
    if record.head.disabled {
        return None;
    }
    let observation = record.observation.as_ref();
    let capabilities = required
        .iter()
        .copied()
        .filter(|capability| declared.serves.iter().any(|class| class == capability))
        .collect();
    let availability = match (
        roster::freshness(&record.head, observation, &snapshot.now, roster::MAX_TTL_MS),
        observation,
    ) {
        (Freshness::Fresh | Freshness::Expired, Some(observed)) => {
            match roster::age_ms(observed, &snapshot.now) {
                Some(age_ms) => Observation::Observed {
                    availability: observed.input.availability,
                    age_ms,
                },
                None => Observation::Unobserved,
            }
        }
        _ => Observation::Unobserved,
    };
    Some(Joined {
        declared,
        revision: record.head.record_version.clone(),
        capabilities,
        locality: record.head.definition.locality,
        availability,
    })
}

/// The wire's exclusion code for a screening that stopped at `rule` (design P7), or `None` for a
/// rule a filter cannot stop at.
const fn code(rule: Rule, excluded: bool) -> Option<&'static str> {
    match rule {
        Rule::R02RequiredCapabilities | Rule::R03ContextLimit | Rule::R08QualityFloor => {
            Some("capability")
        }
        Rule::R04PrivacyClass => Some("privacy"),
        Rule::R05Availability if excluded => Some("unavailable"),
        Rule::R05Availability => Some("stale"),
        Rule::R06CostCeiling | Rule::R07Deadline => Some("budget"),
        _ => None,
    }
}

fn reference(recipe: &DeclaredRecipe) -> Value {
    json!({
        "recipe_id": recipe.id,
        "recipe_version": recipe.version,
        "adapter_id": recipe.adapter,
        "actual_model_required": recipe.actual_model_required,
    })
}

/// Preview `spec` over `routing` and the caller's `snapshot` (pure): the `task.preview` result
/// body.
///
/// # Errors
///
/// `unavailable` when the baseline's record is not in the caller's snapshot or is disabled ("the
/// route baseline's roster record is not available to this caller"), or when the baseline cannot
/// serve by its roster facts ("the route baseline is not usable"); `internal` for a value route's
/// own contract excludes.
pub fn preview(routing: &Routing, snapshot: &Snapshot, spec: &Spec) -> Result<Value, Fault> {
    let parsed = &routing.parsed;
    let required = [spec.task_class];
    let task = Task {
        required_capabilities: &required,
        context_tokens: MAX_INPUT_TOKENS,
        privacy: PrivacyClass::LocalOnly,
        cost_ceiling_microunits: Some(0),
        deadline_ms: Some(spec.work_ms),
        quality_floor_basis_points: None,
    };
    let mut exclusions: Vec<(&str, &str)> = Vec::new();
    let mut joined = Vec::with_capacity(parsed.recipes().len());
    for declared in parsed.recipes() {
        match join(declared, snapshot, &required) {
            Some(recipe) => joined.push(recipe),
            None if declared.id == parsed.baseline() => {
                return Err(unavailable(
                    "the route baseline's roster record is not available to this caller",
                ));
            }
            None => exclusions.push((&declared.id, "unavailable")),
        }
    }
    let at = joined
        .iter()
        .position(|recipe| recipe.declared.id == parsed.baseline())
        .ok_or_else(internal)?;
    let baseline = joined.remove(at);
    let policy = parsed
        .policy(&baseline.recipe())
        .map_err(|error| match error {
            // The baseline's figures were checked at composition; its locality is the roster's.
            ConfigError::BaselineNotLocal { .. } => unavailable("the route baseline is not usable"),
            _ => internal(),
        })?;
    let candidates: Vec<Recipe<'_>> = joined.iter().map(Joined::recipe).collect();
    let decision =
        route::route(&policy, &task, &candidates, &baseline.recipe()).map_err(|_| internal())?;
    let mut eligible: Vec<&str> = Vec::new();
    for step in &decision.explanation().steps {
        match step {
            // Every filter passed: a guarded baseline, an eligible candidate, or a candidate that
            // lacks only a ranking figure (R11).
            Step::Guarded { recipe, .. }
            | Step::Eligible { recipe, .. }
            | Step::Gap {
                recipe,
                rule: Rule::R11Ranking,
                ..
            } => eligible.push(recipe),
            Step::Excluded { recipe, rule, .. } => {
                exclusions.push((recipe, code(*rule, true).ok_or_else(internal)?));
            }
            Step::Gap { recipe, rule, .. } => {
                exclusions.push((recipe, code(*rule, false).ok_or_else(internal)?));
            }
            Step::Ranked { .. } | Step::Decided { .. } => {}
        }
    }
    eligible.sort_unstable();
    exclusions.sort_unstable();
    let by_id = |id: &str| {
        parsed
            .recipes()
            .iter()
            .find(|recipe| recipe.id == id)
            .ok_or_else(internal)
    };
    let eligible = eligible
        .into_iter()
        .map(|id| by_id(id).map(reference))
        .collect::<Result<Vec<_>, _>>()?;
    let cost_mode = if eligible.is_empty() {
        "unknown"
    } else {
        "bounded"
    };
    Ok(json!({
        "eligible": eligible,
        "exclusions": exclusions
            .into_iter()
            .map(|(recipe_id, code)| json!({"recipe_id": recipe_id, "code": code}))
            .collect::<Vec<_>>(),
        "cost_mode": cost_mode,
        "observations_cutoff_unix_ms": snapshot.now.unix_ms.to_string(),
    }))
}
