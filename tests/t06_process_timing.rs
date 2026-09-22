//! Actual child controls for interruption timestamps used by receipt causality.
use habitat_engine::worker::process::{
    self, Interruption, Observer, ObserverDecision, ProcessObservation, ProcessSpec,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

fn spec(executable: &str, args: &[&str]) -> ProcessSpec {
    ProcessSpec {
        executable: executable.into(),
        arguments: args.iter().map(Into::into).collect(),
        directory: std::env::temp_dir().canonicalize().unwrap(),
        environment: vec![("LC_ALL".into(), "C".into())],
        input: Vec::new(),
        stream_limit: 1024,
    }
}

#[test]
fn ordinary_completion_has_no_invented_interruption_time() {
    let before = Instant::now();
    let report = process::run(
        &spec("/usr/bin/true", &[]),
        before + Duration::from_secs(10),
        &AtomicBool::new(false),
    )
    .unwrap();
    assert!(report.started_at >= before);
    assert_eq!(report.exit_code, Some(0));
    assert_eq!(report.interruption, None);
    assert_eq!(report.interruption_observed_at, None);
    assert!(report.leader_reaped && report.process_group_settled && report.pending.is_none());
}

#[test]
fn timeout_records_actual_observation_after_the_original_cutoff() {
    let cutoff = Instant::now() + Duration::from_millis(100);
    let report = process::run(
        &spec("/usr/bin/sleep", &["5"]),
        cutoff,
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(report.interruption, Some(Interruption::Timeout));
    let stop_time = report.interruption_observed_at.unwrap();
    assert!(stop_time >= cutoff);
    assert!(stop_time.duration_since(report.started_at) <= report.elapsed);
    assert!(report.leader_reaped && report.process_group_settled && report.pending.is_none());
}

struct CancelAfterDispatch<'a> {
    cancel: &'a AtomicBool,
    requested: Option<Instant>,
    stops: Vec<(Instant, Instant)>,
}
impl Observer for CancelAfterDispatch<'_> {
    fn observe(&mut self, observation: &ProcessObservation<'_>) -> ObserverDecision {
        if self.requested.is_none() {
            self.requested = Some(Instant::now());
            self.cancel.store(true, Ordering::Release);
        }
        if let Some(stop) = observation.stopping_since {
            self.stops
                .push((stop, observation.cleanup_deadline.unwrap()));
        }
        ObserverDecision {
            hold_stdin: false,
            ready_to_finish: true,
            stop: None,
        }
    }
}

#[test]
fn cancellation_and_cleanup_share_the_first_observation_without_renewal() {
    let cancelled = AtomicBool::new(false);
    let mut observer = CancelAfterDispatch {
        cancel: &cancelled,
        requested: None,
        stops: Vec::new(),
    };
    let report = process::run_observed(
        &spec("/usr/bin/sleep", &["5"]),
        Instant::now() + Duration::from_secs(10),
        &cancelled,
        &mut observer,
    )
    .unwrap();
    assert_eq!(report.interruption, Some(Interruption::Cancelled));
    let stop_time = report.interruption_observed_at.unwrap();
    assert!(stop_time >= observer.requested.unwrap());
    assert!(stop_time.duration_since(report.started_at) <= report.elapsed);
    assert!(!observer.stops.is_empty());
    for (stop, cutoff) in observer.stops {
        assert_eq!(stop, stop_time);
        assert_eq!(cutoff.duration_since(stop), Duration::from_secs(10));
    }
    assert!(report.leader_reaped && report.process_group_settled && report.pending.is_none());
}
