//! Built-in Handlebars helpers

use handlebars::{
    Context, Handlebars, Helper, HelperResult, JsonRender, Output, RenderContext, RenderError,
};

/// Register all built-in helpers
pub fn register_builtin_helpers(handlebars: &mut Handlebars) {
    handlebars.register_helper("eq", Box::new(eq_helper));
    handlebars.register_helper("ne", Box::new(ne_helper));
    handlebars.register_helper("lt", Box::new(lt_helper));
    handlebars.register_helper("gt", Box::new(gt_helper));
    handlebars.register_helper("lte", Box::new(lte_helper));
    handlebars.register_helper("gte", Box::new(gte_helper));
    handlebars.register_helper("and", Box::new(and_helper));
    handlebars.register_helper("or", Box::new(or_helper));
    handlebars.register_helper("not", Box::new(not_helper));
    handlebars.register_helper("len", Box::new(len_helper));
    handlebars.register_helper("upper", Box::new(upper_helper));
    handlebars.register_helper("lower", Box::new(lower_helper));
    handlebars.register_helper("capitalize", Box::new(capitalize_helper));
    handlebars.register_helper("json", Box::new(json_helper));
    handlebars.register_helper("default", Box::new(default_helper));
}

/// Equal comparison helper: {{#if (eq a b)}}
fn eq_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param1 = h.param(0).ok_or_else(|| RenderError::new("eq requires 2 parameters"))?;
    let param2 = h.param(1).ok_or_else(|| RenderError::new("eq requires 2 parameters"))?;

    let result = param1.value() == param2.value();
    out.write(&result.to_string())?;
    Ok(())
}

/// Not equal comparison helper: {{#if (ne a b)}}
fn ne_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param1 = h.param(0).ok_or_else(|| RenderError::new("ne requires 2 parameters"))?;
    let param2 = h.param(1).ok_or_else(|| RenderError::new("ne requires 2 parameters"))?;

    let result = param1.value() != param2.value();
    out.write(&result.to_string())?;
    Ok(())
}

/// Less than comparison helper: {{#if (lt a b)}}
fn lt_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param1 = h.param(0).ok_or_else(|| RenderError::new("lt requires 2 parameters"))?;
    let param2 = h.param(1).ok_or_else(|| RenderError::new("lt requires 2 parameters"))?;

    if let (Some(n1), Some(n2)) = (param1.value().as_f64(), param2.value().as_f64()) {
        out.write(&(n1 < n2).to_string())?;
    } else {
        out.write("false")?;
    }
    Ok(())
}

/// Greater than comparison helper: {{#if (gt a b)}}
fn gt_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param1 = h.param(0).ok_or_else(|| RenderError::new("gt requires 2 parameters"))?;
    let param2 = h.param(1).ok_or_else(|| RenderError::new("gt requires 2 parameters"))?;

    if let (Some(n1), Some(n2)) = (param1.value().as_f64(), param2.value().as_f64()) {
        out.write(&(n1 > n2).to_string())?;
    } else {
        out.write("false")?;
    }
    Ok(())
}

/// Less than or equal comparison helper: {{#if (lte a b)}}
fn lte_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param1 = h.param(0).ok_or_else(|| RenderError::new("lte requires 2 parameters"))?;
    let param2 = h.param(1).ok_or_else(|| RenderError::new("lte requires 2 parameters"))?;

    if let (Some(n1), Some(n2)) = (param1.value().as_f64(), param2.value().as_f64()) {
        out.write(&(n1 <= n2).to_string())?;
    } else {
        out.write("false")?;
    }
    Ok(())
}

/// Greater than or equal comparison helper: {{#if (gte a b)}}
fn gte_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param1 = h.param(0).ok_or_else(|| RenderError::new("gte requires 2 parameters"))?;
    let param2 = h.param(1).ok_or_else(|| RenderError::new("gte requires 2 parameters"))?;

    if let (Some(n1), Some(n2)) = (param1.value().as_f64(), param2.value().as_f64()) {
        out.write(&(n1 >= n2).to_string())?;
    } else {
        out.write("false")?;
    }
    Ok(())
}

/// Logical AND helper: {{#if (and a b)}}
fn and_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param1 = h.param(0).ok_or_else(|| RenderError::new("and requires 2 parameters"))?;
    let param2 = h.param(1).ok_or_else(|| RenderError::new("and requires 2 parameters"))?;

    let result = param1.value().as_bool().unwrap_or(false)
        && param2.value().as_bool().unwrap_or(false);
    out.write(&result.to_string())?;
    Ok(())
}

/// Logical OR helper: {{#if (or a b)}}
fn or_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param1 = h.param(0).ok_or_else(|| RenderError::new("or requires 2 parameters"))?;
    let param2 = h.param(1).ok_or_else(|| RenderError::new("or requires 2 parameters"))?;

    let result = param1.value().as_bool().unwrap_or(false)
        || param2.value().as_bool().unwrap_or(false);
    out.write(&result.to_string())?;
    Ok(())
}

/// Logical NOT helper: {{#if (not a)}}
fn not_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).ok_or_else(|| RenderError::new("not requires 1 parameter"))?;

    let result = !param.value().as_bool().unwrap_or(false);
    out.write(&result.to_string())?;
    Ok(())
}

/// Length helper: {{len array}}
fn len_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).ok_or_else(|| RenderError::new("len requires 1 parameter"))?;

    let len = match param.value() {
        serde_json::Value::Array(arr) => arr.len(),
        serde_json::Value::Object(obj) => obj.len(),
        serde_json::Value::String(s) => s.len(),
        _ => 0,
    };

    out.write(&len.to_string())?;
    Ok(())
}

/// Uppercase helper: {{upper text}}
fn upper_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).ok_or_else(|| RenderError::new("upper requires 1 parameter"))?;

    if let Some(s) = param.value().as_str() {
        out.write(&s.to_uppercase())?;
    }
    Ok(())
}

/// Lowercase helper: {{lower text}}
fn lower_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).ok_or_else(|| RenderError::new("lower requires 1 parameter"))?;

    if let Some(s) = param.value().as_str() {
        out.write(&s.to_lowercase())?;
    }
    Ok(())
}

/// Capitalize helper: {{capitalize text}}
fn capitalize_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).ok_or_else(|| RenderError::new("capitalize requires 1 parameter"))?;

    if let Some(s) = param.value().as_str() {
        let mut chars = s.chars();
        if let Some(first) = chars.next() {
            let capitalized = first.to_uppercase().chain(chars).collect::<String>();
            out.write(&capitalized)?;
        }
    }
    Ok(())
}

/// JSON stringify helper: {{json data}}
fn json_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).ok_or_else(|| RenderError::new("json requires 1 parameter"))?;

    out.write(&param.value().render())?;
    Ok(())
}

/// Default value helper: {{default value "fallback"}}
fn default_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).ok_or_else(|| RenderError::new("default requires 2 parameters"))?;
    let default = h.param(1).ok_or_else(|| RenderError::new("default requires 2 parameters"))?;

    let value = if param.value().is_null() {
        default.value()
    } else {
        param.value()
    };

    out.write(&value.render())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_eq_helper() {
        let mut handlebars = Handlebars::new();
        register_builtin_helpers(&mut handlebars);

        handlebars.register_template_string("test", "{{#if (eq a b)}}equal{{else}}not equal{{/if}}").unwrap();

        let data = json!({"a": 5, "b": 5});
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "equal");

        let data = json!({"a": 5, "b": 10});
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "not equal");
    }

    #[test]
    fn test_upper_helper() {
        let mut handlebars = Handlebars::new();
        register_builtin_helpers(&mut handlebars);

        handlebars.register_template_string("test", "{{upper name}}").unwrap();

        let data = json!({"name": "hello"});
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "HELLO");
    }

    #[test]
    fn test_len_helper() {
        let mut handlebars = Handlebars::new();
        register_builtin_helpers(&mut handlebars);

        handlebars.register_template_string("test", "{{len items}}").unwrap();

        let data = json!({"items": [1, 2, 3, 4, 5]});
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "5");
    }
}

