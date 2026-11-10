//! Optional CLI soft-budget adapter. No prompts are inferred from noninteractive
//! stdin; --proceed-over-budget is the explicit decision for this invocation.
use super::{args::ValidateArgs, bundle::SelectedBundle};
use lintra::*;
use std::{
    error::Error,
    fs,
    io::{self, Write},
};

pub(super) fn prepare(
    args: &ValidateArgs,
    selected: &SelectedBundle,
    cancellation: &CancellationToken,
) -> Result<Option<AcceptedResourcePreflight>, Box<dyn Error>> {
    let Some(path) = &args.resource_budget else {
        return Ok(None);
    };
    let budget: ResourceBudget = serde_json::from_slice(&fs::read(path)?)?;
    #[cfg(feature = "embedded-bundle")]
    let embedded;
    let references = match selected.roots.get(1) {
        Some(root) => ResourceReferenceSource::Filesystem(root),
        None => {
            #[cfg(feature = "embedded-bundle")]
            {
                embedded = EmbeddedBundle::builtin()?;
                ResourceReferenceSource::Embedded(&embedded)
            }
            #[cfg(not(feature = "embedded-bundle"))]
            {
                return Err(io::Error::other("selected reference source is unavailable").into());
            }
        }
    };
    let prepared = selected.service.resource_preflight(
        args.request.clone(),
        references,
        &budget,
        cancellation,
    )?;
    if prepared.audit().requires_confirmation() {
        writeln!(
            io::stderr().lock(),
            "Avviso: budget operativo soft superato. Annulla o scegli Procedi comunque con --proceed-over-budget per questa richiesta."
        )?;
    }
    let accepted = prepared.accept(args.proceed_over_budget)?;
    // Only aggregate counters and the decision escape: no file names, hashes,
    // measurement-source text, raw records or patient identifiers in this log.
    serde_json::to_writer(io::stderr().lock(), &accepted.audit())?;
    writeln!(io::stderr().lock())?;
    Ok(Some(accepted))
}
