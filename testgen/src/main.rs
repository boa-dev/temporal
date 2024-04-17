//! Test262 test runner
//!
//! This crate will run the full ECMAScript test suite (Test262) and report compliance of the
//! `boa` engine.
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![allow(
    clippy::too_many_lines,
    clippy::redundant_pub_crate,
    clippy::cast_precision_loss
)]

mod edition;
mod exec;
mod read;

use self::read::{read_harness, read_suite, MetaData, Negative, TestFlag};
use bitflags::bitflags;
use color_eyre::{
    eyre::{bail, eyre, Report, WrapErr},
    Result,
};
use edition::SpecEdition;
use log::{info, warn};
use read::ErrorType;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{
    de::{Unexpected, Visitor},
    Deserialize, Deserializer,
};
use std::{path::Path, process::Command};

const DEFAULT_TEST262_DIRECTORY: &str = "test262";

const TESTDATA_DIRECTORY: &str = "temporal_rs/testdata";

/// Program entry point.
fn main() -> Result<()> {
    const TEST262_COMMIT: &str = "6f7ae1f311a7b01ef2358de7f4f6fd42c3ae3839";
    color_eyre::install()?;
    env_logger::init();

    let threading = true;

    clone_test262(Some(TEST262_COMMIT))?;
    let test262_path = Path::new(DEFAULT_TEST262_DIRECTORY);

    run_temporal_suite(threading, test262_path)
}

/// Returns the commit hash and commit message of the provided branch name.
fn get_last_branch_commit(branch: &str) -> Result<(String, String)> {
    info!("Getting last commit on '{branch}' branch");

    let result = Command::new("git")
        .arg("log")
        .args(["-n", "1"])
        .arg("--pretty=format:%H %s")
        .arg(branch)
        .current_dir(DEFAULT_TEST262_DIRECTORY)
        .output()?;

    if !result.status.success() {
        bail!(
            "test262 getting commit hash and message failed with return code {:?}",
            result.status.code()
        );
    }

    let output = std::str::from_utf8(&result.stdout)?.trim();

    let (hash, message) = output
        .split_once(' ')
        .expect("git log output to contain hash and message");

    Ok((hash.into(), message.into()))
}

fn reset_test262_commit(commit: &str) -> Result<()> {
    info!("Reset test262 to commit: {commit}...");

    let result = Command::new("git")
        .arg("reset")
        .arg("--hard")
        .arg(commit)
        .current_dir(DEFAULT_TEST262_DIRECTORY)
        .status()?;

    if !result.success() {
        bail!(
            "test262 commit {commit} checkout failed with return code: {:?}",
            result.code()
        );
    }

    Ok(())
}

fn clone_test262(commit: Option<&str>) -> Result<()> {
    const TEST262_REPOSITORY: &str = "https://github.com/tc39/test262";

    let update = commit.is_none();

    if Path::new(DEFAULT_TEST262_DIRECTORY).is_dir() {
        let (current_commit_hash, current_commit_message) = get_last_branch_commit("HEAD")?;

        if let Some(commit) = commit {
            if current_commit_hash == commit {
                return Ok(());
            }
        }

        info!("Fetching latest test262 commits...");

        let result = Command::new("git")
            .arg("fetch")
            .current_dir(DEFAULT_TEST262_DIRECTORY)
            .status()?;

        if !result.success() {
            bail!(
                "Test262 fetching latest failed with return code {:?}",
                result.code()
            );
        }

        if let Some(commit) = commit {
            println!("Test262 switching to commit {commit}...");
            reset_test262_commit(commit)?;
            return Ok(());
        }

        info!("Checking latest Test262 with current HEAD...");

        let (latest_commit_hash, latest_commit_message) = get_last_branch_commit("origin/main")?;

        if current_commit_hash != latest_commit_hash {
            if update {
                info!("Updating Test262 repository:");
            } else {
                warn!("Test262 repository is not in sync, use '--test262-commit latest' to automatically update it:");
            }

            info!("    Current commit: {current_commit_hash} {current_commit_message}");
            info!("    Latest commit:  {latest_commit_hash} {latest_commit_message}");

            if update {
                reset_test262_commit(&latest_commit_hash)?;
            }
        }

        return Ok(());
    }

    println!("Cloning test262...");
    let result = Command::new("git")
        .arg("clone")
        .arg(TEST262_REPOSITORY)
        .arg(DEFAULT_TEST262_DIRECTORY)
        .status()?;

    if !result.success() {
        bail!(
            "Cloning Test262 repository failed with return code {:?}",
            result.code()
        );
    }

    if let Some(commit) = commit {
        info!("Reset Test262 to commit: {commit}...");

        reset_test262_commit(commit)?;
    }

    Ok(())
}

/// Runs the full test suite.
fn run_temporal_suite(parallel: bool, test262_path: &Path) -> Result<()> {
    info!("Loading the test suite...");
    let harness = read_harness(test262_path).wrap_err("could not read the harness file")?;

    let suite_path = test262_path.join(Path::new("test/built-ins/Temporal"));

    let suite = read_suite(&suite_path).wrap_err_with(|| {
        eyre!(
            "could not read the Temporal suite at `{}`",
            suite_path.display()
        )
    })?;
    info!("Test suite loaded, purging old tests...");
    if let Err(e) = std::fs::remove_dir_all(TESTDATA_DIRECTORY) {
        if e.kind() != std::io::ErrorKind::NotFound {
            return Err(Report::new(e)
                .wrap_err(format!("failed to delete directory `{TESTDATA_DIRECTORY}`")));
        }
        info!("Creating new directory `{TESTDATA_DIRECTORY}`");
    }
    std::fs::create_dir_all(TESTDATA_DIRECTORY)
        .wrap_err_with(|| format!("failed to create directory `{TESTDATA_DIRECTORY}`"))?;

    suite.run(&harness, parallel, SpecEdition::ESNext)?;

    Ok(())
}

/// All the harness include files.
#[derive(Debug, Clone)]
struct Harness {
    assert: HarnessFile,
    sta: HarnessFile,
    includes: FxHashMap<Box<str>, HarnessFile>,
}

#[derive(Debug, Clone)]
struct HarnessFile {
    content: Box<str>,
    path: Box<Path>,
}

/// Represents a test suite.
#[derive(Debug, Clone)]
struct TestSuite {
    name: Box<str>,
    path: Box<Path>,
    suites: Box<[TestSuite]>,
    tests: Box<[Test]>,
}

/// Represents a test.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Test {
    name: Box<str>,
    path: Box<Path>,
    description: Box<str>,
    esid: Option<Box<str>>,
    edition: SpecEdition,
    flags: TestFlags,
    information: Box<str>,
    expected_outcome: Outcome,
    features: FxHashSet<Box<str>>,
    includes: FxHashSet<Box<str>>,
    locale: Locale,
    ignored: bool,
}

impl Test {
    /// Creates a new test.
    fn new<N, C>(name: N, path: C, metadata: MetaData) -> Result<Self>
    where
        N: Into<Box<str>>,
        C: Into<Box<Path>>,
    {
        let edition = SpecEdition::from_test_metadata(&metadata)
            .map_err(|feats| eyre!("test metadata contained unknown features: {feats:?}"))?;

        Ok(Self {
            edition,
            name: name.into(),
            description: metadata.description,
            esid: metadata.esid,
            flags: metadata.flags.into(),
            information: metadata.info,
            features: metadata.features.into_vec().into_iter().collect(),
            expected_outcome: Outcome::from(metadata.negative),
            includes: metadata.includes.into_vec().into_iter().collect(),
            locale: metadata.locale,
            path: path.into(),
            ignored: false,
        })
    }

    /// Sets the test as ignored.
    #[inline]
    fn set_ignored(&mut self) {
        self.ignored = true;
    }

    /// Checks if this is a module test.
    #[inline]
    const fn is_module(&self) -> bool {
        self.flags.contains(TestFlags::MODULE)
    }
}

/// An outcome for a test.
#[derive(Debug, Clone)]
enum Outcome {
    Positive,
    Negative { phase: Phase, error_type: ErrorType },
}

impl Default for Outcome {
    fn default() -> Self {
        Self::Positive
    }
}

impl From<Option<Negative>> for Outcome {
    fn from(neg: Option<Negative>) -> Self {
        neg.map(|neg| Self::Negative {
            phase: neg.phase,
            error_type: neg.error_type,
        })
        .unwrap_or_default()
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    struct TestFlags: u16 {
        const STRICT = 0b0_0000_0001;
        const NO_STRICT = 0b0_0000_0010;
        const MODULE = 0b0_0000_0100;
        const RAW = 0b0_0000_1000;
        const ASYNC = 0b0_0001_0000;
        const GENERATED = 0b0_0010_0000;
        const CAN_BLOCK_IS_FALSE = 0b0_0100_0000;
        const CAN_BLOCK_IS_TRUE = 0b0_1000_0000;
        const NON_DETERMINISTIC = 0b1_0000_0000;
    }
}

impl Default for TestFlags {
    fn default() -> Self {
        Self::STRICT | Self::NO_STRICT
    }
}

impl From<TestFlag> for TestFlags {
    fn from(flag: TestFlag) -> Self {
        match flag {
            TestFlag::OnlyStrict => Self::STRICT,
            TestFlag::NoStrict => Self::NO_STRICT,
            TestFlag::Module => Self::MODULE,
            TestFlag::Raw => Self::RAW,
            TestFlag::Async => Self::ASYNC,
            TestFlag::Generated => Self::GENERATED,
            TestFlag::CanBlockIsFalse => Self::CAN_BLOCK_IS_FALSE,
            TestFlag::CanBlockIsTrue => Self::CAN_BLOCK_IS_TRUE,
            TestFlag::NonDeterministic => Self::NON_DETERMINISTIC,
        }
    }
}

impl<T> From<T> for TestFlags
where
    T: AsRef<[TestFlag]>,
{
    fn from(flags: T) -> Self {
        let flags = flags.as_ref();
        if flags.is_empty() {
            Self::default()
        } else {
            let mut result = Self::empty();
            for flag in flags {
                result |= Self::from(*flag);
            }

            if !result.intersects(Self::default()) {
                result |= Self::default();
            }

            result
        }
    }
}

impl<'de> Deserialize<'de> for TestFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FlagsVisitor;

        impl<'de> Visitor<'de> for FlagsVisitor {
            type Value = TestFlags;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "a sequence of flags")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut flags = TestFlags::empty();
                while let Some(elem) = seq.next_element::<TestFlag>()? {
                    flags |= elem.into();
                }
                Ok(flags)
            }
        }

        struct RawFlagsVisitor;

        impl Visitor<'_> for RawFlagsVisitor {
            type Value = TestFlags;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "a flags number")
            }

            fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                TestFlags::from_bits(v).ok_or_else(|| {
                    E::invalid_value(Unexpected::Unsigned(v.into()), &"a valid flag number")
                })
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_seq(FlagsVisitor)
        } else {
            deserializer.deserialize_u16(RawFlagsVisitor)
        }
    }
}

/// Phase for an error.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Phase {
    Parse,
    Resolution,
    Runtime,
}

/// Locale information structure.
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(transparent)]
#[allow(dead_code)]
struct Locale {
    locale: Box<[Box<str>]>,
}
