//! The routine part of the TypeScript bindings (spec 20.8).

use std::fmt::Write as _;

use super::super::*;
use super::Out;

pub(super) fn decls(out: &mut Out) {
    out.decl::<Schedule>();
    out.decl::<Overlap>();
    out.decl::<Missed>();
    out.decl::<RunStatus>();
    out.decl::<SkipReason>();
    out.decl::<RoutineRun>();
    out.decl::<Routine>();
    out.decl::<RoutinesListParams>();
    out.decl::<RoutinesCreateParams>();
    out.decl::<RoutinesUpdateParams>();
    out.decl::<RoutinesSetEnabledParams>();
    out.decl::<RoutineIdParams>();
    out.decl::<RoutinesRunsParams>();
}

pub(super) fn methods(out: &mut Out) {
    let routine = out.name::<Routine>();
    let run = out.name::<RoutineRun>();
    out.method(
        method::ROUTINES_LIST,
        &out.name::<RoutinesListParams>(),
        &out.name::<Vec<Routine>>(),
    );
    out.method(
        method::ROUTINES_CREATE,
        &out.name::<RoutinesCreateParams>(),
        &routine,
    );
    out.method(
        method::ROUTINES_UPDATE,
        &out.name::<RoutinesUpdateParams>(),
        &routine,
    );
    out.method(
        method::ROUTINES_SET_ENABLED,
        &out.name::<RoutinesSetEnabledParams>(),
        &routine,
    );
    out.method(
        method::ROUTINES_RUN_NOW,
        &out.name::<RoutineIdParams>(),
        &run,
    );
    out.method(
        method::ROUTINES_ARCHIVE,
        &out.name::<RoutineIdParams>(),
        &routine,
    );
    out.method(
        method::ROUTINES_RUNS,
        &out.name::<RoutinesRunsParams>(),
        &out.name::<Vec<RoutineRun>>(),
    );
}

pub(super) fn notifications(out: &mut Out) {
    let routine = out.name::<Routine>();
    let run = out.name::<RoutineRun>();
    let _ = writeln!(
        out.text,
        "  \"{}\": {routine};",
        notification::ROUTINE_CHANGED
    );
    let _ = writeln!(out.text, "  \"{}\": {run};", notification::ROUTINE_RUN);
}
