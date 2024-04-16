use boa_engine::{
    js_string,
    native_function::NativeFunction,
    object::{JsObject, ObjectInitializer},
    property::Attribute,
    Context, JsError, JsNativeError, JsResult,
};
use boa_gc::{Finalize, Trace};
use serde::Serialize;

use self::duration::Duration;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct TemporalUnsupported;

impl From<TemporalUnsupported> for JsError {
    fn from(value: TemporalUnsupported) -> Self {
        JsNativeError::error()
            .with_message("Temporal:unsupported")
            .into()
    }
}

mod duration;

enum Assertion {
    DurationFullValue {
        target: Duration,
        years: i32,
        months: i32,
        weeks: i32,
        days: i32,
        hours: i32,
        minutes: i32,
        seconds: i32,
        milliseconds: i32,
        microseconds: i32,
        nanoseconds: i32,
    },
    DurationDateValue {},
}

struct AssertionsTracker {
    assertions: Vec<Assertion>,
}

pub(crate) fn inject(context: &mut Context, tracker: TemporalTracker) -> JsResult<()> {}

/// Creates the object $262 in the context.
pub(super) fn register_js262(context: &mut Context) -> JsObject {
    let global_obj = context.global_object();

    let throw_fn = NativeFunction::from_fn_ptr(|_, _, _| {
        Err(JsNativeError::error()
            .with_message("TemporalTester:unsupported")
            .into())
    });
    let throw_fn_obj = throw_fn.clone().to_js_function(context.realm());

    let js262 = ObjectInitializer::new(context)
        .function(throw_fn.clone(), js_string!("createRealm"), 0)
        .function(throw_fn.clone(), js_string!("detachArrayBuffer"), 2)
        .function(throw_fn.clone(), js_string!("evalScript"), 1)
        .function(throw_fn.clone(), js_string!("gc"), 0)
        .property(
            js_string!("global"),
            global_obj,
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )
        .accessor(
            js_string!("agent"),
            Some(throw_fn_obj),
            None,
            Attribute::CONFIGURABLE,
        )
        .build();

    context
        .register_global_property(
            js_string!("$262"),
            js262.clone(),
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )
        .expect("shouldn't fail with the default global");

    js262
}
