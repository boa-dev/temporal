use boa_engine::prelude::*;
use boa_engine::{
    class::{Class, ClassBuilder},
    js_string, Context, JsArgs, JsResult,
};
use boa_gc::{Finalize, Trace};
use serde::{Deserialize, Serialize};

use super::TemporalUnsupported;

#[derive(Clone, Debug, Trace, Finalize, Serialize, Deserialize)]
enum DurationInitializer {
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
    const NAME: &'static str = "Temporal.Duration";

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
        new_target: &boa_engine::prelude::JsValue,
        args: &[boa_engine::prelude::JsValue],
        context: &mut Context,
    ) -> boa_engine::JsResult<Self> {
        fn to_integer_if_integral(args: &[JsValue], idx: usize) -> JsResult<i32> {
            let value = args.get_or_undefined(0);
            if value.is_undefined() {
                Ok(0)
            } else if let JsValue::Integer(i) = value {
                Ok(*i)
            } else {
                Err(JsNativeError::error()
                    .with_message("TemporalTester:unsupported")
                    .into())
            }
        }
        let years = to_integer_if_integral(args, 0)?;
        let months = to_integer_if_integral(args, 1)?;
        let weeks = to_integer_if_integral(args, 2)?;
        let days = to_integer_if_integral(args, 3)?;
        let hours = to_integer_if_integral(args, 4)?;
        let minutes = to_integer_if_integral(args, 5)?;
        let seconds = to_integer_if_integral(args, 6)?;
        let milliseconds = to_integer_if_integral(args, 7)?;
        let microseconds = to_integer_if_integral(args, 8)?;
        let nanoseconds = to_integer_if_integral(args, 9)?;

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
    fn from_js_value(duration: &JsValue, context: &mut Context) -> JsResult<Duration> {
        if let Some(o) = duration.as_object().and_then(|o| o.downcast_ref::<Self>()) {
            return Ok(o.clone());
        }

        if let JsValue::String(s) = duration {
            return Ok(Duration::Plain(DurationInitializer::String(
                s.to_std_string_escaped(),
            )));
        }

        Err(JsNativeError::error()
            .with_message("TemporalTester:unsupported")
            .into())
    }
    fn from(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let duration = args.get_or_undefined(0);
        if args.len() > 1 {
            return Err(JsNativeError::error()
                .with_message("TemporalTester:unsupported")
                .into());
        }

        let prototype = context
            .eval(Source::from_bytes("Temporal.Duration.prototype"))?
            .as_object()
            .cloned()
            .ok_or_else(|| JsNativeError::error().with_message("TemporalTester:unsupported"))?;

        let duration = Self::from_js_value(duration, context)?;

        Ok(JsObject::from_proto_and_data(prototype, duration).into())
    }

    fn add(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        if args.len() > 1 {
            return Err(JsNativeError::error()
                .with_message("TemporalTester:unsupported")
                .into());
        }
        let lhs = this
            .as_object()
            .cloned()
            .and_then(|o| o.downcast::<Duration>().ok())
            .ok_or(TemporalUnsupported)?;
        let rhs = Self::from_js_value(args.get_or_undefined(0), context)?;

        let prototype = context
            .eval(Source::from_bytes("Temporal.Duration.prototype"))?
            .as_object()
            .cloned()
            .ok_or_else(|| JsNativeError::error().with_message("TemporalTester:unsupported"))?;

        Ok(JsObject::from_proto_and_data(
            prototype,
            Duration::Add {
                target: Box::new(lhs.borrow().data().clone()),
                other: Box::new(rhs),
            },
        ).into())
    }
}
