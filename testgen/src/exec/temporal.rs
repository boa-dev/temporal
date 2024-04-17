use boa_engine::{
    js_string, native_function::NativeFunction, object::ObjectInitializer, property::Attribute,
    Context, JsArgs, JsData, JsError, JsNativeError, JsResult, JsValue,
};
use boa_gc::{Finalize, Trace};
use serde::Serialize;

use self::duration::Duration;

mod duration;

/// Initializes the required properties on the global object for testing.
pub(super) fn setup_context(context: &mut Context) -> JsResult<()> {
    let global_obj = context.global_object();

    let throw_fn = NativeFunction::from_fn_ptr(|_, _, _| Err(TemporalUnsupported.into()));
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

    context.register_global_property(
        js_string!("$262"),
        js262,
        Attribute::WRITABLE | Attribute::CONFIGURABLE,
    )?;

    context.register_global_class::<Duration>()?;
    context
        .global_object()
        .delete_property_or_throw(js_string!("Duration"), context)?;
    let constructor = context
        .get_global_class::<Duration>()
        .expect("already registered the class")
        .constructor();

    let temporal = ObjectInitializer::new(context)
        .property(js_string!("Duration"), constructor, Attribute::all())
        .build();

    context.register_global_property(js_string!("Temporal"), temporal, Attribute::all())?;

    context
        .realm()
        .host_defined_mut()
        .insert_default::<AssertionsTracker>();

    Ok(())
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TemporalUnsupported;

impl From<TemporalUnsupported> for JsError {
    fn from(_value: TemporalUnsupported) -> Self {
        JsNativeError::error()
            .with_message("TemporalTester:unsupported")
            .into()
    }
}

#[derive(Clone, Trace, Finalize, Serialize)]
pub(crate) enum Assertion {
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

#[derive(Default, Clone, Serialize, Trace, Finalize, JsData)]
pub(crate) struct AssertionsTracker {
    pub(crate) assertions: Vec<Assertion>,
}

pub(crate) fn patch_harness(context: &mut Context) -> JsResult<()> {
    let helpers = context
        .global_object()
        .get(js_string!("TemporalHelpers"), context)?;

    let Some(o) = helpers.as_object() else {
        return Ok(());
    };

    o.set(
        js_string!("assertDuration"),
        NativeFunction::from_fn_ptr(assert_duration).to_js_function(context.realm()),
        true,
        context,
    )?;

    Ok(())
}

fn assert_duration(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let target = args
        .get_or_undefined(0)
        .as_object()
        .and_then(|o| o.downcast_ref::<Duration>())
        .map(|o| o.clone())
        .ok_or_else(|| JsNativeError::typ().with_message("invalid call to `assertDuration`"))?;
    let years: i32 = args.get_or_undefined(1).try_js_into(context)?;
    let months: i32 = args.get_or_undefined(2).try_js_into(context)?;
    let weeks: i32 = args.get_or_undefined(3).try_js_into(context)?;
    let days: i32 = args.get_or_undefined(4).try_js_into(context)?;
    let hours: i32 = args.get_or_undefined(5).try_js_into(context)?;
    let minutes: i32 = args.get_or_undefined(6).try_js_into(context)?;
    let seconds: i32 = args.get_or_undefined(7).try_js_into(context)?;
    let milliseconds: i32 = args.get_or_undefined(8).try_js_into(context)?;
    let microseconds: i32 = args.get_or_undefined(8).try_js_into(context)?;
    let nanoseconds: i32 = args.get_or_undefined(9).try_js_into(context)?;

    context
        .realm()
        .host_defined_mut()
        .get_mut::<AssertionsTracker>()
        .ok_or_else(|| JsNativeError::typ().with_message("missing assertions tracker on context"))?
        .assertions
        .push(Assertion::DurationFullValue {
            target,
            years,
            months,
            weeks,
            days,
            hours,
            minutes,
            seconds,
            milliseconds,
            microseconds,
            nanoseconds,
        });

    Ok(JsValue::undefined())
}
