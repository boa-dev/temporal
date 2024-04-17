//! Execution module for the test runner.

mod temporal;

use std::{io::BufWriter, path::PathBuf};

use crate::{
    exec::temporal::AssertionsTracker, Harness, Outcome, Phase, SpecEdition, Test, TestFlags,
    TestSuite, TESTDATA_DIRECTORY,
};
use boa_engine::{Context, JsNativeError, JsResult, Source};
use color_eyre::{
    eyre::{bail, WrapErr},
    Result,
};
use log::{debug, error, info, warn};
use rayon::prelude::*;

impl TestSuite {
    /// Runs the test suite.
    pub(crate) fn run(
        &self,
        harness: &Harness,
        parallel: bool,
        max_edition: SpecEdition,
    ) -> Result<()> {
        info!(target: "testgen", "Running suite `{}`.", self.path.display());

        let suites: Result<Vec<_>> = if parallel {
            self.suites
                .par_iter()
                .map(|suite| suite.run(harness, parallel, max_edition))
                .collect()
        } else {
            self.suites
                .iter()
                .map(|suite| suite.run(harness, parallel, max_edition))
                .collect()
        };

        suites?;

        let tests: Result<Vec<_>> = if parallel {
            self.tests
                .par_iter()
                .filter(|test| test.edition <= max_edition)
                .map(|test| test.run(harness))
                .collect()
        } else {
            self.tests
                .iter()
                .filter(|test| test.edition <= max_edition)
                .map(|test| test.run(harness))
                .collect()
        };

        tests?;

        Ok(())
    }
}

impl Test {
    /// Runs the test.
    pub(crate) fn run(&self, harness: &Harness) -> Result<()> {
        let skip = if self
            .flags
            .intersects(TestFlags::MODULE | TestFlags::RAW | TestFlags::ASYNC)
            || !self
                .flags
                .contains(TestFlags::STRICT | TestFlags::NO_STRICT)
        {
            Some("incompatible test flags")
        } else if matches!(
            self.expected_outcome,
            Outcome::Negative {
                phase: Phase::Parse | Phase::Resolution,
                ..
            }
        ) {
            Some("can only run negative outcomes in the execution phase")
        } else {
            None
        };

        if let Some(skip) = skip {
            debug!(target: "testgen", "Skipping test `{}` ({})", self.path.display(), skip);
            return Ok(());
        }

        self.run_once(harness)
    }

    /// Runs the test once, in strict or non-strict mode
    fn run_once(&self, harness: &Harness) -> Result<()> {
        let Ok(source) = Source::from_filepath(&self.path) else {
            bail!("could not read file `{}`", self.path.display());
        };

        debug!(target: "testgen", "starting test `{}`", self.path.display());

        let result: JsResult<_> = match self.expected_outcome {
            Outcome::Positive => (|| {
                let context = &mut Context::default();
                self.set_up_env(harness, context)?;
                let value = context.eval(source)?;

                let tracker: AssertionsTracker = *context
                    .realm()
                    .host_defined_mut()
                    .remove::<AssertionsTracker>()
                    .ok_or_else(|| {
                        JsNativeError::typ().with_message("missing tracker on context")
                    })?;

                Ok((value, tracker))
            })(),
            _ => Err(temporal::TemporalUnsupported.into()),
        };

        match result {
            Ok((v, tracker)) => {
                debug!(target: "testgen", "`{}`: result text", self.path.display());
                debug!(target: "testgen", "{}", v.display());
                if tracker.assertions.is_empty() {
                    warn!(target: "testgen", "`{}`: generated empty tracker. Skipping.", self.path.display());
                    return Ok(());
                }

                let mut new_path = PathBuf::from(TESTDATA_DIRECTORY);
                new_path.push(self.path.strip_prefix("test262/test/built-ins/Temporal")?);
                new_path.set_extension("json");
                if let Some(parent) = new_path.parent() {
                    std::fs::create_dir_all(parent).wrap_err_with(|| {
                        format!("could not create directory `{}`", parent.display())
                    })?
                }

                let file = std::fs::OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(&new_path)
                    .wrap_err_with(|| format!("could not write file `{}`", new_path.display()))?;

                let file = &mut BufWriter::new(file);

                serde_json::to_writer_pretty(file, &tracker).wrap_err_with(|| {
                    format!(
                        "could not serialize tracker for test `{}`",
                        self.path.display()
                    )
                })?;
            }
            Err(e) => {
                if e.to_string() == "Error: TemporalTester:unsupported" {
                    debug!(target: "testgen", "`{}`: Filtered test", self.path.display());
                    return Ok(());
                }

                warn!(target: "testgen", "`{}`: FAILED. Ignoring...", self.path.display());
                warn!(target: "testgen", "`{}`: {e}", self.path.display());
            }
        }

        Ok(())
    }

    /// Sets the environment up to run the test.
    fn set_up_env(&self, harness: &Harness, context: &mut Context) -> JsResult<()> {
        // add the $262 object.
        temporal::setup_context(context)?;

        let assert = Source::from_reader(
            harness.assert.content.as_bytes(),
            Some(&harness.assert.path),
        );
        let sta = Source::from_reader(harness.sta.content.as_bytes(), Some(&harness.sta.path));

        context.eval(assert).map_err(|e| {
            JsNativeError::eval()
                .with_message("could not run assert.js")
                .with_cause(e)
        })?;
        context.eval(sta).map_err(|e| {
            JsNativeError::eval()
                .with_message("could not run sta.js")
                .with_cause(e)
        })?;

        for include_name in &self.includes {
            let include = harness.includes.get(include_name).ok_or_else(|| {
                JsNativeError::typ()
                    .with_message(format!("could not find the {include_name} include file."))
            })?;
            let source = Source::from_reader(include.content.as_bytes(), Some(&include.path));
            context.eval(source).map_err(|e| {
                JsNativeError::eval()
                    .with_message(format!("could not run the harness `{include_name}`"))
                    .with_cause(e)
            })?;
        }

        temporal::patch_harness(context).map_err(|e| {
            JsNativeError::typ()
                .with_message("could not patch the `Temporal` utils")
                .with_cause(e)
        })?;

        Ok(())
    }
}
