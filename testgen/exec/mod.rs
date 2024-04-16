//! Execution module for the test runner.

mod temporal;

use crate::{
    exec::temporal::inject, read::ErrorType, Harness, Outcome, Phase, SpecEdition, Test, TestFlags,
    TestSuite,
};
use boa_engine::{
    error::JsErasedError, js_string, Context, JsError, JsNativeErrorKind, JsValue, Source,
};
use color_eyre::{
    eyre::{bail, eyre, WrapErr},
    Result,
};
use log::{debug, info, warn};
use rayon::prelude::*;

impl TestSuite {
    /// Runs the test suite.
    pub(crate) fn run(
        &self,
        harness: &Harness,
        parallel: bool,
        max_edition: SpecEdition,
    ) -> Result<()> {
        info!("Running suite `{}`.", self.path.display());

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
            debug!("Skipping test `{}` ({})", self.path.display(), skip);
            return Ok(());
        }

        self.run_once(harness)
    }

    /// Runs the test once, in strict or non-strict mode
    fn run_once(&self, harness: &Harness) -> Result<()> {
        let Ok(source) = Source::from_filepath(&self.path) else {
            bail!("could not read file `{}`", self.path.display());
        };

        debug!(target: "tests", "starting test `{}`", self.path.display());

        let value = std::panic::catch_unwind(|| match self.expected_outcome {
            Outcome::Positive => {
                let context = &mut Context::default();

                self.set_up_env(harness, context)?;

                context
                    .eval(source)
                    .map_err(|e| e.into_erased(context).into())
            }
            _ => Err(eyre!("invalid outcome for test `{}`", self.path.display())),
        })
        .map_err(|_| eyre!("detected panic on test `{}`", self.path.display()))?;

        match value {
            Ok(v) => {
                debug!(target: "tests", "`{}`: result text", self.path.display());
                debug!(target: "tests", "{}", v.display());
            }
            Err(e) => {
                if let Some(e) = e
                    .downcast_ref::<JsErasedError>()
                    .and_then(JsErasedError::as_native)
                {
                    if e.to_string() == "Error: TemporalTester:unsupported" {
                        debug!(target: "tests", "`{}`: Filtered test", self.path.display());
                        return Ok(());
                    }
                }

                warn!(target: "tests", "`{}`: FAILED. Ignoring...", self.path.display());
                warn!(target: "tests", "`{}`: {e}", self.path.display());
            }
        }

        Ok(())
    }

    /// Sets the environment up to run the test.
    fn set_up_env(&self, harness: &Harness, context: &mut Context) -> Result<()> {
        // add the $262 object.
        let _js262 = temporal::register_js262(context);

        let assert = Source::from_reader(
            harness.assert.content.as_bytes(),
            Some(&harness.assert.path),
        );
        let sta = Source::from_reader(harness.sta.content.as_bytes(), Some(&harness.sta.path));

        context
            .eval(assert)
            .map_err(|e| e.into_erased(context))
            .wrap_err("could not run assert.js")?;
        context
            .eval(sta)
            .map_err(|e| e.into_erased(context))
            .wrap_err("could not run sta.js")?;

        for include_name in &self.includes {
            let include = harness
                .includes
                .get(include_name)
                .ok_or_else(|| eyre!("could not find the {include_name} include file."))?;
            let source = Source::from_reader(include.content.as_bytes(), Some(&include.path));
            context
                .eval(source)
                .map_err(|e| e.into_erased(context))
                .wrap_err_with(|| eyre!("could not run the harness `{include_name}`"))?;
        }

        inject(context)
            .map_err(|e| e.into_erased(context))
            .wrap_err("could not patch the `Temporal` utils")?;

        Ok(())
    }
}

/// Returns `true` if `error` is a `target_type` error.
fn is_error_type(error: &JsError, target_type: ErrorType, context: &mut Context) -> bool {
    if let Ok(error) = error.try_native(context) {
        match &error.kind {
            JsNativeErrorKind::Syntax if target_type == ErrorType::SyntaxError => {}
            JsNativeErrorKind::Reference if target_type == ErrorType::ReferenceError => {}
            JsNativeErrorKind::Range if target_type == ErrorType::RangeError => {}
            JsNativeErrorKind::Type if target_type == ErrorType::TypeError => {}
            _ => return false,
        }
        true
    } else {
        let passed = error
            .as_opaque()
            .expect("try_native cannot fail if e is not opaque")
            .as_object()
            .and_then(|o| o.get(js_string!("constructor"), context).ok())
            .as_ref()
            .and_then(JsValue::as_object)
            .and_then(|o| o.get(js_string!("name"), context).ok())
            .as_ref()
            .and_then(JsValue::as_string)
            .map(|s| s == target_type.as_str())
            .unwrap_or_default();
        passed
    }
}
