use boa_engine::prelude::*;
use boa_engine::{
    class::{Class, ClassBuilder},
    js_string, Context, JsArgs, JsResult,
};
use boa_gc::{Finalize, Trace};
use serde::{Deserialize, Serialize};

use super::TemporalUnsupported;

#[derive(Clone, Debug, Trace, Finalize, Serialize, Deserialize)]
pub(crate) enum DurationInitializer {
    Full {
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
    String(String),
}

#[derive(Clone, Debug, Trace, Finalize, Serialize, Deserialize, JsData)]
pub(crate) enum Duration {
    Plain(DurationInitializer),
    Add {
        target: Box<Duration>,
        other: Box<Duration>,
        // TODO: relativeTo,
    },
    Subtract {
        target: Box<Duration>,
        other: Box<Duration>,
        // TODO: relativeTo,
    },
    Negated(Box<Duration>),
    Abs(Box<Duration>),
    // TODO: round
}

impl Class for Duration {
    const NAME: &'static str = "Duration";

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        class
            .static_method(
                js_string!("from"),
                1,
                NativeFunction::from_fn_ptr(Self::from),
            )
            .method(js_string!("add"), 1, NativeFunction::from_fn_ptr(Self::add));

        Ok(())
    }

    fn data_constructor(
        _new_target: &boa_engine::prelude::JsValue,
        args: &[boa_engine::prelude::JsValue],
        context: &mut Context,
    ) -> boa_engine::JsResult<Self> {
        let years: i32 = args
            .get_or_undefined(0)
            .try_js_into(context)
            .map_err(|_| TemporalUnsupported)?;
        let months = args
            .get_or_undefined(1)
            .try_js_into(context)
            .map_err(|_| TemporalUnsupported)?;
        let weeks = args
            .get_or_undefined(2)
            .try_js_into(context)
            .map_err(|_| TemporalUnsupported)?;
        let days = args
            .get_or_undefined(3)
            .try_js_into(context)
            .map_err(|_| TemporalUnsupported)?;
        let hours = args
            .get_or_undefined(4)
            .try_js_into(context)
            .map_err(|_| TemporalUnsupported)?;
        let minutes = args
            .get_or_undefined(5)
            .try_js_into(context)
            .map_err(|_| TemporalUnsupported)?;
        let seconds = args
            .get_or_undefined(6)
            .try_js_into(context)
            .map_err(|_| TemporalUnsupported)?;
        let milliseconds = args
            .get_or_undefined(7)
            .try_js_into(context)
            .map_err(|_| TemporalUnsupported)?;
        let microseconds = args
            .get_or_undefined(8)
            .try_js_into(context)
            .map_err(|_| TemporalUnsupported)?;
        let nanoseconds = args
            .get_or_undefined(9)
            .try_js_into(context)
            .map_err(|_| TemporalUnsupported)?;

        Ok(Self::Plain(DurationInitializer::Full {
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
        }))
    }
}

impl Duration {
    fn from_js_value(duration: &JsValue) -> JsResult<Duration> {
        if let Some(o) = duration.as_object().and_then(|o| o.downcast_ref::<Self>()) {
            return Ok(o.clone());
        }

        if let JsValue::String(s) = duration {
            return Ok(Duration::Plain(DurationInitializer::String(
                s.to_std_string_escaped(),
            )));
        }

        Err(TemporalUnsupported.into())
    }
    fn from(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let duration = args.get_or_undefined(0);
        if args.len() > 1 {
            return Err(TemporalUnsupported.into());
        }

        let prototype = context
            .get_global_class::<Self>()
            .ok_or_else(|| JsNativeError::typ().with_message("missing class `Temporal.Duration`"))?
            .prototype();
        let duration = Self::from_js_value(duration)?;

        Ok(JsObject::from_proto_and_data(prototype, duration).into())
    }

    fn add(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        if args.len() > 1 {
            return Err(TemporalUnsupported.into());
        }
        let lhs = this
            .as_object()
            .cloned()
            .and_then(|o| o.downcast::<Duration>().ok())
            .ok_or(TemporalUnsupported)?;
        let lhs = lhs.borrow().data().clone();
        let rhs = Self::from_js_value(args.get_or_undefined(0))?;

        let prototype = context
            .eval(Source::from_bytes("Temporal.Duration.prototype"))?
            .as_object()
            .cloned()
            .ok_or(TemporalUnsupported)?;

        Ok(JsObject::from_proto_and_data(
            prototype,
            Duration::Add {
                target: Box::new(lhs),
                other: Box::new(rhs),
            },
        )
        .into())
    }
}
