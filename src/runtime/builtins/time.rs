use crate::runtime::builtin::{BuiltinMethod, BuiltinStruct};
use crate::runtime::value::Value;
use crate::runtime::{RuntimeError, RuntimeResult};
use loft_builtin_macros::loft_builtin;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Sleep for the specified number of milliseconds and return a promise
#[loft_builtin(time.sleep)]
fn time_sleep(#[required] _this: &Value, #[types(number)] args: &[Value]) -> RuntimeResult<Value> {
    let duration_ms = match &args[0] {
        Value::Number(n) => {
            
            n.to_f64().unwrap_or(0.0) as u64
        }
        _ => unreachable!(),
    };

    // Actually sleep (this blocks the current thread)
    // In a real async runtime, this would create a timer future
    thread::sleep(Duration::from_millis(duration_ms));

    // Return a promise that resolves to Unit (void)
    Ok(Value::Promise(Box::new(Value::Unit)))
}

/// Get the current time in milliseconds since Unix epoch
#[loft_builtin(time.now)]
fn time_now(_this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| RuntimeError::new(format!("Failed to get current time: {}", e)))?;

    let millis = now.as_millis() as u64;
    Ok(Value::Number(Decimal::from(millis)))
}

/// Create a high-resolution timer for performance measurement
#[loft_builtin(time.perf_now)]
fn time_perf_now(_this: &Value, _args: &[Value]) -> RuntimeResult<Value> {
    // For simplicity, we'll use a static start time
    // In a real implementation, this would be more sophisticated
    static START_TIME: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

    let start = START_TIME.get_or_init(Instant::now);
    let elapsed = start.elapsed();
    let millis = elapsed.as_millis() as f64;

    Ok(Value::Number(
        Decimal::try_from(millis).unwrap_or(Decimal::from(0)),
    ))
}

/// Format a duration in milliseconds to a human-readable string
#[loft_builtin(time.format)]
fn time_format(#[required] _this: &Value, #[types(number)] args: &[Value]) -> RuntimeResult<Value> {
    let duration_ms = match &args[0] {
        Value::Number(n) => n.to_f64().unwrap_or(0.0),
        _ => unreachable!(),
    };

    let formatted = if duration_ms < 1000.0 {
        format!("{:.2}ms", duration_ms)
    } else if duration_ms < 60000.0 {
        format!("{:.2}s", duration_ms / 1000.0)
    } else if duration_ms < 3600000.0 {
        let minutes = duration_ms / 60000.0;
        format!("{:.2}m", minutes)
    } else {
        let hours = duration_ms / 3600000.0;
        format!("{:.2}h", hours)
    };

    Ok(Value::String(formatted))
}

/// Create a benchmark function that measures execution time
#[loft_builtin(time.benchmark)]
fn time_benchmark(_this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new(
            "time.benchmark() requires a function argument",
        ));
    }

    // For now, just return a placeholder since we don't have full function execution
    // In a full implementation, this would execute the function and measure time
    Ok(Value::String(
        "Benchmark not yet fully implemented".to_string(),
    ))
}

/// Create the Time builtin struct
pub fn create_time_builtin() -> BuiltinStruct {
    let mut time = BuiltinStruct::new("time");

    time.add_method("sleep", time_sleep as BuiltinMethod);
    time.add_method("now", time_now as BuiltinMethod);
    time.add_method("perf_now", time_perf_now as BuiltinMethod);
    time.add_method("format", time_format as BuiltinMethod);
    time.add_method("benchmark", time_benchmark as BuiltinMethod);

    time
}

// Register the builtin automatically
crate::submit_builtin!("time", create_time_builtin);
