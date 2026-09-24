//! T22 cohort cases (`T22-CO-nn`). Every case builds its world as values. Nothing here
//! schedules, sleeps or counts votes.
//!
//! The contract's required proof is *"disjoint work synergy; stale brief; overlapping
//! writes; missing child; contradictory evidence; dissent preservation; failed join repair
//! and final integrated verification"*, and each family below names which it covers.

use std::error::Error;

use habitat_engine::cohort::{
    Blocked, Claim, Cohort, Join, MAX_CLAIMS, MAX_DEPENDENCIES, MAX_EVIDENCE_BYTES, MAX_REBRIEFS,
    MAX_ROLE_BYTES, MAX_THREADS, Outcome, Refusal, Role, SCHEMA_VERSION,
};

type Outcome_ = Result<(), Box<dyn Error>>;

fn id(index: usize) -> String {
    format!("{index:08x}-0000-4000-8000-000000000000")
}

fn role() -> Result<Role, Refusal> {
    Role::new("specialist")
}

fn claim(path: &str) -> Result<Vec<Claim>, Box<dyn Error>> {
    Ok(vec![Claim::new(path)?])
}

/// A cohort at brief 1 with `count` required threads, each claiming its own path.
fn cohort_of(count: usize) -> Result<Cohort, Box<dyn Error>> {
    let mut cohort = Cohort::new(1);
    for index in 1..=count {
        cohort.assign(
            &id(index),
            role()?,
            &[],
            claim(&format!("src/m{index}"))?,
            true,
        )?;
    }
    Ok(cohort)
}

fn blocked_names(join: &Join) -> Vec<&'static str> {
    join.reasons().iter().map(Blocked::name).collect()
}

// ------------------------------------------------------- disjoint work and synergy

/// T22-CO-01 · disjoint valid work converges: every required thread meets its criterion on a
/// current brief and the parent may integrate.
#[test]
fn disjoint_valid_work_converges() -> Outcome_ {
    let mut cohort = cohort_of(3)?;
    for index in 1..=3 {
        cohort.report(&id(index), 0, Outcome::Met, "evidence")?;
    }
    let join = cohort.join();
    assert!(join.is_integrable());
    assert_eq!(
        join,
        Join::Integrable {
            threads: vec![id(1), id(2), id(3)]
        }
    );
    Ok(())
}

/// T22-CO-02 · an integrable join is an integration **candidate**, not an acceptance: it
/// names the threads whose work may be integrated and asserts nothing about the parent.
#[test]
fn an_integrable_join_names_its_threads_and_nothing_more() -> Outcome_ {
    let mut cohort = cohort_of(2)?;
    cohort.report(&id(1), 0, Outcome::Met, "a")?;
    cohort.report(&id(2), 0, Outcome::Met, "b")?;
    match cohort.join() {
        Join::Integrable { threads } => assert_eq!(threads, vec![id(1), id(2)]),
        other @ Join::Blocked(_) => {
            return Err(format!("expected integrable, got {other:?}").into());
        }
    }
    Ok(())
}

/// T22-CO-03 · an empty cohort joins integrably with no threads. Synergy is about useful
/// integration, not agent count, so zero agents integrating zero work is coherent.
#[test]
fn an_empty_cohort_joins_with_no_threads() {
    let cohort = Cohort::new(1);
    assert!(cohort.is_empty());
    assert_eq!(cohort.len(), 0);
    assert_eq!(cohort.join(), Join::Integrable { threads: vec![] });
}

/// T22-CO-04 · an optional thread that is unmet does not block; a required one does. The
/// asymmetry is what makes integration earned rather than tallied.
#[test]
fn an_optional_unmet_thread_does_not_block() -> Outcome_ {
    let mut cohort = Cohort::new(1);
    cohort.assign(&id(1), role()?, &[], claim("src/a")?, true)?;
    cohort.assign(&id(2), role()?, &[], claim("src/b")?, false)?;
    cohort.report(&id(1), 0, Outcome::Met, "ok")?;
    cohort.report(&id(2), 0, Outcome::Unmet, "did not reach it")?;
    let join = cohort.join();
    assert!(join.is_integrable());
    assert_eq!(
        join,
        Join::Integrable {
            threads: vec![id(1)]
        },
        "only the met thread is integrated"
    );
    Ok(())
}

/// T22-CO-05 · an optional thread that never reports does not block either.
#[test]
fn an_optional_silent_thread_does_not_block() -> Outcome_ {
    let mut cohort = Cohort::new(1);
    cohort.assign(&id(1), role()?, &[], claim("src/a")?, true)?;
    cohort.assign(&id(2), role()?, &[], claim("src/b")?, false)?;
    cohort.report(&id(1), 0, Outcome::Met, "ok")?;
    assert!(cohort.join().is_integrable());
    Ok(())
}

/// T22-CO-06 · only `Met` permits integration; three of the four outcomes do not.
#[test]
fn only_met_permits_integration() {
    assert_eq!(Outcome::ALL.len(), 4);
    let permitting: Vec<&str> = Outcome::ALL
        .into_iter()
        .filter(|outcome| outcome.permits_integration())
        .map(Outcome::name)
        .collect();
    assert_eq!(permitting, vec!["met"]);
    for outcome in Outcome::ALL {
        assert_eq!(outcome.to_string(), outcome.name());
    }
}

// ------------------------------------------------------- missing children

/// T22-CO-07 · a required thread that never reports blocks the join and is named.
#[test]
fn a_missing_required_child_blocks_and_is_named() -> Outcome_ {
    let mut cohort = cohort_of(3)?;
    cohort.report(&id(1), 0, Outcome::Met, "a")?;
    let join = cohort.join();
    assert!(!join.is_integrable());
    assert_eq!(blocked_names(&join), vec!["missing-child"]);
    assert_eq!(join.reasons()[0], Blocked::Missing(vec![id(2), id(3)]));
    Ok(())
}

/// T22-CO-08 · an unmet required thread blocks and is named, distinctly from a missing one.
#[test]
fn an_unmet_required_child_blocks_distinctly() -> Outcome_ {
    let mut cohort = cohort_of(2)?;
    cohort.report(&id(1), 0, Outcome::Met, "a")?;
    cohort.report(&id(2), 0, Outcome::Unmet, "b")?;
    let join = cohort.join();
    assert_eq!(blocked_names(&join), vec!["unmet"]);
    assert_eq!(join.reasons()[0], Blocked::Unmet(vec![id(2)]));
    Ok(())
}

/// T22-CO-09 · an indeterminate required thread blocks as unmet: not reaching a conclusion
/// is not the same as meeting the criterion.
#[test]
fn an_indeterminate_required_child_blocks() -> Outcome_ {
    let mut cohort = cohort_of(1)?;
    cohort.report(&id(1), 0, Outcome::Indeterminate, "could not tell")?;
    assert_eq!(blocked_names(&cohort.join()), vec!["unmet"]);
    Ok(())
}

/// T22-CO-10 · every blocking reason is reported, not just the first, so a caller repairing
/// one does not discover the next at a time.
#[test]
fn every_blocking_reason_is_reported() -> Outcome_ {
    let mut cohort = Cohort::new(1);
    for index in 1..=3 {
        cohort.assign(
            &id(index),
            role()?,
            &[],
            claim(&format!("src/m{index}"))?,
            true,
        )?;
    }
    cohort.report(&id(1), 0, Outcome::Unmet, "no")?;
    cohort.report(&id(2), 0, Outcome::Dissent, "I disagree")?;
    let join = cohort.join();
    assert_eq!(
        blocked_names(&join),
        vec!["missing-child", "unmet", "dissent"],
        "all three at once"
    );
    Ok(())
}

// ------------------------------------------------------- dissent preservation

/// T22-CO-11 · dissent blocks the join and is preserved with its reason. It is never
/// outvoted, even by an overwhelming majority.
#[test]
fn dissent_blocks_and_is_preserved_with_its_reason() -> Outcome_ {
    let mut cohort = cohort_of(5)?;
    for index in 1..=4 {
        cohort.report(&id(index), 0, Outcome::Met, "agree")?;
    }
    cohort.report(
        &id(5),
        0,
        Outcome::Dissent,
        "the fixture contradicts the brief",
    )?;
    let join = cohort.join();
    assert!(!join.is_integrable(), "four to one is not a decision");
    assert_eq!(
        join.reasons()[0],
        Blocked::Dissent(vec![(
            id(5),
            "the fixture contradicts the brief".to_owned()
        )])
    );
    Ok(())
}

/// T22-CO-12 · an **optional** thread's dissent blocks too: dissent is a claim about the
/// work, not about one thread's share of it.
#[test]
fn an_optional_threads_dissent_still_blocks() -> Outcome_ {
    let mut cohort = Cohort::new(1);
    cohort.assign(&id(1), role()?, &[], claim("src/a")?, true)?;
    cohort.assign(&id(2), role()?, &[], claim("src/b")?, false)?;
    cohort.report(&id(1), 0, Outcome::Met, "ok")?;
    cohort.report(&id(2), 0, Outcome::Dissent, "contradictory evidence")?;
    assert_eq!(blocked_names(&cohort.join()), vec!["dissent"]);
    Ok(())
}

/// T22-CO-13 · several dissents are all preserved, each with its own reason.
#[test]
fn several_dissents_are_all_preserved() -> Outcome_ {
    let mut cohort = cohort_of(3)?;
    cohort.report(&id(1), 0, Outcome::Met, "ok")?;
    cohort.report(&id(2), 0, Outcome::Dissent, "reason two")?;
    cohort.report(&id(3), 0, Outcome::Dissent, "reason three")?;
    let join = cohort.join();
    assert_eq!(
        join.reasons()[0],
        Blocked::Dissent(vec![
            (id(2), "reason two".to_owned()),
            (id(3), "reason three".to_owned())
        ])
    );
    Ok(())
}

/// T22-CO-14 · recorded evidence is readable for any reported thread and absent before.
#[test]
fn evidence_is_readable_and_absent_before_reporting() -> Outcome_ {
    let mut cohort = cohort_of(2)?;
    assert_eq!(cohort.evidence(&id(1))?, None);
    cohort.report(&id(1), 0, Outcome::Dissent, "the store disagrees")?;
    assert_eq!(cohort.evidence(&id(1))?, Some("the store disagrees"));
    Ok(())
}

// ------------------------------------------------------- overlapping writes

/// T22-CO-15 · two threads cannot claim the same path. Overlap is refused at assignment, not
/// detected at join when the damage is already done.
#[test]
fn an_identical_claim_is_refused_at_assignment() -> Outcome_ {
    let mut cohort = Cohort::new(1);
    cohort.assign(&id(1), role()?, &[], claim("src/store.rs")?, true)?;
    assert_eq!(
        cohort.assign(&id(2), role()?, &[], claim("src/store.rs")?, true),
        Err(Refusal::OverlappingClaim)
    );
    assert_eq!(cohort.len(), 1, "the refused thread was not assigned");
    Ok(())
}

/// T22-CO-16 · a claim nested under another is an overlap, in both directions: owning a
/// directory and owning a file inside it is the same conflict as owning the file twice.
#[test]
fn a_nested_claim_overlaps_in_both_directions() -> Outcome_ {
    let mut wide_first = Cohort::new(1);
    wide_first.assign(&id(1), role()?, &[], claim("src/store")?, true)?;
    assert_eq!(
        wide_first.assign(
            &id(2),
            role()?,
            &[],
            claim("src/store/reconciliation.rs")?,
            true
        ),
        Err(Refusal::OverlappingClaim)
    );
    let mut narrow_first = Cohort::new(1);
    narrow_first.assign(
        &id(1),
        role()?,
        &[],
        claim("src/store/reconciliation.rs")?,
        true,
    )?;
    assert_eq!(
        narrow_first.assign(&id(2), role()?, &[], claim("src/store")?, true),
        Err(Refusal::OverlappingClaim)
    );
    Ok(())
}

/// T22-CO-17 · a sibling path that merely shares a textual prefix is NOT an overlap.
/// `src/store` must not be read as a prefix of `src/storefront`; comparing whole segments is
/// what makes that true.
#[test]
fn a_shared_text_prefix_is_not_an_overlap() -> Outcome_ {
    let mut cohort = Cohort::new(1);
    cohort.assign(&id(1), role()?, &[], claim("src/store")?, true)?;
    cohort.assign(&id(2), role()?, &[], claim("src/storefront")?, true)?;
    assert_eq!(cohort.len(), 2);
    assert!(cohort.claims_are_disjoint());
    Ok(())
}

/// T22-CO-18 · the conflict predicate agrees with the assignment refusal, so the two cannot
/// drift apart.
#[test]
fn the_conflict_predicate_agrees_with_assignment() -> Outcome_ {
    let pairs = [
        ("src/a", "src/a", true),
        ("src/a", "src/a/b", true),
        ("src/a/b", "src/a", true),
        ("src/a", "src/ab", false),
        ("src/a", "src/b", false),
        ("a", "a/b/c", true),
    ];
    for (left, right, conflicts) in pairs {
        assert_eq!(
            Claim::new(left)?.conflicts_with(&Claim::new(right)?),
            conflicts,
            "{left} vs {right}"
        );
        let mut cohort = Cohort::new(1);
        cohort.assign(&id(1), role()?, &[], claim(left)?, true)?;
        let outcome = cohort.assign(&id(2), role()?, &[], claim(right)?, true);
        assert_eq!(
            outcome.is_err(),
            conflicts,
            "{left} vs {right} at assignment"
        );
    }
    Ok(())
}

/// T22-CO-19 · a thread may hold several disjoint claims, and any one of them conflicting is
/// enough to refuse.
#[test]
fn any_one_conflicting_claim_refuses_the_whole_assignment() -> Outcome_ {
    let mut cohort = Cohort::new(1);
    cohort.assign(&id(1), role()?, &[], claim("src/a")?, true)?;
    let mixed = vec![Claim::new("src/b")?, Claim::new("src/a/deep")?];
    assert_eq!(
        cohort.assign(&id(2), role()?, &[], mixed, true),
        Err(Refusal::OverlappingClaim)
    );
    assert_eq!(cohort.len(), 1);
    Ok(())
}

/// T22-CO-20 · an empty claim path is refused; a claim must name something.
#[test]
fn an_empty_claim_is_refused() {
    assert_eq!(Claim::new("").map(|_| ()), Err(Refusal::EmptyClaim));
    assert_eq!(
        Claim::new("src/a").map(|c| c.path().to_owned()),
        Ok("src/a".to_owned())
    );
}

/// T22-CO-21 · a thread with no claims is admissible — not all work writes.
#[test]
fn a_thread_may_hold_no_claims() -> Outcome_ {
    let mut cohort = Cohort::new(1);
    cohort.assign(&id(1), role()?, &[], vec![], true)?;
    cohort.assign(&id(2), role()?, &[], vec![], true)?;
    assert!(cohort.claims_are_disjoint());
    assert_eq!(cohort.len(), 2);
    Ok(())
}

/// T22-CO-22 · disjointness holds across a whole cohort, not just pairwise with the newest.
#[test]
fn disjointness_holds_across_the_whole_cohort() -> Outcome_ {
    let cohort = cohort_of(8)?;
    assert!(cohort.claims_are_disjoint());
    assert_eq!(cohort.len(), 8);
    Ok(())
}

// ------------------------------------------------------- stale briefs and repair

/// T22-CO-23 · revising the brief makes existing threads stale, and a stale thread blocks the
/// join even though it reported `Met`.
#[test]
fn revising_the_brief_makes_reported_work_stale() -> Outcome_ {
    let mut cohort = cohort_of(2)?;
    cohort.report(&id(1), 0, Outcome::Met, "a")?;
    cohort.report(&id(2), 0, Outcome::Met, "b")?;
    assert!(cohort.join().is_integrable());
    cohort.revise(2)?;
    let join = cohort.join();
    assert!(
        !join.is_integrable(),
        "work against an old brief is different work"
    );
    assert_eq!(join.reasons()[0], Blocked::Stale(vec![id(1), id(2)]));
    Ok(())
}

/// T22-CO-24 · reporting against a stale brief is refused, so a thread cannot answer a
/// question that has changed.
#[test]
fn reporting_against_a_stale_brief_is_refused() -> Outcome_ {
    let mut cohort = cohort_of(1)?;
    cohort.revise(2)?;
    assert_eq!(
        cohort.report(&id(1), 0, Outcome::Met, "against the old brief"),
        Err(Refusal::StaleBrief)
    );
    Ok(())
}

/// T22-CO-25 · rebriefing repairs a stale thread by clearing its conclusion; the failed join
/// then succeeds once the thread re-reports. This is the repair path end to end.
#[test]
fn rebriefing_repairs_a_stale_thread() -> Outcome_ {
    let mut cohort = cohort_of(2)?;
    cohort.report(&id(1), 0, Outcome::Met, "a")?;
    cohort.report(&id(2), 0, Outcome::Met, "b")?;
    cohort.revise(2)?;
    assert!(!cohort.join().is_integrable());
    for index in 1..=2 {
        cohort.rebrief(&id(index))?;
        assert_eq!(
            cohort.thread(&id(index))?.outcome,
            None,
            "the old conclusion is discarded, not carried forward"
        );
        cohort.report(&id(index), 1, Outcome::Met, "redone")?;
    }
    assert!(cohort.join().is_integrable());
    Ok(())
}

/// T22-CO-26 · a thread assigned after a revision is current, while one assigned before is
/// stale — the brief travels with the assignment.
#[test]
fn the_brief_travels_with_the_assignment() -> Outcome_ {
    let mut cohort = Cohort::new(1);
    cohort.assign(&id(1), role()?, &[], claim("src/a")?, true)?;
    cohort.revise(5)?;
    cohort.assign(&id(2), role()?, &[], claim("src/b")?, true)?;
    assert_eq!(cohort.thread(&id(1))?.brief, 1);
    assert_eq!(cohort.thread(&id(2))?.brief, 5);
    assert_eq!(cohort.brief(), 5);
    Ok(())
}

/// T22-CO-27 · a stale thread is reported as stale rather than missing, even if it never
/// reported: the more specific fact wins.
#[test]
fn a_stale_thread_is_stale_rather_than_missing() -> Outcome_ {
    let mut cohort = cohort_of(1)?;
    cohort.revise(2)?;
    let join = cohort.join();
    assert_eq!(blocked_names(&join), vec!["stale-brief"]);
    Ok(())
}

/// T22-CO-28 · an outcome is not revised in place; a second report is refused so a silently
/// rewritten conclusion cannot be mistaken for the first.
#[test]
fn a_second_report_is_refused() -> Outcome_ {
    let mut cohort = cohort_of(1)?;
    cohort.report(&id(1), 0, Outcome::Met, "first")?;
    assert_eq!(
        cohort.report(&id(1), 0, Outcome::Unmet, "second"),
        Err(Refusal::AlreadyReported)
    );
    assert_eq!(cohort.evidence(&id(1))?, Some("first"));
    Ok(())
}

// ------------------------------------------------------- the child DAG

/// T22-CO-29 · a thread may depend on one already assigned, and the dependency is reported
/// back in declaration order.
#[test]
fn dependencies_are_recorded_in_declaration_order() -> Outcome_ {
    let mut cohort = Cohort::new(1);
    cohort.assign(&id(1), role()?, &[], claim("src/a")?, true)?;
    cohort.assign(&id(2), role()?, &[], claim("src/b")?, true)?;
    cohort.assign(&id(3), role()?, &[&id(2), &id(1)], claim("src/c")?, true)?;
    assert_eq!(cohort.thread(&id(3))?.dependencies, vec![id(2), id(1)]);
    Ok(())
}

/// T22-CO-30 · a thread cannot depend on itself.
#[test]
fn a_thread_cannot_depend_on_itself() -> Outcome_ {
    let mut cohort = Cohort::new(1);
    assert_eq!(
        cohort.assign(&id(1), role()?, &[&id(1)], claim("src/a")?, true),
        Err(Refusal::SelfDependency)
    );
    assert!(cohort.is_empty());
    Ok(())
}

/// T22-CO-31 · a dependency on an unassigned thread is refused, which is what makes the
/// graph finite: a thread can only name work that already exists.
#[test]
fn a_dependency_on_an_unassigned_thread_is_refused() -> Outcome_ {
    let mut cohort = Cohort::new(1);
    assert_eq!(
        cohort.assign(&id(1), role()?, &[&id(9)], claim("src/a")?, true),
        Err(Refusal::UnknownDependency)
    );
    Ok(())
}

/// T22-CO-32 · a chain of dependencies is admissible and acyclic; the cycle check does not
/// false-positive on a long chain.
#[test]
fn a_dependency_chain_is_admissible() -> Outcome_ {
    let mut cohort = Cohort::new(1);
    cohort.assign(&id(1), role()?, &[], claim("src/m1")?, true)?;
    for index in 2..=6 {
        let previous = id(index - 1);
        cohort.assign(
            &id(index),
            role()?,
            &[&previous],
            claim(&format!("src/m{index}"))?,
            true,
        )?;
    }
    assert_eq!(cohort.len(), 6);
    Ok(())
}

/// T22-CO-33 · a diamond is admissible: two threads may share a dependency.
#[test]
fn a_diamond_of_dependencies_is_admissible() -> Outcome_ {
    let mut cohort = Cohort::new(1);
    cohort.assign(&id(1), role()?, &[], claim("src/m1")?, true)?;
    cohort.assign(&id(2), role()?, &[&id(1)], claim("src/m2")?, true)?;
    cohort.assign(&id(3), role()?, &[&id(1)], claim("src/m3")?, true)?;
    cohort.assign(&id(4), role()?, &[&id(2), &id(3)], claim("src/m4")?, true)?;
    assert_eq!(cohort.threads()?.len(), 4);
    Ok(())
}

/// T22-CO-34 · a duplicate thread identity is refused.
#[test]
fn a_duplicate_thread_identity_is_refused() -> Outcome_ {
    let mut cohort = Cohort::new(1);
    cohort.assign(&id(1), role()?, &[], claim("src/a")?, true)?;
    assert_eq!(
        cohort.assign(&id(1), role()?, &[], claim("src/b")?, true),
        Err(Refusal::DuplicateThread)
    );
    Ok(())
}

/// T22-CO-35 · a malformed thread or dependency identity is refused.
#[test]
fn malformed_identities_are_refused() -> Outcome_ {
    use habitat_engine::contracts::ScalarError;
    let mut cohort = Cohort::new(1);
    assert_eq!(
        cohort.assign("nope", role()?, &[], vec![], true),
        Err(Refusal::MalformedIdentity(ScalarError::InvalidUuid))
    );
    cohort.assign(&id(1), role()?, &[], vec![], true)?;
    assert_eq!(
        cohort.assign(&id(2), role()?, &["nope"], vec![], true),
        Err(Refusal::MalformedIdentity(ScalarError::InvalidUuid))
    );
    Ok(())
}

/// T22-CO-36 · an unknown thread is refused by every operation that names one.
#[test]
fn an_unknown_thread_is_refused_everywhere() {
    let mut cohort = Cohort::new(1);
    assert_eq!(
        cohort.report(&id(9), 0, Outcome::Met, "x"),
        Err(Refusal::UnknownThread)
    );
    assert_eq!(cohort.rebrief(&id(9)), Err(Refusal::UnknownThread));
    assert_eq!(
        cohort.thread(&id(9)).map(|_| ()),
        Err(Refusal::UnknownThread)
    );
    assert_eq!(cohort.evidence(&id(9)), Err(Refusal::UnknownThread));
}

// ------------------------------------------------------- bounds

/// T22-CO-37 · the thread bound refuses at its limit, before the thread is built.
#[test]
fn the_thread_bound_refuses_before_building() -> Outcome_ {
    let mut cohort = Cohort::new(1);
    for index in 1..=MAX_THREADS {
        cohort.assign(&id(index), role()?, &[], vec![], true)?;
    }
    assert_eq!(cohort.len(), MAX_THREADS);
    assert_eq!(
        cohort.assign(&id(MAX_THREADS + 1), role()?, &[], vec![], true),
        Err(Refusal::ThreadLimit)
    );
    assert_eq!(cohort.len(), MAX_THREADS);
    Ok(())
}

/// T22-CO-38 · the claim bound refuses at its limit, and exactly the bound is admitted.
#[test]
fn the_claim_bound_admits_exactly_its_limit() -> Outcome_ {
    let too_many: Vec<Claim> = (0..=MAX_CLAIMS)
        .map(|i| Claim::new(&format!("src/c{i}")))
        .collect::<Result<_, _>>()?;
    let exact: Vec<Claim> = (0..MAX_CLAIMS)
        .map(|i| Claim::new(&format!("src/d{i}")))
        .collect::<Result<_, _>>()?;
    let mut cohort = Cohort::new(1);
    assert_eq!(
        cohort.assign(&id(1), role()?, &[], too_many, true),
        Err(Refusal::ClaimLimit)
    );
    cohort.assign(&id(1), role()?, &[], exact, true)?;
    Ok(())
}

/// T22-CO-39 · the dependency bound is an **acquisition** bound on the caller's array: it
/// refuses before any of the entries is parsed or resolved, so a caller cannot make the
/// cohort walk a list it has already declared too long.
///
/// The fixture deliberately uses a small cohort. Filling the cohort to `MAX_THREADS` first
/// would trip `ThreadLimit` instead — which is what the first version of this case did, and
/// the refusal it got was the correct one for the world it had built.
#[test]
fn the_dependency_bound_refuses_before_resolving_any_entry() -> Outcome_ {
    let mut cohort = Cohort::new(1);
    cohort.assign(&id(1), role()?, &[], vec![], true)?;
    // None of these resolve to a thread; the bound must fire before that is discovered.
    let owned: Vec<String> = (900..=900 + MAX_DEPENDENCIES).map(id).collect();
    let refs: Vec<&str> = owned.iter().map(String::as_str).collect();
    assert_eq!(refs.len(), MAX_DEPENDENCIES + 1);
    assert_eq!(
        cohort.assign(&id(2), role()?, &refs, vec![], true),
        Err(Refusal::DependencyLimit)
    );
    assert_eq!(
        cohort.assign(&id(2), role()?, &refs[..MAX_DEPENDENCIES], vec![], true),
        Err(Refusal::UnknownDependency),
        "exactly the bound is admitted, and then the entries are resolved"
    );
    assert_eq!(cohort.len(), 1);
    Ok(())
}

/// T22-CO-40 · the declared bounds are the values enforced.
#[test]
fn declared_bounds_are_the_enforced_bounds() {
    assert_eq!(MAX_THREADS, 64);
    assert_eq!(MAX_CLAIMS, 64);
    assert_eq!(MAX_DEPENDENCIES, 64);
    assert_eq!(SCHEMA_VERSION, 1);
}

// ------------------------------------------------------- reads and diagnostics

/// T22-CO-41 · the thread list returns every thread in assignment order with its state.
#[test]
fn the_thread_list_returns_every_thread_in_order() -> Outcome_ {
    let mut cohort = cohort_of(4)?;
    cohort.report(&id(2), 0, Outcome::Met, "b")?;
    let threads = cohort.threads()?;
    assert_eq!(threads.len(), 4);
    let identities: Vec<&str> = threads.iter().map(|t| t.identity.as_str()).collect();
    assert_eq!(identities, (1..=4).map(id).collect::<Vec<_>>());
    assert_eq!(threads[1].outcome, Some(Outcome::Met));
    assert_eq!(threads[0].outcome, None);
    Ok(())
}

/// T22-CO-42 · an assignment reports its claims, so a caller can see what a thread owns
/// without guessing from the path it wrote.
#[test]
fn an_assignment_reports_its_claims() -> Outcome_ {
    let mut cohort = Cohort::new(1);
    cohort.assign(
        &id(1),
        role()?,
        &[],
        vec![Claim::new("src/a")?, Claim::new("docs/a.md")?],
        true,
    )?;
    let thread = cohort.thread(&id(1))?;
    let paths: Vec<&str> = thread.claims.iter().map(Claim::path).collect();
    assert_eq!(paths, vec!["src/a", "docs/a.md"]);
    assert!(thread.required);
    Ok(())
}

/// T22-CO-43 · every refusal has a distinct name and none is a substring of another.
#[test]
fn refusal_names_are_distinct_and_non_overlapping() {
    use habitat_engine::contracts::ScalarError;
    let all = [
        Refusal::MalformedIdentity(ScalarError::InvalidUuid),
        Refusal::ThreadLimit,
        Refusal::ClaimLimit,
        Refusal::DependencyLimit,
        Refusal::DuplicateThread,
        Refusal::UnknownThread,
        Refusal::UnknownDependency,
        Refusal::SelfDependency,
        Refusal::DependencyCycle,
        Refusal::OverlappingClaim,
        Refusal::StaleBrief,
        Refusal::AlreadyReported,
        Refusal::ChildOutstanding,
        Refusal::EmptyClaim,
        Refusal::NonCanonicalClaim,
        Refusal::BriefRegressed,
        Refusal::EvidenceLimit,
        Refusal::EmptyRole,
        Refusal::RoleLimit,
        Refusal::RebriefLimit,
        Refusal::DissentOutstanding,
        Refusal::StaleGeneration,
    ];
    for (i, a) in all.iter().enumerate() {
        assert!(!a.name().is_empty());
        for (j, b) in all.iter().enumerate() {
            if i != j {
                assert!(
                    !a.name().contains(b.name()),
                    "{} contains {}",
                    a.name(),
                    b.name()
                );
            }
        }
    }
}

/// T22-CO-44 · a refusal carrying a scalar error shows both parts.
#[test]
fn refusal_display_shows_the_carried_error() {
    use habitat_engine::contracts::ScalarError;
    assert_eq!(
        Refusal::MalformedIdentity(ScalarError::InvalidUuid).to_string(),
        "malformed cohort identity: expected a lowercase hyphenated UUIDv4"
    );
    assert_eq!(
        Refusal::OverlappingClaim.to_string(),
        "resource claim overlaps a live claim"
    );
}

/// T22-CO-45 · blocked reasons name themselves stably.
#[test]
fn blocked_reasons_name_themselves() {
    assert_eq!(Blocked::Missing(vec![]).name(), "missing-child");
    assert_eq!(Blocked::Unmet(vec![]).name(), "unmet");
    assert_eq!(Blocked::Dissent(vec![]).name(), "dissent");
    assert_eq!(Blocked::Stale(vec![]).name(), "stale-brief");
}

/// T22-CO-46 · an integrable join has no reasons, and a blocked one is never integrable.
#[test]
fn integrable_and_blocked_are_exclusive() -> Outcome_ {
    let mut cohort = cohort_of(1)?;
    cohort.report(&id(1), 0, Outcome::Met, "ok")?;
    let good = cohort.join();
    assert!(good.is_integrable() && good.reasons().is_empty());
    let mut bad = cohort_of(1)?;
    bad.report(&id(1), 0, Outcome::Unmet, "no")?;
    let blocked = bad.join();
    assert!(!blocked.is_integrable() && !blocked.reasons().is_empty());
    Ok(())
}

/// T22-CO-47 · the integrated thread list excludes optional threads that did not meet, so a
/// caller integrating from it cannot pick up work that was not endorsed.
#[test]
fn the_integrated_list_excludes_unmet_optional_work() -> Outcome_ {
    let mut cohort = Cohort::new(1);
    cohort.assign(&id(1), role()?, &[], claim("src/a")?, true)?;
    cohort.assign(&id(2), role()?, &[], claim("src/b")?, false)?;
    cohort.assign(&id(3), role()?, &[], claim("src/c")?, false)?;
    cohort.report(&id(1), 0, Outcome::Met, "a")?;
    cohort.report(&id(2), 0, Outcome::Met, "b")?;
    cohort.report(&id(3), 0, Outcome::Unmet, "c")?;
    assert_eq!(
        cohort.join(),
        Join::Integrable {
            threads: vec![id(1), id(2)]
        }
    );
    Ok(())
}

/// T22-CO-48 · the full scenario: split independent work, return compatible results, detect
/// an integration failure, repair it and re-verify. This is the contract's integrated proof.
#[test]
fn the_integrated_scenario_splits_fails_repairs_and_reverifies() -> Outcome_ {
    let mut cohort = Cohort::new(1);
    cohort.assign(&id(1), role()?, &[], claim("src/store")?, true)?;
    cohort.assign(&id(2), role()?, &[], claim("src/route")?, true)?;
    cohort.assign(
        &id(3),
        role()?,
        &[&id(1), &id(2)],
        claim("docs/contract.md")?,
        true,
    )?;
    assert!(cohort.claims_are_disjoint());

    cohort.report(&id(1), 0, Outcome::Met, "store done")?;
    cohort.report(&id(2), 0, Outcome::Met, "route done")?;
    cohort.report(&id(3), 0, Outcome::Dissent, "the two contracts disagree")?;
    let failed = cohort.join();
    assert_eq!(blocked_names(&failed), vec!["dissent"]);

    // Repair: the brief moves to resolve the disagreement, every thread redoes its work.
    cohort.revise(2)?;
    for index in 1..=3 {
        cohort.rebrief(&id(index))?;
        cohort.report(&id(index), 1, Outcome::Met, "reconciled")?;
    }
    let repaired = cohort.join();
    assert!(repaired.is_integrable());
    assert_eq!(
        repaired,
        Join::Integrable {
            threads: vec![id(1), id(2), id(3)]
        }
    );
    assert!(cohort.claims_are_disjoint());
    Ok(())
}

/// T22-CO-49 · rebriefing a current thread clears its outcome too, so a caller repairing a
/// wrong conclusion does not have to revise the brief to do it.
#[test]
fn rebriefing_a_current_thread_clears_its_outcome() -> Outcome_ {
    let mut cohort = cohort_of(1)?;
    cohort.report(&id(1), 0, Outcome::Unmet, "wrong")?;
    cohort.rebrief(&id(1))?;
    assert_eq!(cohort.thread(&id(1))?.outcome, None);
    assert_eq!(cohort.evidence(&id(1))?, None);
    cohort.report(&id(1), 1, Outcome::Met, "right")?;
    assert!(cohort.join().is_integrable());
    Ok(())
}

/// T22-CO-50 · a cohort where every thread dissents is still not a decision: unanimity
/// against is reported as dissent, not converted into a verdict.
#[test]
fn unanimous_dissent_is_still_dissent() -> Outcome_ {
    let mut cohort = cohort_of(3)?;
    for index in 1..=3 {
        cohort.report(&id(index), 0, Outcome::Dissent, "no")?;
    }
    let join = cohort.join();
    assert_eq!(blocked_names(&join), vec!["dissent"]);
    assert_eq!(join.reasons()[0].name(), "dissent");
    Ok(())
}

/// T22-CO-51 · claims remain disjoint through reporting and rebriefing: state transitions do
/// not release ownership.
#[test]
fn ownership_survives_reporting_and_rebriefing() -> Outcome_ {
    let mut cohort = cohort_of(4)?;
    for index in 1..=4 {
        cohort.report(&id(index), 0, Outcome::Met, "x")?;
        assert!(cohort.claims_are_disjoint(), "after report {index}");
    }
    cohort.revise(2)?;
    for index in 1..=4 {
        cohort.rebrief(&id(index))?;
        assert!(cohort.claims_are_disjoint(), "after rebrief {index}");
    }
    assert_eq!(
        cohort.assign(&id(99), role()?, &[], claim("src/m1")?, true),
        Err(Refusal::OverlappingClaim),
        "a rebriefed thread still owns its path"
    );
    Ok(())
}

/// T22-CO-52 · a stale thread and a dissenting thread block together, each named.
#[test]
fn stale_and_dissent_block_together() -> Outcome_ {
    let mut cohort = Cohort::new(1);
    cohort.assign(&id(1), role()?, &[], claim("src/a")?, true)?;
    cohort.report(&id(1), 0, Outcome::Dissent, "disagree")?;
    cohort.revise(2)?;
    cohort.assign(&id(2), role()?, &[], claim("src/b")?, true)?;
    cohort.report(&id(2), 0, Outcome::Met, "ok")?;
    let join = cohort.join();
    assert_eq!(blocked_names(&join), vec!["stale-brief"]);
    Ok(())
}

/// T22-CO-53 · `Cohort::is_empty` and `len` are asserted in both directions and away from
/// the identity element: a predicate only ever asked to say yes is pinned by nothing.
#[test]
fn cohort_emptiness_and_count_are_pinned_in_both_directions() -> Outcome_ {
    let mut cohort = Cohort::new(1);
    assert!(cohort.is_empty());
    assert_eq!(cohort.len(), 0);
    for index in 1..=4 {
        cohort.assign(
            &id(index),
            role()?,
            &[],
            claim(&format!("src/m{index}"))?,
            true,
        )?;
        assert!(!cohort.is_empty(), "after assigning {index}");
        assert_eq!(cohort.len(), index);
    }
    Ok(())
}

/// T22-CO-54 · `claims_are_disjoint` is asserted true on a cohort built through the API.
///
/// It cannot be made to return **false** through any public path: `assign` refuses an
/// overlapping claim, so a cohort holding one is unreachable. The predicate exists for a
/// cohort read back from storage, where the invariant is a claim about bytes rather than a
/// consequence of the transitions — which is exactly why it must not be deleted, and why its
/// `-> true` mutant is recorded as equivalent-with-reason rather than killed by a test that
/// merely looks like it works.
#[test]
fn disjointness_holds_and_cannot_be_violated_through_the_api() -> Outcome_ {
    let cohort = cohort_of(6)?;
    assert!(cohort.claims_are_disjoint());
    let mut attempted = cohort_of(2)?;
    assert_eq!(
        attempted.assign(&id(99), role()?, &[], claim("src/m1")?, true),
        Err(Refusal::OverlappingClaim),
        "the only route to a non-disjoint cohort is refused"
    );
    assert!(attempted.claims_are_disjoint());
    Ok(())
}

/// T22-CO-55 · the dependency bound's comparison is pinned at its exact boundary in both
/// directions, so `>` cannot be weakened to `>=` or inverted without a case failing.
#[test]
fn the_dependency_bound_comparison_is_pinned_at_the_boundary() -> Outcome_ {
    let owned: Vec<String> = (900..=900 + MAX_DEPENDENCIES).map(id).collect();
    let refs: Vec<&str> = owned.iter().map(String::as_str).collect();
    for (count, refused_by_bound) in [
        (MAX_DEPENDENCIES - 1, false),
        (MAX_DEPENDENCIES, false),
        (MAX_DEPENDENCIES + 1, true),
    ] {
        let mut cohort = Cohort::new(1);
        let outcome = cohort.assign(&id(1), role()?, &refs[..count], vec![], true);
        if refused_by_bound {
            assert_eq!(
                outcome,
                Err(Refusal::DependencyLimit),
                "{count} dependencies"
            );
        } else {
            assert_eq!(
                outcome,
                Err(Refusal::UnknownDependency),
                "{count} dependencies must pass the bound and fail on resolution"
            );
        }
    }
    Ok(())
}

/// T22-CO-56 · the cycle guard is unreachable through the public API, recorded with its
/// reason rather than pretended away.
///
/// `assign` refuses a dependency that names an unassigned thread, so a thread can only
/// depend on work that already exists, and a graph built that way is acyclic by
/// construction. `closes_cycle` therefore always returns `false` today and its mutants are
/// **equivalent**. It is kept because the property it defends is a consequence of one line
/// in `assign` — if that line ever admits a forward reference, this guard is what stops the
/// walk from running forever.
#[test]
fn the_cycle_guard_is_unreachable_and_that_is_the_point() -> Outcome_ {
    let mut cohort = Cohort::new(1);
    cohort.assign(&id(1), role()?, &[], vec![], true)?;
    cohort.assign(&id(2), role()?, &[&id(1)], vec![], true)?;
    cohort.assign(&id(3), role()?, &[&id(2), &id(1)], vec![], true)?;
    // The only way to name a thread that could close a cycle is to name one that is not yet
    // assigned, and that is refused first.
    assert_eq!(
        cohort.assign(&id(4), role()?, &[&id(5)], vec![], true),
        Err(Refusal::UnknownDependency)
    );
    assert_eq!(
        cohort.assign(&id(4), role()?, &[&id(4)], vec![], true),
        Err(Refusal::SelfDependency)
    );
    assert_eq!(cohort.len(), 3);
    Ok(())
}

/// `T22-CO-overlap-table` — the Rust half of the one-rule-two-implementations check.
///
/// `Claim::conflicts_with` and `julia/src/Cohesion.jl::claims_conflict` state the same rule
/// in two languages. Two doors keeping one rule is a promise; this makes it a mechanism —
/// `evaluation/cohorts/claim-overlap-v1.json` is generated from a third statement of the rule
/// and read by a test on each side, so a change to either implementation alone goes red here.
#[test]
fn claim_overlap_agrees_with_the_shared_table() -> Result<(), Box<dyn Error>> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("evaluation/cohorts/claim-overlap-v1.json");
    let table: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&path)?)?;
    assert_eq!(
        table["schema"].as_str(),
        Some("hee3.evaluation.claim-overlap.v1"),
        "the table this test reads is not the one it was written against"
    );
    let cases = table["cases"]
        .as_array()
        .ok_or("claim-overlap table has no case list")?;
    // The denominator comes from the table, not from a literal beside it: a case removed
    // from the file must not quietly shrink what this test covers.
    assert!(
        cases.len() >= 20,
        "the shared table has shrunk to {} cases",
        cases.len()
    );
    let mut answers = std::collections::BTreeSet::new();
    for case in cases {
        let name = case["name"].as_str().ok_or("case without a name")?;
        let a = case["a"].as_str().ok_or("case without a left path")?;
        let b = case["b"].as_str().ok_or("case without a right path")?;
        let want = case["overlap"].as_bool().ok_or("case without an answer")?;
        answers.insert(want);
        let (left, right) = (Claim::new(a)?, Claim::new(b)?);
        assert_eq!(
            left.conflicts_with(&right),
            want,
            "{name}: conflicts_with({a:?}, {b:?})"
        );
        // The rule is symmetric; the implementation orders by length internally, so a table
        // read in one direction only would leave that ordering unpinned.
        assert_eq!(
            right.conflicts_with(&left),
            want,
            "{name} reversed: conflicts_with({b:?}, {a:?})"
        );
    }
    assert_eq!(
        answers,
        [false, true].into_iter().collect(),
        "a table that answers one way discriminates nothing"
    );
    // Review N3: the paths the table says no implementation may admit as a claim.
    let refused = table["refused"]
        .as_array()
        .ok_or("claim-overlap table has no refused list")?;
    assert!(
        refused.iter().any(|path| path == "src/store/"),
        "the refused list lost review N3's own example"
    );
    for path in refused {
        let path = path.as_str().ok_or("refused entry is not a string")?;
        assert!(
            Claim::new(path).is_err(),
            "{path:?} was admitted as a claim"
        );
    }
    Ok(())
}

/// Replay one cohesion fixture's rows through [`Cohort`] and return what `join` decides.
///
/// Claims are replaced by one private path per row: they bear on assignment, not on the join,
/// and C02 deliberately carries an overlap `assign` refuses. Rows are assigned in ascending
/// brief order with the cohort revised to each brief first, so a row's revision is the one it
/// was assigned against; each reports on its own brief, and the cohort is then revised to the
/// fixture's current brief — which is what makes the earlier rows stale.
fn replay(fixture: &serde_json::Value) -> Result<Join, Box<dyn Error>> {
    let decimal = |value: &serde_json::Value| -> Result<u64, Box<dyn Error>> {
        Ok(value
            .as_str()
            .ok_or("revision is not a decimal string")?
            .parse()?)
    };
    let current = decimal(&fixture["brief_revision"])?;
    let rows = fixture["threads"]
        .as_array()
        .ok_or("fixture without threads")?;
    let mut ordered = Vec::with_capacity(rows.len());
    for (index, row) in rows.iter().enumerate() {
        ordered.push((decimal(&row["brief_revision"])?, index, row));
    }
    ordered.sort_by_key(|&(brief, index, _)| (brief, index));
    let mut cohort = Cohort::new(0);
    for (brief, index, row) in ordered {
        cohort.revise(brief)?;
        let identity = row["thread_id"].as_str().ok_or("row without identity")?;
        let required = row["required"].as_bool().ok_or("row without required")?;
        cohort.assign(
            identity,
            role()?,
            &[],
            claim(&format!("fixture/{index}"))?,
            required,
        )?;
        let name = row["outcome"].as_str().ok_or("row without outcome")?;
        let outcome = Outcome::ALL
            .into_iter()
            .find(|outcome| outcome.name() == name)
            .ok_or("row with an unknown outcome")?;
        cohort.report(identity, 0, outcome, "fixture")?;
    }
    cohort.revise(current)?;
    Ok(cohort.join())
}

/// The join a cohesion request declares is `cohort::join`'s verdict over the same rows.
///
/// Julia's `cohesion` derives the join and refuses a declaration that disagrees, so the
/// fixtures' declarations are the expected answers on that side; this test is where they come
/// from. Before it existed both fixtures carried hand-typed joins that no implementation
/// computed (review N10): C02 declared `integrable` over a required dissent and a stale row.
#[test]
fn fixture_joins_agree_with_cohort_join() -> Result<(), Box<dyn Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/t22");
    let mut verdicts = std::collections::BTreeSet::new();
    for name in ["C01", "C02"] {
        let fixture: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(root.join(format!("{name}.json")))?)?;
        let join = replay(&fixture)?;
        let declared = &fixture["join"];
        let verdict = if join.is_integrable() {
            "integrable"
        } else {
            "blocked"
        };
        assert_eq!(
            declared["verdict"].as_str(),
            Some(verdict),
            "{name}: verdict"
        );
        let reasons: Vec<&str> = declared["reasons"]
            .as_array()
            .ok_or("declared join without reasons")?
            .iter()
            .map(|reason| reason.as_str().unwrap_or("<not a string>"))
            .collect();
        assert_eq!(
            reasons,
            blocked_names(&join),
            "{name}: reasons, in the rule's order"
        );
        verdicts.insert(blocked_names(&join).join(","));
    }
    assert_eq!(
        verdicts.len(),
        2,
        "two fixtures with one join pin that join only where they agree"
    );
    Ok(())
}

/// Review N3: a claim with an empty segment slipped past the overlap rule — `src/store/` did
/// not conflict with `src/store/x.rs`, because every statement of the rule (Rust, Julia and
/// the shared table's generator) split it into `[src, store, ""]`. A claim is now admitted only
/// in canonical form, so a writer cannot spell its way around a sibling's claim.
#[test]
fn a_non_canonical_claim_is_refused_at_construction() {
    for path in [
        "src/store/",
        "/src/store",
        "src//store",
        "./src",
        "src/../x",
        "src/./x",
    ] {
        assert!(Claim::new(path).is_err(), "{path:?} was admitted");
    }
    for path in ["src/store", "src/store/x.rs", "a", "docs/a.md"] {
        assert!(Claim::new(path).is_ok(), "{path:?} was refused");
    }
}

/// Review D5: a brief revision never moves backwards. `revise` used to assign any value, so a
/// regression made threads assigned against the newer brief read as stale and threads from an
/// OLDER brief read as current again — stale work re-admitted to the join.
#[test]
fn a_brief_revision_cannot_move_backwards() -> Result<(), Box<dyn Error>> {
    let mut cohort = Cohort::new(3);
    cohort.assign(&id(1), role()?, &[], claim("src/a")?, true)?;
    cohort.report(&id(1), 0, Outcome::Met, "on brief 3")?;
    assert_eq!(cohort.revise(2), Err(Refusal::BriefRegressed));
    assert_eq!(
        cohort.revise(3),
        Ok(()),
        "revising to the current value changes nothing"
    );
    assert_eq!(cohort.brief(), 3);
    assert!(
        cohort.join().is_integrable(),
        "the brief-3 work is still current"
    );
    Ok(())
}

// ------------------------------------------------------- wave 1 gaps COH-01..COH-05

/// T22-CO-57 · COH-01: a current dissent cannot be erased by rebriefing. Before this refusal
/// `report(Dissent) -> rebrief -> report(Met)` joined integrably with no trace of the dissent —
/// a false parent join built from the repair path.
#[test]
fn a_current_dissent_is_not_erased_by_rebrief() -> Outcome_ {
    let mut cohort = cohort_of(2)?;
    cohort.report(&id(1), 0, Outcome::Met, "ok")?;
    cohort.report(
        &id(2),
        0,
        Outcome::Dissent,
        "the brief contradicts the fixture",
    )?;
    assert_eq!(cohort.rebrief(&id(2)), Err(Refusal::DissentOutstanding));
    // Nothing moved: the dissent, its reason and the generation are all where they were.
    let thread = cohort.thread(&id(2))?;
    assert_eq!(thread.outcome, Some(Outcome::Dissent));
    assert_eq!(thread.generation, 0);
    assert_eq!(
        cohort.evidence(&id(2))?,
        Some("the brief contradicts the fixture")
    );
    assert_eq!(
        cohort.report(&id(2), 1, Outcome::Met, "overwrite"),
        Err(Refusal::StaleGeneration),
        "no later generation exists to report a Met under"
    );
    assert_eq!(
        cohort.join(),
        Join::Blocked(vec![Blocked::Dissent(vec![(
            id(2),
            "the brief contradicts the fixture".to_owned()
        )])])
    );
    Ok(())
}

/// T22-CO-58 · COH-01's repair path stays open: revising the brief makes the dissent stale,
/// a stale dissent may be rebriefed, and the join is blocked as stale until it is redone. A
/// current Unmet or Indeterminate thread is still rebriefable in place (T22-CO-49).
#[test]
fn a_dissent_is_repaired_by_revising_the_brief() -> Outcome_ {
    let mut cohort = cohort_of(1)?;
    cohort.report(&id(1), 0, Outcome::Dissent, "disagree")?;
    cohort.revise(2)?;
    assert_eq!(blocked_names(&cohort.join()), vec!["stale-brief"]);
    cohort.rebrief(&id(1))?;
    cohort.report(&id(1), 1, Outcome::Met, "on the revised brief")?;
    assert!(cohort.join().is_integrable());

    let mut current = cohort_of(2)?;
    current.report(&id(1), 0, Outcome::Indeterminate, "no conclusion")?;
    current.report(&id(2), 0, Outcome::Unmet, "not reached")?;
    current.rebrief(&id(1))?;
    current.rebrief(&id(2))?;
    assert_eq!(current.thread(&id(1))?.generation, 1);
    assert_eq!(current.thread(&id(2))?.generation, 1);
    Ok(())
}

/// T22-CO-59 · COH-02: evidence is bounded at acquisition — exactly `MAX_EVIDENCE_BYTES` is
/// recorded, one byte more is refused before anything is parsed, and a refused report leaves
/// the thread unreported.
#[test]
fn evidence_is_bounded_at_acquisition() -> Outcome_ {
    let mut cohort = cohort_of(2)?;
    let exact = "e".repeat(MAX_EVIDENCE_BYTES);
    let over = "e".repeat(MAX_EVIDENCE_BYTES + 1);
    assert_eq!(
        cohort.report(&id(1), 0, Outcome::Met, &over),
        Err(Refusal::EvidenceLimit)
    );
    assert_eq!(cohort.thread(&id(1))?.outcome, None);
    assert_eq!(cohort.evidence(&id(1))?, None);
    // The bound fires before the identity is read: a malformed identity with over-long
    // evidence names the bound, not the identity.
    assert_eq!(
        cohort.report("nope", 0, Outcome::Met, &over),
        Err(Refusal::EvidenceLimit)
    );
    cohort.report(&id(1), 0, Outcome::Met, &exact)?;
    assert_eq!(
        cohort.evidence(&id(1))?.map(str::len),
        Some(MAX_EVIDENCE_BYTES)
    );
    // Multi-byte text is bounded in bytes, not characters.
    let wide = "\u{e9}".repeat(MAX_EVIDENCE_BYTES / 2 + 1);
    assert_eq!(
        cohort.report(&id(2), 0, Outcome::Met, &wide),
        Err(Refusal::EvidenceLimit)
    );
    Ok(())
}

/// T22-CO-60 · COH-03: a thread carries a bounded role, reported back on its assignment.
/// Empty and over-long names are refused at construction; exactly the bound is admitted.
#[test]
fn a_thread_carries_a_bounded_role() -> Outcome_ {
    assert_eq!(Role::new(""), Err(Refusal::EmptyRole));
    assert_eq!(
        Role::new(&"r".repeat(MAX_ROLE_BYTES + 1)),
        Err(Refusal::RoleLimit)
    );
    let widest = Role::new(&"r".repeat(MAX_ROLE_BYTES))?;
    assert_eq!(widest.as_str().len(), MAX_ROLE_BYTES);
    let mut cohort = Cohort::new(1);
    cohort.assign(&id(1), Role::new("reviewer")?, &[], claim("src/a")?, true)?;
    cohort.assign(&id(2), widest, &[], claim("src/b")?, false)?;
    let roles: Vec<&str> = cohort
        .threads()?
        .iter()
        .map(|thread| thread.role.as_str())
        .collect();
    assert_eq!(roles, vec!["reviewer", "r".repeat(MAX_ROLE_BYTES).as_str()]);
    Ok(())
}

/// T22-CO-61 · COH-04: rebriefs are a bounded retry count. Exactly `MAX_REBRIEFS` are
/// admitted, the next is refused and leaves the thread as it was, and the count is visible on
/// the assignment as its generation.
#[test]
fn rebriefs_are_bounded_by_count() -> Outcome_ {
    let mut cohort = cohort_of(1)?;
    for attempt in 1..=MAX_REBRIEFS {
        cohort.rebrief(&id(1))?;
        assert_eq!(cohort.thread(&id(1))?.generation, attempt);
    }
    cohort.report(&id(1), MAX_REBRIEFS, Outcome::Unmet, "last attempt")?;
    assert_eq!(cohort.rebrief(&id(1)), Err(Refusal::RebriefLimit));
    let thread = cohort.thread(&id(1))?;
    assert_eq!(thread.generation, MAX_REBRIEFS);
    assert_eq!(thread.outcome, Some(Outcome::Unmet));
    Ok(())
}

/// T22-CO-62 · COH-05: a callback from a superseded attempt is refused. After a rebrief of a
/// CURRENT thread the brief is unchanged, so the brief check alone cannot tell the two
/// attempts apart; the generation does.
#[test]
fn a_superseded_generation_callback_is_refused() -> Outcome_ {
    let mut cohort = cohort_of(1)?;
    cohort.report(&id(1), 0, Outcome::Unmet, "first attempt")?;
    cohort.rebrief(&id(1))?;
    cohort.rebrief(&id(1))?;
    for late in [0, 1] {
        assert_eq!(
            cohort.report(&id(1), late, Outcome::Met, "late callback"),
            Err(Refusal::StaleGeneration),
            "generation {late}"
        );
    }
    assert_eq!(
        cohort.report(&id(1), 3, Outcome::Met, "from the future"),
        Err(Refusal::StaleGeneration)
    );
    assert_eq!(cohort.thread(&id(1))?.outcome, None);
    cohort.report(&id(1), 2, Outcome::Met, "current attempt")?;
    assert_eq!(cohort.evidence(&id(1))?, Some("current attempt"));
    assert!(cohort.join().is_integrable());
    Ok(())
}

/// T22-CO-63 · the new bounds are the values declared, asserted against the literal so a
/// planted change to a constant is seen (F122).
#[test]
fn the_wave_one_bounds_are_the_declared_values() {
    assert_eq!(MAX_EVIDENCE_BYTES, 65_536);
    assert_eq!(MAX_ROLE_BYTES, 64);
    assert_eq!(MAX_REBRIEFS, 8);
}
